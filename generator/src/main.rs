mod code;
mod documentation;

use serde::Deserialize;
use std::path::PathBuf;
use zint_rs::options::symbology::SYMBOLOGY_SUMMARY;

#[derive(Deserialize)]
pub struct Symbology {
    pub name: String,
    pub kebab_case: String,
    #[serde(default)]
    pub doc: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub options: Vec<OptionField>,
}

#[derive(Deserialize)]
pub struct OptionField {
    #[serde(default)]
    pub name: String,
    pub ty: String,
    #[serde(default)]
    pub doc: String,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub is_binding: bool,
}

pub fn typst_pkg_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("typst-package")
}

fn main() {
    let symbologies: Vec<Symbology> =
        serde_json::from_str(SYMBOLOGY_SUMMARY).expect("failed to parse symbology summary");

    let command = std::env::args().nth(1);
    match command.as_deref() {
        Some("wrappers") => code::gen_wrappers(&symbologies),
        Some("docs") => documentation::gen_docs(&symbologies),
        _ => {
            eprintln!("Usage: generator <wrappers|docs>");
            std::process::exit(1);
        }
    }
}
