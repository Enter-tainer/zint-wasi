use std::path::{Path, PathBuf};

use crate::action::macros::*;
use crate::action::ActionResult;
use crate::tools::*;
use crate::{state, state_path};

const WASI_PATH_VAR: &str = "WASI_SDK_PATH";

fn has_wasi_sdk() -> bool {
    match std::env::var(WASI_PATH_VAR) {
        Ok(it) => exists(it),
        Err(_) => false,
    }
}

#[allow(unreachable_code)]
fn wasi_url(version: impl AsRef<str>) -> String {
    let version = version.as_ref();
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return format!("https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-{version}/wasi-sdk-{version}.0-arm64-linux.tar.gz");
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return format!("https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-{version}/wasi-sdk-{version}.0-x86_64-linux.tar.gz");
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return format!("https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-{version}/wasi-sdk-{version}.0-arm64-macos.tar.gz");
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return format!("https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-{version}/wasi-sdk-{version}.0-x86_64-macos.tar.gz");
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return format!("https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-{version}/wasi-sdk-{version}.0-x86_64-windows.tar.gz");
    panic!("no prebuild WASI SDK available for current platform; please build and specify `WASI_SDK_PATH` environment variable manually")
}

pub fn action_ensure_wasi_sdk() -> ActionResult {
    if has_wasi_sdk() {
        action_skip!("WASI SDK is set with environment variable");
    }

    let work_dir = state_path!(WORK_DIR);
    let download_path = work_dir.join("wasi_sdk.tar.gz");
    let extract_path = match std::env::var(WASI_PATH_VAR) {
        Ok(it) if !it.is_empty() => PathBuf::from(it),
        _ => work_dir.join("wasi_sdk"),
    };

    if !exists(&extract_path) {
        if !exists(&download_path) {
            let url = wasi_url(state!(WASI_SDK_VERSION, default: "24"));
            action_expect!(download(url, &download_path));
        }

        let _ = std::fs::create_dir_all(&extract_path);
        action_expect!(untar(
            download_path,
            &extract_path,
            ["--strip-components=1"]
        ));
    }

    let wasi_sdk_path = action_expect!(extract_path.canonicalize());

    unsafe {
        std::env::set_var(WASI_PATH_VAR, wasi_sdk_path);
    }

    action_ok!();
}

pub fn action_build_plugin() -> ActionResult {
    action_expect_0!(cargo(["build", "--release", "--target", "wasm32-wasip1"]));
    action_ok!();
}

pub fn action_stub_plugin() -> ActionResult {
    let base_path = state_path!(WORK_DIR).join("release");
    let release = base_path.join(state!(PLUGIN_WASM));
    let stubbed = base_path.join(state!(PLUGIN_STUB_WASM, default: "plugin_stub.wasm"));
    action_expect!(wasi_stub(release, stubbed));
    action_ok!();
}

#[allow(unreachable_code)]
fn binaryen_url(version: impl AsRef<str>) -> String {
    let version = version.as_ref();
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return format!("https://github.com/WebAssembly/binaryen/releases/download/version_{version}/binaryen-version_{version}-arm64-linux.tar.gz");
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return format!("https://github.com/WebAssembly/binaryen/releases/download/version_{version}/binaryen-version_{version}-x86_64-linux.tar.gz");
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return format!("https://github.com/WebAssembly/binaryen/releases/download/version_{version}/binaryen-version_{version}-arm64-macos.tar.gz");
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return format!("https://github.com/WebAssembly/binaryen/releases/download/version_{version}/binaryen-version_{version}-x86_64-macos.tar.gz");
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return format!("https://github.com/WebAssembly/binaryen/releases/download/version_{version}/binaryen-version_{version}-x86_64-windows.tar.gz");
    panic!("no prebuild binaryen available for current platform")
}

pub fn action_prepare_wasm_opt() -> ActionResult {
    if has_command(WASM_OPT) {
        action_ok!();
    }

    let work_dir = state_path!(WORK_DIR);
    let binaryen_tar = work_dir.join("binaryen.tar.gz");
    let wasm_opt_dir = work_dir.join("tools");
    let wasm_opt_bin = wasm_opt_dir.join(WASM_OPT);
    if !exists(wasm_opt_bin) {
        if !exists(&binaryen_tar) {
            action_expect!(download(
                binaryen_url(state!(BINARYEN_VERSION, default: "119")),
                &binaryen_tar
            ));
        }
        action_expect!(untar(
            binaryen_tar,
            wasm_opt_dir,
            [
                "--strip-components".to_string(),
                format!(
                    "/binaryen-version_{}/bin/{WASM_OPT}",
                    state!(BINARYEN_VERSION, default: "119")
                )
            ]
        ));
    }
    action_ok!();
}

pub fn action_opt_plugin() -> ActionResult {
    let base_path = state_path!(WORK_DIR).join("release");
    let stub_path = base_path.join(state!(PLUGIN_STUB_WASM, default: "plugin_stub.wasm"));
    let stub_opt_path = base_path.join(state!(PLUGIN_STUB_OPT_WASM, default: "plugin_stub_opt.wasm"));
    action_expect!(wasm_opt(stub_path, &stub_opt_path));
    let target_path = Path::new(state!(TYPST_PKG)).join(state!(PLUGIN_WASM_OUT, default: "plugin.wasm"));
    action_expect!(std::fs::copy(stub_opt_path, target_path));
    action_ok!();
}

pub fn action_copy_license() -> ActionResult {
    let source_path = state_path!(LICENSE_FILE, default: "./LICENSE");
    let target_path = Path::new(state!(TYPST_PKG)).join("LICENSE");
    action_expect!(std::fs::copy(source_path, target_path));
    action_ok!();
}