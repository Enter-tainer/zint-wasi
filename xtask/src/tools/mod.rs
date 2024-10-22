use std::{
    collections::{BTreeSet, HashMap},
    ffi::{OsStr, OsString},
    fmt::{Display, Write},
    hash::{Hash, Hasher},
    io::{self, Read, Seek},
    mem::MaybeUninit,
    path::{Path, PathBuf},
    process::{Command, Output as ProgramOutput},
    str::FromStr,
    sync::OnceLock,
    time::SystemTime,
};

use crate::arguments::*;
use crate::log::*;
use crate::state_path;

mod error;
pub mod name;

pub use error::*;
pub use name::*;

include!("macro.rs");

#[inline(always)]
pub fn exists(path: impl AsRef<Path>) -> bool {
    std::fs::exists(path.as_ref()).ok().unwrap_or_default()
}
pub fn local_tool_path(name: impl AsRef<Path>) -> Option<PathBuf> {
    let tools_dir = state_path!(WORK_DIR).join("tools").join(name);
    if exists(&tools_dir) {
        Some(tools_dir)
    } else {
        local_tool_path_in_project(state_path!(PROJECT_ROOT))
    }
}
pub fn local_tool_path_in_project(project: impl AsRef<Path>) -> Option<PathBuf> {
    let project = project.as_ref();
    let check = |kind: &str| {
        let executable_path = project.join("target").join(kind).join(WASI_STUB);
        if !exists(&executable_path) {
            return None;
        }
        let executable_path = executable_path
            .canonicalize()
            .expect("unable to canonicalize path that exists");
        Some(executable_path)
    };
    check("release").or(check("debug"))
}

pub fn cmd(program: impl AsRef<OsStr>, args: impl ArgList) -> Command {
    let mut result = Command::new(program.as_ref());
    result.args(args.os_string_args());
    result
}

