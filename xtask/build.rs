use std::{path::PathBuf, str::FromStr};

fn main() {
    println!("cargo::rustc-check-cfg=cfg(ci)");
    println!("cargo::rustc-check-cfg=cfg(ci, values(\"github\"))");
    let on_github = std::env::var("GITHUB_ACTION")
        .map(|it| !it.is_empty())
        .unwrap_or_default();
    let on_ci = on_github
        || std::env::var("CI")
            .map(|it| !it.eq_ignore_ascii_case("false") && it != "0")
            .unwrap_or_default();

    // `ci = "github"` does not satisfy a bare `cfg(ci)`, so a GitHub Actions run
    // has to set both forms or every plain `cfg(ci)` is compiled out there.
    if on_ci {
        println!("cargo::rustc-cfg=ci");
    }
    if on_github {
        println!("cargo::rustc-cfg=ci=\"github\"");
    }

    if !std::env::var("XTASK_PROJECT_ROOT")
        .map(|it| !it.is_empty())
        .unwrap_or_default()
    {
        let project_root = PathBuf::from_str(std::env::var("CARGO_MANIFEST_DIR").unwrap().as_str())
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        println!(
            "cargo::rustc-env=XTASK_PROJECT_ROOT={}",
            project_root.display()
        );
    }
}
