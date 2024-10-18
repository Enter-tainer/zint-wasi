use std::{
    ffi::{OsStr, OsString},
    fmt::{Debug, Display},
    io,
    path::Path,
    process as proc,
    sync::{Once, OnceLock},
};

use crate::state_path;

pub fn exists(path: impl AsRef<Path>) -> bool {
    std::fs::exists(path.as_ref()).ok().unwrap_or_default()
}

pub fn cmd<S: AsRef<OsStr>>(
    program: impl AsRef<OsStr>,
    args: impl IntoIterator<Item = S>,
) -> proc::Command {
    let mut result = proc::Command::new(program.as_ref());
    result.args(args);
    result
}

/* USE: path.canonicalize()
pub fn realpath(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    cmd("realpath", [path.as_ref()]).output().map(|it| PathBuf::from(OsString::from_vec(it.stdout)))
}
*/

pub fn has_command(name: impl AsRef<OsStr>) -> bool {
    let which = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };
    let output = match cmd(which, [name]).output() {
        Ok(it) => it,
        Err(_) => panic!("can't run '{}' to check evirnoment", which),
    };
    output.status.success() && !output.stdout.is_empty()
}

pub fn cargo<S: AsRef<OsStr>>(args: impl IntoIterator<Item = S>) -> io::Result<proc::ExitStatus> {
    cmd("cargo", args)
        //.env("WASI_SDK_PATH", "./target/wasi_sdk")
        .status()
}

#[cfg(target_os = "windows")]
pub fn run_powershell<S: AsRef<str>>(
    code: impl IntoIterator<Item = S>,
) -> io::Result<proc::ExitStatus> {
    // not tested
    let mut ps = proc::Command::new("powershell")
        .args(["-Command", "-"])
        .spawn()
        .expect("unable to run powershell");
    let mut stdin = ps.stdin.take().expect("can't pipe to powershell");
    for line in code.into_iter() {
        stdin
            .write_all(line.as_ref().as_bytes())
            .expect("can't write commands to powershell");
        stdin
            .write_all(b"\n")
            .expect("can't write commands to powershell");
    }
    stdin
        .write_all(b"exit\n")
        .expect("can't terminate powershell");
    ps.wait()
}

#[cfg(not(target_os = "windows"))]
const WGET: &str = "wget";
#[cfg(target_os = "windows")]
const WGET: &str = "wget.exe";
#[cfg(not(target_os = "windows"))]
const CURL: &str = "curl";
#[cfg(target_os = "windows")]
const CURL: &str = "curl.exe";

fn download_wget(url: &str, path: &Path) -> Result<(), DownloadError> {
    let status = cmd(
        WGET,
        [
            OsStr::new(url),
            OsStr::new("-q"),
            OsStr::new("--show-progress"),
            OsStr::new("-O"),
            path.as_os_str(),
        ],
    )
    .status()
    .map_err(DownloadError::IO)?;
    // https://www.gnu.org/software/wget/manual/html_node/Exit-Status.html
    match status.code() {
        Some(0) => Ok(()),
        Some(3) => Err(DownloadError::IO(io::Error::new(
            io::ErrorKind::Other,
            format!("file I/O error: {}", path.display()),
        ))),
        Some(4) => Err(DownloadError::BadUrl {
            url: url.to_string(),
        }),
        _ => Err(DownloadError::CommandError(
            CommandError::from(status).program(WGET),
        )),
    }
}
fn download_curl(url: &str, path: &Path) -> Result<(), DownloadError> {
    let status = cmd(
        CURL,
        [
            OsStr::new("-L"),
            OsStr::new(url),
            OsStr::new("--output"),
            path.as_os_str(),
        ],
    )
    .status()
    .map_err(DownloadError::IO)?;

    // https://everything.curl.dev/cmdline/exitcode.html
    match status.code() {
        Some(0) => Ok(()),
        Some(3) | Some(5) | Some(6) | Some(7) => Err(DownloadError::BadUrl {
            url: url.to_string(),
        }),
        Some(23) => Err(DownloadError::IO(io::Error::new(
            io::ErrorKind::Other,
            format!("file I/O error: {}", path.display()),
        ))),
        _ => Err(DownloadError::CommandError(
            CommandError::from(status).program(CURL),
        )),
    }
}

