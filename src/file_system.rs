use std::path::{PathBuf, Path};

/// Allows mocking of the file-system for 
/// testing.
pub trait FsProvider {
    fn current_dir(&self) -> Result<PathBuf, std::io::Error>;
    fn read_to_string(&self, path: &Path) -> Result<String, std::io::Error>;
}

pub struct FsWrapper { }

impl FsProvider for FsWrapper {
    fn current_dir(&self) -> Result<PathBuf, std::io::Error> {
        std::env::current_dir()
    }

    fn read_to_string(&self, path: &Path) -> Result<String, std::io::Error> {
        std::fs::read_to_string(path)
    }
}