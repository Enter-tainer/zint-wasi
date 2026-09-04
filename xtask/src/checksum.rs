//! Digests for the toolchains the build downloads.
//!
//! Every archive that is fetched over the network is checked against a digest
//! that is part of the source tree, so that a build either uses the bytes this
//! repository was tested against or stops and says which ones it got instead.

use std::{
    fmt::Display,
    io::{self, Read},
    path::Path,
};

use sha2::{Digest, Sha256};

/// The digest of every archive the build is allowed to download, in the format
/// `sha256sum` writes, so that the same file checks out with
/// `sha256sum --check`.
const PINNED: &str = include_str!("../toolchain.sha256");

/// Reads the pinned digest of `artifact`, named the way the release publishes
/// it.
///
/// Input:  `"binaryen-version_119-x86_64-linux.tar.gz"`
/// Output: the 64 hex characters that archive has to hash to
pub fn pinned(artifact: &str) -> Option<&'static str> {
    PINNED.lines().find_map(|line| {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }
        // `sha256sum` separates the two with two spaces, the second of which
        // says the file was read as binary.
        let (digest, name) = line.split_once("  ")?;
        (name.trim() == artifact).then_some(digest)
    })
}

/// Checks a downloaded archive against the digest pinned for it.
pub fn verify(path: impl AsRef<Path>, artifact: &str) -> Result<(), ChecksumError> {
    let actual = of_file(path.as_ref()).map_err(ChecksumError::IO)?;

    match pinned(artifact) {
        Some(expected) if expected.eq_ignore_ascii_case(&actual) => Ok(()),
        Some(expected) => Err(ChecksumError::Mismatch {
            artifact: artifact.to_string(),
            expected: expected.to_string(),
            actual,
        }),
        None => Err(ChecksumError::Unpinned {
            artifact: artifact.to_string(),
            actual,
        }),
    }
}

/// The SHA-256 of a file, as lowercase hex.
pub fn of_file(path: impl AsRef<Path>) -> io::Result<String> {
    let mut file = io::BufReader::new(std::fs::File::open(path)?);
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 8 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(hex(&digest.finalize()))
}

/// The name of any one of the pinned archives, for tests that need a name this
/// build knows without hardcoding a version that later moves.
#[cfg(test)]
pub fn some_pinned_artifact() -> &'static str {
    PINNED
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .find_map(|line| line.split_once("  ").map(|(_, name)| name.trim()))
        .expect("at least one pinned archive")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[derive(Debug)]
pub enum ChecksumError {
    /// The archive is not the one that was pinned
    Mismatch {
        artifact: String,
        expected: String,
        actual: String,
    },
    /// The archive is one this build has never seen
    Unpinned {
        artifact: String,
        actual: String,
    },
    IO(io::Error),
}

impl Display for ChecksumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChecksumError::Mismatch {
                artifact,
                expected,
                actual,
            } => write!(
                f,
                "'{artifact}' is not the archive this build pins:\n\
                 \texpected {expected}\n\
                 \tgot      {actual}\n\
                 the downloaded file has been removed"
            ),
            ChecksumError::Unpinned { artifact, actual } => write!(
                f,
                "no digest is pinned for '{artifact}'; after checking where it \
                 came from, add\n\n\t{actual}  {artifact}\n\nto \
                 xtask/toolchain.sha256, or set XTASK_ALLOW_UNPINNED_DOWNLOADS=1 \
                 to build without this check"
            ),
            ChecksumError::IO(io) => write!(f, "cannot read the downloaded file: {io}"),
        }
    }
}

impl std::error::Error for ChecksumError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IO(source) => Some(source),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{pinned, verify, ChecksumError, PINNED};
    use crate::test_support::TempFile;

    #[test]
    fn a_file_is_hashed_by_its_contents() {
        let file = TempFile::holding("abc");
        assert_eq!(
            super::of_file(file.path()).expect("the file written above"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    /// Every line has to be readable as `sha256sum` output, or the digest of a
    /// tool would silently go missing.
    #[test]
    fn the_pinned_file_is_a_list_of_digests_and_names() {
        let entries: Vec<(&str, &str)> = PINNED
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| {
                line.split_once("  ")
                    .unwrap_or_else(|| panic!("two spaces between digest and name: {line}"))
            })
            .collect();

        assert!(!entries.is_empty(), "the digests are missing");
        for (digest, name) in &entries {
            assert_eq!(digest.len(), 64, "{name} has a digest of the wrong length");
            assert!(
                digest.chars().all(|it| it.is_ascii_hexdigit()),
                "{name} has a digest that is not hex"
            );
        }

        let mut names: Vec<&str> = entries.iter().map(|(_, name)| *name).collect();
        names.sort_unstable();
        let unique = names.len();
        names.dedup();
        assert_eq!(unique, names.len(), "an archive is pinned twice");
    }

    #[test]
    fn an_archive_that_is_not_pinned_has_no_digest() {
        assert!(pinned("binaryen-version_1-x86_64-linux.tar.gz").is_none());
    }

    /// The digest of a pinned archive is what says the download is the one the
    /// build was tested against, so a file that is not it has to be refused.
    #[test]
    fn a_file_that_is_not_the_pinned_archive_is_reported() {
        let file = TempFile::holding("abc");
        let artifact = super::some_pinned_artifact();

        let error = verify(file.path(), artifact).expect_err("that is not the archive");
        assert!(
            matches!(error, ChecksumError::Mismatch { .. }),
            "unexpected error: {error}"
        );
        assert!(
            error
                .to_string()
                .contains("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
            "the error should carry the digest that was found: {error}"
        );
    }

    #[test]
    fn a_file_nothing_is_pinned_for_says_what_to_add() {
        let file = TempFile::holding("abc");

        let error = verify(file.path(), "something-nobody-pinned.tar.gz")
            .expect_err("nothing is pinned for it");

        assert!(
            matches!(error, ChecksumError::Unpinned { .. }),
            "unexpected error: {error}"
        );
        let message = error.to_string();
        assert!(
            message.contains(
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  \
                 something-nobody-pinned.tar.gz"
            ),
            "the error should spell out the line to add: {message}"
        );
    }
}