pub fn download(url: impl AsRef<str>, target: impl AsRef<Path>) -> Result<(), DownloadError> {
    #[allow(clippy::type_complexity)]
    static EXECUTOR: OnceLock<fn(&str, &Path) -> Result<(), DownloadError>> = OnceLock::new();
    let run = EXECUTOR.get_or_init(|| {
        if has_command(WGET) {
            return download_wget;
        }
        if has_command(CURL) {
            return download_curl;
        }
        #[cfg(target_os = "windows")]
        {
            // untested code from SO
            return |url, path| {
                run_powershell([
                    "$client = new-object System.Net.WebClient".to_string(),
                    format!("$client.DownloadFile(\"{url}\",\"{path}\")"),
                ])
            };
        }
        panic!("no supported program that can download files is present")
    });
    println!(
        "\t- downloading '{}' to '{}'",
        url.as_ref(),
        target.as_ref().display()
    );
    run(url.as_ref(), target.as_ref())
}

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

const TAR: &str = "tar";
pub fn untar<S>(
    archive: impl AsRef<Path>,
    target: impl AsRef<Path>,
    args: impl IntoIterator<Item = S>,
) -> io::Result<()>
where
    S: AsRef<OsStr>,
{
    static CHECK: Once = Once::new();
    CHECK.call_once(|| {
        if !has_command(TAR) {
            panic!("missing `tar` command in path")
        }
    });

    println!(
        "\t- extracting '{}' into '{}'",
        archive.as_ref().display(),
        target.as_ref().display()
    );
    let status = cmd(
        TAR,
        [
            OsStr::new("-xsf"),
            OsStr::new(archive.as_ref().as_os_str()),
            OsStr::new("-C"),
            OsStr::new(target.as_ref().as_os_str()),
        ]
        .into_iter()
        .map(|it| it.to_os_string())
        .chain(args.into_iter().map(|it| it.as_ref().to_os_string())),
    )
    .status()?;

    match status.code() {
        Some(0) => Ok(()),
        None => Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "tar process interrupted",
        )),
        Some(other) => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("tar exited with code #{other}"),
        )),
    }
}

type WasiStubRunner =
    Box<dyn Fn(&Path, &Path) -> io::Result<proc::ExitStatus> + Send + Sync + 'static>;
#[inline(always)]
fn wasi_stub_runner(executable: impl AsRef<OsStr>) -> WasiStubRunner {
    let executable = Box::leak(Box::new(executable.as_ref().to_os_string()));
    Box::new(move |file: &Path, output: &Path| {
        let r = OsString::from("-r");
        let zero = OsString::from("0");
        let o = OsString::from("-o");
        cmd(
            executable.as_os_str(),
            [
                r.as_os_str(),
                zero.as_os_str(),
                file.as_os_str(),
                o.as_os_str(),
                output.as_os_str(),
            ],
        )
        .status()
    })
}

