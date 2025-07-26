use crate::editor::{Buffer, Position};
use crate::error::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum VimCommand {
    // Movement commands
    MoveLeft,
    MoveDown,
    MoveUp,
    MoveRight,
    MoveLineStart,
    MoveLineEnd,
    MoveBufferStart,
    MoveBufferEnd,
    MoveWordForward,
    MoveWordBackward,

    // Editing commands
    InsertChar(char),
    DeleteChar,
    DeleteLine,
    NewLine,

    // Mode transitions
    EnterInsert,
    EnterInsertAfter,
    EnterInsertNewLine,
    EnterVisual,
    EnterCommand,
    ExitToNormal,

    // File operations
    Save,
    Quit,
    SaveAndQuit,
    ForceQuit,

    // Undo/Redo
    Undo,
    Redo,

    // No operation
    Noop,
}

impl VimCommand {
    pub fn execute(&self, buffer: &mut Buffer, cursor: &mut Position) -> Result<CommandResult> {
        match self {
            VimCommand::MoveLeft => {
                cursor.move_left().ok(); // エラーを無視して境界で停止
                Ok(CommandResult::None)
            }
            VimCommand::MoveDown => {
                let total_lines = buffer.line_count();
                cursor.move_down(total_lines).ok(); // エラーを無視して境界で停止

                // 新しい行の長さに合わせてカーソル位置を調整
                if let Ok(line_length) = buffer.line_length(cursor.row) {
                    cursor.clamp_to_line(line_length);
                }
                Ok(CommandResult::None)
            }
            VimCommand::MoveUp => {
                cursor.move_up().ok(); // エラーを無視して境界で停止

                // 新しい行の長さに合わせてカーソル位置を調整
                if let Ok(line_length) = buffer.line_length(cursor.row) {
                    cursor.clamp_to_line(line_length);
                }
                Ok(CommandResult::None)
            }
            VimCommand::MoveRight => {
                if let Ok(line_length) = buffer.line_length(cursor.row) {
                    cursor.move_right(line_length).ok(); // エラーを無視して境界で停止
                }
                Ok(CommandResult::None)
            }
            VimCommand::MoveLineStart => {
                cursor.move_to_line_start();
                Ok(CommandResult::None)
            }
            VimCommand::MoveLineEnd => {
                if let Ok(line_length) = buffer.line_length(cursor.row) {
                    cursor.move_to_line_end(line_length);
                }
                Ok(CommandResult::None)
            }
            VimCommand::MoveBufferStart => {
                cursor.move_to_buffer_start();
                Ok(CommandResult::None)
            }
            VimCommand::MoveBufferEnd => {
                let total_lines = buffer.line_count();
                let last_line_length = if total_lines > 0 {
                    buffer.line_length(total_lines - 1).unwrap_or(0)
                } else {
                    0
                };
                cursor.move_to_buffer_end(total_lines, last_line_length);
                Ok(CommandResult::None)
            }
            VimCommand::InsertChar(ch) => {
                buffer.insert_char(*cursor, *ch)?;
                cursor.move_right(buffer.line_length(cursor.row)?).ok();
                Ok(CommandResult::None)
            }
            VimCommand::DeleteChar => {
                if let Some(deleted) = buffer.delete_char(*cursor)? {
                    Ok(CommandResult::DeletedChar(deleted))
                } else {
                    Ok(CommandResult::None)
                }
            }
            VimCommand::DeleteLine => {
                if let Some(deleted) = buffer.delete_line(cursor.row)? {
                    // カーソル位置を調整
                    let total_lines = buffer.line_count();
                    if cursor.row >= total_lines && total_lines > 0 {
                        cursor.row = total_lines - 1;
                    }
                    if let Ok(line_length) = buffer.line_length(cursor.row) {
                        cursor.clamp_to_line(line_length);
                    }
                    Ok(CommandResult::DeletedLine(deleted))
                } else {
                    Ok(CommandResult::None)
                }
            }
            VimCommand::NewLine => {
                buffer.insert_line(cursor.row + 1)?;
                cursor.row += 1;
                cursor.col = 0;
                Ok(CommandResult::None)
            }
            VimCommand::Undo => {
                buffer.undo()?;
                Ok(CommandResult::None)
            }
            VimCommand::Redo => {
                buffer.redo()?;
                Ok(CommandResult::None)
            }
            VimCommand::EnterInsert
            | VimCommand::EnterInsertAfter
            | VimCommand::EnterInsertNewLine
            | VimCommand::EnterVisual
            | VimCommand::EnterCommand
            | VimCommand::ExitToNormal => {
                // Mode transitions are handled by the mode manager
                Ok(CommandResult::ModeTransition)
            }
            VimCommand::Save => Ok(CommandResult::SaveRequested),
            VimCommand::Quit => Ok(CommandResult::QuitRequested),
            VimCommand::SaveAndQuit => Ok(CommandResult::SaveAndQuitRequested),
            VimCommand::ForceQuit => Ok(CommandResult::ForceQuitRequested),
            VimCommand::MoveWordForward | VimCommand::MoveWordBackward => {
                // TODO: Implement word movement
                Ok(CommandResult::None)
            }
            VimCommand::Noop => Ok(CommandResult::None),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CommandResult {
    None,
    DeletedChar(char),
    DeletedLine(String),
    ModeTransition,
    SaveRequested,
    QuitRequested,
    SaveAndQuitRequested,
    ForceQuitRequested,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::buffer::test_helpers::BufferBuilder;

    #[test]
    fn test_move_right_command() {
        let mut buffer = BufferBuilder::with_content("Hello").build();
        let mut cursor = Position::new(0, 0);

        let result = VimCommand::MoveRight.execute(&mut buffer, &mut cursor);
        assert!(result.is_ok());
        assert_eq!(cursor.col, 1);
    }

    #[test]
    fn test_move_right_at_line_end() {
        let mut buffer = BufferBuilder::with_content("Hello").build();
        let mut cursor = Position::new(0, 5);

        let result = VimCommand::MoveRight.execute(&mut buffer, &mut cursor);
        assert!(result.is_ok());
        assert_eq!(cursor.col, 5); // 境界で停止
    }

    #[test]
    fn test_move_down_command() {
        let mut buffer = BufferBuilder::with_content("Line 1\nLine 2").build();
        let mut cursor = Position::new(0, 3);

        let result = VimCommand::MoveDown.execute(&mut buffer, &mut cursor);
        assert!(result.is_ok());
        assert_eq!(cursor.row, 1);
        assert_eq!(cursor.col, 3);
    }

    #[test]
    fn test_move_down_cursor_clamping() {
        let mut buffer = BufferBuilder::with_content("Long line\nShort").build();
        let mut cursor = Position::new(0, 8);

        let result = VimCommand::MoveDown.execute(&mut buffer, &mut cursor);
        assert!(result.is_ok());
        assert_eq!(cursor.row, 1);
        assert_eq!(cursor.col, 5); // "Short"の長さでクランプ
    }

    #[test]
    fn test_insert_char_command() {
        let mut buffer = BufferBuilder::with_content("Hello").build();
        let mut cursor = Position::new(0, 5);

        let result = VimCommand::InsertChar('!').execute(&mut buffer, &mut cursor);
        assert!(result.is_ok());
        assert_eq!(buffer.line(0).unwrap(), "Hello!");
        assert_eq!(cursor.col, 6);
    }

    #[test]
    fn test_delete_char_command() {
        let mut buffer = BufferBuilder::with_content("Hello").build();
        let mut cursor = Position::new(0, 1);

        let result = VimCommand::DeleteChar.execute(&mut buffer, &mut cursor);
        assert!(result.is_ok());
        if let Ok(CommandResult::DeletedChar(ch)) = result {
            assert_eq!(ch, 'e');
        } else {
            panic!("Expected DeletedChar result");
        }
        assert_eq!(buffer.line(0).unwrap(), "Hllo");
    }

    #[test]
    fn test_delete_char_at_end() {
        let mut buffer = BufferBuilder::with_content("Hello").build();
        let mut cursor = Position::new(0, 5);

        let result = VimCommand::DeleteChar.execute(&mut buffer, &mut cursor);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CommandResult::None);
        assert_eq!(buffer.line(0).unwrap(), "Hello");
    }

    #[test]
    fn test_delete_line_command() {
        let mut buffer = BufferBuilder::with_content("Line 1\nLine 2\nLine 3").build();
        let mut cursor = Position::new(1, 3);

        let result = VimCommand::DeleteLine.execute(&mut buffer, &mut cursor);
        assert!(result.is_ok());
        if let Ok(CommandResult::DeletedLine(line)) = result {
            assert_eq!(line, "Line 2");
        } else {
            panic!("Expected DeletedLine result");
        }
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.line(0).unwrap(), "Line 1");
        assert_eq!(buffer.line(1).unwrap(), "Line 3");
    }

    #[test]
    fn test_new_line_command() {
        let mut buffer = BufferBuilder::with_content("Hello").build();
        let mut cursor = Position::new(0, 2);

        let result = VimCommand::NewLine.execute(&mut buffer, &mut cursor);
        assert!(result.is_ok());
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(cursor.row, 1);
        assert_eq!(cursor.col, 0);
    }

    #[test]
    fn test_move_line_start_command() {
        let mut buffer = BufferBuilder::with_content("Hello").build();
        let mut cursor = Position::new(0, 3);

        let result = VimCommand::MoveLineStart.execute(&mut buffer, &mut cursor);
        assert!(result.is_ok());
        assert_eq!(cursor.col, 0);
    }

    #[test]
    fn test_move_line_end_command() {
        let mut buffer = BufferBuilder::with_content("Hello").build();
        let mut cursor = Position::new(0, 2);

        let result = VimCommand::MoveLineEnd.execute(&mut buffer, &mut cursor);
        assert!(result.is_ok());
        assert_eq!(cursor.col, 5);
    }

    #[test]
    fn test_move_buffer_start_command() {
        let mut buffer = BufferBuilder::with_content("Line 1\nLine 2").build();
        let mut cursor = Position::new(1, 3);

        let result = VimCommand::MoveBufferStart.execute(&mut buffer, &mut cursor);
        assert!(result.is_ok());
        assert_eq!(cursor.row, 0);
        assert_eq!(cursor.col, 0);
    }

    #[test]
    fn test_move_buffer_end_command() {
        let mut buffer = BufferBuilder::with_content("Line 1\nLine 2").build();
        let mut cursor = Position::new(0, 0);

        let result = VimCommand::MoveBufferEnd.execute(&mut buffer, &mut cursor);
        assert!(result.is_ok());
        assert_eq!(cursor.row, 1);
        assert_eq!(cursor.col, 6);
    }

    #[test]
    fn test_undo_command() {
        let mut buffer = BufferBuilder::with_content("Hello").build();
        let mut cursor = Position::new(0, 5);

        // まず変更を行う
        buffer.insert_char(cursor, '!').unwrap();

        let result = VimCommand::Undo.execute(&mut buffer, &mut cursor);
        assert!(result.is_ok());
        assert_eq!(buffer.line(0).unwrap(), "Hello");
    }

    #[test]
    fn test_redo_command() {
        let mut buffer = BufferBuilder::with_content("Hello").build();
        let mut cursor = Position::new(0, 5);

        // 変更してからundoする
        buffer.insert_char(cursor, '!').unwrap();
        buffer.undo().unwrap();

        let result = VimCommand::Redo.execute(&mut buffer, &mut cursor);
        assert!(result.is_ok());
        assert_eq!(buffer.line(0).unwrap(), "Hello!");
    }

    #[test]
    fn test_mode_transition_commands() {
        let mut buffer = BufferBuilder::new().build();
        let mut cursor = Position::new(0, 0);

        let commands = [
            VimCommand::EnterInsert,
            VimCommand::EnterVisual,
            VimCommand::EnterCommand,
            VimCommand::ExitToNormal,
        ];

        for cmd in &commands {
            let result = cmd.execute(&mut buffer, &mut cursor);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), CommandResult::ModeTransition);
        }
    }

    #[test]
    fn test_file_operation_commands() {
        let mut buffer = BufferBuilder::new().build();
        let mut cursor = Position::new(0, 0);

        let test_cases = [
            (VimCommand::Save, CommandResult::SaveRequested),
            (VimCommand::Quit, CommandResult::QuitRequested),
            (VimCommand::SaveAndQuit, CommandResult::SaveAndQuitRequested),
            (VimCommand::ForceQuit, CommandResult::ForceQuitRequested),
        ];

        for (cmd, expected) in &test_cases {
            let result = cmd.execute(&mut buffer, &mut cursor);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), *expected);
        }
    }

    #[test]
    fn test_noop_command() {
        let mut buffer = BufferBuilder::new().build();
        let mut cursor = Position::new(0, 0);

        let result = VimCommand::Noop.execute(&mut buffer, &mut cursor);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CommandResult::None);
    }
}
