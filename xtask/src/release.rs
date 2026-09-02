//! Support for cutting a release: the package manifest as far as a release
//! needs it, moving the version everywhere the repository repeats it, and
//! laying out the files that make up the published package.

use std::{
    fmt, io,
    ops::Range,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

/// Where the release code stops and reports instead of guessing.
#[derive(Debug)]
pub enum ReleaseError {
    /// `set-version` was asked for without a version.
    MissingVersionArgument,
    /// Not a `major.minor.patch` triple.
    InvalidVersion(String),
    /// A manifest key that has to be there once is absent or repeated.
    Key {
        key: &'static str,
        problem: &'static str,
    },
    /// The text does not contain what a rewrite was going to replace.
    NotFound(String),
    /// An `exclude` pattern uses a feature the bundler does not implement.
    UnsupportedPattern(String),
    /// The files that repeat the version disagree; one line per file.
    Mismatch(Vec<String>),
    Io(io::Error),
}

impl fmt::Display for ReleaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReleaseError::MissingVersionArgument => {
                write!(f, "usage: cargo xtask set-version <major.minor.patch>")
            }
            ReleaseError::InvalidVersion(version) => {
                write!(f, "'{version}' is not a major.minor.patch version")
            }
            ReleaseError::Key { key, problem } => {
                write!(f, "manifest key '{key}' under [package] {problem}")
            }
            ReleaseError::NotFound(what) => write!(f, "found no {what}"),
            ReleaseError::UnsupportedPattern(pattern) => write!(
                f,
                "exclude pattern '{pattern}' is not supported: only plain file and directory names, optionally anchored with a leading '/', can be bundled"
            ),
            ReleaseError::Mismatch(lines) => {
                write!(f, "the version is not the same everywhere:")?;
                for line in lines {
                    write!(f, "\n  {line}")?;
                }
                Ok(())
            }
            ReleaseError::Io(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for ReleaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ReleaseError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for ReleaseError {
    fn from(err: io::Error) -> Self {
        ReleaseError::Io(err)
    }
}

/// Checks that `version` is the `major.minor.patch` triple Typst Universe
/// requires: three decimal numbers, no prefix, no pre-release suffix.
pub fn check_version(version: &str) -> Result<(), ReleaseError> {
    let parts: Vec<&str> = version.split('.').collect();
    let is_number = |part: &&str| {
        !part.is_empty()
            && part.bytes().all(|byte| byte.is_ascii_digit())
            && (part.len() == 1 || !part.starts_with('0'))
    };
    if parts.len() == 3 && parts.iter().all(is_number) {
        Ok(())
    } else {
        Err(ReleaseError::InvalidVersion(version.to_string()))
    }
}

/// The parts of `typst.toml` that a release reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub exclude: Vec<String>,
}

impl Manifest {
    /// Reads the keys out of a manifest.
    ///
    /// Only the shape this project's manifest has is understood: under the
    /// `[package]` table, one quoted string per key, and for `exclude` an
    /// array of quoted strings that may span lines. That is far from TOML,
    /// but it keeps a parser out of xtask, and anything else fails here
    /// rather than being half-read.
    pub fn parse(text: &str) -> Result<Self, ReleaseError> {
        Ok(Self {
            name: package_string(text, "name")?,
            version: package_string(text, "version")?,
            exclude: package_string_array(text, "exclude")?,
        })
    }
}

/// The lines under `[package]`, with comments and surrounding whitespace
/// removed, so that the key of each is what precedes its first `=`.
fn package_lines(text: &str) -> impl Iterator<Item = &str> {
    let mut in_package = false;
    text.lines().filter_map(move |line| {
        let line = strip_comment(line).trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            return None;
        }
        in_package.then_some(line)
    })
}

/// Cuts a `#` comment off a line, leaving one inside a quoted string alone.
///
/// Input:  `exclude = ["a#b"] # docs`
/// Output: `exclude = ["a#b"] `
fn strip_comment(line: &str) -> &str {
    let mut quoted = false;
    for (index, ch) in line.char_indices() {
        match ch {
            '"' => quoted = !quoted,
            '#' if !quoted => return &line[..index],
            _ => {}
        }
    }
    line
}

/// The text after `key =` on the one line under `[package]` that has it.
fn package_value<'t>(text: &'t str, key: &'static str) -> Result<Option<&'t str>, ReleaseError> {
    let mut values = package_lines(text).filter_map(|line| {
        let (found, value) = line.split_once('=')?;
        (found.trim() == key).then_some(value.trim())
    });
    let value = values.next();
    if values.next().is_some() {
        return Err(ReleaseError::Key {
            key,
            problem: "appears more than once",
        });
    }
    Ok(value)
}

