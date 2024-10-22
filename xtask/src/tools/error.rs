use std::{
    fmt::{Debug, Display},
    io,
    path::Path,
    process::ExitStatus,
};

#[derive(Debug)]
pub enum DownloadError {
    CommandError(CommandError),
    BadUrl { url: String },
    IO(io::Error),
}
impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DownloadError::CommandError(exit) => std::fmt::Display::fmt(exit, f),
            DownloadError::BadUrl { url } => write!(f, "can't resolve url: '{url}'"),
            DownloadError::IO(io) => write!(f, "io error: {io}"),
        }
    }
}
impl std::error::Error for DownloadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CommandError(source) => Some(source),
            Self::IO(source) => Some(source),
            _ => None,
        }
    }
}
impl From<CommandError> for DownloadError {
    fn from(err: CommandError) -> Self {
        if let CommandError::Other(other) = err {
            match other.downcast() {
                Ok(this) => *this,
                Err(other) => Self::CommandError(CommandError::Other(other)),
            }
        } else {
            Self::CommandError(err)
        }
    }
}

pub enum CommandError {
    MissingTool {
        program: &'static str,
        install_from: Option<&'static str>,
    },
    ExitError {
        program: Option<&'static str>,
        code: std::num::NonZeroI32,
    },
    Interrupted {
        program: Option<&'static str>,
        interrupt: i32,
    },
    BadArgument {
        program: Option<&'static str>,
        argument: &'static str,
        expect_found: Option<(&'static str, Box<dyn Display + Send + Sync + 'static>)>,
        reason: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    },
    Other(Box<dyn std::error::Error + Send + Sync + 'static>),
}

impl CommandError {
    pub fn new(code: i32) -> Self {
        assert!(code != 0, "exit code 0 doesn't indicate an error");
        unsafe {
            Self::ExitError {
                program: None,
                code: std::num::NonZeroI32::new_unchecked(code),
            }
        }
    }
    pub fn interrupt(interrupt: i32) -> Self {
        Self::Interrupted {
            program: None,
            interrupt,
        }
    }
    pub fn missing_tool(name: &'static str, source: Option<&'static str>) -> Self {
        Self::MissingTool {
            program: name,
            install_from: source,
        }
    }
    pub fn file_not_found(argument: &'static str, file: impl AsRef<Path>) -> Self {
        Self::BadArgument {
            program: None,
            argument,
            expect_found: None,
            reason: Some(Box::new(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "file not found or inaccessible (path: '{}')",
                    file.as_ref().display()
                ),
            ))),
        }
    }
    pub fn inaccessible(argument: &'static str, source: io::Error) -> Self {
        Self::BadArgument {
            program: None,
            argument,
            expect_found: None,
            reason: Some(Box::new(source)),
        }
    }
    #[inline]
    pub fn other<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Other(Box::new(error))
    }

    pub fn program(mut self, name: &'static str) -> Self {
        match &mut self {
            Self::MissingTool { program, .. } => *program = name,
            Self::ExitError { program, .. } => *program = Some(name),
            Self::Interrupted { program, .. } => *program = Some(name),
            Self::BadArgument { program, .. } => *program = Some(name),
            Self::Other(_) => {}
        }
        self
    }

    pub fn from_exit_status(exit: &ExitStatus) -> Result<(), Self> {
        #[allow(unreachable_patterns)]
        match exit.code() {
            Some(0) => Ok(()),
            Some(code) => Err(Self::new(code)),
            #[cfg(unix)]
            None => Err(Self::interrupt(
                std::os::unix::prelude::ExitStatusExt::signal(exit)
                    .expect("program terminated with no exit code, nor interrupt signal"),
            )),
            _ => unreachable!("program terminated with no exit code"),
        }
    }
}

