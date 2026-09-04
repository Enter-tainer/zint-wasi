use std::{
    env::consts::{ARCH, OS},
    ffi::OsStr,
    path::PathBuf,
};

use super::*;
use crate::state::GlobalState;
use crate::tools::*;
use crate::{state, state_path};

const WASI_PATH_VAR: &str = "WASI_SDK_PATH";

fn has_wasi_sdk() -> bool {
    match std::env::var(WASI_PATH_VAR) {
        Ok(it) => exists(it),
        Err(_) => false,
    }
}

/// URL of the prebuilt WASI SDK release for a platform, as reported by
/// [`std::env::consts`].
///
/// Input:  ("linux", "x86_64", "24")
/// Output: ".../wasi-sdk-24/wasi-sdk-24.0-x86_64-linux.tar.gz"
fn wasi_url(os: &str, arch: &str, version: &str) -> Option<String> {
    let platform = match (os, arch) {
        ("linux", "aarch64") => "arm64-linux",
        ("linux", "x86_64") => "x86_64-linux",
        ("macos", "aarch64") => "arm64-macos",
        ("macos", "x86_64") => "x86_64-macos",
        ("windows", "x86_64") => "x86_64-windows",
        _ => return None,
    };
    Some(format!("https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-{version}/wasi-sdk-{version}.0-{platform}.tar.gz"))
}

