use anyhow::Result;
use std::{
    env,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

fn watch_files(path: impl AsRef<Path>) {
    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        if entry.file_type().is_file() {
            println!("cargo:rerun-if-changed={}", entry.path().display());
        }
    }
}

/// Paths into a WASI SDK installation, located through `WASI_SDK_PATH`.
struct WasiSdk {
    clang: PathBuf,
    ar: PathBuf,
    sysroot: PathBuf,
}

impl WasiSdk {
    /// Panics with a message naming the offending path rather than letting a
    /// missing SDK surface later as an opaque "failed to find tool" from `cc`.
    fn locate() -> Self {
        let root = match env::var_os("WASI_SDK_PATH") {
            Some(it) => strip_verbatim(PathBuf::from(it)),
            None if cfg!(windows) => {
                panic!("WASI_SDK_PATH must be set to build for wasm32-wasip1 on Windows")
            }
            None => PathBuf::from("/opt/wasi-sdk"),
        };
        match std::fs::exists(&root) {
            Ok(true) => {}
            Ok(false) => panic!("WASI SDK not installed, misconfigured: {}", root.display()),
            Err(_) => panic!("missing permissions to access WASI SDK: {}", root.display()),
        }

        let bin = root.join("bin");
        let pick = |names: &[&str]| {
            names
                .iter()
                .map(|name| bin.join(format!("{name}{}", env::consts::EXE_SUFFIX)))
                .find(|it| std::fs::exists(it).unwrap_or_default())
                .unwrap_or_else(|| {
                    panic!("no {} in WASI SDK: {}", names.join(" or "), bin.display())
                })
        };

        Self {
            clang: pick(&["clang"]),
            // The Windows build of the SDK only ships the llvm- prefixed archiver.
            ar: pick(&["ar", "llvm-ar"]),
            sysroot: root.join("share").join("wasi-sysroot"),
        }
    }
}

/// Drops a Windows extended-length prefix, which `Path::join` would otherwise
/// carry into a path that clang cannot resolve: the prefix disables path
/// normalization, so every separator below it has to already be a backslash.
/// `Path::canonicalize` returns such a path, and that is what xtask exports as
/// `WASI_SDK_PATH`.
///
/// Input:  `\\?\D:\src\target\wasi_sdk`
/// Output: `D:\src\target\wasi_sdk`
fn strip_verbatim(path: PathBuf) -> PathBuf {
    match path.to_str().and_then(|it| it.strip_prefix(r"\\?\")) {
        Some(stripped) => PathBuf::from(stripped),
        None => path,
    }
}

fn main() -> Result<()> {
    #[allow(non_snake_case)]
    let WASM = env::var("CARGO_CFG_TARGET_FAMILY")
        .map(|it| it == "wasm")
        .unwrap_or_default();
    #[allow(non_snake_case)]
    let WASM32_WASIP1 = WASM
        && env::var("TARGET")
            .map(|it| it == "wasm32-wasip1")
            .unwrap_or_default();

    let wasi_sdk = WASM32_WASIP1.then(WasiSdk::locate);

    let files = [
        "zint/backend/2of5.c",
        "zint/backend/auspost.c",
        "zint/backend/aztec.c",
        "zint/backend/bc412.c",
        // "zint/backend/bmp.c",
        "zint/backend/codablock.c",
        "zint/backend/code128.c",
        "zint/backend/code16k.c",
        "zint/backend/code1.c",
        "zint/backend/code49.c",
        "zint/backend/code.c",
        "zint/backend/common.c",
        "zint/backend/composite.c",
        "zint/backend/dllversion.c",
        "zint/backend/dmatrix.c",
        "zint/backend/dotcode.c",
        "zint/backend/eci.c",
        // "zint/backend/emf.c",
        "zint/backend/filemem.c",
        "zint/backend/general_field.c",
        // "zint/backend/gif.c",
        "zint/backend/gridmtx.c",
        "zint/backend/gs1.c",
        "zint/backend/hanxin.c",
        "zint/backend/imail.c",
        "zint/backend/large.c",
        "zint/backend/library.c",
        "zint/backend/mailmark.c",
        "zint/backend/maxicode.c",
        "zint/backend/medical.c",
        "zint/backend/output.c",
        // "zint/backend/pcx.c",
        "zint/backend/pdf417.c",
        "zint/backend/plessey.c",
        // "zint/backend/png.c",
        "zint/backend/postal.c",
        // "zint/backend/ps.c",
        "zint/backend/qr.c",
        // "zint/backend/raster.c",
        "zint/backend/reedsol.c",
        "zint/backend/rss.c",
        "zint/backend/svg.c",
        "zint/backend/telepen.c",
        // "zint/backend/tif.c",
        "zint/backend/ultra.c",
        "zint/backend/upcean.c",
        "zint/backend/vector.c",
        "patch/patch.c",
    ];

    // Build zint as a static library.
    let mut build = cc::Build::new();

    build
        .files(files)
        .define("_GNU_SOURCE", None)
        // The below flags are used by the official Makefile.
        .flag_if_supported("-Wchar-subscripts")
        .flag_if_supported("-Wno-array-bounds")
        .flag_if_supported("-Wno-format-truncation")
        .flag_if_supported("-Wno-missing-field-initializers")
        .flag_if_supported("-Wno-sign-compare")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wuninitialized")
        .flag_if_supported("-Wunused")
        .flag_if_supported("-Wwrite-strings")
        .flag_if_supported("-funsigned-char")
        .flag_if_supported("-Wno-cast-function-type")
        .flag_if_supported("-Wno-implicit-fallthrough")
        .flag_if_supported("-Wno-enum-conversion")
        .flag_if_supported("-Wno-implicit-function-declaration")
        .flag_if_supported("-Wno-implicit-const-int-float-conversion")
        .flag_if_supported("-Wno-shift-op-parentheses")
        .opt_level(2);

    if let Some(sdk) = &wasi_sdk {
        build
            .target("wasm32-wasip1")
            .compiler(&sdk.clang)
            .archiver(&sdk.ar)
            .flag(format!("--sysroot={}", sdk.sysroot.display()));
    }
    build.compile("zint");

    // Generate bindings for zint
    let bindings = bindgen::Builder::default()
        .header("zint/backend/zint.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_arg("-fvisibility=hidden")
        .size_t_is_usize(false);

    let bindings = if WASM32_WASIP1 {
        bindings.clang_arg("--target=wasm32-wasip1")
    } else {
        bindings
    };

    let bindings = bindings.generate()?;

    watch_files("zint");
    watch_files("patch");

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    bindings.write_to_file(out_dir.join("bindings.rs"))?;
    Ok(())
}
