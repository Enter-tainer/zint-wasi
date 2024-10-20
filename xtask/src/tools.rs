use std::{
    collections::{BTreeSet, HashMap},
    ffi::{OsStr, OsString},
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    io::{self, Read, Seek},
    mem::MaybeUninit,
    os::unix::ffi::OsStringExt,
    path::{Path, PathBuf},
    process as proc,
    str::FromStr,
    sync::OnceLock,
};

use crate::log::*;
use crate::state_path;

pub fn exists(path: impl AsRef<Path>) -> bool {
    std::fs::exists(path.as_ref()).ok().unwrap_or_default()
}
pub fn local_tool_path(name: impl AsRef<Path>) -> PathBuf {
    state_path!(WORK_DIR).join("tools").join(name)
}

pub fn cmd<S: AsRef<OsStr>>(
    program: impl AsRef<OsStr>,
    args: impl IntoIterator<Item = S>,
) -> proc::Command {
    let mut result = proc::Command::new(program.as_ref());
    result.args(args);
    result
}

pub fn has_command(name: impl AsRef<OsStr>) -> bool {
    use std::sync::RwLock;
    static CACHE: OnceLock<RwLock<HashMap<OsString, bool>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| RwLock::new(HashMap::new()));

    if let Ok(cache) = cache.try_read() {
        if let Some(cached) = cache.get(name.as_ref()) {
            return *cached;
        }
    }

    let which = if cfg!(target_os = "windows") {
        "where"
    } else if cfg!(unix) {
        "which"
    } else {
        panic!("no known alternative for UNIX 'which' command on current platform")
    };
    let output = match cmd(which, [name.as_ref()]).output() {
        Ok(it) => it,
        Err(_) => panic!("can't run '{}' to check evirnoment", which),
    };

    let result = output.status.success() && !output.stdout.is_empty();
    if let Ok(mut cache) = cache.try_write() {
        cache.insert(name.as_ref().to_os_string(), result);
    }

    result
}

const CARGO: &str = "cargo";
pub fn cargo<S: AsRef<OsStr>>(
    args: impl IntoIterator<Item = S>,
) -> Result<proc::Command, CommandError> {
    if !has_command(CARGO) {
        return Err(CommandError::missing_tool(
            CARGO,
            Some("https://rustup.rs/"),
        ));
    }
    Ok(cmd(CARGO, args))
}

