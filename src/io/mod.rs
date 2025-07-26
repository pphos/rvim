pub mod fs;
pub mod terminal;

pub use fs::FileSystem;
pub use terminal::{Terminal, TerminalPosition, TerminalSize};

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_io_module_imports() {
        // モジュールが正常にインポートできることを確認
        let _fs = FileSystem::new();
        let _size = TerminalSize::new(80, 24);
        let _pos = TerminalPosition::new(0, 0);
    }

    #[test]
    fn test_filesystem_integration() -> crate::error::Result<()> {
        // 実際のファイルシステム操作のテスト
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("integration_test.txt");
        let content = "Integration test content";

        // ファイル書き込み
        FileSystem::write_file(&file_path, content)?;

        // ファイル存在確認
        assert!(FileSystem::file_exists(&file_path));

        // ファイル読み込み
        let read_content = FileSystem::read_file(&file_path)?;
        assert_eq!(read_content, content);

        // ファイルサイズ確認
        let size = FileSystem::get_file_size(&file_path)?;
        assert_eq!(size, content.len() as u64);

        // バックアップ作成
        FileSystem::create_backup(&file_path)?;
        let backup_path = file_path.with_extension("txt.bak");
        assert!(FileSystem::file_exists(&backup_path));

        let backup_content = FileSystem::read_file(&backup_path)?;
        assert_eq!(backup_content, content);

        Ok(())
    }

    #[test]
    fn test_terminal_structures() {
        let size = TerminalSize::new(120, 30);
        assert_eq!(size.width, 120);
        assert_eq!(size.height, 30);

        let pos = TerminalPosition::new(15, 10);
        assert_eq!(pos.col, 15);
        assert_eq!(pos.row, 10);

        // Clone and PartialEq implementations
        let size2 = size;
        assert_eq!(size, size2);

        let pos2 = pos;
        assert_eq!(pos, pos2);
    }

    #[test]
    fn test_error_handling() {
        // 存在しないファイルの読み込み
        let result = FileSystem::read_file("/nonexistent/path/file.txt");
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(matches!(e, crate::error::EditorError::FileNotFound { .. }));
        }
    }

    #[test]
    fn test_file_operations_with_paths() -> crate::error::Result<()> {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // 異なる形式のパスでのテスト
        let file1 = temp_dir.path().join("file1.txt");
        let file2: PathBuf = temp_dir.path().join("file2.txt").into();
        let file3 = temp_dir.path().join("subdirectory").join("file3.txt");

        let content = "Test content";

        // String path
        FileSystem::write_file(&file1, content)?;
        assert!(FileSystem::file_exists(&file1));

        // PathBuf
        FileSystem::write_file(&file2, content)?;
        assert!(FileSystem::file_exists(&file2));

        // ネストしたディレクトリ（自動作成される）
        FileSystem::write_file(&file3, content)?;
        assert!(FileSystem::file_exists(&file3));

        // すべてのファイルの内容確認
        assert_eq!(FileSystem::read_file(&file1)?, content);
        assert_eq!(FileSystem::read_file(&file2)?, content);
        assert_eq!(FileSystem::read_file(&file3)?, content);

        Ok(())
    }
}