pub trait CommandResultExt<T> {
    fn map_exit_codes<M>(self, mapping: M) -> Result<T, CommandError>
    where
        M: Fn(i32) -> Option<CommandError>;
}
impl<T> CommandResultExt<T> for Result<T, CommandError> {
    fn map_exit_codes<M>(self, mapping: M) -> Result<T, CommandError>
    where
        M: Fn(i32) -> Option<CommandError>,
    {
        match self {
            Ok(it) => Ok(it),
            Err(err) => Err(
                if let CommandError::ExitError { code: err_code, .. } = err {
                    mapping(Into::<i32>::into(err_code)).unwrap_or(err)
                } else {
                    err
                },
            ),
        }
    }
}

impl From<CommandError> for io::Error {
    fn from(err: CommandError) -> Self {
        match err {
            CommandError::MissingTool { .. } => io::Error::new(io::ErrorKind::NotFound, err),
            CommandError::ExitError { .. } => io::Error::new(io::ErrorKind::Other, err),
            CommandError::Interrupted { .. } => io::Error::new(io::ErrorKind::Interrupted, err),
            CommandError::BadArgument { .. } => io::Error::new(io::ErrorKind::InvalidInput, err),
            CommandError::Other(other) => io::Error::new(io::ErrorKind::Other, other),
        }
    }
}

impl Debug for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = f.debug_struct(match self {
            Self::MissingTool { .. } => "CommandError::MissingTool",
            Self::ExitError { .. } => "CommandError::ExitError",
            Self::Interrupted { .. } => "CommandError::Interrupted",
            Self::BadArgument { .. } => "CommandError::BadArgument",
            Self::Other(other) => {
                return std::fmt::Debug::fmt(other, f);
            }
        });
        match self {
            Self::MissingTool { program, .. } => {
                result.field("program", program);
            }
            Self::ExitError { program, code } => {
                result.field("program", program);
                result.field("code", code);
            }
            Self::Interrupted { program, interrupt } => {
                result.field("program", program);
                result.field("interrupt", interrupt);
            }
            Self::BadArgument {
                program,
                argument,
                expect_found,
                reason,
            } => {
                result.field("program", program);
                result.field("argument", argument);
                if let Some((expected, found)) = expect_found {
                    result.field("expected", expected);
                    result.field("found", &found.as_ref().to_string());
                } else if let Some(reason) = reason {
                    result.field("reason", reason);
                }
            }
            Self::Other(_) => unreachable!(),
        }
        result.finish()
    }
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingTool {
                program,
                install_from,
            } => {
                write!(f, "{program} is not in PATH, and it's required for running requested tasks. Install it using {} or from",
                    if cfg!(target_os = "macos") {
                        "brew"
                    } else if cfg!(target_os = "windows") {
                        "win-get"
                    } else {
                        "a package manager"
                    }
                )?;
                if let Some(from) = install_from {
                    write!(f, ": '{from}'")
                } else {
                    write!(f, " a credible source.")
                }
            }
            Self::ExitError { program, code } => {
                write!(
                    f,
                    "{} exited with code #{}",
                    program.unwrap_or("process"),
                    code
                )
            }
            Self::Interrupted { program, interrupt } => {
                write!(
                    f,
                    "{} interrupted (signal: {})",
                    program.unwrap_or("process"),
                    interrupt
                )
            }
            Self::BadArgument {
                program,
                argument,
                expect_found,
                reason,
            } => {
                let detail = if let Some((expected, found)) = expect_found {
                    format!(" {expected} expected, but found {found}")
                } else if let Some(why) = reason {
                    format!(": {why}")
                } else {
                    "".to_string()
                };
                write!(
                    f,
                    "{}bad '{argument}' argument{detail}",
                    program
                        .map(|p| format!("{p} executed with "))
                        .unwrap_or_default()
                )
            }
            Self::Other(other) => std::fmt::Display::fmt(other, f),
        }
    }
}

impl std::error::Error for CommandError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            CommandError::BadArgument {
                reason: Some(reason),
                ..
            } => Some(reason.as_ref()),
            CommandError::Other(other) => Some(other.as_ref()),
            _ => None,
        }
    }
}