/// The string between the quotes of a `key = "..."` value.
fn unquote(value: &str) -> Option<&str> {
    let inner = value.strip_prefix('"')?.strip_suffix('"')?;
    (!inner.contains('"')).then_some(inner)
}

fn package_string(text: &str, key: &'static str) -> Result<String, ReleaseError> {
    let value = package_value(text, key)?.ok_or(ReleaseError::Key {
        key,
        problem: "is missing",
    })?;
    unquote(value).map(str::to_string).ok_or(ReleaseError::Key {
        key,
        problem: "is not a quoted string",
    })
}

/// A `key = ["a", "b"]` array, which may continue over the following lines.
/// An absent key is an empty array.
///
/// Input:  `exclude = [\n  "manual.pdf",\n  "example.svg"\n]`
/// Output: `["manual.pdf", "example.svg"]`
fn package_string_array(text: &str, key: &'static str) -> Result<Vec<String>, ReleaseError> {
    let not_an_array = ReleaseError::Key {
        key,
        problem: "is not an array of quoted strings",
    };
    let Some(first) = package_value(text, key)? else {
        return Ok(Vec::new());
    };
    let Some(mut body) = first.strip_prefix('[').map(str::to_string) else {
        return Err(not_an_array);
    };

    // The line with the key is only found again by scanning, so the
    // continuation lines are those after it.
    if !body.contains(']') {
        let mut lines = package_lines(text).skip_while(|line| !line.starts_with(key));
        lines.next();
        for line in lines {
            body.push(' ');
            body.push_str(line);
            if line.contains(']') {
                break;
            }
        }
    }
    let Some((inner, rest)) = body.split_once(']') else {
        return Err(not_an_array);
    };
    if !rest.trim().is_empty() {
        return Err(not_an_array);
    }

    inner
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(|item| {
            unquote(item).map(str::to_string).ok_or(ReleaseError::Key {
                key,
                problem: "is not an array of quoted strings",
            })
        })
        .collect()
}

/// Replaces the `version` under `[package]`, leaving the rest of the file as
/// it is. Works for `typst.toml` and for a crate's `Cargo.toml` alike, where
/// the dependency versions sit under other tables and are left alone.
///
/// Input:  `[package]\nname = "tiaoma"\nversion = "0.3.0" # released\n`
/// Output: `[package]\nname = "tiaoma"\nversion = "0.4.0" # released\n`
pub fn set_manifest_version(text: &str, version: &str) -> Result<String, ReleaseError> {
    let mut in_package = false;
    let mut replaced = 0;
    let mut result = String::with_capacity(text.len());

    for line in text.split_inclusive('\n') {
        let content = strip_comment(line).trim();
        if content.starts_with('[') {
            in_package = content == "[package]";
        }
        let is_version = in_package
            && content
                .split_once('=')
                .is_some_and(|(key, _)| key.trim() == "version");
        if !is_version {
            result.push_str(line);
            continue;
        }

        let open = line.find('"');
        let close = open.and_then(|open| line[open + 1..].find('"').map(|it| open + 1 + it));
        let (Some(open), Some(close)) = (open, close) else {
            return Err(ReleaseError::Key {
                key: "version",
                problem: "is not a quoted string",
            });
        };
        result.push_str(&line[..=open]);
        result.push_str(version);
        result.push_str(&line[close..]);
        replaced += 1;
    }

    match replaced {
        1 => Ok(result),
        0 => Err(ReleaseError::Key {
            key: "version",
            problem: "is missing",
        }),
        _ => Err(ReleaseError::Key {
            key: "version",
            problem: "appears more than once",
        }),
    }
}

/// Where the version sits in every `@preview/<name>:<version>` import of
/// `text`, so that the same scan serves reading and rewriting them.
///
/// Input:  `#import "@preview/tiaoma:0.3.0"` with name `tiaoma`
/// Output: the range covering `0.3.0`
fn import_version_ranges(text: &str, name: &str) -> Vec<Range<usize>> {
    let needle = format!("@preview/{name}:");
    let mut ranges = Vec::new();
    let mut from = 0;
    while let Some(found) = text[from..].find(&needle) {
        let start = from + found + needle.len();
        let end = start
            + text[start..]
                .find(|ch: char| !ch.is_ascii_digit() && ch != '.')
                .unwrap_or(text.len() - start);
        if end > start {
            ranges.push(start..end);
            from = end;
            continue;
        }
        // Nothing version-shaped followed the marker. The scan carries on
        // past a whole character, because the byte after the marker can sit
        // inside a multi-byte one, and stops when the marker ended the text.
        match text[start..].chars().next() {
            Some(next) => from = start + next.len_utf8(),
            None => break,
        }
    }
    ranges
}