pub fn cargo_has_tool(tool: impl AsRef<str>) -> bool {
    if !has_command(CARGO) {
        return false;
    }

    let mut install_list = cmd(CARGO, ["install", "--list"]);
    let install_list = match install_list.output() {
        Ok(it) => OsString::from_vec(it.stdout).to_string_lossy().into_owned(),
        Err(_) => panic!("can't query installed crates from {CARGO}"),
    };

    install_list
        .lines()
        .filter(|it| {
            it.chars()
                .next()
                .map(|it| it.is_whitespace())
                .unwrap_or_default()
        })
        .any(|it| it.trim() == tool.as_ref())
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

macro_rules! make_runner {
    (
        Fn($($arg: ident: $arg_ty: ty),*) -> Result<$returned: ty, $error: ty> $init: block
    ) => {
        {
            type Runner =
                Box<dyn Fn($($arg_ty),*) -> Result<$returned, $error> + Send + Sync + 'static>;
            static RUNNER: OnceLock<Runner> = OnceLock::new();
            RUNNER.get_or_init(|| $init)
        }
    };
    (
        fn($($arg: ident: $arg_ty: ty),*) -> Result<$returned: ty, $error: ty> $init: block
    ) => {
        {
            type Runner = fn($($arg_ty),*) -> Result<$returned, $error>;
            static RUNNER: OnceLock<Runner> = OnceLock::new();
            RUNNER.get_or_init(|| $init)
        }
    };
}

pub fn download(url: impl AsRef<str>, target: impl AsRef<Path>) -> Result<(), DownloadError> {
    if let Some(parent) = target.as_ref().parent() {
        std::fs::create_dir_all(parent).map_err(DownloadError::IO)?;
    }

    let runner = make_runner!(fn(url: &str, target: &Path) -> Result<(), DownloadError> {
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

        |_url, _target| {
            Err(DownloadError::CommandError(CommandError::missing_tool(
                "wget",
                Some("https://www.gnu.org/software/wget/"),
            )))
        }
    });
    group!("Download: {}", target.as_ref().display());
    info!(
        "\t- downloading '{}' to '{}'",
        url.as_ref(),
        target.as_ref().display()
    );
    let result = (runner)(url.as_ref(), target.as_ref());
    end_group!();
    result
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
    output: impl AsRef<Path>,
    args: impl IntoIterator<Item = S>,
) -> Result<(), CommandError>
where
    S: AsRef<OsStr>,
{
    if !has_command(TAR) {
        return Err(CommandError::missing_tool(
            TAR,
            Some("https://www.gnu.org/software/tar/"),
        ));
    }

    if !exists(&archive) {
        return Err(CommandError::file_not_found("archive", archive));
    }
    if let Err(err) = std::fs::create_dir_all(&output) {
        return Err(CommandError::inaccessible("output", err));
    }

    group!("Extract: {}", archive.as_ref().display());
    info!(
        "\t- extracting '{}' into '{}'",
        archive.as_ref().display(),
        output.as_ref().display()
    );
    let result = cmd(
        TAR,
        [
            OsStr::new("-xvsf"),
            OsStr::new(archive.as_ref().as_os_str()),
            OsStr::new("-C"),
            OsStr::new(output.as_ref().as_os_str()),
        ]
        .into_iter()
        .map(|it| it.to_os_string())
        .chain(args.into_iter().map(|it| it.as_ref().to_os_string())),
    )
    .program_status(TAR);
    end_group!();
    result
}

const WASI_STUB: &str = "wasi-stub";
/// Tries running wasi-stub from PATH, then from `./target/release` dir, then
/// from `./target/debug`, if all else fails, builds it with cargo.
pub fn wasi_stub(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<(), CommandError> {
    if !exists(&input) {
        return Err(CommandError::file_not_found("input", &input).program(WASI_STUB));
    }
    if let Some(parent) = output.as_ref().parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            return Err(CommandError::inaccessible("output", err));
        }
    }

    let runner = make_runner!(Fn(input: &Path, output: &Path) -> Result<(), CommandError> {
        let runner = |executable: &OsStr| {
            let executable = executable.to_owned();
            Box::new(move |file: &Path, output: &Path| {
                cmd(
                    executable.as_os_str(),
                    [
                        OsStr::new("-r"),
                        OsStr::new("0"),
                        file.as_os_str(),
                        OsStr::new("-o"),
                        output.as_os_str(),
                    ],
                )
                .program_status(WASI_STUB)
            })
        };

        if has_command(WASI_STUB) {
            return runner(OsStr::new(WASI_STUB));
        }

        let min_proto_path = state_path!(WASM_MIN_PROTOCOL_DIR, default: "$<root>/zint-typst-plugin/vendor/wasm-minimal-protocol");
        let try_prebuilt = |kind: &str| {
            let executable_path = min_proto_path.join("target").join(kind).join(WASI_STUB);
            if !exists(&executable_path) {
                return None;
            }
            let executable_path = executable_path
                .canonicalize()
                .expect("unable to canonicalize path that exists");
            return Some(runner(executable_path.as_os_str()));
        };
        if let Some(it) = try_prebuilt("release") {
            return it;
        }
        if let Some(it) = try_prebuilt("debug") {
            return it;
        }

        Box::new(move |file: &Path, output: &Path| {
            let min_proto_path = min_proto_path.join("Cargo.toml");
            cargo([
                OsStr::new("run"),
                OsStr::new("--manifest-path"),
                min_proto_path.as_os_str(),
                OsStr::new("--bin=wasi-stub"),
                OsStr::new("--release"),
                OsStr::new("--"),
                OsStr::new("-r"),
                OsStr::new("0"),
                file.as_os_str(),
                OsStr::new("-o"),
                output.as_os_str(),
            ])?.program_status(WASI_STUB)
        })
    });
    (runner)(input.as_ref(), output.as_ref()).map_err(|err| err.program(WASI_STUB))
}

pub const WASM_OPT: &str = "wasm-opt";
pub fn wasm_opt(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<(), CommandError> {
    if !exists(&input) {
        return Err(CommandError::file_not_found("input", &input).program(WASM_OPT));
    }
    if let Some(parent) = output.as_ref().parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            return Err(CommandError::inaccessible("output", err));
        }
    }

    let runner = make_runner!(Fn(input: &Path, output: &Path) -> Result<(), CommandError> {
        let tool_path = local_tool_path(WASM_OPT);
        let command = if has_command(WASM_OPT) {
            Some(OsString::from_str(WASM_OPT).unwrap())
        } else if exists(&tool_path) {
            Some(tool_path.as_os_str().to_owned())
        } else {
            None
        };
        if let Some(command) = command {
            return Box::new(move |file: &Path, output: &Path| {
                cmd(
                    &command,
                    [
                        file.as_os_str(),
                        OsStr::new("-O3"),
                        OsStr::new("--enable-bulk-memory"),
                        OsStr::new("-o"),
                        output.as_os_str(),
                    ],
                )
                .program_status(WASM_OPT)
            })
        }

        return Box::new(|_input, _output| {
            Err(CommandError::missing_tool(
                "wasm-opt",
                Some("https://github.com/WebAssembly/binaryen/releases"),
            ))
        })
    });

    (runner)(input.as_ref(), output.as_ref()).map_err(|err| err.program(WASM_OPT))
}

