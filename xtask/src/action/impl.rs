use std::{ffi::OsStr, path::PathBuf};

use super::macros::*;
use super::ActionResult;

use crate::arguments::ArgumentList;
use crate::log::*;
use crate::state::{configure, GlobalState};
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

pub fn action_ensure_wasi_sdk(_args: &ArgumentList) -> ActionResult {
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

pub fn action_build_plugin(args: &ArgumentList) -> ActionResult {
    GlobalState::set_temporary(
        "BUILD_PROFILE",
        match args.has("debug") {
            true => "plugin-debug",
            false => "plugin-release",
        },
    );

    GlobalState::set_temporary("PREV_PLUGIN_WASM_HASH", state!(PLUGIN_WASM_HASH));
    GlobalState::set(
        "PLUGIN_WASM_HASH",
        hash_configured_paths!([
            "$<root>/zint-wasm-sys/src",
            "$<root>/zint-wasm-sys/build.rs",
            "$<root>/zint-wasm-rs/src",
            "$<root>/zint-typst-plugin/src",
        ])
        .to_string(),
    );
    action_expect!(cargo((
        "build",
        "--profile",
        state!(BUILD_PROFILE),
        "--target",
        state!(TARGET)
    )));
    action_ok!();
}

pub fn action_stub_plugin(args: &ArgumentList) -> ActionResult {
    let release = state_path!(PROJECT_ROOT)
        .join("target")
        .join(state!(TARGET))
        .join(state!(BUILD_PROFILE))
        .join(state!(PLUGIN_WASM));
    let stub_path = state_path!(WORK_DIR)
        .join(state!(TARGET))
        .join(state!(BUILD_PROFILE))
        .join(state!(PLUGIN_STUB_WASM, default: "plugin_stub.wasm"));

    let input_changed = state!(PREV_PLUGIN_WASM_HASH) != state!(PLUGIN_WASM_HASH);
    if !exists(&stub_path) || input_changed {
        group!("Stubbing '{}'", release.display());
        action_expect!(wasi_stub(release, &stub_path));
        end_group!();
    }

    // report stubbed file size because WASI module can't actually be ran by
    // typst, so this is the first "usable" module
    summary!(
        "- Compiled WASM size: {}",
        action_expect!(FileSize::of(&stub_path))
    );
    if args.has("debug") {
        let target_path =
            state_path!(TYPST_PKG).join(state!(PLUGIN_WASM_OUT, default: "plugin.wasm"));
        action_expect!(std::fs::copy(stub_path, target_path));
    }
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

pub fn action_prepare_wasm_opt(args: &ArgumentList) -> ActionResult {
    if args.has("debug") {
        action_skip!("building in debug mode");
    }
    if has_command(WASM_OPT) {
        action_skip!("{} already in PATH", WASM_OPT);
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
        action_expect!(std::fs::create_dir_all(&wasm_opt_dir));
        action_expect!(untar(
            binaryen_tar,
            wasm_opt_dir,
            (
                "--strip-components=2",
                format!(
                    "binaryen-version_{}/bin/{WASM_OPT}",
                    state!(BINARYEN_VERSION, default: "119")
                )
            )
        ));
    }
    action_ok!();
}

pub fn action_opt_plugin(args: &ArgumentList) -> ActionResult {
    if args.has("debug") {
        action_skip!("building in debug mode");
    }
    let base_path = state_path!(WORK_DIR)
        .join(state!(TARGET))
        .join(state!(BUILD_PROFILE));
    let stub_path = base_path.join(state!(PLUGIN_STUB_WASM, default: "plugin_stub.wasm"));
    let stub_opt_path =
        base_path.join(state!(PLUGIN_STUB_OPT_WASM, default: "plugin_stub_opt.wasm"));
    let target_path = state_path!(TYPST_PKG).join(state!(PLUGIN_WASM_OUT, default: "plugin.wasm"));

    let input_changed = state!(PREV_PLUGIN_WASM_HASH) != state!(PLUGIN_WASM_HASH);
    if !exists(&stub_opt_path) || input_changed {
        action_expect!(wasm_opt(stub_path, &stub_opt_path));
        action_expect!(std::fs::copy(stub_opt_path, &target_path));
    }
    summary!(
        "- Optimized WASM size: {}",
        action_expect!(FileSize::of(target_path))
    );
    action_ok!();
}

pub fn action_build_manual(_args: &ArgumentList) -> ActionResult {
    let manual_source = state_path!(TYPST_MANUAL_SOURCE, default: "$<root>/dist/manual.typ");
    let manual_target = state_path!(TYPST_PKG).join("manual.pdf");

    let create_time = action_expect!(last_edit(&manual_source));
    action_expect!(typst_compile(
        &manual_source,
        &manual_target,
        (
            "--root",
            state!(PROJECT_ROOT),
            "--ignore-system-fonts",
            "--font-path",
            configure!("$<root>/dist/fonts"),
            "--creation-timestamp",
            create_time.to_string(),
        )
    ));
    GlobalState::set("MANUAL_CHANGE_TIME", create_time.to_string());

    action_ok!();
}

pub fn action_build_example(_args: &ArgumentList) -> ActionResult {
    let example_source = state_path!(TYPST_EXAMPLE_SOURCE, default: "$<root>/dist/example.typ");
    let example_target = state_path!(TYPST_PKG).join("example.svg");

    let create_time = action_expect!(last_edit(&example_source));
    action_expect!(typst_compile(
        example_source,
        example_target,
        (
            "--root",
            state!(PROJECT_ROOT),
            "--ignore-system-fonts",
            "--font-path",
            configure!("$<root>/dist/fonts"),
            "--creation-timestamp",
            create_time.to_string(),
        )
    ));
    action_ok!();
}

pub fn action_ensure_cargo_about(_args: &ArgumentList) -> ActionResult {
    if !cargo_has_tool("cargo-about") {
        action_expect!(cargo(["install", "cargo-about"]));
    }
    action_ok!();
}

pub fn action_make_3rdparty_license_list(_args: &ArgumentList) -> ActionResult {
    let about_input =
        state_path!(THIRDPARTY_LICENSE_PATH, default: "$<root>/dist/3rdparty_license.hbs");
    let output = cargo([
        OsStr::new("about"),
        OsStr::new("generate"),
        about_input.as_os_str(),
    ]);
    let output = action_expect!(action_expect!(output).output());
    let generated = action_expect!(std::str::from_utf8(&output.stdout));

    let about_output_file = state_path!(TYPST_PKG).join("3rdparty_license.html");
    action_expect!(std::fs::write(about_output_file, generated));
    action_ok!();
}

pub fn action_copy_license(_args: &ArgumentList) -> ActionResult {
    let source_path = state_path!(LICENSE_FILE, default: "$<root>/LICENSE");
    let target_path = state_path!(TYPST_PKG).join("LICENSE");
    action_expect!(std::fs::copy(source_path, target_path));
    action_ok!();
}

#[allow(unreachable_code)]
fn typst_url(version: impl AsRef<str>) -> (String, &'static str, &'static str) {
    let version = version.as_ref();
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    return (format!("https://github.com/typst/typst/releases/download/v{version}/typst-aarch64-unknown-linux-musl.tar.xz"), "typst-aarch64-unknown-linux-musl", "tar.xz");
    #[cfg(all(target_os = "linux", target_arch = "arm"))]
    return (format!("https://github.com/typst/typst/releases/download/v{version}/typst-armv7-unknown-linux-musleabi.tar.xz"), "typst-armv7-unknown-linux-musleabi", "tar.xz");
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return (format!("https://github.com/typst/typst/releases/download/v{version}/typst-x86_64-unknown-linux-musl.tar.xz "), "typst-x86_64-unknown-linux-musl", "tar.xz");
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return (format!("https://github.com/typst/typst/releases/download/v{version}/typst-aarch64-apple-darwin.tar.xz"), "typst-aarch64-apple-darwin", "tar.xz");
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return (format!("https://github.com/typst/typst/releases/download/v{version}/typst-x86_64-apple-darwin.tar.xz"), "typst-x86_64-apple-darwin", "tar.xz");
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return (format!("https://github.com/typst/typst/releases/download/v{version}/typst-x86_64-pc-windows-msvc.zip"), "typst-x86_64-pc-windows-msvc", "zip");
    panic!("no prebuild binaryen available for current platform")
}

// should be only used for CI
pub fn action_install_typst(_args: &ArgumentList) -> ActionResult {
    if has_command(TYPST) {
        action_skip!("{} already in PATH", TYPST);
    }

    let (url, base_dir, ext) = typst_url(state!(TYPST_VERSION));
    let work_dir = state_path!(WORK_DIR);
    let typst_archive = work_dir.join(format!("typst.{ext}"));
    let typst_dir = work_dir.join("tools");
    let typst_bin = typst_dir.join(TYPST);

    if !exists(typst_bin) {
        if !exists(&typst_archive) {
            action_expect!(download(url, &typst_archive));
        }
        action_expect!(std::fs::create_dir_all(&typst_dir));
        action_expect!(untar(
            typst_archive,
            typst_dir,
            [
                "--strip-components=1".to_string(),
                format!("{base_dir}/{TYPST}",)
            ]
        ));
    }

    action_ok!();
}