pub fn git(command: impl AsRef<OsStr>, args: impl ArgList) -> Command {
    cmd(GIT, (command.as_ref(), args))
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

pub fn cargo(args: impl ArgList) -> Result<Command, CommandError> {
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

    let install_list = cmd(CARGO, ["install", "--list"]).program_stdout();
    let install_list = match install_list {
        Ok(it) => it.to_string_lossy().into_owned(),
        Err(err) => panic!("can't query installed crates: {err}"),
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
pub fn run_powershell<S: AsRef<str>>(code: impl IntoIterator<Item = S>) -> io::Result<ExitStatus> {
    use std::io::Write;
    // not tested
    let mut ps = cmd("powershell", ["-Command", "-"])
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

mod download {
    use super::*;

    fn download_wget(url: &str, path: &Path) -> Result<(), DownloadError> {
        cmd(WGET, (url, "-q", "--show-progress", "-O", path))
            .program_status()
            .map_exit_codes(|code| match code {
                // https://www.gnu.org/software/wget/manual/html_node/Exit-Status.html
                3 => Some(CommandError::other(io::Error::new(
                    io::ErrorKind::Other,
                    format!("file I/O error: {}", path.display()),
                ))),
                4 => Some(CommandError::other(DownloadError::BadUrl {
                    url: url.to_string(),
                })),
                _ => None,
            })
            .map(|_| ())
            .map_err(DownloadError::from)
    }
    fn download_curl(url: &str, path: &Path) -> Result<(), DownloadError> {
        cmd(CURL, ("-L", url, "--output", path.as_os_str()))
            .program_status()
            .map_exit_codes(|code| match code {
                // https://everything.curl.dev/cmdline/exitcode.html
                3 | 5 | 6 | 7 => Some(CommandError::other(DownloadError::BadUrl {
                    url: url.to_string(),
                })),
                23 => Some(CommandError::other(io::Error::new(
                    io::ErrorKind::Other,
                    format!("file I/O error: {}", path.display()),
                ))),
                _ => None,
            })
            .map(|_| ())
            .map_err(DownloadError::from)
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
}
pub use download::download;

pub fn untar(
    archive: impl AsRef<Path>,
    output: impl AsRef<Path>,
    args: impl ArgList,
) -> Result<(), CommandError> {
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
    let args = args.into_args();
    let result = cmd(
        TAR,
        ("-xvsf", archive.as_ref(), "-C", output.as_ref()).chain_args(args),
    )
    .program_status();
    end_group!();
    result
}

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
                    (
                        "-r",
                        "0",
                        file,
                        "-o",
                        output,
                    ),
                )
                .program_status()
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
            cargo((
                "run",
                "--manifest-path",
                min_proto_path,
                "--bin=wasi-stub",
                "--release",
                "--",
                "-r",
                "0",
                file,
                "-o",
                output,
            ))?.program_status()
        })
    });
    (runner)(input.as_ref(), output.as_ref()).map_err(|err| err.program(WASI_STUB))
}

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
        let command = if has_command(WASM_OPT) {
            Some(OsString::from_str(WASM_OPT).unwrap())
        } else {
            local_tool_path(WASM_OPT)
                .map(|tool| tool.as_os_str().to_owned())
        };
        if let Some(command) = command {
            return Box::new(move |file: &Path, output: &Path| {
                cmd(
                    &command,
                    (
                        file,
                        "-O3",
                        "--enable-bulk-memory",
                        "-o",
                        output,
                    ),
                )
                .program_status()
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

//#[cfg(ci = "github")]
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

pub fn typst_compile(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    options: impl ArgList,
) -> Result<(), CommandError> {
    if !exists(&input) {
        return Err(CommandError::file_not_found("input", &input).program(TYPST));
    }
    if let Some(parent) = output.as_ref().parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            return Err(CommandError::inaccessible("output", err));
        }
    }

    let runner = make_runner!(Fn(
        input: &Path,
        output: &Path,
        options: ArgumentList,
    ) -> Result<(Result<ProgramOutput, CommandError>, std::time::Duration), CommandError>
    {
        if has_command(TYPST) {
            return Box::new(|input, output, options| {
                let begin = std::time::Instant::now();
                let result = cmd(
                    TYPST,
                    (
                        "compile",
                        options,
                        input,
                        output
                    )
                )
                .output().map_err(|_|
                    CommandError::missing_tool(
                        TYPST,
                        Some("https://github.com/typst/typst/releases"),
                    )
                );
                let duration = std::time::Instant::now() - begin;
                Ok((result, duration))
            })
        }

        if let Some(typst) = local_tool_path(TYPST) {
            return Box::new(move |input, output, options| {
                let begin = std::time::Instant::now();
                let result = cmd(
                    &typst,
                    (
                        "compile",
                        options,
                        input,
                        output
                    )
                )
                .output().map_err(|_|
                    CommandError::missing_tool(
                        TYPST,
                        Some("https://github.com/typst/typst/releases"),
                    )
                );
                let duration = std::time::Instant::now() - begin;
                Ok((result, duration))
            })
        }

        Box::new(|_input, _output, _options| {
            Err(CommandError::missing_tool(
                TYPST,
                Some("https://github.com/typst/typst/releases"),
            ))
        })
    });

    #[allow(unused_variables)]
    let (output, duration) = (runner)(input.as_ref(), output.as_ref(), options.into_args())?;
    let output = output?;
    let stderr = unsafe {
        OsString::from_encoded_bytes_unchecked(output.stderr)
            .to_string_lossy()
            .to_string()
    };

    //[cfg(ci = "github")]
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

    CommandError::from_exit_status(&output.status)
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

pub fn last_edit(file: impl AsRef<Path>) -> io::Result<u128> {
    let metadata = std::fs::metadata(file.as_ref())?;

    let status = GitStatus::index_and_working_tree(&file)?;
    let commit_time = GitStatus::has_pending_changes(status);
    let commit_time = match !commit_time {
        true => {
            let commit_time = git("log", ("-1", "--pretty=\"format:%ct\"", file.as_ref()))
                .program_stdout()?
                .to_string_lossy()
                .to_string();
            if !commit_time.is_empty() {
                commit_time.parse::<u128>().ok()
            } else {
                None
            }
        }
        false => None,
    };

    match commit_time {
        Some(it) => Ok(it),
        None => metadata.modified().and_then(|time| {
            time.duration_since(SystemTime::UNIX_EPOCH)
                .map(|tse| tse.as_millis())
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
        }),
    }
}

#[allow(dead_code)]
trait ProgramExt {
    fn program_status(self) -> Result<(), CommandError>;
    fn program_output(self) -> Result<ProgramOutput, CommandError>;
    fn program_stdout(self) -> Result<OsString, CommandError>;
    fn program_stderr(self) -> Result<OsString, CommandError>;
}
impl ProgramExt for Command {
    fn program_status(mut self) -> Result<(), CommandError> {
        let path = PathBuf::from(self.get_program());
        let mut result = match self.status() {
            Ok(it) => CommandError::from_exit_status(&it),
            Err(_) => panic!("unable to run '{}'", self.get_program().to_string_lossy()),
        };
        if let Some(name) = path.file_name() {
            result = result.map_err(|err| err.program(name.to_string_lossy().to_string().leak()));
        }
        result
    }
    fn program_output(mut self) -> Result<ProgramOutput, CommandError> {
        let path = PathBuf::from(self.get_program());
        let mut result = match self.output() {
            Ok(it) if it.status.success() => return Ok(it),
            Ok(it) => CommandError::from_exit_status(&it.status).map(|_| it),
            Err(_) => panic!("unable to run '{}'", self.get_program().to_string_lossy()),
        };
        if let Some(name) = path.file_name() {
            result = result.map_err(|err| err.program(name.to_string_lossy().to_string().leak()));
        }
        result
    }
    fn program_stdout(self) -> Result<OsString, CommandError> {
        self.program_output()
            .map(|ok| unsafe { OsString::from_encoded_bytes_unchecked(ok.stdout) })
    }
    fn program_stderr(self) -> Result<OsString, CommandError> {
        self.program_output()
            .map(|ok| unsafe { OsString::from_encoded_bytes_unchecked(ok.stderr) })
    }
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GitStatus {
    /// unmodified
    #[default]
    Unmodified,
    /// modified
    Modified,
    /// file type changed (regular file, symbolic link or submodule)
    TypeChanged,
    /// added
    Added,
    /// deleted
    Deleted,
    /// renamed
    Renamed,
    /// copied (if config option status.renames is set to "copies")
    Copied,
    /// updated but unmerged
    Updated,
    /// untracked
    Untracked,
    /// ignored
    Ignored,
}

impl GitStatus {
    fn from_char(c: char) -> Option<Self> {
        Some(match c {
            ' ' => Self::Unmodified,
            'M' => Self::Modified,
            'T' => Self::TypeChanged,
            'A' => Self::Added,
            'D' => Self::Deleted,
            'R' => Self::Renamed,
            'C' => Self::Copied,
            'U' => Self::Updated,
            '?' => Self::Untracked,
            '!' => Self::Ignored,
            _ => return None,
        })
    }

    pub fn index_and_working_tree(file: impl AsRef<Path>) -> io::Result<(Self, Self)> {
        if file.as_ref().is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("'{}' is directory", file.as_ref().display()),
            ));
        }
        let status = git("status", ("--ignored", "--porcelain", file.as_ref()))
            .program_stdout()?
            .to_string_lossy()
            .to_string();

        if status.trim().is_empty() {
            if !file.as_ref().exists() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("file '{}' doesn't exist", file.as_ref().display()),
                ));
            }

            return Ok((Self::default(), Self::default()));
        }
        Ok((
            status
                .chars()
                .nth(0)
                .and_then(Self::from_char)
                .unwrap_or_default(),
            status
                .chars()
                .nth(1)
                .and_then(Self::from_char)
                .unwrap_or_default(),
        ))
    }

    #[inline]
    pub fn index(file: impl AsRef<Path>) -> io::Result<Self> {
        Self::index_and_working_tree(file).map(|(index, _)| index)
    }

    #[inline]
    pub fn working_tree(file: impl AsRef<Path>) -> io::Result<Self> {
        Self::index_and_working_tree(file).map(|(_, wt)| wt)
    }

    /// Returns `true` if file has local changes.
    ///
    /// This function is used by [`last_edit`] to see whether local or remote
    /// metadata should be used to infer file edit time.
    fn has_pending_changes(status: (GitStatus, GitStatus)) -> bool {
        match status {
            (_, GitStatus::Modified)
            | (GitStatus::Modified, _)
            | (_, GitStatus::Updated)
            | (GitStatus::Updated, _)
            | (_, GitStatus::Added)
            | (GitStatus::Added, _)
            | (GitStatus::TypeChanged, _)
            | (_, GitStatus::TypeChanged) => {
                // File was modified or created, so local changes are newer.
                // If type changed then any status info tracked by git is invalid.
                true
            }
            (_, GitStatus::Untracked) => {
                // File was created
                true
            }
            (GitStatus::Ignored, _) | (_, GitStatus::Ignored) => {
                // File isn't tracked, only system info is available
                true
            }
            (GitStatus::Deleted, GitStatus::Renamed)
            | (GitStatus::Deleted, GitStatus::Copied)
            | (_, GitStatus::Renamed)
            | (GitStatus::Renamed, _) => {
                // Different file, so local changes are newer
                true
            }
            (GitStatus::Deleted, _) | (_, GitStatus::Deleted) => {
                // File no longer exists
                false
            }
            (_, GitStatus::Unmodified) | (GitStatus::Unmodified, _) => false,
            (index, working_tree) => unreachable!(
                "GitStatus::has_pending_changes not implemented for {}{}",
                index, working_tree
            ),
        }
    }
}

impl From<GitStatus> for char {
    fn from(value: GitStatus) -> char {
        match value {
            GitStatus::Unmodified => ' ',
            GitStatus::Modified => 'M',
            GitStatus::TypeChanged => 'T',
            GitStatus::Added => 'A',
            GitStatus::Deleted => 'D',
            GitStatus::Renamed => 'R',
            GitStatus::Copied => 'C',
            GitStatus::Updated => 'U',
            GitStatus::Untracked => '?',
            GitStatus::Ignored => '!',
        }
    }
}
impl Display for GitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char((*self).into())
    }
}
