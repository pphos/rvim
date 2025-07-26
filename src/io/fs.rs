use crate::error::{EditorError, Result};
use std::fs;
use std::path::Path;

pub struct FileSystem;

impl FileSystem {
    pub fn new() -> Self {
        Self
    }

    pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(EditorError::file_not_found(path));
        }

        match fs::read_to_string(path) {
            Ok(content) => Ok(content),
            Err(e) => match e.kind() {
                std::io::ErrorKind::PermissionDenied => Err(EditorError::permission_denied(path)),
                _ => Err(EditorError::Io(e)),
            },
        }
    }

    pub fn write_file<P: AsRef<Path>>(path: P, content: &str) -> Result<()> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        match fs::write(path, content) {
            Ok(()) => Ok(()),
            Err(e) => match e.kind() {
                std::io::ErrorKind::PermissionDenied => Err(EditorError::permission_denied(path)),
                _ => Err(EditorError::Io(e)),
            },
        }
    }

    pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }

    pub fn create_backup<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(()); // バックアップ対象が存在しない場合はOK
        }

        let backup_path = path.with_extension(format!(
            "{}.bak",
            path.extension().and_then(|s| s.to_str()).unwrap_or("")
        ));

        match fs::copy(path, &backup_path) {
            Ok(_) => Ok(()),
            Err(e) => match e.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    Err(EditorError::permission_denied(&backup_path))
                }
                _ => Err(EditorError::Io(e)),
            },
        }
    }

    pub fn get_file_size<P: AsRef<Path>>(path: P) -> Result<u64> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(EditorError::file_not_found(path));
        }

        match fs::metadata(path) {
            Ok(metadata) => Ok(metadata.len()),
            Err(e) => Err(EditorError::Io(e)),
        }
    }
}

impl Default for FileSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::{NamedTempFile, TempDir};

    #[test]
    fn test_file_system_new() {
        let fs = FileSystem::new();
        // 単純に作成できることを確認
        assert_eq!(std::mem::size_of_val(&fs), 0); // ZSTであることを確認
    }

    #[test]
    fn test_file_system_default() {
        let fs = FileSystem::default();
        assert_eq!(std::mem::size_of_val(&fs), 0);
    }

    #[test]
    fn test_read_existing_file() -> Result<()> {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "Hello, World!";
        std::io::Write::write_all(&mut temp_file, content.as_bytes()).unwrap();

        let result = FileSystem::read_file(temp_file.path())?;
        assert_eq!(result, content);
        Ok(())
    }

    #[test]
    fn test_read_nonexistent_file() {
        let path = PathBuf::from("/nonexistent/file.txt");
        let result = FileSystem::read_file(&path);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EditorError::FileNotFound { .. }
        ));
    }

    #[test]
    fn test_write_file() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "Test content";

        FileSystem::write_file(&file_path, content)?;

        let read_content = FileSystem::read_file(&file_path)?;
        assert_eq!(read_content, content);
        Ok(())
    }

    #[test]
    fn test_write_file_create_directory() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("subdir").join("test.txt");
        let content = "Test content";

        FileSystem::write_file(&file_path, content)?;

        let read_content = FileSystem::read_file(&file_path)?;
        assert_eq!(read_content, content);
        Ok(())
    }

    #[test]
    fn test_file_exists() -> Result<()> {
        let temp_file = NamedTempFile::new().unwrap();

        assert!(FileSystem::file_exists(temp_file.path()));
        assert!(!FileSystem::file_exists("/nonexistent/file.txt"));
        Ok(())
    }

    #[test]
    fn test_create_backup_existing_file() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let content = "Original content";

        FileSystem::write_file(&file_path, content)?;
        FileSystem::create_backup(&file_path)?;

        let backup_path = temp_dir.path().join("test.txt.bak");
        assert!(FileSystem::file_exists(&backup_path));

        let backup_content = FileSystem::read_file(&backup_path)?;
        assert_eq!(backup_content, content);
        Ok(())
    }

    #[test]
    fn test_create_backup_with_extension() -> Result<()> {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");
        let content = "fn main() {}";

        FileSystem::write_file(&file_path, content)?;
        FileSystem::create_backup(&file_path)?;

        let backup_path = temp_dir.path().join("test.rs.bak");
        assert!(FileSystem::file_exists(&backup_path));
        Ok(())
    }

    #[test]
    fn test_create_backup_nonexistent_file() -> Result<()> {
        let path = PathBuf::from("/nonexistent/file.txt");
        let result = FileSystem::create_backup(&path);

        // 存在しないファイルのバックアップは成功とする
        assert!(result.is_ok());
        Ok(())
    }

    #[test]
    fn test_get_file_size() -> Result<()> {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "Hello, World!";
        std::io::Write::write_all(&mut temp_file, content.as_bytes()).unwrap();

        let size = FileSystem::get_file_size(temp_file.path())?;
        assert_eq!(size, content.len() as u64);
        Ok(())
    }

    #[test]
    fn test_get_file_size_nonexistent() {
        let path = PathBuf::from("/nonexistent/file.txt");
        let result = FileSystem::get_file_size(&path);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            EditorError::FileNotFound { .. }
        ));
    }

    #[test]
    fn test_get_file_size_empty_file() -> Result<()> {
        let temp_file = NamedTempFile::new().unwrap();

        let size = FileSystem::get_file_size(temp_file.path())?;
        assert_eq!(size, 0);
        Ok(())
    }
}

#[cfg(test)]
pub mod test_helpers {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    pub struct TestFileHelper {
        pub temp_dir: TempDir,
    }

    impl TestFileHelper {
        pub fn new() -> Self {
            Self {
                temp_dir: TempDir::new().unwrap(),
            }
        }

        pub fn create_test_file(&self, name: &str, content: &str) -> PathBuf {
            let file_path = self.temp_dir.path().join(name);
            FileSystem::write_file(&file_path, content).unwrap();
            file_path
        }

        pub fn get_path(&self, name: &str) -> PathBuf {
            self.temp_dir.path().join(name)
        }
    }

    impl Default for TestFileHelper {
        fn default() -> Self {
            Self::new()
        }
    }

    #[test]
    fn test_file_helper() {
        let helper = TestFileHelper::new();
        let file_path = helper.create_test_file("test.txt", "test content");

        assert!(FileSystem::file_exists(&file_path));
        let content = FileSystem::read_file(&file_path).unwrap();
        assert_eq!(content, "test content");
    }
}
