//! Helpers shared by the unit tests across this crate.

use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU32, Ordering},
};

/// A file in the temporary directory that deletes itself again, so that a
/// failing run leaves nothing behind for the next one to read.
pub struct TempFile(PathBuf);

impl TempFile {
    pub fn holding(contents: &str) -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);

        // The process id keeps concurrent runs apart, the counter keeps the
        // tests within one run apart.
        let name = format!(
            "zint-wasi-xtask-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(name);
        std::fs::write(&path, contents).expect("writable temporary directory");

        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    pub fn read(&self) -> String {
        std::fs::read_to_string(&self.0).expect("the file written above")
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// A directory in the temporary directory that removes itself again, with
/// everything a test put into it.
pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new() -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);

        let name = format!(
            "zint-wasi-xtask-dir-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(name);
        std::fs::create_dir_all(&path).expect("writable temporary directory");

        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    /// Writes `contents` to `relative`, creating the directories on the way.
    pub fn write(&self, relative: &str, contents: &str) {
        let path = self.0.join(relative);
        std::fs::create_dir_all(path.parent().expect("a file below the directory"))
            .expect("writable temporary directory");
        std::fs::write(path, contents).expect("writable temporary directory");
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
