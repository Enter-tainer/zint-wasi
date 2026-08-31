use std::{
    collections::{BTreeSet, HashMap},
    ffi::{OsStr, OsString},
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    io::{self, Read, Seek},
    mem::MaybeUninit,
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

/// `name` with the platform executable suffix appended (`.exe` on Windows).
///
/// Only needed where a tool is addressed as a file: a path on disk, or a member
/// name inside a release archive. Looking a tool up in `PATH` doesn't need it.
pub fn exe_name(name: impl AsRef<str>) -> String {
    format!("{}{}", name.as_ref(), std::env::consts::EXE_SUFFIX)
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
        Ok(it) => String::from_utf8_lossy(&it.stdout).into_owned(),
        Err(_) => panic!("can't query installed crates from {CARGO}"),
    };

    // The indented lines under each package name are its executables, which
    // carry the platform suffix:
    //   cargo-about v0.9.2:
    //       cargo-about.exe
    let executable = exe_name(tool);
    install_list
        .lines()
        .filter(|it| {
            it.chars()
                .next()
                .map(|it| it.is_whitespace())
                .unwrap_or_default()
        })
        .any(|it| it.trim() == executable)
}

#[cfg(target_os = "windows")]
pub fn run_powershell<S: AsRef<str>>(
    code: impl IntoIterator<Item = S>,
) -> io::Result<proc::ExitStatus> {
    use io::Write;

    let mut ps = proc::Command::new("powershell")
        .args(["-NoProfile", "-Command", "-"])
        .stdin(proc::Stdio::piped())
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
    drop(stdin);
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
        Some(3) => Err(DownloadError::IO(io::Error::other(format!(
            "file I/O error: {}",
            path.display()
        )))),
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
        Some(23) => Err(DownloadError::IO(io::Error::other(format!(
            "file I/O error: {}",
            path.display()
        )))),
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

/// Escapes a value for a single-quoted PowerShell string, where `'` is the only
/// character with a meaning and is escaped by doubling it.
///
/// Input:  `C:\Ann's Files\sdk`
/// Output: `C:\Ann''s Files\sdk`
#[cfg(target_os = "windows")]
fn quote_powershell(value: &str) -> String {
    value.replace('\'', "''")
}

// The PowerShell fallback below ends in a `return`, leaving the final closure
// unreachable on Windows.
#[allow(unreachable_code)]
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
            return |url: &str, path: &Path| {
                // Single quotes so a '$' in either string isn't expanded, and an
                // explicit non-zero exit because a failed Invoke-WebRequest
                // otherwise leaves the shell itself exiting successfully.
                // Windows PowerShell renders a progress bar per chunk, which
                // costs more than the transfer on an SDK-sized download.
                let status = run_powershell([
                    "$ProgressPreference = 'SilentlyContinue'".to_string(),
                    format!(
                        "try {{ Invoke-WebRequest -Uri '{}' -OutFile '{}' -ErrorAction Stop }} \
                         catch {{ exit 1 }}",
                        quote_powershell(url),
                        quote_powershell(&path.display().to_string())
                    ),
                ])
                .map_err(DownloadError::IO)?;
                CommandError::from_exit(status)
                    .map_err(|err| DownloadError::CommandError(err.program("powershell")))
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

/// The `tar` to extract with.
///
/// On Windows this is deliberately the bsdtar in System32 addressed by
/// absolute path, never whatever `PATH` happens to resolve: Git for Windows,
/// MSYS2 and Cygwin all put a GNU tar in front of it, and GNU tar reads
/// neither the zip that typst ships for Windows nor an archive path that
/// starts with a drive letter, which it mistakes for a `host:path` remote.
#[cfg(target_os = "windows")]
fn tar_program() -> Option<OsString> {
    let system_root = std::env::var_os("SystemRoot")?;
    let bsdtar = Path::new(&system_root).join("System32").join("tar.exe");
    exists(&bsdtar).then(|| bsdtar.into_os_string())
}
#[cfg(not(target_os = "windows"))]
fn tar_program() -> Option<OsString> {
    has_command(TAR).then(|| OsString::from(TAR))
}

pub fn untar<S>(
    archive: impl AsRef<Path>,
    output: impl AsRef<Path>,
    args: impl IntoIterator<Item = S>,
) -> Result<(), CommandError>
where
    S: AsRef<OsStr>,
{
    let Some(tar) = tar_program() else {
        return Err(CommandError::missing_tool(
            TAR,
            Some("https://www.gnu.org/software/tar/"),
        ));
    };

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
    // No '-s': it means --preserve-order to GNU tar, which is meaningless for
    // these archives, while bsdtar reads it as a substitution expression and
    // consumes the following argument.
    let result = cmd(
        &tar,
        [
            OsStr::new("-xvf"),
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
            let executable_path = min_proto_path
                .join("target")
                .join(kind)
                .join(exe_name(WASI_STUB));
            if !exists(&executable_path) {
                return None;
            }
            let executable_path = executable_path
                .canonicalize()
                .expect("unable to canonicalize path that exists");
            Some(runner(executable_path.as_os_str()))
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

const TARGET_FEATURES_SECTION: &str = "target_features";

/// Lists the names of the custom sections of a WebAssembly module.
///
/// Only the top level section headers are walked. Section id `0` marks a
/// custom section, whose payload begins with its own length-prefixed name.
/// Returns [`None`] if `module` isn't a WebAssembly binary.
fn wasm_custom_sections(module: &[u8]) -> Option<Vec<String>> {
    fn leb128_u32(input: &mut &[u8]) -> Option<u32> {
        let mut result = 0;
        for shift in [0u32, 7, 14, 21, 28] {
            let (byte, rest) = input.split_first()?;
            *input = rest;
            result |= ((byte & 0x7f) as u32) << shift;
            if byte & 0x80 == 0 {
                return Some(result);
            }
        }
        None
    }

    // 4 byte magic number followed by a 4 byte version.
    let mut rest = module.strip_prefix(b"\0asm")?.get(4..)?;
    let mut names = Vec::new();
    while let Some((id, tail)) = rest.split_first() {
        rest = tail;
        let size = leb128_u32(&mut rest)? as usize;
        let mut payload = rest.get(..size)?;
        rest = rest.get(size..)?;
        if *id == 0 {
            let length = leb128_u32(&mut payload)? as usize;
            names.push(String::from_utf8_lossy(payload.get(..length)?).into_owned());
        }
    }
    Some(names)
}

pub const WASM_OPT: &str = "wasm-opt";
pub fn wasm_opt(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<(), CommandError> {
    if !exists(&input) {
        return Err(CommandError::file_not_found("input", &input).program(WASM_OPT));
    }

    // wasm-opt takes the set of allowed proposals from this section. Without it
    // the module is checked against wasm-opt's own defaults, which reject
    // anything the Rust toolchain enabled after that release of binaryen, with
    // an error that doesn't say so.
    let missing_features_section = std::fs::read(input.as_ref())
        .ok()
        .and_then(|module| wasm_custom_sections(&module))
        .is_some_and(|sections| !sections.iter().any(|it| it == TARGET_FEATURES_SECTION));
    if missing_features_section {
        return Err(CommandError::BadArgument {
            program: Some(WASM_OPT),
            argument: "input",
            expect_found: None,
            reason: Some(Box::new(io::Error::other(format!(
                "module has no '{}' section, so wasm-opt cannot know which \
                 WebAssembly proposals it may use; the plugin profile has to \
                 keep that section (`strip = \"debuginfo\"`, not `strip = true`)",
                TARGET_FEATURES_SECTION
            )))),
        });
    }
    if let Some(parent) = output.as_ref().parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            return Err(CommandError::inaccessible("output", err));
        }
    }

    let runner = make_runner!(Fn(input: &Path, output: &Path) -> Result<(), CommandError> {
        let tool_path = local_tool_path(exe_name(WASM_OPT));
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
                        // Enabled proposals are read from the module itself, so
                        // this list never has to track the Rust toolchain.
                        OsStr::new("--strip-producers"),
                        OsStr::new("--strip-target-features"),
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

#[cfg(ci = "github")]
macro_rules! typst_report {
    ($output: ident, $kind: literal) => {{
        let matches: Vec<_> = $output
            .lines()
            .filter(|it| it.starts_with(concat![$kind, ":"]))
            .map(|it| it.strip_prefix(concat![$kind, ":"]).unwrap().trim())
            .collect();
        let mut items = std::collections::BTreeMap::new();
        for item in matches {
            let count = items.entry(item).or_insert(0);
            *count += 1;
        }
        if !items.is_empty() {
            summary!("<details>");
            match $kind {
                "error" => summary!("  <summary><h4>🚨 {} Errors</h4></summary>\n", items.len()),
                "warning" => summary!(
                    "  <summary><h4>⚠️ {} Warnings</h4></summary>\n",
                    items.len()
                ),
                _ => summary!("  <summary><h4>{} {}</h4></summary>\n", items.len(), $kind),
            }

            for (item, count) in items {
                if count > 1 {
                    summary!("  - {} **\\[x{}]**", item, count);
                } else {
                    summary!("  - {}", item);
                }
            }
            summary!("\n</details>");
        }
    }};
}

pub const TYPST: &str = "typst";
pub fn typst_compile(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
) -> Result<(), CommandError> {
    if !exists(&input) {
        return Err(CommandError::file_not_found("input", &input).program(TYPST));
    }
    if let Some(parent) = output.as_ref().parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            return Err(CommandError::inaccessible("output", err));
        }
    }

    let runner = make_runner!(fn(input: &Path, output: &Path) -> Result<(proc::Output, std::time::Duration), CommandError> {
        if has_command(TYPST) {
            return |input, output| {
                let begin = std::time::Instant::now();
                let result = cmd(
                    TYPST,
                    [
                        OsStr::new("compile"),
                        OsStr::new("--creation-timestamp=0"),
                        input.as_os_str(),
                        output.as_os_str(),
                    ],
                )
                .program_output(TYPST);
                let duration = std::time::Instant::now() - begin;
                Ok((result, duration))
            }
        }

        if exists(local_tool_path(exe_name(TYPST))) {
            return |input, output| {
                let begin = std::time::Instant::now();
                let result = cmd(
                    local_tool_path(exe_name(TYPST)),
                    [
                        OsStr::new("compile"),
                        input.as_os_str(),
                        output.as_os_str(),
                    ],
                )
                .program_output(TYPST);
                let duration = std::time::Instant::now() - begin;
                Ok((result, duration))
            }
        }

        |_input, _output| {
            Err(CommandError::missing_tool(
                TYPST,
                Some("https://github.com/typst/typst/releases"),
            ))
        }
    });

    #[allow(unused_variables)]
    let (output, duration) = (runner)(input.as_ref(), output.as_ref())?;
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    #[cfg(ci = "github")]
    {
        summary!(
            "### Typst compiled '{}' in {}",
            input.as_ref().file_stem().unwrap().to_string_lossy(),
            DisplayDuration {
                duration,
                show_ms: true,
            }
        );

        typst_report!(stderr, "error");
        typst_report!(stderr, "warning");
    }
    group!("Typst compile: {}", input.as_ref().display());
    if !stderr.trim().is_empty() {
        if output.status.success() {
            info!(stderr)
        } else {
            error!(stderr)
        }
    } else if cfg!(ci) {
        info!("typst produced no output");
    }
    end_group!();

    CommandError::from_exit(output.status)
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

#[cfg(test)]
mod tests {
    use super::{
        exists, hash_files, wasm_custom_sections, CommandError, DownloadError, FileSize,
        TARGET_FEATURES_SECTION,
    };
    use crate::test_support::TempFile;

    /// Builds a module header followed by `sections`.
    fn module(sections: &[u8]) -> Vec<u8> {
        // "\0asm" magic number, then version 1.
        let mut result = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        result.extend_from_slice(sections);
        result
    }

    /// Encodes `value` as LEB128, the length encoding the format uses.
    ///
    /// 199 becomes `[0xc7, 0x01]`: 7 bits per byte, high bit marks "continues".
    fn leb128(mut value: u32) -> Vec<u8> {
        let mut result = Vec::new();
        loop {
            let byte = (value & 0x7f) as u8;
            value >>= 7;
            if value == 0 {
                result.push(byte);
                return result;
            }
            result.push(byte | 0x80);
        }
    }

    /// Builds a custom section (id 0) wrapping a length prefixed `name`.
    fn custom_section(name: &str) -> Vec<u8> {
        let mut payload = leb128(name.len() as u32);
        payload.extend_from_slice(name.as_bytes());
        let mut result = vec![0x00];
        result.extend_from_slice(&leb128(payload.len() as u32));
        result.extend_from_slice(&payload);
        result
    }

    #[test]
    fn finds_custom_sections_and_skips_known_ones() {
        let mut sections = custom_section(TARGET_FEATURES_SECTION);
        // A type section, which carries no name and must not be reported.
        sections.extend_from_slice(&[0x01, 0x01, 0x00]);
        sections.extend_from_slice(&custom_section("producers"));

        let found = wasm_custom_sections(&module(&sections)).expect("valid module");
        assert_eq!(found, [TARGET_FEATURES_SECTION, "producers"]);
    }

    #[test]
    fn reads_sections_past_a_multi_byte_length() {
        // 200 bytes of payload needs two LEB128 bytes to encode the length.
        let name = "x".repeat(198);
        let found = wasm_custom_sections(&module(&custom_section(&name))).expect("valid module");
        assert_eq!(found, [name]);
    }

    #[test]
    fn rejects_input_that_is_not_wasm() {
        assert!(wasm_custom_sections(b"not a wasm module at all").is_none());
        // Truncated section body.
        assert!(wasm_custom_sections(&module(&[0x00, 0x7f, 0x01])).is_none());
    }

    /// The plugin's size before and after optimisation is reported to the
    /// build log, so each step between the units is worth pinning.
    #[test]
    fn file_sizes_are_reported_in_the_largest_unit_that_fits() {
        for (bytes, expected) in [
            (0u64, "0 B"),
            (1, "1 B"),
            (1023, "1023 B"),
            (1024, "1.000 KiB"),
            (1536, "1.500 KiB"),
            (1024 * 1024 - 1, "1023.999 KiB"),
            (1024 * 1024, "1.000 MiB"),
            (1024 * 1024 * 1024, "1.000 GiB"),
        ] {
            assert_eq!(FileSize::from(bytes).to_string(), expected, "{bytes} bytes");
        }
    }

    #[test]
    fn a_file_size_converts_back_to_a_number_of_bytes() {
        assert_eq!(u64::from(FileSize::from(4096)), 4096);
        assert_eq!(usize::from(FileSize::from(4096)), 4096);
    }

    #[test]
    fn the_size_of_a_missing_file_is_an_error() {
        let missing = std::env::temp_dir().join("zint-wasi-xtask-no-such-file");
        let Err(error) = FileSize::of(missing) else {
            panic!("the file does not exist and has no size")
        };

        assert!(
            error.to_string().contains("file not found"),
            "unexpected error: {error}"
        );
    }

    /// The hash decides whether a build step can be skipped, so it has to
    /// depend on what the files contain and not on how they were listed.
    #[test]
    fn hashing_ignores_the_order_the_files_are_listed_in() {
        let first = TempFile::holding("one");
        let second = TempFile::holding("two");

        assert_eq!(
            hash_files([first.path(), second.path()]),
            hash_files([second.path(), first.path()])
        );
    }

    #[test]
    fn hashing_follows_the_contents_and_not_the_name() {
        let one = TempFile::holding("same contents");
        let copy = TempFile::holding("same contents");
        let different = TempFile::holding("other contents");

        assert_eq!(hash_files([one.path()]), hash_files([copy.path()]));
        assert_ne!(hash_files([one.path()]), hash_files([different.path()]));
    }

    /// The exit code is put into a `NonZeroI32` without checking, so the guard
    /// in front of that is what keeps it sound.
    #[test]
    #[should_panic(expected = "exit code 0 doesn't indicate an error")]
    fn a_successful_exit_is_not_an_error() {
        let _ = CommandError::new(0);
    }

    #[test]
    fn a_failure_names_the_program_it_came_from() {
        assert_eq!(
            CommandError::new(2).to_string(),
            "process exited with code #2"
        );
        assert_eq!(
            CommandError::new(2).program("wasm-opt").to_string(),
            "wasm-opt exited with code #2"
        );
        assert_eq!(
            CommandError::interrupt(9).program("typst").to_string(),
            "typst interrupted (signal: 9)"
        );
    }

    /// A missing tool is the one failure a contributor can do something about,
    /// so the message has to say where to get it.
    #[test]
    fn a_missing_tool_says_where_to_get_it() {
        let known = CommandError::missing_tool("cargo", Some("https://rustup.rs/")).to_string();
        assert!(known.starts_with("cargo is not in PATH"), "{known}");
        assert!(known.ends_with("'https://rustup.rs/'"), "{known}");

        let unknown = CommandError::missing_tool("wasm-opt", None).to_string();
        assert!(unknown.ends_with("a credible source."), "{unknown}");
    }

    /// A download failure is reported to whoever is building the plugin, so
    /// each of the three ways it can fail has to say something different.
    #[test]
    fn a_download_failure_says_which_way_it_failed() {
        let bad_url = DownloadError::BadUrl {
            url: "htp://example.invalid/wasi-sdk.tar.gz".to_string(),
        };
        assert_eq!(
            bad_url.to_string(),
            "can't resolve url: 'htp://example.invalid/wasi-sdk.tar.gz'"
        );

        let exited = DownloadError::CommandError(CommandError::new(8).program("wget"));
        assert_eq!(exited.to_string(), "wget exited with code #8");

        let io = DownloadError::IO(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "no write access",
        ));
        assert_eq!(io.to_string(), "io error: no write access");
    }

    /// Only the failures that wrap another error have one to point at.
    #[test]
    fn a_download_failure_points_at_the_error_underneath_it() {
        use std::error::Error;

        assert!(DownloadError::CommandError(CommandError::new(8))
            .source()
            .is_some());
        assert!(DownloadError::IO(std::io::Error::other("no write access"))
            .source()
            .is_some());
        assert!(DownloadError::BadUrl {
            url: "htp://example.invalid".to_string()
        }
        .source()
        .is_none());
    }

    /// A path that cannot be looked at counts as missing, because every caller
    /// treats the answer as "is this tool already here".
    #[test]
    fn a_path_that_cannot_be_read_counts_as_missing() {
        let file = TempFile::holding("");

        assert!(exists(file.path()));
        assert!(!exists(
            std::env::temp_dir().join("zint-wasi-xtask-no-such-path")
        ));
    }

    #[test]
    fn a_bad_argument_explains_what_was_wrong_with_it() {
        let error =
            CommandError::file_not_found("input", "/nowhere/plugin.wasm").program("wasm-opt");
        let message = error.to_string();

        assert!(message.contains("wasm-opt"), "{message}");
        assert!(message.contains("'input'"), "{message}");
        assert!(message.contains("/nowhere/plugin.wasm"), "{message}");
    }
}