pub fn action_ensure_wasi_sdk(_args: &[String]) -> ActionResult {
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
            let url = match wasi_url(OS, ARCH, &state!(WASI_SDK_VERSION, default: "24")) {
                Some(it) => it,
                None => action_error!(std::io::Error::other(format!(
                    "no prebuilt WASI SDK for {OS}-{ARCH}; build one and set {WASI_PATH_VAR}"
                ))),
            };
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

pub fn action_build_plugin(args: &[String]) -> ActionResult {
    GlobalState::set_temporary(
        "BUILD_PROFILE",
        match args.contains(&"--debug".to_string()) {
            true => "plugin-debug",
            false => "plugin-release",
        },
    );

    action_expect!(cargo([
        "build".to_string(),
        "--profile".to_string(),
        state!(BUILD_PROFILE),
        "--target".to_string(),
        state!(TARGET)
    ]));
    action_ok!();
}

pub fn action_stub_plugin(args: &[String]) -> ActionResult {
    let release = state_path!(PROJECT_ROOT)
        .join("target")
        .join(state!(TARGET))
        .join(state!(BUILD_PROFILE))
        .join(state!(PLUGIN_WASM));
    let stub_path = state_path!(WORK_DIR)
        .join(state!(TARGET))
        .join(state!(BUILD_PROFILE))
        .join(state!(PLUGIN_STUB_WASM, default: "plugin_stub.wasm"));

    let input_changed = did_files_change!([
        "$<root>/zint-wasm-sys/src",
        "$<root>/zint-wasm-sys/build.rs",
        "$<root>/zint-wasm-rs/src",
        "$<root>/zint-typst-plugin/src",
    ] as PLUGIN_WASM_HASH);
    if !exists(&stub_path) || input_changed {
        group!("Stubbing '{}'", release.display());
        action_expect!(wasi_stub(release, &stub_path));
        end_group!();

        if input_changed {
            GlobalState::set("PLUGIN_WASM_STUB_HASH", state!(PLUGIN_WASM_HASH));
        }
    }

    // report stubbed file size because WASI module can't actually be ran by
    // typst, so this is the first "usable" module
    summary!(
        "- Compiled WASM size: {}",
        action_expect!(FileSize::of(&stub_path))
    );
    if args.contains(&"--debug".to_string()) {
        let target_path =
            state_path!(TYPST_PKG).join(state!(PLUGIN_WASM_OUT, default: "plugin.wasm"));
        action_expect!(std::fs::copy(stub_path, target_path));
    }
    action_ok!();
}

/// URL of the prebuilt binaryen release for a platform, as reported by
/// [`std::env::consts`].
///
/// Input:  ("macos", "aarch64", "119")
/// Output: ".../version_119/binaryen-version_119-arm64-macos.tar.gz"
fn binaryen_url(os: &str, arch: &str, version: &str) -> Option<String> {
    let platform = match (os, arch) {
        ("linux", "aarch64") => "aarch64-linux",
        ("linux", "x86_64") => "x86_64-linux",
        ("macos", "aarch64") => "arm64-macos",
        ("macos", "x86_64") => "x86_64-macos",
        ("windows", "x86_64") => "x86_64-windows",
        _ => return None,
    };
    Some(format!("https://github.com/WebAssembly/binaryen/releases/download/version_{version}/binaryen-version_{version}-{platform}.tar.gz"))
}

pub fn action_prepare_wasm_opt(args: &[String]) -> ActionResult {
    if args.contains(&"--debug".to_string()) {
        action_skip!("building in debug mode");
    }
    if has_command(WASM_OPT) {
        action_skip!("{} already in PATH", WASM_OPT);
    }

    let work_dir = state_path!(WORK_DIR);
    let binaryen_tar = work_dir.join("binaryen.tar.gz");
    let wasm_opt_dir = work_dir.join("tools");
    let wasm_opt_bin = wasm_opt_dir.join(exe_name(WASM_OPT));
    if !exists(wasm_opt_bin) {
        if !exists(&binaryen_tar) {
            let url = match binaryen_url(OS, ARCH, &state!(BINARYEN_VERSION, default: "119")) {
                Some(it) => it,
                None => action_error!(std::io::Error::other(format!(
                    "no prebuilt binaryen for {OS}-{ARCH}"
                ))),
            };
            action_expect!(download(url, &binaryen_tar));
        }
        action_expect!(std::fs::create_dir_all(&wasm_opt_dir));
        action_expect!(untar(
            binaryen_tar,
            wasm_opt_dir,
            [
                "--strip-components=2".to_string(),
                format!(
                    "binaryen-version_{}/bin/{}",
                    state!(BINARYEN_VERSION, default: "119"),
                    exe_name(WASM_OPT)
                )
            ]
        ));
    }
    action_ok!();
}

pub fn action_opt_plugin(args: &[String]) -> ActionResult {
    if args.contains(&"--debug".to_string()) {
        action_skip!("building in debug mode");
    }
    let base_path = state_path!(WORK_DIR)
        .join(state!(TARGET))
        .join(state!(BUILD_PROFILE));
    let stub_path = base_path.join(state!(PLUGIN_STUB_WASM, default: "plugin_stub.wasm"));
    let stub_opt_path =
        base_path.join(state!(PLUGIN_STUB_OPT_WASM, default: "plugin_stub_opt.wasm"));
    let target_path = state_path!(TYPST_PKG).join(state!(PLUGIN_WASM_OUT, default: "plugin.wasm"));

    let stub_hash = state!(PLUGIN_WASM_STUB_HASH, default: "");
    let input_changed = state!(PLUGIN_WASM_HASH) != stub_hash;
    if !exists(&stub_opt_path) || input_changed {
        action_expect!(wasm_opt(stub_path, &stub_opt_path));
        action_expect!(std::fs::copy(stub_opt_path, &target_path));
    }
    GlobalState::set("PLUGIN_WASM_HASH", stub_hash);
    summary!(
        "- Optimized WASM size: {}",
        action_expect!(FileSize::of(target_path))
    );
    action_ok!();
}

pub fn action_build_manual(_args: &[String]) -> ActionResult {
    let manual_source = state_path!(MANUAL_SOURCE, default: || {
        state_path!(TYPST_PKG).join("manual.typ").to_string_lossy().to_string()
    });
    let manual_target = state_path!(TYPST_PKG).join("manual.pdf");
    action_expect!(typst_compile(&manual_source, &manual_target));

    action_ok!();
}

pub fn action_build_example(_args: &[String]) -> ActionResult {
    let example_source = state_path!(MANUAL_SOURCE, default: || {
        state_path!(TYPST_PKG).join("example.typ").to_string_lossy().to_string()
    });
    let example_target = state_path!(TYPST_PKG).join("example.svg");
    action_expect!(typst_compile(example_source, example_target));
    action_ok!();
}

pub fn action_ensure_cargo_about(_args: &[String]) -> ActionResult {
    if !cargo_has_tool("cargo-about") {
        // The binary sits behind a feature; without it the install reports
        // success while producing nothing to run.
        action_expect!(cargo(["install", "cargo-about", "--features", "cli"]));
    }
    action_ok!();
}

pub fn action_make_3rdparty_license_list(_args: &[String]) -> ActionResult {
    let about_input =
        state_path!(THIRDPARTY_LICENSE_PATH, default: "$<root>/dist/3rdparty_license.hbs");
    let about_output_file = state_path!(TYPST_PKG).join("3rdparty_license.html");
    // Let cargo-about write the file itself: it refuses to run at all when its
    // stdout is redirected under PowerShell, and capturing it gained nothing.
    let mut command = action_expect!(cargo([
        OsStr::new("about"),
        OsStr::new("generate"),
        OsStr::new("--output-file"),
        about_output_file.as_os_str(),
        about_input.as_os_str(),
    ]));
    let status = action_expect!(command.status());
    // cargo-about reports failures on stderr, so without this the package
    // ships an empty license list.
    action_expect!(CommandError::from_exit(status).map_err(|err| err.program("cargo about")));
    action_ok!();
}

pub fn action_copy_license(_args: &[String]) -> ActionResult {
    let source_path = state_path!(LICENSE_FILE, default: "$<root>/LICENSE");
    let target_path = state_path!(TYPST_PKG).join("LICENSE");
    action_expect!(std::fs::copy(source_path, target_path));

    // The plugin links the Zint backend, so the package is distributed under
    // `MIT AND BSD-3-Clause` and has to carry that notice as well.
    let zint_source_path =
        state_path!(ZINT_LICENSE_FILE, default: "$<root>/zint-wasm-sys/LICENSE-BSD-3-CLAUSE");
    let zint_target_path = state_path!(TYPST_PKG).join("LICENSE-BSD-3-CLAUSE");
    action_expect!(std::fs::copy(zint_source_path, zint_target_path));

    action_ok!();
}

/// URL, archive root directory and extension of the prebuilt typst release for
/// a platform, as reported by [`std::env::consts`].
///
/// Windows is the one platform typst ships as a zip rather than a tarball.
fn typst_url(os: &str, arch: &str, version: &str) -> Option<(String, String, &'static str)> {
    let (target, ext) = match (os, arch) {
        ("linux", "aarch64") => ("aarch64-unknown-linux-musl", "tar.xz"),
        ("linux", "arm") => ("armv7-unknown-linux-musleabi", "tar.xz"),
        ("linux", "x86_64") => ("x86_64-unknown-linux-musl", "tar.xz"),
        ("macos", "aarch64") => ("aarch64-apple-darwin", "tar.xz"),
        ("macos", "x86_64") => ("x86_64-apple-darwin", "tar.xz"),
        ("windows", "x86_64") => ("x86_64-pc-windows-msvc", "zip"),
        _ => return None,
    };
    Some((
        format!("https://github.com/typst/typst/releases/download/v{version}/typst-{target}.{ext}"),
        format!("typst-{target}"),
        ext,
    ))
}

// should be only used for CI
pub fn action_install_typst(_args: &[String]) -> ActionResult {
    if has_command(TYPST) {
        action_skip!("{} already in PATH", TYPST);
    }

    let (url, base_dir, ext) = match typst_url(OS, ARCH, &state!(TYPST_VERSION)) {
        Some(it) => it,
        None => action_error!(std::io::Error::other(format!(
            "no prebuilt typst for {OS}-{ARCH}"
        ))),
    };
    let work_dir = state_path!(WORK_DIR);
    let typst_archive = work_dir.join(format!("typst.{ext}"));
    let typst_dir = work_dir.join("tools");
    let typst_bin = typst_dir.join(exe_name(TYPST));

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
                format!("{base_dir}/{}", exe_name(TYPST))
            ]
        ));
    }

    action_ok!();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every platform this project claims to support, so a URL for one host
    /// can't rot while another host is the only one building.
    const SUPPORTED: &[(&str, &str)] = &[
        ("linux", "aarch64"),
        ("linux", "x86_64"),
        ("macos", "aarch64"),
        ("macos", "x86_64"),
        ("windows", "x86_64"),
    ];

    fn assert_downloadable(url: &str) {
        assert!(url.starts_with("https://"), "not an https url: '{url}'");
        assert!(
            !url.contains(char::is_whitespace),
            "url contains whitespace: '{url}'"
        );
    }

    #[test]
    fn wasi_urls_are_well_formed() {
        for (os, arch) in SUPPORTED {
            let url = wasi_url(os, arch, "24").expect("no WASI SDK for {os}-{arch}");
            assert_downloadable(&url);
            assert!(
                url.starts_with(
                    "https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-24/"
                ),
                "not the release that was asked for: '{url}'"
            );
            assert!(url.ends_with(".tar.gz"), "'{url}' is not a tarball");
        }
    }

    #[test]
    fn binaryen_urls_are_well_formed() {
        for (os, arch) in SUPPORTED {
            let url = binaryen_url(os, arch, "119").expect("no binaryen for {os}-{arch}");
            assert_downloadable(&url);
            assert!(
                url.starts_with(
                    "https://github.com/WebAssembly/binaryen/releases/download/version_119/"
                ),
                "not the release that was asked for: '{url}'"
            );
            assert!(url.ends_with(".tar.gz"), "'{url}' is not a tarball");
        }
    }

    /// Binaryen names its ARM Linux archive after the target the compiler
    /// uses, unlike the WASI SDK, which calls the same platform arm64; the
    /// other name is only carried by the macOS archive.
    #[test]
    fn binaryen_names_arm_linux_after_the_compiler() {
        let url = binaryen_url("linux", "aarch64", "119").expect("no binaryen for linux-aarch64");
        assert!(
            url.ends_with("/binaryen-version_119-aarch64-linux.tar.gz"),
            "not the archive binaryen publishes: '{url}'"
        );
    }

    #[test]
    fn typst_urls_are_well_formed() {
        for (os, arch) in SUPPORTED {
            let (url, base_dir, ext) = typst_url(os, arch, "0.13.1").expect("no typst");
            assert_downloadable(&url);
            assert!(
                url.starts_with("https://github.com/typst/typst/releases/download/v0.13.1/"),
                "not the release that was asked for: '{url}'"
            );
            // The archive root is the member path prefix, so it has to match
            // what the URL actually names.
            assert!(url.contains(&base_dir), "'{base_dir}' not named by '{url}'");
            assert!(url.ends_with(ext), "'{url}' is not a '{ext}'");
        }
    }

    /// Windows is the one platform typst ships as a zip; everything else is a
    /// tarball, and the extraction path has to keep coping with both.
    #[test]
    fn typst_ships_a_zip_only_for_windows() {
        for (os, arch) in SUPPORTED {
            let (_, _, ext) = typst_url(os, arch, "0.13.1").expect("no typst");
            let expected = if *os == "windows" { "zip" } else { "tar.xz" };
            assert_eq!(ext, expected, "unexpected typst archive for {os}-{arch}");
        }
    }

    /// A tool can only be moved to another version together with the digest of
    /// what that version downloads, or the build stops on the archive it has
    /// never seen. The versions come from the same state the build reads, and
    /// every platform is checked from any one of them, so this fails on the
    /// bump rather than on the machine that builds next.
    #[test]
    fn every_archive_a_supported_platform_downloads_is_pinned() {
        let wasi = state!(WASI_SDK_VERSION, default: "24");
        let binaryen = state!(BINARYEN_VERSION, default: "119");
        let typst = state!(TYPST_VERSION, default: "0.13.1");

        for (os, arch) in SUPPORTED {
            let urls = [
                wasi_url(os, arch, &wasi).expect("no WASI SDK for {os}-{arch}"),
                binaryen_url(os, arch, &binaryen).expect("no binaryen for {os}-{arch}"),
                typst_url(os, arch, &typst)
                    .expect("no typst for {os}-{arch}")
                    .0,
            ];
            for url in urls {
                let artifact = crate::tools::release_and_name(&url);
                assert!(
                    crate::checksum::pinned(&artifact).is_some(),
                    "no digest is pinned for '{artifact}'"
                );
            }
        }
    }

    #[test]
    fn unsupported_platforms_have_no_url() {
        assert_eq!(wasi_url("linux", "riscv64", "24"), None);
        assert_eq!(binaryen_url("windows", "aarch64", "119"), None);
        assert_eq!(typst_url("freebsd", "x86_64", "0.13.1"), None);
    }
}
