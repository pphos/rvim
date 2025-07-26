pub mod command;
pub mod keymap;
pub mod mode;

pub use command::{CommandResult, VimCommand};
pub use keymap::{Key, KeyMapper};
pub use mode::{Mode, ModeManager};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::{Buffer, Position};

    #[test]
    fn test_vim_module_integration() {
        // VIMモジュール全体の統合テスト
        let mut mode_manager = ModeManager::new();
        let key_mapper = KeyMapper::new();
        let mut buffer = Buffer::new();
        let mut cursor = Position::new(0, 0);

        // Normal modeでの基本操作
        assert!(mode_manager.current().is_normal());

        // 'i'キーでInsert modeに遷移
        let key = Key::char('i');
        let cmd = key_mapper.map_key(&key, mode_manager.current());
        assert_eq!(cmd, VimCommand::EnterInsert);

        mode_manager.enter_insert();
        assert!(mode_manager.current().is_insert());

        // Insert modeで文字入力
        let key = Key::char('H');
        let cmd = key_mapper.map_key(&key, mode_manager.current());
        assert_eq!(cmd, VimCommand::InsertChar('H'));

        let result = cmd.execute(&mut buffer, &mut cursor);
        assert!(result.is_ok());
        assert_eq!(buffer.line(0).unwrap(), "H");
        assert_eq!(cursor.col, 1);

        // Escapeで Normal modeに戻る
        let key = Key::escape();
        let cmd = key_mapper.map_key(&key, mode_manager.current());
        assert_eq!(cmd, VimCommand::ExitToNormal);

        mode_manager.enter_normal();
        assert!(mode_manager.current().is_normal());
    }

    #[test]
    fn test_vim_movement_integration() {
        let key_mapper = KeyMapper::new();
        let mut buffer = Buffer::from_content("Hello\nWorld");
        let mut cursor = Position::new(0, 0);

        // 右移動
        let key = Key::char('l');
        let cmd = key_mapper.map_key(&key, &Mode::Normal);
        cmd.execute(&mut buffer, &mut cursor).unwrap();
        assert_eq!(cursor.col, 1);

        // 下移動
        let key = Key::char('j');
        let cmd = key_mapper.map_key(&key, &Mode::Normal);
        cmd.execute(&mut buffer, &mut cursor).unwrap();
        assert_eq!(cursor.row, 1);
        assert_eq!(cursor.col, 1);

        // 行頭移動
        let key = Key::char('0');
        let cmd = key_mapper.map_key(&key, &Mode::Normal);
        cmd.execute(&mut buffer, &mut cursor).unwrap();
        assert_eq!(cursor.col, 0);
    }

    #[test]
    fn test_vim_editing_integration() {
        let key_mapper = KeyMapper::new();
        let mut buffer = Buffer::from_content("Hello");
        let mut cursor = Position::new(0, 5);

        // 文字削除
        let key = Key::char('x');
        let cmd = key_mapper.map_key(&key, &Mode::Normal);
        let result = cmd.execute(&mut buffer, &mut cursor).unwrap();
        assert_eq!(result, CommandResult::None); // 行末では削除されない

        // カーソルを文字上に移動してから削除
        cursor.col = 1;
        let result = cmd.execute(&mut buffer, &mut cursor).unwrap();
        assert_eq!(result, CommandResult::DeletedChar('e'));
        assert_eq!(buffer.line(0).unwrap(), "Hllo");

        // Undo
        let key = Key::char('u');
        let cmd = key_mapper.map_key(&key, &Mode::Normal);
        cmd.execute(&mut buffer, &mut cursor).unwrap();
        assert_eq!(buffer.line(0).unwrap(), "Hello");

        // Redo
        let key = Key::ctrl('r');
        let cmd = key_mapper.map_key(&key, &Mode::Normal);
        cmd.execute(&mut buffer, &mut cursor).unwrap();
        assert_eq!(buffer.line(0).unwrap(), "Hllo");
    }

    #[test]
    fn test_mode_transitions_integration() {
        let mut mode_manager = ModeManager::new();
        let key_mapper = KeyMapper::new();

        // Normal -> Insert
        let key = Key::char('i');
        let cmd = key_mapper.map_key(&key, mode_manager.current());
        assert_eq!(cmd, VimCommand::EnterInsert);
        mode_manager.enter_insert();
        assert!(mode_manager.current().is_insert());

        // Insert -> Normal
        let key = Key::escape();
        let cmd = key_mapper.map_key(&key, mode_manager.current());
        assert_eq!(cmd, VimCommand::ExitToNormal);
        mode_manager.enter_normal();
        assert!(mode_manager.current().is_normal());

        // Normal -> Visual
        let key = Key::char('v');
        let cmd = key_mapper.map_key(&key, mode_manager.current());
        assert_eq!(cmd, VimCommand::EnterVisual);
        mode_manager.enter_visual(Position::new(0, 0));
        assert!(mode_manager.current().is_visual());

        // Visual -> Normal
        let key = Key::escape();
        let cmd = key_mapper.map_key(&key, mode_manager.current());
        assert_eq!(cmd, VimCommand::ExitToNormal);
        mode_manager.enter_normal();
        assert!(mode_manager.current().is_normal());

        // Normal -> Command
        let key = Key::char(':');
        let cmd = key_mapper.map_key(&key, mode_manager.current());
        assert_eq!(cmd, VimCommand::EnterCommand);
        mode_manager.enter_command();
        assert!(mode_manager.current().is_command());
    }
}
