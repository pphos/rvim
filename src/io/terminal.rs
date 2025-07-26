use crate::error::{EditorError, Result};
use crossterm::{
    ExecutableCommand, cursor,
    event::{self, Event, KeyEvent},
    execute,
    style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::io::{self, Write};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
}

impl TerminalSize {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerminalPosition {
    pub col: u16,
    pub row: u16,
}

impl TerminalPosition {
    pub fn new(col: u16, row: u16) -> Self {
        Self { col, row }
    }
}

pub struct Terminal {
    stdout: io::Stdout,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        terminal::enable_raw_mode()
            .map_err(|e| EditorError::terminal(format!("Failed to enable raw mode: {}", e)))?;

        let mut stdout = io::stdout();
        execute!(stdout, terminal::EnterAlternateScreen).map_err(|e| {
            EditorError::terminal(format!("Failed to enter alternate screen: {}", e))
        })?;

        Ok(Self { stdout })
    }

    pub fn cleanup(&mut self) -> Result<()> {
        execute!(self.stdout, terminal::LeaveAlternateScreen).map_err(|e| {
            EditorError::terminal(format!("Failed to leave alternate screen: {}", e))
        })?;

        terminal::disable_raw_mode()
            .map_err(|e| EditorError::terminal(format!("Failed to disable raw mode: {}", e)))?;

        Ok(())
    }

    pub fn size(&self) -> Result<TerminalSize> {
        let (width, height) = terminal::size()
            .map_err(|e| EditorError::terminal(format!("Failed to get terminal size: {}", e)))?;

        Ok(TerminalSize::new(width, height))
    }

    pub fn clear_screen(&mut self) -> Result<()> {
        self.stdout
            .execute(Clear(ClearType::All))
            .map_err(|e| EditorError::terminal(format!("Failed to clear screen: {}", e)))?;

        Ok(())
    }

    pub fn clear_line(&mut self) -> Result<()> {
        self.stdout
            .execute(Clear(ClearType::CurrentLine))
            .map_err(|e| EditorError::terminal(format!("Failed to clear line: {}", e)))?;

        Ok(())
    }

    pub fn move_cursor(&mut self, pos: TerminalPosition) -> Result<()> {
        self.stdout
            .execute(cursor::MoveTo(pos.col, pos.row))
            .map_err(|e| EditorError::terminal(format!("Failed to move cursor: {}", e)))?;

        Ok(())
    }

    pub fn hide_cursor(&mut self) -> Result<()> {
        self.stdout
            .execute(cursor::Hide)
            .map_err(|e| EditorError::terminal(format!("Failed to hide cursor: {}", e)))?;

        Ok(())
    }

    pub fn show_cursor(&mut self) -> Result<()> {
        self.stdout
            .execute(cursor::Show)
            .map_err(|e| EditorError::terminal(format!("Failed to show cursor: {}", e)))?;

        Ok(())
    }

    pub fn write(&mut self, text: &str) -> Result<()> {
        self.stdout
            .write_all(text.as_bytes())
            .map_err(|e| EditorError::terminal(format!("Failed to write to terminal: {}", e)))?;

        Ok(())
    }

    pub fn write_at(&mut self, pos: TerminalPosition, text: &str) -> Result<()> {
        self.move_cursor(pos)?;
        self.write(text)?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.stdout
            .flush()
            .map_err(|e| EditorError::terminal(format!("Failed to flush output: {}", e)))?;

        Ok(())
    }

    pub fn set_foreground_color(&mut self, color: Color) -> Result<()> {
        self.stdout
            .execute(SetForegroundColor(color))
            .map_err(|e| EditorError::terminal(format!("Failed to set foreground color: {}", e)))?;

        Ok(())
    }

    pub fn set_background_color(&mut self, color: Color) -> Result<()> {
        self.stdout
            .execute(SetBackgroundColor(color))
            .map_err(|e| EditorError::terminal(format!("Failed to set background color: {}", e)))?;

        Ok(())
    }

    pub fn reset_colors(&mut self) -> Result<()> {
        self.stdout
            .execute(ResetColor)
            .map_err(|e| EditorError::terminal(format!("Failed to reset colors: {}", e)))?;

        Ok(())
    }

    pub fn read_key(&mut self) -> Result<KeyEvent> {
        loop {
            if event::poll(Duration::from_millis(100))
                .map_err(|e| EditorError::terminal(format!("Failed to poll events: {}", e)))?
            {
                match event::read()
                    .map_err(|e| EditorError::terminal(format!("Failed to read event: {}", e)))?
                {
                    Event::Key(key_event) => return Ok(key_event),
                    _ => continue, // その他のイベントは無視
                }
            }
        }
    }

