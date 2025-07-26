use anyhow::Result;
use crossterm::event::KeyEvent;
use std::path::PathBuf;

use crate::{
    application::vim_bindings::{VimBindings, VimCommand},
    domain::{cursor::CursorPosition, editor_mode::EditorMode, text_buffer::TextBuffer},
    ports::{
        file_system::FileSystem,
        terminal::{Position, Terminal, TerminalSize},
    },
};

#[derive(Debug, Clone)]
pub struct EditorState {
    pub cursor: CursorPosition,
    pub buffer: TextBuffer,
    pub file_path: Option<PathBuf>,
    pub should_quit: bool,
    pub status_message: String,
    pub command_line: CommandLine,
}

#[derive(Debug, Clone)]
pub struct CommandLine {
    pub active: bool,
    pub content: String,
    pub cursor_pos: usize,
}

impl CommandLine {
    pub fn new() -> Self {
        Self {
            active: false,
            content: String::new(),
            cursor_pos: 0,
        }
    }
    
    pub fn activate(&mut self) {
        self.active = true;
        self.content = ":".to_string();
        self.cursor_pos = 1;
    }
    
    pub fn deactivate(&mut self) {
        self.active = false;
        self.content.clear();
        self.cursor_pos = 0;
    }
    
    pub fn insert_char(&mut self, ch: char) {
        self.content.insert(self.cursor_pos, ch);
        self.cursor_pos += 1;
    }
    
    pub fn delete_char(&mut self) {
        if self.cursor_pos > 1 { // Don't delete the ':'
            self.cursor_pos -= 1;
            self.content.remove(self.cursor_pos);
        }
    }
    
