use crate::vim::command::VimCommand;
use crate::vim::mode::Mode;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl Key {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub fn char(c: char) -> Self {
        Self {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn ctrl(c: char) -> Self {
        Self {
            code: KeyCode::Char(c),
            modifiers: KeyModifiers::CONTROL,
        }
    }

    pub fn escape() -> Self {
        Self {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn enter() -> Self {
        Self {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn backspace() -> Self {
        Self {
            code: KeyCode::Backspace,
            modifiers: KeyModifiers::NONE,
        }
    }
}

impl From<KeyEvent> for Key {
    fn from(event: KeyEvent) -> Self {
        Self {
            code: event.code,
            modifiers: event.modifiers,
        }
    }
}

pub struct KeyMapper {
    // 現在は静的マッピングだが、将来的にはカスタマイズ可能にできる
}

impl KeyMapper {
    pub fn new() -> Self {
        Self {}
    }

    pub fn map_key(&self, key: &Key, mode: &Mode) -> VimCommand {
        match mode {
            Mode::Normal => self.map_normal_mode(key),
            Mode::Insert => self.map_insert_mode(key),
            Mode::Visual { .. } => self.map_visual_mode(key),
            Mode::Command { .. } => self.map_command_mode(key, mode),
        }
    }

    fn map_normal_mode(&self, key: &Key) -> VimCommand {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return self.map_ctrl_keys(key);
        }

        match key.code {
            // Movement
            KeyCode::Char('h') => VimCommand::MoveLeft,
            KeyCode::Char('j') => VimCommand::MoveDown,
            KeyCode::Char('k') => VimCommand::MoveUp,
            KeyCode::Char('l') => VimCommand::MoveRight,
            KeyCode::Char('0') => VimCommand::MoveLineStart,
            KeyCode::Char('$') => VimCommand::MoveLineEnd,
            KeyCode::Char('w') => VimCommand::MoveWordForward,
            KeyCode::Char('b') => VimCommand::MoveWordBackward,

            // Mode transitions
            KeyCode::Char('i') => VimCommand::EnterInsert,
            KeyCode::Char('a') => VimCommand::EnterInsertAfter,
            KeyCode::Char('o') => VimCommand::EnterInsertNewLine,
            KeyCode::Char('v') => VimCommand::EnterVisual,
            KeyCode::Char(':') => VimCommand::EnterCommand,

            // Editing
            KeyCode::Char('x') => VimCommand::DeleteChar,
            KeyCode::Char('d') => VimCommand::DeleteLine, // 簡略化: ddの代わり
            KeyCode::Char('u') => VimCommand::Undo,

            // Special keys
            KeyCode::Left => VimCommand::MoveLeft,
            KeyCode::Down => VimCommand::MoveDown,
            KeyCode::Up => VimCommand::MoveUp,
            KeyCode::Right => VimCommand::MoveRight,

            _ => VimCommand::Noop,
        }
    }

    fn map_insert_mode(&self, key: &Key) -> VimCommand {
        match key.code {
            KeyCode::Esc => VimCommand::ExitToNormal,
            KeyCode::Char(c) => VimCommand::InsertChar(c),
            KeyCode::Enter => VimCommand::NewLine,
            KeyCode::Backspace => VimCommand::DeleteCharBackward,
            _ => VimCommand::Noop,
        }
    }

    fn map_visual_mode(&self, key: &Key) -> VimCommand {
        match key.code {
            KeyCode::Esc => VimCommand::ExitToNormal,
            KeyCode::Char('i') => VimCommand::EnterInsert,

            // Movement (visual mode specific behavior could be added later)
            KeyCode::Char('h') => VimCommand::MoveLeft,
            KeyCode::Char('j') => VimCommand::MoveDown,
            KeyCode::Char('k') => VimCommand::MoveUp,
            KeyCode::Char('l') => VimCommand::MoveRight,

            _ => VimCommand::Noop,
        }
    }

    fn map_command_mode(&self, key: &Key, mode: &Mode) -> VimCommand {
        match key.code {
            KeyCode::Esc => VimCommand::ExitToNormal,
            KeyCode::Enter => {
                // Execute the command with current input
                if let Mode::Command { input } = mode {
                    VimCommand::ExecuteCommand(input.clone())
                } else {
                    VimCommand::ExitToNormal
                }
            }
            KeyCode::Char(c) => VimCommand::CommandInput(c),
            KeyCode::Backspace => VimCommand::CommandBackspace,
            _ => VimCommand::Noop,
        }
    }

    fn map_ctrl_keys(&self, key: &Key) -> VimCommand {
        match key.code {
            KeyCode::Char('r') => VimCommand::Redo,
            KeyCode::Char('f') => VimCommand::MoveBufferEnd, // Ctrl+F: page down
            KeyCode::Char('b') => VimCommand::MoveBufferStart, // Ctrl+B: page up
            _ => VimCommand::Noop,
        }
    }
}

impl Default for KeyMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::Position;

    #[test]
    fn test_key_creation() {
        let key = Key::char('h');
        assert_eq!(key.code, KeyCode::Char('h'));
        assert_eq!(key.modifiers, KeyModifiers::NONE);

        let ctrl_key = Key::ctrl('r');
        assert_eq!(ctrl_key.code, KeyCode::Char('r'));
        assert_eq!(ctrl_key.modifiers, KeyModifiers::CONTROL);

        let escape = Key::escape();
        assert_eq!(escape.code, KeyCode::Esc);
        assert_eq!(escape.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn test_key_from_event() {
        let event = KeyEvent {
            code: KeyCode::Char('h'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        let key = Key::from(event);
        assert_eq!(key.code, KeyCode::Char('h'));
        assert_eq!(key.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn test_normal_mode_movement_mapping() {
        let mapper = KeyMapper::new();
        let mode = Mode::Normal;

        let test_cases = [
            (Key::char('h'), VimCommand::MoveLeft),
            (Key::char('j'), VimCommand::MoveDown),
            (Key::char('k'), VimCommand::MoveUp),
            (Key::char('l'), VimCommand::MoveRight),
            (Key::char('0'), VimCommand::MoveLineStart),
            (Key::char('$'), VimCommand::MoveLineEnd),
            (Key::char('w'), VimCommand::MoveWordForward),
            (Key::char('b'), VimCommand::MoveWordBackward),
        ];

        for (key, expected_cmd) in &test_cases {
            let cmd = mapper.map_key(key, &mode);
            assert_eq!(cmd, *expected_cmd);
        }
    }

    #[test]
    fn test_normal_mode_editing_mapping() {
        let mapper = KeyMapper::new();
        let mode = Mode::Normal;

        let test_cases = [
            (Key::char('i'), VimCommand::EnterInsert),
            (Key::char('a'), VimCommand::EnterInsertAfter),
            (Key::char('o'), VimCommand::EnterInsertNewLine),
            (Key::char('v'), VimCommand::EnterVisual),
            (Key::char(':'), VimCommand::EnterCommand),
            (Key::char('x'), VimCommand::DeleteChar),
            (Key::char('d'), VimCommand::DeleteLine),
            (Key::char('u'), VimCommand::Undo),
        ];

        for (key, expected_cmd) in &test_cases {
            let cmd = mapper.map_key(key, &mode);
            assert_eq!(cmd, *expected_cmd);
        }
    }

    #[test]
    fn test_normal_mode_arrow_keys() {
        let mapper = KeyMapper::new();
        let mode = Mode::Normal;

        let test_cases = [
            (
                Key::new(KeyCode::Left, KeyModifiers::NONE),
                VimCommand::MoveLeft,
            ),
            (
                Key::new(KeyCode::Down, KeyModifiers::NONE),
                VimCommand::MoveDown,
            ),
            (
                Key::new(KeyCode::Up, KeyModifiers::NONE),
                VimCommand::MoveUp,
            ),
            (
                Key::new(KeyCode::Right, KeyModifiers::NONE),
                VimCommand::MoveRight,
            ),
        ];

        for (key, expected_cmd) in &test_cases {
            let cmd = mapper.map_key(key, &mode);
            assert_eq!(cmd, *expected_cmd);
        }
    }

    #[test]
    fn test_insert_mode_mapping() {
        let mapper = KeyMapper::new();
        let mode = Mode::Insert;

        // Character insertion
        let cmd = mapper.map_key(&Key::char('a'), &mode);
        assert_eq!(cmd, VimCommand::InsertChar('a'));

        // Special keys
        let test_cases = [
            (Key::escape(), VimCommand::ExitToNormal),
            (Key::enter(), VimCommand::NewLine),
            (Key::backspace(), VimCommand::DeleteCharBackward),
        ];

        for (key, expected_cmd) in &test_cases {
            let cmd = mapper.map_key(key, &mode);
            assert_eq!(cmd, *expected_cmd);
        }
    }

    #[test]
    fn test_visual_mode_mapping() {
        let mapper = KeyMapper::new();
        let mode = Mode::Visual {
            start: Position::new(0, 0),
        };

        let test_cases = [
            (Key::escape(), VimCommand::ExitToNormal),
            (Key::char('i'), VimCommand::EnterInsert),
            (Key::char('h'), VimCommand::MoveLeft),
            (Key::char('j'), VimCommand::MoveDown),
            (Key::char('k'), VimCommand::MoveUp),
            (Key::char('l'), VimCommand::MoveRight),
        ];

        for (key, expected_cmd) in &test_cases {
            let cmd = mapper.map_key(key, &mode);
            assert_eq!(cmd, *expected_cmd);
        }
    }

    #[test]
    fn test_command_mode_mapping() {
        let mapper = KeyMapper::new();
        let mode = Mode::Command {
            input: String::new(),
        };

        // Test escape
        let cmd = mapper.map_key(&Key::escape(), &mode);
        assert_eq!(cmd, VimCommand::ExitToNormal);

        // Test enter with empty command
        let cmd = mapper.map_key(&Key::enter(), &mode);
        assert_eq!(cmd, VimCommand::ExecuteCommand(String::new()));

        // Test enter with command
        let mode_with_input = Mode::Command {
            input: "q".to_string(),
        };
        let cmd = mapper.map_key(&Key::enter(), &mode_with_input);
        assert_eq!(cmd, VimCommand::ExecuteCommand("q".to_string()));

        // Test character input
        let cmd = mapper.map_key(&Key::char('q'), &mode);
        assert_eq!(cmd, VimCommand::CommandInput('q'));

        // Test backspace
        let cmd = mapper.map_key(&Key::backspace(), &mode);
        assert_eq!(cmd, VimCommand::CommandBackspace);
    }

    #[test]
    fn test_ctrl_key_mapping() {
        let mapper = KeyMapper::new();
        let mode = Mode::Normal;

        let test_cases = [
            (Key::ctrl('r'), VimCommand::Redo),
            (Key::ctrl('f'), VimCommand::MoveBufferEnd),
            (Key::ctrl('b'), VimCommand::MoveBufferStart),
        ];

        for (key, expected_cmd) in &test_cases {
            let cmd = mapper.map_key(key, &mode);
            assert_eq!(cmd, *expected_cmd);
        }
    }

    #[test]
    fn test_unmapped_key() {
        let mapper = KeyMapper::new();
        let mode = Mode::Normal;

        let cmd = mapper.map_key(&Key::char('z'), &mode);
        assert_eq!(cmd, VimCommand::Noop);
    }
}
