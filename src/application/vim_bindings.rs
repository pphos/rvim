use crate::domain::{
    cursor::CursorPosition,
    editor_mode::{EditorMode, ModeManager},
    text_buffer::TextBuffer,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum VimCommand {
    // Movement commands
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveWordForward,
    MoveWordBackward,
    MoveLineStart,
    MoveLineEnd,
    MoveBufferStart,
    MoveBufferEnd,

    // Mode transitions
    EnterInsertMode,
    EnterInsertModeAfter,
    EnterInsertModeNewLine,
    EnterInsertModeNewLineAbove,
    EnterVisualMode,
    EnterCommandMode,
    ExitToNormal,

    // Edit commands
    InsertChar(char),
    DeleteChar,
    Backspace,
    DeleteLine,
    Undo,
    Redo,

    // File operations
    Save,
    Quit,
    SaveAndQuit,
    ForceQuit,

    // Special
    PageUp,
    PageDown,
    
    // Command line
    CommandChar(char),
    CommandBackspace,
    CommandExecute,
    Unknown,
}

pub struct VimBindings {
    mode_manager: ModeManager,
}

impl VimBindings {
    pub fn new() -> Self {
        Self {
            mode_manager: ModeManager::new(),
        }
    }

    pub fn current_mode(&self) -> &EditorMode {
        self.mode_manager.current_mode()
    }

    pub fn parse_key(&mut self, key_event: KeyEvent) -> VimCommand {
        match self.mode_manager.current_mode() {
            EditorMode::Normal => self.parse_normal_mode_key(key_event),
            EditorMode::Insert => self.parse_insert_mode_key(key_event),
            EditorMode::Visual => self.parse_visual_mode_key(key_event),
            EditorMode::Command => self.parse_command_mode_key(key_event),
        }
    }

    fn parse_normal_mode_key(&mut self, key_event: KeyEvent) -> VimCommand {
        match key_event {
            // Movement
            KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimCommand::MoveLeft,
            KeyEvent {
                code: KeyCode::Char('j'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimCommand::MoveDown,
            KeyEvent {
                code: KeyCode::Char('k'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimCommand::MoveUp,
            KeyEvent {
                code: KeyCode::Char('l'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimCommand::MoveRight,
            KeyEvent {
                code: KeyCode::Char('w'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimCommand::MoveWordForward,
            KeyEvent {
                code: KeyCode::Char('b'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimCommand::MoveWordBackward,
            KeyEvent {
                code: KeyCode::Char('0'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimCommand::MoveLineStart,
            KeyEvent {
                code: KeyCode::Char('$'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimCommand::MoveLineEnd,

            // Special movements
            KeyEvent {
                code: KeyCode::Char('g'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                // TODO: Handle 'gg' for buffer start
                VimCommand::MoveBufferStart
            }
            KeyEvent {
                code: KeyCode::Char('G'),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => VimCommand::MoveBufferEnd,

            // Mode transitions
            KeyEvent {
                code: KeyCode::Char('i'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.mode_manager.transition_with_key('i');
                VimCommand::EnterInsertMode
            }
            KeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.mode_manager.transition_with_key('a');
                VimCommand::EnterInsertModeAfter
            }
            KeyEvent {
                code: KeyCode::Char('o'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.mode_manager.transition_with_key('o');
                VimCommand::EnterInsertModeNewLine
            }
            KeyEvent {
                code: KeyCode::Char('O'),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => {
                self.mode_manager.transition_with_key('O');
                VimCommand::EnterInsertModeNewLineAbove
            }
            KeyEvent {
                code: KeyCode::Char('v'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.mode_manager.transition_with_key('v');
                VimCommand::EnterVisualMode
            }
            KeyEvent {
                code: KeyCode::Char(':'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                self.mode_manager.transition_with_key(':');
                VimCommand::EnterCommandMode
            }

            // Edit commands
            KeyEvent {
                code: KeyCode::Char('x'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimCommand::DeleteChar,
            KeyEvent {
                code: KeyCode::Char('d'),
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                // TODO: Handle 'dd' for delete line
                VimCommand::DeleteLine
            }
            KeyEvent {
                code: KeyCode::Char('u'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimCommand::Undo,
            KeyEvent {
                code: KeyCode::Char('r'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => VimCommand::Redo,

            // Page navigation
            KeyEvent {
                code: KeyCode::Char('f'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => VimCommand::PageDown,
            KeyEvent {
                code: KeyCode::Char('b'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => VimCommand::PageUp,

            _ => VimCommand::Unknown,
        }
    }

    fn parse_insert_mode_key(&mut self, key_event: KeyEvent) -> VimCommand {
        match key_event {
            KeyEvent {
                code: KeyCode::Esc, ..
            } => {
                self.mode_manager.transition_with_escape();
                VimCommand::ExitToNormal
            }
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimCommand::InsertChar(c),
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => VimCommand::InsertChar(c),
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => VimCommand::Backspace,
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => VimCommand::InsertChar('\n'),
            KeyEvent {
                code: KeyCode::Tab, ..
            } => VimCommand::InsertChar('\t'),
            _ => VimCommand::Unknown,
        }
    }

    fn parse_visual_mode_key(&mut self, key_event: KeyEvent) -> VimCommand {
        match key_event {
            KeyEvent {
                code: KeyCode::Esc, ..
            } => {
                self.mode_manager.transition_with_escape();
                VimCommand::ExitToNormal
            }
            // Movement commands (same as normal mode)
            KeyEvent {
                code: KeyCode::Char('h'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimCommand::MoveLeft,
            KeyEvent {
                code: KeyCode::Char('j'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimCommand::MoveDown,
            KeyEvent {
                code: KeyCode::Char('k'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimCommand::MoveUp,
            KeyEvent {
                code: KeyCode::Char('l'),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimCommand::MoveRight,
            _ => VimCommand::Unknown,
        }
    }

    fn parse_command_mode_key(&mut self, key_event: KeyEvent) -> VimCommand {
        match key_event {
            KeyEvent {
                code: KeyCode::Esc, ..
            } => {
                self.mode_manager.transition_with_escape();
                VimCommand::ExitToNormal
            }
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => {
                VimCommand::CommandExecute
            }
            KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => {
                VimCommand::CommandBackspace
            }
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
                ..
            } => VimCommand::CommandChar(c),
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::SHIFT,
                ..
            } => VimCommand::CommandChar(c),
            _ => VimCommand::Unknown,
        }
    }

    pub fn execute_command(
        &self,
        command: VimCommand,
        cursor: &mut CursorPosition,
        buffer: &mut TextBuffer,
    ) -> Result<(), String> {
        match command {
            // Movement commands
            VimCommand::MoveLeft => {
                cursor.move_left();
                Ok(())
            }
            VimCommand::MoveRight => {
                let line_length = buffer.get_line_length(cursor.row);
                let original_col = cursor.col;
                cursor.move_right(line_length);
                // Vim doesn't move past end of line in normal mode
                if cursor.col > line_length && line_length > 0 {
                    cursor.col = line_length.saturating_sub(1);
                }
                Ok(())
            }
            VimCommand::MoveUp => {
                cursor.move_up();
                Ok(())
            }
            VimCommand::MoveDown => {
                cursor.move_down(buffer.line_count());
                Ok(())
            }
            VimCommand::MoveLineStart => {
                cursor.move_to_line_start();
                Ok(())
            }
            VimCommand::MoveLineEnd => {
                let line_length = buffer.get_line_length(cursor.row);
                cursor.move_to_line_end(line_length);
                Ok(())
            }
            VimCommand::MoveBufferStart => {
                cursor.move_to_buffer_start();
                Ok(())
            }
            VimCommand::MoveBufferEnd => {
                let last_line_length = if buffer.line_count() > 0 {
                    buffer.get_line_length(buffer.line_count() - 1)
                } else {
                    0
                };
                cursor.move_to_buffer_end(buffer.line_count(), last_line_length);
                Ok(())
            }

            // Edit commands
            VimCommand::InsertChar(c) => {
                if c == '\n' {
                    buffer.insert_line(cursor.row + 1)?;
                    cursor.move_down(buffer.line_count());
                    cursor.move_to_line_start();
                } else {
                    buffer.insert_char(cursor.row, cursor.col, c)?;
                    cursor.move_right(buffer.get_line_length(cursor.row));
                }
                Ok(())
            }
            VimCommand::DeleteChar => {
                buffer.delete_char(cursor.row, cursor.col)?;
                Ok(())
            }
            VimCommand::Backspace => {
                if cursor.col > 0 {
                    cursor.move_left();
                    buffer.delete_char(cursor.row, cursor.col)?;
                } else if cursor.row > 0 {
                    // Join with previous line
                    let current_line = buffer.get_line(cursor.row).unwrap_or(&String::new()).clone();
                    let prev_line_length = buffer.get_line_length(cursor.row - 1);
                    
                    buffer.delete_line(cursor.row)?;
                    cursor.row -= 1;
                    cursor.col = prev_line_length;
                    
                    // Insert the content of the deleted line to the end of previous line
                    for ch in current_line.chars() {
                        buffer.insert_char(cursor.row, cursor.col, ch)?;
                        cursor.col += 1;
                    }
                }
                Ok(())
            }
            VimCommand::DeleteLine => {
                buffer.delete_line(cursor.row)?;
                cursor.move_to_line_start();
                Ok(())
            }
            VimCommand::Undo => {
                buffer.undo()?;
                Ok(())
            }
            VimCommand::Redo => {
                buffer.redo()?;
                Ok(())
            }

            // Mode transitions and other commands
            _ => Ok(()),
        }
    }
}

impl Default for VimBindings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn create_key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_vim_bindings_new() {
        let bindings = VimBindings::new();
        assert_eq!(bindings.current_mode(), &EditorMode::Normal);
    }

    #[test]
    fn test_normal_mode_movement_keys() {
        let mut bindings = VimBindings::new();

        let h_key = create_key_event(KeyCode::Char('h'), KeyModifiers::NONE);
        assert_eq!(bindings.parse_key(h_key), VimCommand::MoveLeft);

        let j_key = create_key_event(KeyCode::Char('j'), KeyModifiers::NONE);
        assert_eq!(bindings.parse_key(j_key), VimCommand::MoveDown);

        let k_key = create_key_event(KeyCode::Char('k'), KeyModifiers::NONE);
        assert_eq!(bindings.parse_key(k_key), VimCommand::MoveUp);

        let l_key = create_key_event(KeyCode::Char('l'), KeyModifiers::NONE);
        assert_eq!(bindings.parse_key(l_key), VimCommand::MoveRight);
    }

    #[test]
    fn test_normal_to_insert_mode_transition() {
        let mut bindings = VimBindings::new();

        let i_key = create_key_event(KeyCode::Char('i'), KeyModifiers::NONE);
        let command = bindings.parse_key(i_key);

        assert_eq!(command, VimCommand::EnterInsertMode);
        assert_eq!(bindings.current_mode(), &EditorMode::Insert);
    }

    #[test]
    fn test_insert_mode_character_input() {
        let mut bindings = VimBindings::new();

        // Enter insert mode
        let i_key = create_key_event(KeyCode::Char('i'), KeyModifiers::NONE);
        bindings.parse_key(i_key);

        let char_key = create_key_event(KeyCode::Char('a'), KeyModifiers::NONE);
        let command = bindings.parse_key(char_key);

        assert_eq!(command, VimCommand::InsertChar('a'));
    }

    #[test]
    fn test_insert_to_normal_mode_transition() {
        let mut bindings = VimBindings::new();

        // Enter insert mode
        let i_key = create_key_event(KeyCode::Char('i'), KeyModifiers::NONE);
        bindings.parse_key(i_key);

        // Exit to normal mode
        let esc_key = create_key_event(KeyCode::Esc, KeyModifiers::NONE);
        let command = bindings.parse_key(esc_key);

        assert_eq!(command, VimCommand::ExitToNormal);
        assert_eq!(bindings.current_mode(), &EditorMode::Normal);
    }

    #[test]
    fn test_execute_movement_commands() {
        let bindings = VimBindings::new();
        let mut cursor = CursorPosition::new(0, 2);
        let mut buffer = TextBuffer::from_content("Hello\nWorld\nTest");

        // Test move left
        bindings
            .execute_command(VimCommand::MoveLeft, &mut cursor, &mut buffer)
            .unwrap();
        assert_eq!(cursor.col, 1);

        // Test move right
        bindings
            .execute_command(VimCommand::MoveRight, &mut cursor, &mut buffer)
            .unwrap();
        assert_eq!(cursor.col, 2);
    }

    #[test]
    fn test_execute_insert_char() {
        let bindings = VimBindings::new();
        let mut cursor = CursorPosition::new(0, 0);
        let mut buffer = TextBuffer::new();

        bindings
            .execute_command(VimCommand::InsertChar('H'), &mut cursor, &mut buffer)
            .unwrap();
        bindings
            .execute_command(VimCommand::InsertChar('i'), &mut cursor, &mut buffer)
            .unwrap();

        assert_eq!(buffer.get_line(0), Some(&"Hi".to_string()));
        assert_eq!(cursor.col, 2);
    }

    #[test]
    fn test_execute_delete_char() {
        let bindings = VimBindings::new();
        let mut cursor = CursorPosition::new(0, 1);
        let mut buffer = TextBuffer::from_content("Hello");

        bindings
            .execute_command(VimCommand::DeleteChar, &mut cursor, &mut buffer)
            .unwrap();

        assert_eq!(buffer.get_line(0), Some(&"Hllo".to_string()));
    }

    #[test]
    fn test_execute_undo_redo() {
        let bindings = VimBindings::new();
        let mut cursor = CursorPosition::new(0, 0);
        let mut buffer = TextBuffer::new();

        // Insert character
        bindings
            .execute_command(VimCommand::InsertChar('X'), &mut cursor, &mut buffer)
            .unwrap();
        assert_eq!(buffer.get_line(0), Some(&"X".to_string()));

        // Undo
        bindings
            .execute_command(VimCommand::Undo, &mut cursor, &mut buffer)
            .unwrap();
        assert_eq!(buffer.get_line(0), Some(&"".to_string()));

        // Redo
        bindings
            .execute_command(VimCommand::Redo, &mut cursor, &mut buffer)
            .unwrap();
        assert_eq!(buffer.get_line(0), Some(&"X".to_string()));
    }

    #[test]
    fn test_default_implementation() {
        let _bindings = VimBindings::default();
    }
}
