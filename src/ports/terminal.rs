use anyhow::Result;
use crossterm::event::KeyEvent;

#[derive(Debug, Clone, PartialEq)]
pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

pub trait Terminal {
    fn read_key(&self) -> Result<KeyEvent>;
    fn write(&self, content: &str) -> Result<()>;
    fn write_highlighted(&self, content: &str) -> Result<()>;
    fn clear(&self) -> Result<()>;
    fn clear_line(&self) -> Result<()>;
    fn move_cursor(&self, position: Position) -> Result<()>;
    fn hide_cursor(&self) -> Result<()>;
    fn show_cursor(&self) -> Result<()>;
    fn get_size(&self) -> Result<TerminalSize>;
    fn enable_raw_mode(&self) -> Result<()>;
    fn disable_raw_mode(&self) -> Result<()>;
    fn flush(&self) -> Result<()>;
}

#[cfg(test)]
pub mod mock {
    use super::*;
    use mockall::mock;

    mock! {
        pub TestTerminal {}

        impl Terminal for TestTerminal {
            fn read_key(&self) -> Result<KeyEvent>;
            fn write(&self, content: &str) -> Result<()>;
            fn write_highlighted(&self, content: &str) -> Result<()>;
            fn clear(&self) -> Result<()>;
            fn clear_line(&self) -> Result<()>;
            fn move_cursor(&self, position: Position) -> Result<()>;
            fn hide_cursor(&self) -> Result<()>;
            fn show_cursor(&self) -> Result<()>;
            fn get_size(&self) -> Result<TerminalSize>;
            fn enable_raw_mode(&self) -> Result<()>;
            fn disable_raw_mode(&self) -> Result<()>;
            fn flush(&self) -> Result<()>;
        }
    }
}