    pub fn get_command(&self) -> String {
        if self.content.len() > 1 {
            self.content[1..].to_string() // Remove the ':' prefix
        } else {
            String::new()
        }
    }
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            cursor: CursorPosition::new(0, 0),
            buffer: TextBuffer::new(),
            file_path: None,
            should_quit: false,
            status_message: String::new(),
            command_line: CommandLine::new(),
        }
    }

    pub fn with_file(file_path: PathBuf, content: String) -> Self {
        Self {
            cursor: CursorPosition::new(0, 0),
            buffer: TextBuffer::from_content(&content).with_file_path(file_path.clone()),
            file_path: Some(file_path),
            should_quit: false,
            status_message: String::new(),
            command_line: CommandLine::new(),
        }
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EditorService<F: FileSystem, T: Terminal> {
    state: EditorState,
    vim_bindings: VimBindings,
    file_system: F,
    terminal: T,
    terminal_size: TerminalSize,
}

impl<F: FileSystem, T: Terminal> EditorService<F, T> {
    pub fn new(file_system: F, terminal: T) -> Result<Self> {
        let terminal_size = terminal.get_size()?;

        Ok(Self {
            state: EditorState::new(),
            vim_bindings: VimBindings::new(),
            file_system,
            terminal,
            terminal_size,
        })
    }

    pub fn with_file(file_system: F, terminal: T, file_path: PathBuf) -> Result<Self> {
        let terminal_size = terminal.get_size()?;

        let content = if file_system.file_exists(&file_path) {
            file_system.read_file(&file_path)?
        } else {
            String::new()
        };

        Ok(Self {
            state: EditorState::with_file(file_path, content),
            vim_bindings: VimBindings::new(),
            file_system,
            terminal,
            terminal_size,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.terminal.clear()?;
        self.terminal.hide_cursor()?;

        while !self.state.should_quit {
            self.render()?;
            self.handle_input()?;
        }

        self.terminal.show_cursor()?;
        Ok(())
    }

    fn handle_input(&mut self) -> Result<()> {
        let key_event = self.terminal.read_key()?;
        let command = self.vim_bindings.parse_key(key_event);

        self.execute_command(command)?;
        Ok(())
    }

    fn execute_command(&mut self, command: VimCommand) -> Result<()> {
        match command {
            VimCommand::EnterCommandMode => {
                self.state.command_line.activate();
            }
            VimCommand::CommandChar(c) => {
                if self.state.command_line.active {
                    self.state.command_line.insert_char(c);
                }
            }
            VimCommand::CommandBackspace => {
                if self.state.command_line.active {
                    self.state.command_line.delete_char();
                }
            }
            VimCommand::CommandExecute => {
                if self.state.command_line.active {
                    let command = self.state.command_line.get_command();
                    self.execute_vim_command(&command)?;
                    self.state.command_line.deactivate();
                    self.vim_bindings.current_mode(); // Ensure we're back to normal mode
                }
            }
            VimCommand::ExitToNormal => {
                if self.state.command_line.active {
                    self.state.command_line.deactivate();
                }
            }
            VimCommand::Save => {
                if let Some(ref file_path) = self.state.file_path {
                    let content = self.state.buffer.to_string();
                    self.file_system.write_file(file_path, &content)?;
                    self.state.buffer.mark_saved();
                    self.state.status_message = format!("\"{}\" written", file_path.display());
                } else {
                    self.state.status_message = "No file name".to_string();
                }
            }
            VimCommand::Quit => {
                if self.state.buffer.is_modified() {
                    self.state.status_message =
                        "No write since last change (add ! to override)".to_string();
                } else {
                    self.state.should_quit = true;
                }
            }
            VimCommand::SaveAndQuit => {
                if let Some(ref file_path) = self.state.file_path {
                    let content = self.state.buffer.to_string();
                    self.file_system.write_file(file_path, &content)?;
                    self.state.buffer.mark_saved();
                }
                self.state.should_quit = true;
            }
            VimCommand::ForceQuit => {
                self.state.should_quit = true;
            }
            _ => {
                if let Err(e) = self.vim_bindings.execute_command(
                    command,
                    &mut self.state.cursor,
                    &mut self.state.buffer,
                ) {
                    self.state.status_message = e;
                }
            }
        }

        // Adjust cursor position to stay within bounds
        self.adjust_cursor_position();
        Ok(())
    }
    
    fn execute_vim_command(&mut self, command: &str) -> Result<()> {
        match command {
            "w" => {
                if let Some(ref file_path) = self.state.file_path {
                    let content = self.state.buffer.to_string();
                    self.file_system.write_file(file_path, &content)?;
                    self.state.buffer.mark_saved();
                    self.state.status_message = format!("\"{}\" written", file_path.display());
                } else {
                    self.state.status_message = "No file name".to_string();
                }
            }
            "q" => {
                if self.state.buffer.is_modified() {
                    self.state.status_message = "No write since last change (add ! to override)".to_string();
                } else {
                    self.state.should_quit = true;
                }
            }
            "wq" => {
                if let Some(ref file_path) = self.state.file_path {
                    let content = self.state.buffer.to_string();
                    self.file_system.write_file(file_path, &content)?;
                    self.state.buffer.mark_saved();
                }
                self.state.should_quit = true;
            }
            "q!" => {
                self.state.should_quit = true;
            }
            _ => {
                self.state.status_message = format!("Unknown command: {}", command);
            }
        }
        Ok(())
    }

    fn adjust_cursor_position(&mut self) {
        let buffer_height = self.state.buffer.line_count();
        if buffer_height > 0 && self.state.cursor.row >= buffer_height {
            self.state.cursor.row = buffer_height - 1;
        }

        let line_length = self.state.buffer.get_line_length(self.state.cursor.row);
        
        // In insert mode, cursor can be at the end of line
        let is_insert_mode = matches!(self.vim_bindings.current_mode(), EditorMode::Insert);
        let max_col = if is_insert_mode {
            line_length
        } else {
            line_length.saturating_sub(1).max(0)
        };
        
        if self.state.cursor.col > max_col {
            self.state.cursor.col = max_col;
        }
    }

    fn render(&mut self) -> Result<()> {
        self.terminal.clear()?;

        // Render buffer content
        self.render_buffer()?;

        // Render status line
        self.render_status_line()?;

        // Keep cursor hidden and use highlighting for position indication
        self.terminal.flush()?;
        Ok(())
    }

    fn render_buffer(&mut self) -> Result<()> {
        let buffer_height = (self.terminal_size.height as usize).saturating_sub(2); // Leave space for status line

        for row in 0..buffer_height {
            let buffer_row = row;

            if buffer_row < self.state.buffer.line_count() {
                if let Some(line) = self.state.buffer.get_line(buffer_row) {
                    let display_line = if line.len() > self.terminal_size.width as usize {
                        line[..self.terminal_size.width as usize].to_string()
                    } else {
                        line.clone()
                    };
                    
                    // Render line with cursor highlighting
                    if buffer_row == self.state.cursor.row {
                        let cursor_col = self.state.cursor.col;
                        self.render_line_with_cursor(&display_line, cursor_col)?;
                    } else {
                        self.terminal.write(&display_line)?;
                    }
                }
            } else {
                // Empty line indicator
                if buffer_row == self.state.cursor.row && self.state.cursor.col == 0 {
                    self.terminal.write_highlighted(" ")?;
                } else {
                    self.terminal.write("~")?;
                }
            }

            if row < buffer_height - 1 {
                self.terminal.write("\r\n")?;
            }
        }

        Ok(())
    }
    
    fn render_line_with_cursor(&mut self, line: &str, cursor_col: usize) -> Result<()> {
        let chars: Vec<char> = line.chars().collect();
        
        for (col, &ch) in chars.iter().enumerate() {
            if col == cursor_col {
                self.terminal.write_highlighted(&ch.to_string())?;
            } else {
                self.terminal.write(&ch.to_string())?;
            }
        }
        
        // If cursor is at the end of line, show a highlighted space
        if cursor_col >= chars.len() {
            self.terminal.write_highlighted(" ")?;
        }
        
        Ok(())
    }

    fn render_status_line(&mut self) -> Result<()> {
        let status_y = self.terminal_size.height.saturating_sub(1);
        self.terminal.move_cursor(Position { x: 0, y: status_y })?;

        // If command line is active, show the command being typed
        if self.state.command_line.active {
            self.render_command_line()?;
            return Ok(());
        }

        let mode_str = match self.vim_bindings.current_mode() {
            EditorMode::Normal => "",
            EditorMode::Insert => "-- INSERT --",
            EditorMode::Visual => "-- VISUAL --",
            EditorMode::Command => "-- COMMAND --",
        };

        let file_info = if let Some(ref path) = self.state.file_path {
            format!(" \"{}\"", path.display())
        } else {
            " [No Name]".to_string()
        };

        let modified_indicator = if self.state.buffer.is_modified() {
            " [+]"
        } else {
            ""
        };

        let position_info = format!(
            " {}:{}",
            self.state.cursor.row + 1,
            self.state.cursor.col + 1
        );

        let status_line = if !self.state.status_message.is_empty() {
            self.state.status_message.clone()
        } else {
            format!(
                "{}{}{}{}",
                mode_str, file_info, modified_indicator, position_info
            )
        };

        // Clear the status message after displaying it
        if !self.state.status_message.is_empty() {
            self.state.status_message.clear();
        }

        self.terminal.write(&status_line)?;
        Ok(())
    }
    
    fn render_command_line(&mut self) -> Result<()> {
        // Clear the line first
        self.terminal.write(&" ".repeat(self.terminal_size.width as usize))?;
        let status_y = self.terminal_size.height.saturating_sub(1);
        self.terminal.move_cursor(Position { x: 0, y: status_y })?;
        
        // Render command line with cursor
        let command_chars: Vec<char> = self.state.command_line.content.chars().collect();
        
        for (i, &ch) in command_chars.iter().enumerate() {
            if i == self.state.command_line.cursor_pos {
                self.terminal.write_highlighted(&ch.to_string())?;
            } else {
                self.terminal.write(&ch.to_string())?;
            }
        }
        
        // If cursor is at the end, show highlighted space
        if self.state.command_line.cursor_pos >= command_chars.len() {
            self.terminal.write_highlighted(" ")?;
        }
        
        Ok(())
    }

    pub fn state(&self) -> &EditorState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{file_system::mock::MockTestFileSystem, terminal::mock::MockTestTerminal};
    use mockall::predicate::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_editor_state_new() {
        let state = EditorState::new();
        assert_eq!(state.cursor.row, 0);
        assert_eq!(state.cursor.col, 0);
        assert!(state.file_path.is_none());
        assert!(!state.should_quit);
        assert!(state.status_message.is_empty());
    }

    #[test]
    fn test_editor_state_with_file() {
        let file_path = PathBuf::from("test.txt");
        let content = "Hello\nWorld".to_string();

        let state = EditorState::with_file(file_path.clone(), content);
        assert_eq!(state.file_path, Some(file_path));
        assert_eq!(state.buffer.line_count(), 2);
    }

    #[test]
    fn test_editor_service_new() {
        let mut file_system = MockTestFileSystem::new();
        let mut terminal = MockTestTerminal::new();

        terminal.expect_get_size().times(1).returning(|| {
            Ok(TerminalSize {
                width: 80,
                height: 24,
            })
        });

        let service = EditorService::new(file_system, terminal);
        assert!(service.is_ok());
    }

    #[test]
    fn test_editor_service_with_existing_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let content = "Hello, World!";
        fs::write(&file_path, content).unwrap();

        let mut file_system = MockTestFileSystem::new();
        let mut terminal = MockTestTerminal::new();

        file_system
            .expect_file_exists()
            .with(eq(file_path.clone()))
            .times(1)
            .returning(|_| true);

        file_system
            .expect_read_file()
            .with(eq(file_path.clone()))
            .times(1)
            .returning(|_| Ok("Hello, World!".to_string()));

        terminal.expect_get_size().times(1).returning(|| {
            Ok(TerminalSize {
                width: 80,
                height: 24,
            })
        });

        let service = EditorService::with_file(file_system, terminal, file_path);
        assert!(service.is_ok());

        let service = service.unwrap();
        assert_eq!(
            service.state.buffer.get_line(0),
            Some(&"Hello, World!".to_string())
        );
    }

    #[test]
    fn test_editor_service_with_nonexistent_file() {
        let file_path = PathBuf::from("nonexistent.txt");

        let mut file_system = MockTestFileSystem::new();
        let mut terminal = MockTestTerminal::new();

        file_system
            .expect_file_exists()
            .with(eq(file_path.clone()))
            .times(1)
            .returning(|_| false);

        terminal.expect_get_size().times(1).returning(|| {
            Ok(TerminalSize {
                width: 80,
                height: 24,
            })
        });

        let service = EditorService::with_file(file_system, terminal, file_path);
        assert!(service.is_ok());

        let service = service.unwrap();
        assert_eq!(service.state.buffer.line_count(), 1);
        assert_eq!(service.state.buffer.get_line(0), Some(&String::new()));
    }

    #[test]
    fn test_execute_save_command() {
        let file_path = PathBuf::from("test.txt");
        let mut file_system = MockTestFileSystem::new();
        let mut terminal = MockTestTerminal::new();

        terminal.expect_get_size().times(1).returning(|| {
            Ok(TerminalSize {
                width: 80,
                height: 24,
            })
        });

        file_system
            .expect_write_file()
            .with(eq(file_path.clone()), eq(""))
            .times(1)
            .returning(|_, _| Ok(()));

        let mut service = EditorService::new(file_system, terminal).unwrap();
        service.state.file_path = Some(file_path.clone());

        let result = service.execute_command(VimCommand::Save);
        assert!(result.is_ok());
        assert!(!service.state.buffer.is_modified());
    }

    #[test]
    fn test_execute_quit_command_with_unsaved_changes() {
        let mut file_system = MockTestFileSystem::new();
        let mut terminal = MockTestTerminal::new();

        terminal.expect_get_size().times(1).returning(|| {
            Ok(TerminalSize {
                width: 80,
                height: 24,
            })
        });

        let mut service = EditorService::new(file_system, terminal).unwrap();
        service.state.buffer.insert_char(0, 0, 'X').unwrap(); // Make buffer modified

        let result = service.execute_command(VimCommand::Quit);
        assert!(result.is_ok());
        assert!(!service.state.should_quit);
        assert!(!service.state.status_message.is_empty());
    }

    #[test]
    fn test_execute_force_quit_command() {
        let mut file_system = MockTestFileSystem::new();
        let mut terminal = MockTestTerminal::new();

        terminal.expect_get_size().times(1).returning(|| {
            Ok(TerminalSize {
                width: 80,
                height: 24,
            })
        });

        let mut service = EditorService::new(file_system, terminal).unwrap();
        service.state.buffer.insert_char(0, 0, 'X').unwrap(); // Make buffer modified

        let result = service.execute_command(VimCommand::ForceQuit);
        assert!(result.is_ok());
        assert!(service.state.should_quit);
    }

    #[test]
    fn test_adjust_cursor_position() {
        let file_system = MockTestFileSystem::new();
        let mut terminal = MockTestTerminal::new();

        terminal.expect_get_size().times(1).returning(|| {
            Ok(TerminalSize {
                width: 80,
                height: 24,
            })
        });

        let mut service = EditorService::new(file_system, terminal).unwrap();
        service.state.cursor.row = 100; // Out of bounds
        service.state.cursor.col = 100; // Out of bounds

        service.adjust_cursor_position();

        assert_eq!(service.state.cursor.row, 0); // Adjusted to valid position
        assert_eq!(service.state.cursor.col, 0); // Adjusted to valid position
    }

    #[test]
    fn test_command_line_functionality() {
        let file_system = MockTestFileSystem::new();
        let mut terminal = MockTestTerminal::new();
        
        terminal.expect_get_size().times(1).returning(|| {
            Ok(TerminalSize {
                width: 80,
                height: 24,
            })
        });

        let mut service = EditorService::new(file_system, terminal).unwrap();
        
        // Test command line activation
        let result = service.execute_command(VimCommand::EnterCommandMode);
        assert!(result.is_ok());
        assert!(service.state.command_line.active);
        assert_eq!(service.state.command_line.content, ":");
        assert_eq!(service.state.command_line.cursor_pos, 1);
        
        // Test character input
        let result = service.execute_command(VimCommand::CommandChar('w'));
        assert!(result.is_ok());
        assert_eq!(service.state.command_line.content, ":w");
        assert_eq!(service.state.command_line.cursor_pos, 2);
        
        // Test backspace
        let result = service.execute_command(VimCommand::CommandBackspace);
        assert!(result.is_ok());
        assert_eq!(service.state.command_line.content, ":");
        assert_eq!(service.state.command_line.cursor_pos, 1);
        
        // Test escape
        let result = service.execute_command(VimCommand::ExitToNormal);
        assert!(result.is_ok());
        assert!(!service.state.command_line.active);
    }
}