const WASI_STUB: &str = "wasi-stub";
/// Tries running wasi-stub from PATH, then from `./target/release` dir, then
/// from `./target/debug`, if all else fails, builds it with cargo.
pub fn wasi_stub(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<(), CommandError> {
    static RUNNER: OnceLock<WasiStubRunner> = OnceLock::new();
    let runner = RUNNER.get_or_init(|| {
        if has_command(WASI_STUB) {
            return wasi_stub_runner(WASI_STUB);
        }

        let work_dir = state_path!(WORK_DIR);
        let try_prebuilt = |kind: &str| {
            let executable_path = work_dir.join(kind).join(WASI_STUB);
            if !exists(&executable_path) {
                return None;
            }
            let executable_path = executable_path
                .canonicalize()
                .expect("unable to canonicalize path that exists");
            return Some(wasi_stub_runner(executable_path));
        };
        if let Some(it) = try_prebuilt("release") {
            return it;
        }
        if let Some(it) = try_prebuilt("debug") {
            return it;
        }

        return Box::new(move |file: &Path, output: &Path| {
            let run = OsString::from("run");
            let release = OsString::from("--release");
            let bin = OsString::from("--bin=wasi-stub");
            let pass = OsString::from("--");
            let r = OsString::from("-r");
            let zero = OsString::from("0");
            let o = OsString::from("-o");
            cargo([
                run.as_os_str(),
                release.as_os_str(),
                bin.as_os_str(),
                pass.as_os_str(),
                r.as_os_str(),
                zero.as_os_str(),
                file.as_os_str(),
                o.as_os_str(),
                output.as_os_str(),
            ])
        });
    });

    if !exists(&input) {
        return Err(CommandError::file_not_found("input", &input).program(WASI_STUB));
    }

    CommandError::from_exit(match (runner)(input.as_ref(), output.as_ref()) {
        Ok(it) => it,
        Err(_) => panic!("unable to run {WASI_STUB}"),
    })
    .map_err(|err| err.program(WASI_STUB))
}

pub const WASM_OPT: &str = "wasm-opt";
pub fn wasm_opt(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<(), CommandError> {
    static RUNNER: OnceLock<WasmOptRunner> = OnceLock::new();
    type WasmOptRunner =
        Box<dyn Fn(&Path, &Path) -> io::Result<proc::ExitStatus> + Send + Sync + 'static>;

    let runner = RUNNER.get_or_init(|| {
        let runner = |program: &OsStr| {
            let program = OsString::from(program);
            Box::new(move |file: &Path, output: &Path| {
                let opt = OsString::from("-O3");
                let bulk_mem = OsString::from("--enable-bulk-memory");
                let o = OsString::from("-o");
                cmd(
                    &program,
                    [
                        file.as_os_str(),
                        opt.as_os_str(),
                        bulk_mem.as_os_str(),
                        o.as_os_str(),
                        output.as_os_str(),
                    ],
                )
                .status()
            })
        };

        if has_command(WASM_OPT) {
            return runner(OsStr::new(WASM_OPT));
        }

        let tool_path = state_path!(WORK_DIR).join("tools").join(WASM_OPT);
        if exists(&tool_path) {
            return runner(tool_path.as_os_str());
        }

        // not really, checked more places, but this is easier to explain
        panic!("unable to find {WASM_OPT} in PATH")
    });

    if !exists(&input) {
        return Err(CommandError::file_not_found("input", &input).program(WASM_OPT));
    }

    CommandError::from_exit(match (runner)(input.as_ref(), output.as_ref()) {
        Ok(it) => it,
        Err(_) => panic!("unable to run {WASM_OPT}"),
    })
    .map_err(|err| err.program(WASM_OPT))
}

const TYPST: &str = "typst";
pub fn typst_compile(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<(), CommandError> {
    if !has_command(TYPST) {
        return Err(CommandError::missing_tool(TYPST, Some("https://github.com/typst/typst/releases")));
    }
    if !exists(&input) {
        return Err(CommandError::file_not_found("input", &input).program(TYPST));
    }

    let status = cmd(TYPST, [
        OsStr::new("compile"),
        input.as_ref().as_os_str(),
        output.as_ref().as_os_str(),
    ]).status();
    let status = match status {
        Ok(it) => it,
        Err(_) => panic!("unable to run {TYPST}")
    };
    CommandError::from_exit(status)
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

    pub fn program(mut self, name: &'static str) -> Self {
        match &mut self {
            CommandError::MissingTool { program, .. } => *program = name,
            CommandError::ExitError { program, .. } => *program = Some(name),
            CommandError::Interrupted { program, .. } => *program = Some(name),
            CommandError::BadArgument { program, .. } => *program = Some(name),
        }
        self
    }

    pub fn from_exit(exit: proc::ExitStatus) -> Result<(), Self> {
        #[allow(unreachable_patterns)]
        match exit.code() {
            Some(0) => Ok(()),
            Some(code) => Err(Self::new(code)),
            #[cfg(unix)]
            None => Err(Self::interrupt(
                std::os::unix::prelude::ExitStatusExt::signal(&exit)
                    .expect("program terminated with no exit code, nor interrupt signal"),
            )),
            _ => unreachable!("program terminated with no exit code"),
        }
    }
}

impl From<proc::ExitStatus> for CommandError {
    fn from(exit: proc::ExitStatus) -> Self {
        Self::from_exit(exit).expect_err("not an error")
    }
}

impl Debug for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = f.debug_struct(match self {
            CommandError::MissingTool { .. } => "CommandError::MissingTool",
            CommandError::ExitError { .. } => "CommandError::ExitError",
            CommandError::Interrupted { .. } => "CommandError::Interrupted",
            CommandError::BadArgument { .. } => "CommandError::BadArgument",
        });
        match self {
            CommandError::MissingTool { program, .. } => {
                result.field("program", program);
            }
            CommandError::ExitError { program, code } => {
                result.field("program", program);
                result.field("code", code);
            }
            CommandError::Interrupted { program, interrupt } => {
                result.field("program", program);
                result.field("interrupt", interrupt);
            }
            CommandError::BadArgument {
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
        }
        result.finish()
    }
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::MissingTool {
                program,
                install_from,
            } => {
                write!(f, "{program} is not installed, and it's required for running requested tasks. Install it using {} or from",
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
            CommandError::ExitError { program, code } => {
                write!(
                    f,
                    "{} exited with code #{}",
                    program.unwrap_or("process"),
                    code
                )
            }
            CommandError::Interrupted { program, interrupt } => {
                write!(
                    f,
                    "{} interrupted (signal: {})",
                    program.unwrap_or("process"),
                    interrupt
                )
            }
            CommandError::BadArgument {
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
                    "{} called with bad '{argument}' argument{detail}",
                    program.unwrap_or("process")
                )
            }
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
            _ => None,
        }
    }
}