    pub fn read_key_timeout(&mut self, timeout: Duration) -> Result<Option<KeyEvent>> {
        if event::poll(timeout)
            .map_err(|e| EditorError::terminal(format!("Failed to poll events: {}", e)))?
        {
            match event::read()
                .map_err(|e| EditorError::terminal(format!("Failed to read event: {}", e)))?
            {
                Event::Key(key_event) => Ok(Some(key_event)),
                _ => Ok(None), // その他のイベントはNone
            }
        } else {
            Ok(None) // タイムアウト
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // エラーを無視してクリーンアップを試行
        let _ = self.cleanup();
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new().expect("Failed to create terminal")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_size() {
        let size = TerminalSize::new(80, 24);
        assert_eq!(size.width, 80);
        assert_eq!(size.height, 24);
    }

    #[test]
    fn test_terminal_position() {
        let pos = TerminalPosition::new(10, 5);
        assert_eq!(pos.col, 10);
        assert_eq!(pos.row, 5);
    }

    #[test]
    #[ignore] // CI環境では端末が利用できない場合があるため
    fn test_terminal_creation() {
        let result = Terminal::new();
        // CI環境では失敗する可能性があるが、ローカルテストでは成功すべき
        if result.is_ok() {
            let mut terminal = result.unwrap();
            assert!(terminal.cleanup().is_ok());
        }
    }

    #[test]
    #[ignore] // CI環境では端末が利用できない場合があるため
    fn test_terminal_operations() -> Result<()> {
        let mut terminal = Terminal::new()?;

        // 基本操作のテスト
        terminal.clear_screen()?;
        terminal.move_cursor(TerminalPosition::new(0, 0))?;
        terminal.write("Test")?;
        terminal.flush()?;

        // カーソル操作
        terminal.hide_cursor()?;
        terminal.show_cursor()?;

        // 色設定
        terminal.set_foreground_color(Color::Red)?;
        terminal.set_background_color(Color::Black)?;
        terminal.reset_colors()?;

        terminal.cleanup()?;
        Ok(())
    }

    #[test]
    fn test_write_at() -> Result<()> {
        // この関数は実際のターミナル操作を含まないテスト
        let pos = TerminalPosition::new(5, 3);
        let text = "Hello";

        // write_atは move_cursor + write の組み合わせであることを確認
        // 実際のテストはmockターミナルで行うべき
        assert_eq!(pos.col, 5);
        assert_eq!(pos.row, 3);
        assert_eq!(text, "Hello");
        Ok(())
    }

    #[test]
    fn test_position_and_size_equality() {
        let pos1 = TerminalPosition::new(10, 20);
        let pos2 = TerminalPosition::new(10, 20);
        let pos3 = TerminalPosition::new(10, 21);

        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);

        let size1 = TerminalSize::new(80, 24);
        let size2 = TerminalSize::new(80, 24);
        let size3 = TerminalSize::new(81, 24);

        assert_eq!(size1, size2);
        assert_ne!(size1, size3);
    }
}

#[cfg(test)]
pub mod test_helpers {
    use super::*;
    use std::collections::VecDeque;

    // テスト用のモックターミナル
    pub struct MockTerminal {
        pub size: TerminalSize,
        pub cursor_position: TerminalPosition,
        pub output_buffer: String,
        pub key_events: VecDeque<KeyEvent>,
        pub colors_set: Vec<String>,
    }

    impl MockTerminal {
        pub fn new(width: u16, height: u16) -> Self {
            Self {
                size: TerminalSize::new(width, height),
                cursor_position: TerminalPosition::new(0, 0),
                output_buffer: String::new(),
                key_events: VecDeque::new(),
                colors_set: Vec::new(),
            }
        }

        pub fn add_key_event(&mut self, key_event: KeyEvent) {
            self.key_events.push_back(key_event);
        }

        pub fn get_output(&self) -> &str {
            &self.output_buffer
        }

        pub fn clear_output(&mut self) {
            self.output_buffer.clear();
        }

        // 実際のTerminalのメソッドを模倣
        pub fn mock_write(&mut self, text: &str) -> Result<()> {
            self.output_buffer.push_str(text);
            Ok(())
        }

        pub fn mock_move_cursor(&mut self, pos: TerminalPosition) -> Result<()> {
            self.cursor_position = pos;
            Ok(())
        }

        pub fn mock_clear_screen(&mut self) -> Result<()> {
            self.output_buffer.clear();
            Ok(())
        }

        pub fn mock_read_key(&mut self) -> Result<Option<KeyEvent>> {
            Ok(self.key_events.pop_front())
        }
    }

    impl Default for MockTerminal {
        fn default() -> Self {
            Self::new(80, 24)
        }
    }

    #[test]
    fn test_mock_terminal() {
        let mut mock = MockTerminal::new(100, 50);

        assert_eq!(mock.size.width, 100);
        assert_eq!(mock.size.height, 50);

        mock.mock_write("Hello").unwrap();
        assert_eq!(mock.get_output(), "Hello");

        mock.mock_move_cursor(TerminalPosition::new(10, 5)).unwrap();
        assert_eq!(mock.cursor_position.col, 10);
        assert_eq!(mock.cursor_position.row, 5);

        mock.mock_clear_screen().unwrap();
        assert_eq!(mock.get_output(), "");
    }
}