/// The versions that `text` imports the package `name` at.
pub fn import_versions(text: &str, name: &str) -> Vec<String> {
    import_version_ranges(text, name)
        .into_iter()
        .map(|range| text[range].to_string())
        .collect()
}

/// Moves every `@preview/<name>:<version>` import in `text` to `version`.
///
/// Input:  `#import "@preview/tiaoma:0.3.0"` with name `tiaoma`, version `0.4.0`
/// Output: `#import "@preview/tiaoma:0.4.0"`
pub fn set_import_version(text: &str, name: &str, version: &str) -> Result<String, ReleaseError> {
    let ranges = import_version_ranges(text, name);
    if ranges.is_empty() {
        return Err(ReleaseError::NotFound(format!("@preview/{name} import")));
    }

    let mut result = String::with_capacity(text.len());
    let mut from = 0;
    for range in ranges {
        result.push_str(&text[from..range.start]);
        result.push_str(version);
        from = range.end;
    }
    result.push_str(&text[from..]);
    Ok(result)
}

/// The `exclude` patterns of a manifest, as far as the bundler honours them.
///
/// Typst applies them with gitignore semantics. The subset implemented here
/// is a bare file or directory name, which matches at any depth, and a path
/// with a slash in it, which is anchored at the package root whether or not
/// it carries the leading slash gitignore uses to say so. Wildcards and
/// negations are refused rather than approximated, so a manifest that starts
/// using them fails the bundle step instead of shipping what it meant to
/// leave out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Excludes(Vec<Exclude>);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Exclude {
    /// Matches a path component at any depth.
    Name(String),
    /// Matches a path from the package root, and everything below it.
    Anchored(Vec<String>),
}

impl Excludes {
    pub fn parse<S: AsRef<str>>(patterns: &[S]) -> Result<Self, ReleaseError> {
        patterns
            .iter()
            .map(|pattern| Exclude::parse(pattern.as_ref()))
            .collect::<Result<_, _>>()
            .map(Self)
    }

    /// Whether `relative`, a path below the package root, is left out.
    pub fn matches(&self, relative: &Path) -> bool {
        let components: Vec<String> = relative
            .components()
            .map(|it| it.as_os_str().to_string_lossy().into_owned())
            .collect();
        self.0.iter().any(|exclude| match exclude {
            Exclude::Name(name) => components.iter().any(|it| it == name),
            Exclude::Anchored(parts) => components.starts_with(parts),
        })
    }
}

impl Exclude {
    fn parse(pattern: &str) -> Result<Self, ReleaseError> {
        let unsupported = || ReleaseError::UnsupportedPattern(pattern.to_string());
        if pattern.contains(['*', '?', '[', ']', '\\']) || pattern.starts_with('!') {
            return Err(unsupported());
        }
        // A trailing slash only restricts the match to directories, and the
        // bundle copies files, so a directory is excluded by its files anyway.
        let trimmed = pattern.trim_end_matches('/');
        let anchored = trimmed.starts_with('/') || trimmed.contains('/');
        let parts: Vec<String> = trimmed
            .split('/')
            .filter(|part| !part.is_empty())
            .map(str::to_string)
            .collect();
        if parts.is_empty() || parts.iter().any(|part| part == "." || part == "..") {
            return Err(unsupported());
        }
        Ok(if anchored {
            Exclude::Anchored(parts)
        } else {
            Exclude::Name(trimmed.to_string())
        })
    }
}

