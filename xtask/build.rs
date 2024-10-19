use std::{path::PathBuf, str::FromStr};

fn main() {
    println!("cargo::rustc-check-cfg=cfg(ci)");
    println!("cargo::rustc-check-cfg=cfg(ci, values(\"github\"))");
    if std::env::var("GITHUB_ACTION")
        .map(|it| !it.is_empty())
        .unwrap_or_default()
    {
        println!("cargo::rustc-cfg=ci=\"github\"");
    } else if std::env::var("CI")
        .map(|it| it.to_ascii_lowercase() != "false" && it != "0")
        .unwrap_or_default()
    {
        println!("cargo::rustc-cfg=ci");
    }
    
    let project_root = PathBuf::from_str(std::env::var("CARGO_MANIFEST_DIR").unwrap().as_str())
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    println!("cargo::rustc-env=PROJECT_ROOT={}", project_root.display());
}
