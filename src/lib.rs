pub mod editor;
pub mod error;
pub mod io;
pub mod vim;

pub use editor::{Buffer, Position};
pub use error::{EditorError, Result};
pub use io::{FileSystem, Terminal, TerminalPosition, TerminalSize};
pub use vim::{Key, KeyMapper, Mode, ModeManager, VimCommand};

#[cfg(test)]
mod tests {
    #[test]
    fn test_lib_compiles() {
        // ライブラリが正常にコンパイルされることを確認
        assert!(true);
    }
}
