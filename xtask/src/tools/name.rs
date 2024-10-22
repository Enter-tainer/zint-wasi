pub const GIT: &str = "git";
pub const CARGO: &str = "cargo";
pub const TAR: &str = "tar";

#[cfg(not(target_os = "windows"))]
pub const WGET: &str = "wget";
#[cfg(target_os = "windows")]
pub const WGET: &str = "wget.exe";
#[cfg(not(target_os = "windows"))]
pub const CURL: &str = "curl";
#[cfg(target_os = "windows")]
pub const CURL: &str = "curl.exe";

pub const WASI_STUB: &str = "wasi-stub";
pub const WASM_OPT: &str = "wasm-opt";

pub const TYPST: &str = "typst";