/// Copies the package at `package` into `target`, without the excluded files,
/// creating `target` as needed. Returns what was copied, relative to the
/// package root and in a fixed order, so the caller can list it.
pub fn bundle(
    package: &Path,
    target: &Path,
    excludes: &Excludes,
) -> Result<Vec<PathBuf>, ReleaseError> {
    let mut copied = Vec::new();
    let mut walker = WalkDir::new(package)
        .min_depth(1)
        .sort_by_file_name()
        .into_iter();

    while let Some(entry) = walker.next() {
        let entry = entry.map_err(io::Error::other)?;
        let relative = entry
            .path()
            .strip_prefix(package)
            .expect("walked from the package root")
            .to_path_buf();
        if excludes.matches(&relative) {
            if entry.file_type().is_dir() {
                walker.skip_current_dir();
            }
            continue;
        }
        if entry.file_type().is_dir() {
            continue;
        }

        let destination = target.join(&relative);
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(entry.path(), destination)?;
        copied.push(relative);
    }
    Ok(copied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    const TYPST_TOML: &str = r#"[package]
name = "tiaoma"
version = "0.3.0"
entrypoint = "lib.typ"
license = "MIT AND BSD-3-Clause"
keywords = [
    "wasm",
    "zint",
]
exclude = [
  "tidy_style.typ",
  "manual.typ", # the manual is linked from the README
  "manual.pdf",
  "example.typ",
  "example.svg"
]
"#;

    const CARGO_TOML: &str = r#"[package]
name = "zint-typst-plugin"
version = "0.3.0"
license = "MIT"
edition.workspace = true

[dependencies]
serde = { version = "1", features = ["derive"] }

[dependencies.ciborium]
version = "0.2.1"
"#;

    #[test]
    fn a_release_version_is_three_plain_numbers() {
        for version in ["0.0.0", "0.4.0", "1.0.0", "10.20.30", "4294967295.0.1"] {
            assert!(check_version(version).is_ok(), "{version}");
        }
    }

    #[test]
    fn anything_else_is_refused_as_a_version() {
        for version in [
            "",
            "0",
            "0.4",
            "0.4.0.1",
            "v0.4.0",
            "0.4.0-rc1",
            "0.04.0",
            "0.4.0 ",
            " 0.4.0",
            "0..0",
            "a.b.c",
            "٠.٤.٠",
        ] {
            assert!(
                matches!(check_version(version), Err(ReleaseError::InvalidVersion(found)) if found == version),
                "{version:?}"
            );
        }
    }

    /// The package README is prose, so the marker can turn up in it without a
    /// version behind it. Every shape of that is passed over rather than read,
    /// and none of them may cut a character in half on the way.
    #[test]
    fn a_marker_without_a_version_behind_it_is_passed_over() {
        for (text, expected) in [
            ("see @preview/tiaoma:\u{e9} and more", vec![]),
            ("@preview/tiaoma: is how it is imported", vec![]),
            ("the import ends the line: @preview/tiaoma:", vec![]),
            (
                "@preview/tiaoma: became @preview/tiaoma:0.3.0",
                vec!["0.3.0"],
            ),
        ] {
            assert_eq!(import_versions(text, "tiaoma"), expected, "{text:?}");
        }
    }

    #[test]
    fn the_manifest_keys_a_release_needs_are_read() {
        let manifest = Manifest::parse(TYPST_TOML).unwrap();
        assert_eq!(manifest.name, "tiaoma");
        assert_eq!(manifest.version, "0.3.0");
        assert_eq!(
            manifest.exclude,
            [
                "tidy_style.typ",
                "manual.typ",
                "manual.pdf",
                "example.typ",
                "example.svg"
            ]
        );
    }

    #[test]
    fn a_one_line_exclude_array_and_a_missing_one_are_read_too() {
        let one_line =
            "[package]\nname = \"x\"\nversion = \"1.0.0\"\nexclude = [\"a.pdf\", \"b.pdf\"]\n";
        assert_eq!(
            Manifest::parse(one_line).unwrap().exclude,
            ["a.pdf", "b.pdf"]
        );

        let empty = "[package]\nname = \"x\"\nversion = \"1.0.0\"\nexclude = []\n";
        assert!(Manifest::parse(empty).unwrap().exclude.is_empty());

        let absent = "[package]\nname = \"x\"\nversion = \"1.0.0\"\n";
        assert!(Manifest::parse(absent).unwrap().exclude.is_empty());
    }

    #[test]
    fn keys_outside_the_package_table_do_not_count() {
        let text = "[package]\nname = \"x\"\n[template]\nversion = \"9.9.9\"\n";
        assert!(matches!(
            Manifest::parse(text),
            Err(ReleaseError::Key {
                key: "version",
                problem: "is missing"
            })
        ));
    }

    #[test]
    fn a_key_that_is_there_twice_is_reported_rather_than_picked() {
        let text = "[package]\nname = \"x\"\nversion = \"1.0.0\"\nversion = \"2.0.0\"\n";
        assert!(matches!(
            Manifest::parse(text),
            Err(ReleaseError::Key {
                key: "version",
                problem: "appears more than once"
            })
        ));
    }

    #[test]
    fn a_value_of_another_shape_is_reported() {
        let unquoted = "[package]\nname = tiaoma\nversion = \"1.0.0\"\n";
        assert!(matches!(
            Manifest::parse(unquoted),
            Err(ReleaseError::Key {
                key: "name",
                problem: "is not a quoted string"
            })
        ));

        let not_array = "[package]\nname = \"x\"\nversion = \"1.0.0\"\nexclude = \"a.pdf\"\n";
        assert!(matches!(
            Manifest::parse(not_array),
            Err(ReleaseError::Key { key: "exclude", .. })
        ));

        let bare_item = "[package]\nname = \"x\"\nversion = \"1.0.0\"\nexclude = [a.pdf]\n";
        assert!(matches!(
            Manifest::parse(bare_item),
            Err(ReleaseError::Key { key: "exclude", .. })
        ));

        let unterminated =
            "[package]\nname = \"x\"\nversion = \"1.0.0\"\nexclude = [\n  \"a.pdf\",\n";
        assert!(matches!(
            Manifest::parse(unterminated),
            Err(ReleaseError::Key { key: "exclude", .. })
        ));
    }

    #[test]
    fn a_comment_ends_at_a_hash_outside_quotes_only() {
        assert_eq!(
            strip_comment("exclude = [\"a#b\"] # docs"),
            "exclude = [\"a#b\"] "
        );
        assert_eq!(strip_comment("# only a comment"), "");
        assert_eq!(strip_comment("plain"), "plain");
    }

    #[test]
    fn the_manifest_version_moves_and_nothing_else_does() {
        let moved = set_manifest_version(TYPST_TOML, "0.4.0").unwrap();
        assert_eq!(moved, TYPST_TOML.replacen("0.3.0", "0.4.0", 1));

        let moved = set_manifest_version(CARGO_TOML, "0.4.0").unwrap();
        assert_eq!(moved, CARGO_TOML.replacen("0.3.0", "0.4.0", 1));
        assert!(
            moved.contains("version = \"1\""),
            "dependency versions stay"
        );
        assert!(
            moved.contains("version = \"0.2.1\""),
            "dependency tables stay"
        );
    }

    #[test]
    fn the_manifest_version_keeps_its_comment_and_line_ending() {
        let text = "[package]\r\nversion = \"0.3.0\" # released\r\n";
        assert_eq!(
            set_manifest_version(text, "0.4.0").unwrap(),
            "[package]\r\nversion = \"0.4.0\" # released\r\n"
        );
    }

    #[test]
    fn a_manifest_without_a_package_version_cannot_be_moved() {
        assert!(matches!(
            set_manifest_version("[package]\nname = \"x\"\n", "0.4.0"),
            Err(ReleaseError::Key {
                key: "version",
                problem: "is missing"
            })
        ));
        assert!(matches!(
            set_manifest_version("[package]\nversion = 1\n", "0.4.0"),
            Err(ReleaseError::Key {
                key: "version",
                problem: "is not a quoted string"
            })
        ));
        assert!(matches!(
            set_manifest_version(
                "[package]\nversion = \"1.0.0\"\nversion = \"2.0.0\"\n",
                "0.4.0"
            ),
            Err(ReleaseError::Key {
                key: "version",
                problem: "appears more than once"
            })
        ));
    }

    #[test]
    fn every_import_of_the_package_is_found_and_moved() {
        let readme = "#import \"@preview/tiaoma:0.3.0\"\n\nor `@preview/tiaoma:0.2.1`, not @preview/tiaoma-extra:1.0.0\n";
        assert_eq!(import_versions(readme, "tiaoma"), ["0.3.0", "0.2.1"]);
        assert_eq!(
            set_import_version(readme, "tiaoma", "0.4.0").unwrap(),
            "#import \"@preview/tiaoma:0.4.0\"\n\nor `@preview/tiaoma:0.4.0`, not @preview/tiaoma-extra:1.0.0\n"
        );
    }

    #[test]
    fn an_import_at_the_end_of_the_text_is_still_found() {
        assert_eq!(
            import_versions("@preview/tiaoma:0.3.0", "tiaoma"),
            ["0.3.0"]
        );
    }

    #[test]
    fn an_import_without_a_version_is_not_one() {
        assert!(import_versions("@preview/tiaoma:\"", "tiaoma").is_empty());
        assert!(matches!(
            set_import_version("no import here", "tiaoma", "0.4.0"),
            Err(ReleaseError::NotFound(what)) if what == "@preview/tiaoma import"
        ));
    }

    #[test]
    fn a_bare_name_is_excluded_at_any_depth() {
        let excludes = Excludes::parse(&["manual.pdf", "assets"]).unwrap();
        assert!(excludes.matches(Path::new("manual.pdf")));
        assert!(excludes.matches(Path::new("docs/manual.pdf")));
        assert!(excludes.matches(Path::new("assets")));
        assert!(excludes.matches(Path::new("assets/logo.svg")));
        assert!(excludes.matches(Path::new("src/assets/logo.svg")));
        assert!(!excludes.matches(Path::new("manual.typ")));
        assert!(!excludes.matches(Path::new("assets.typ")));
    }

    #[test]
    fn a_path_is_excluded_from_the_root_only() {
        let excludes = Excludes::parse(&["/assets", "docs/manual.pdf", "gallery/"]).unwrap();
        assert!(excludes.matches(Path::new("assets/logo.svg")));
        assert!(!excludes.matches(Path::new("src/assets/logo.svg")));
        assert!(excludes.matches(Path::new("docs/manual.pdf")));
        assert!(!excludes.matches(Path::new("manual.pdf")));
        assert!(excludes.matches(Path::new("gallery/one.png")));
    }

    #[test]
    fn a_pattern_the_bundler_cannot_honour_is_refused() {
        for pattern in [
            "*.pdf",
            "manual.?df",
            "[ab].typ",
            "!lib.typ",
            "",
            "/",
            "./lib.typ",
            "../x",
            "a\\b",
        ] {
            assert!(
                matches!(Excludes::parse(&[pattern]), Err(ReleaseError::UnsupportedPattern(found)) if found == pattern),
                "{pattern:?}"
            );
        }
    }

    #[test]
    fn the_bundle_holds_the_package_without_the_excluded_files() {
        let package = TempDir::new();
        package.write("typst.toml", "[package]\n");
        package.write("lib.typ", "// lib");
        package.write("manual.pdf", "%PDF");
        package.write("docs/guide.pdf", "%PDF");
        package.write("docs/notes.typ", "// notes");
        package.write("assets/logo.svg", "<svg/>");
        package.write("assets/nested/deep.svg", "<svg/>");
        let target = TempDir::new();
        let target_dir = target.path().join("tiaoma").join("0.4.0");

        let excludes = Excludes::parse(&["manual.pdf", "guide.pdf", "/assets"]).unwrap();
        let copied = bundle(package.path(), &target_dir, &excludes).unwrap();

        assert_eq!(
            copied,
            [
                PathBuf::from("docs").join("notes.typ"),
                PathBuf::from("lib.typ"),
                PathBuf::from("typst.toml"),
            ]
        );
        assert_eq!(
            std::fs::read_to_string(target_dir.join("docs").join("notes.typ")).unwrap(),
            "// notes"
        );
        assert!(!target_dir.join("manual.pdf").exists());
        assert!(!target_dir.join("assets").exists());
    }

    #[test]
    fn bundling_an_empty_package_copies_nothing_and_creates_no_target() {
        let package = TempDir::new();
        let target = TempDir::new();
        let target_dir = target.path().join("out");

        let copied = bundle(
            package.path(),
            &target_dir,
            &Excludes::parse::<&str>(&[]).unwrap(),
        )
        .unwrap();

        assert!(copied.is_empty());
        assert!(!target_dir.exists());
    }

    #[test]
    fn errors_say_what_went_wrong() {
        assert_eq!(
            ReleaseError::InvalidVersion("v1".into()).to_string(),
            "'v1' is not a major.minor.patch version"
        );
        assert_eq!(
            ReleaseError::Key {
                key: "version",
                problem: "is missing"
            }
            .to_string(),
            "manifest key 'version' under [package] is missing"
        );
        assert_eq!(
            ReleaseError::Mismatch(vec!["a: 1".into(), "b: 2".into()]).to_string(),
            "the version is not the same everywhere:\n  a: 1\n  b: 2"
        );
        assert_eq!(
            ReleaseError::MissingVersionArgument.to_string(),
            "usage: cargo xtask set-version <major.minor.patch>"
        );
    }
}
