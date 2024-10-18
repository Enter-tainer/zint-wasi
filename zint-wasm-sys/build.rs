use anyhow::Result;
use std::{env, path::PathBuf};

use walkdir::WalkDir;

fn main() -> Result<()> {
    {
        let sdk_path = match env::var("WASI_SDK_PATH") {
            Ok(it) => PathBuf::from(it),
            Err(_) => PathBuf::from("/opt/wasi-sdk"),
        };
        // report these errors early with clear error messages
        match std::fs::exists(&sdk_path) {
            Ok(true) => {}
            Ok(false) => panic!(
                "WASI SDK not installed, misconfigured: {}",
                sdk_path.display()
            ),
            Err(_) => panic!(
                "missing permissions to access WASI SDK: {}",
                sdk_path.display()
            ),
        }

        let sdk_bin = sdk_path.join("bin");
        let sdk_sysroot = sdk_path.join("share/wasi-sysroot");

        unsafe {
            env::set_var("CC", sdk_bin.join("clang"));
            env::set_var("AR", sdk_bin.join("ar"));
            env::set_var("CFLAGS", format!("--sysroot={}", sdk_sysroot.display()));
        }
    }

    let files = [
        "zint/backend/2of5.c",
        "zint/backend/auspost.c",
        "zint/backend/aztec.c",
        "zint/backend/bc412.c",
        "zint/backend/bmp.c",
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
        "zint/backend/emf.c",
        "zint/backend/general_field.c",
        "zint/backend/gif.c",
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
        "zint/backend/pcx.c",
        "zint/backend/pdf417.c",
        "zint/backend/plessey.c",
        // "zint/backend/png.c",
        "zint/backend/postal.c",
        "zint/backend/ps.c",
        "zint/backend/qr.c",
        "zint/backend/raster.c",
        "zint/backend/reedsol.c",
        "zint/backend/rss.c",
        "zint/backend/svg.c",
        "zint/backend/telepen.c",
        "zint/backend/tif.c",
        "zint/backend/ultra.c",
        "zint/backend/upcean.c",
        "zint/backend/vector.c",
        "extension/sds.c",
        "extension/svg.c",
    ];

    // Build zint as a static library.
    let mut build = cc::Build::new();

    build.files(files)
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
        .flag_if_supported("-Wno-shift-op-parentheses");
    
    build.target("wasm32-wasip1");
    #[cfg(target = "wasm32-wasip2")]
    build.target("wasm32-wasip2");

    build.opt_level(2)
        .compile("zint");

    // Generate bindings for quickjs
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_arg("-fvisibility=hidden")
        .size_t_is_usize(false);

    let bindings = if true {
        bindings.clang_arg("--target=wasm32-wasip1")
    } else if cfg!(target = "wasm32-wasip2") {
        bindings.clang_arg("--target=wasm32-wasip2")
    } else {
        bindings
    };

    let bindings = bindings.generate()?;

    println!("cargo:rerun-if-changed=wrapper.h");

    for entry in WalkDir::new("zint") {
        println!("cargo:rerun-if-changed={}", entry?.path().display());
    }

    for entry in WalkDir::new("extension") {
        println!("cargo:rerun-if-changed={}", entry?.path().display());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    bindings.write_to_file(out_dir.join("bindings.rs"))?;
    Ok(())
}
