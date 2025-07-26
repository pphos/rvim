use anyhow::Result;
use std::path::Path;

pub trait FileSystem {
    fn read_file(&self, path: &Path) -> Result<String>;
    fn write_file(&self, path: &Path, content: &str) -> Result<()>;
    fn file_exists(&self, path: &Path) -> bool;
    fn create_backup(&self, path: &Path) -> Result<()>;
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use mockall::mock;

    mock! {
        pub TestFileSystem {}

        impl FileSystem for TestFileSystem {
            fn read_file(&self, path: &Path) -> Result<String>;
            fn write_file(&self, path: &Path, content: &str) -> Result<()>;
            fn file_exists(&self, path: &Path) -> bool;
            fn create_backup(&self, path: &Path) -> Result<()>;
        }
    }
}