pub const TYPST: &str = "typst";
pub fn typst_compile(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> Result<std::time::Duration, CommandError> {
    if !exists(&input) {
        return Err(CommandError::file_not_found("input", &input).program(TYPST));
    }
    if let Some(parent) = output.as_ref().parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            return Err(CommandError::inaccessible("output", err));
        }
    }

    let runner = make_runner!(fn(input: &Path, output: &Path) -> Result<std::time::Duration, CommandError> {
        if has_command(TYPST) {
            return |input, output| {
                let begin = std::time::Instant::now();
                let result = cmd(
                    TYPST,
                    [
                        OsStr::new("compile"),
                        input.as_os_str(),
                        output.as_os_str(),
                    ],
                )
                .program_status(TYPST);
                let duration = std::time::Instant::now() - begin;
                result.map(|_| duration)
            }
        }

        if exists(local_tool_path(TYPST)) {
            return |input, output| {
                let begin = std::time::Instant::now();
                let result = cmd(
                    local_tool_path(TYPST),
                    [
                        OsStr::new("compile"),
                        input.as_os_str(),
                        output.as_os_str(),
                    ],
                )
                .program_status(TYPST);
                let duration = std::time::Instant::now() - begin;
                result.map(|_| duration)
            }
        }

        |_input, _output| {
            Err(CommandError::missing_tool(
                TYPST,
                Some("https://github.com/typst/typst/releases"),
            ))
        }
    });

    (runner)(input.as_ref(), output.as_ref())
}

fn hash_single_file<H>(path: impl AsRef<Path>, state: &mut H) -> io::Result<()>
where
    H: std::hash::Hasher,
{
    let file = std::fs::File::open(path)?;
    let mut file = std::io::BufReader::new(file);
    let mut buffer: MaybeUninit<[u8; 1024]> = MaybeUninit::uninit();
    unsafe {
        let buffer = buffer.as_mut_ptr().as_mut().unwrap_unchecked();
        loop {
            let count = file.read(buffer)?;
            if count == 0 {
                break;
            }
            buffer[..count].hash(state)
        }
    }
    Ok(())
}

pub fn hash_files<P>(files: impl IntoIterator<Item = P>) -> u64
where
    P: AsRef<Path>,
{
    let mut state = xxhash_rust::xxh3::Xxh3::new();

    // We need to discover files separately because listing a directory doesn't
    // need to return files in the same order every time.
    let mut discovered = BTreeSet::new();
    for root in files {
        let root = root.as_ref();
        if root.is_file() {
            discovered.insert(root.to_path_buf());
            continue;
        } else {
            let walk = walkdir::WalkDir::new(root);
            for item in walk.into_iter().filter_map(Result::ok) {
                if item.file_type().is_file() {
                    discovered.insert(item.into_path());
                }
            }
        }
    }

    // Once we have a sorted list of discovered files, we can hash them.
    for file in discovered {
        let _ = hash_single_file(file, &mut state);
    }

    state.finish()
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileSize(u64);
impl FileSize {
    pub fn of(file: impl AsRef<Path>) -> Result<Self, CommandError> {
        let file = file.as_ref();
        if !exists(file) {
            return Err(CommandError::file_not_found("file", file).program("FileSize"));
        }

        let mut file = match std::fs::File::open(file) {
            Ok(it) => it,
            Err(_) => return Ok(FileSize(0)),
        };
        Ok(FileSize(
            file.seek(io::SeekFrom::End(0)).unwrap_or_default(),
        ))
    }
}
impl From<u64> for FileSize {
    fn from(value: u64) -> Self {
        FileSize(value)
    }
}
impl From<FileSize> for u64 {
    fn from(value: FileSize) -> Self {
        value.0
    }
}
impl From<FileSize> for usize {
    fn from(value: FileSize) -> Self {
        value.0 as usize
    }
}
impl Display for FileSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const BYTES_PER_KIB: u64 = 1024;
        const BYTES_PER_MIB: u64 = BYTES_PER_KIB * 1024;
        const BYTES_PER_GIB: u64 = BYTES_PER_MIB * 1024;
        if self.0 / BYTES_PER_GIB >= 1 {
            write!(f, "{:.3} GiB", self.0 as f32 / BYTES_PER_GIB as f32)
        } else if self.0 / BYTES_PER_MIB >= 1 {
            write!(f, "{:.3} MiB", self.0 as f32 / BYTES_PER_MIB as f32)
        } else if self.0 / BYTES_PER_KIB >= 1 {
            write!(f, "{:.3} KiB", self.0 as f32 / BYTES_PER_KIB as f32)
        } else {
            write!(f, "{} B", self.0)
        }
    }
}

#[allow(dead_code)]
trait ProgramExt {
    fn program_status(self, program: impl AsRef<str>) -> Result<(), CommandError>;
    fn program_output(self, program: impl AsRef<str>) -> proc::Output;
}
impl ProgramExt for proc::Command {
    fn program_status(mut self, program: impl AsRef<str>) -> Result<(), CommandError> {
        match self.status() {
            Ok(it) => CommandError::from_exit(it),
            Err(_) => panic!("unable to run {}", program.as_ref()),
        }
    }
    fn program_output(mut self, program: impl AsRef<str>) -> proc::Output {
        match self.output() {
            Ok(it) => it,
            Err(_) => panic!("unable to run {}", program.as_ref()),
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
                    "{}bad '{argument}' argument{detail}",
                    program
                        .map(|p| format!("{p} executed with "))
                        .unwrap_or_default()
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
