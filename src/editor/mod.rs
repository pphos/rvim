pub mod buffer;
pub mod cursor;

pub use buffer::Buffer;
pub use cursor::Position;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_module_imports() {
        // モジュールが正常にインポートできることを確認
        let _pos = Position::new(0, 0);
        let _buffer = Buffer::new();
    }
}
