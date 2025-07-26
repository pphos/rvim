use crate::ports::file_system::FileSystem;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct StdFileSystem;

impl StdFileSystem {
    pub fn new() -> Self {
        Self
    }
}

impl FileSystem for StdFileSystem {
    fn read_file(&self, path: &Path) -> Result<String> {
        fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path.display()))
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        fs::write(path, content)
            .with_context(|| format!("Failed to write file: {}", path.display()))
    }

    fn file_exists(&self, path: &Path) -> bool {
        path.exists() && path.is_file()
    }

    fn create_backup(&self, path: &Path) -> Result<()> {
        if !self.file_exists(path) {
            return Ok(());
        }

        let backup_path = format!("{}.bak", path.display());
        let backup_path = Path::new(&backup_path);

        fs::copy(path, backup_path).with_context(|| {
            format!(
                "Failed to create backup: {} -> {}",
                path.display(),
                backup_path.display()
            )
        })?;

        Ok(())
    }
}

impl Default for StdFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_read_existing_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let content = "Hello, World!";
        fs::write(&file_path, content).unwrap();

        let fs = StdFileSystem::new();
        let result = fs.read_file(&file_path).unwrap();

        assert_eq!(result, content);
    }

    #[test]
    fn test_read_nonexistent_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("nonexistent.txt");

        let fs = StdFileSystem::new();
        let result = fs.read_file(&file_path);

        assert!(result.is_err());
    }

    #[test]
    fn test_write_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("output.txt");
        let content = "Test content";

        let fs = StdFileSystem::new();
        fs.write_file(&file_path, content).unwrap();

        let written_content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(written_content, content);
    }

    #[test]
    fn test_file_exists() {
        let dir = tempdir().unwrap();
        let existing_file = dir.path().join("exists.txt");
        let nonexistent_file = dir.path().join("not_exists.txt");

        fs::write(&existing_file, "content").unwrap();

        let fs = StdFileSystem::new();
        assert!(fs.file_exists(&existing_file));
        assert!(!fs.file_exists(&nonexistent_file));
    }

    #[test]
    fn test_create_backup_existing_file() {
        let dir = tempdir().unwrap();
        let original_file = dir.path().join("original.txt");
        let backup_file = dir.path().join("original.txt.bak");
        let content = "Original content";

        fs::write(&original_file, content).unwrap();

        let fs = StdFileSystem::new();
        fs.create_backup(&original_file).unwrap();

        assert!(backup_file.exists());
        let backup_content = std::fs::read_to_string(&backup_file).unwrap();
        assert_eq!(backup_content, content);
    }

    #[test]
    fn test_create_backup_nonexistent_file() {
        let dir = tempdir().unwrap();
        let nonexistent_file = dir.path().join("not_exists.txt");

        let fs = StdFileSystem::new();
        let result = fs.create_backup(&nonexistent_file);

        assert!(result.is_ok());
    }

    #[test]
    fn test_default() {
        let _fs = StdFileSystem::default();
    }
}
