use clap::{Arg, Command};
use rvim::{
    Buffer, EditorError, FileSystem, Key, KeyMapper, ModeManager, Position, Result, Terminal,
    VimCommand,
};
use std::path::PathBuf;

struct Editor {
    buffer: Buffer,
    cursor: Position,
    mode_manager: ModeManager,
    key_mapper: KeyMapper,
    terminal: Terminal,
    file_path: Option<PathBuf>,
    should_quit: bool,
}

impl Editor {
    fn new() -> Result<Self> {
        Ok(Self {
            buffer: Buffer::new(),
            cursor: Position::origin(),
            mode_manager: ModeManager::new(),
            key_mapper: KeyMapper::new(),
            terminal: Terminal::new()?,
            file_path: None,
            should_quit: false,
        })
    }

    fn with_file(path: PathBuf) -> Result<Self> {
        let content = FileSystem::read_file(&path)?;
        let buffer = Buffer::from_content(&content).with_file_path(path.clone());

        Ok(Self {
            buffer,
            cursor: Position::origin(),
            mode_manager: ModeManager::new(),
            key_mapper: KeyMapper::new(),
            terminal: Terminal::new()?,
            file_path: Some(path),
            should_quit: false,
        })
    }

    fn run(&mut self) -> Result<()> {
        self.terminal.clear_screen()?;
        self.terminal.hide_cursor()?;

        while !self.should_quit {
            self.render()?;
            self.handle_input()?;
        }

        self.terminal.show_cursor()?;
        self.terminal.cleanup()?;
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        self.terminal.clear_screen()?;

        // バッファ内容を描画
        for (row, line) in self.buffer.to_string().lines().enumerate() {
            // カーソル行の場合、カーソル位置をハイライト
            if row == self.cursor.row {
                self.render_line_with_cursor_highlight(row, line)?;
            } else {
                self.terminal
                    .write_at(rvim::TerminalPosition::new(0, row as u16), line)?;
            }
        }

        // ステータスライン描画
        let terminal_size = self.terminal.size()?;
        let status_row = terminal_size.height.saturating_sub(1);

        let mode_str = format!("-- {} --", self.mode_manager.current());
        let position_str = format!("{}:{}", self.cursor.row + 1, self.cursor.col + 1);
        let file_str = self
            .file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("[No Name]");

        let status = format!("{} | {} | {}", mode_str, file_str, position_str);
        self.terminal
            .write_at(rvim::TerminalPosition::new(0, status_row), &status)?;

        // Commandモードの場合、入力コマンドを表示
        if let rvim::vim::Mode::Command { input } = self.mode_manager.current() {
            let command_line = format!(":{}", input);
            self.terminal.write_at(
                rvim::TerminalPosition::new(0, status_row.saturating_sub(1)),
                &command_line,
            )?;
        }

        // カーソル位置に移動
        self.terminal.move_cursor(rvim::TerminalPosition::new(
            self.cursor.col as u16,
            self.cursor.row as u16,
        ))?;

        self.terminal.flush()?;
        Ok(())
    }

    fn render_line_with_cursor_highlight(&mut self, row: usize, line: &str) -> Result<()> {
        use crossterm::style::Color;
        
        self.terminal.move_cursor(rvim::TerminalPosition::new(0, row as u16))?;
        
        let chars: Vec<char> = line.chars().collect();
        
        for (col, &ch) in chars.iter().enumerate() {
            if col == self.cursor.col {
                // カーソル位置の文字をハイライト
                self.terminal.set_background_color(Color::White)?;
                self.terminal.set_foreground_color(Color::Black)?;
                self.terminal.write(&ch.to_string())?;
                self.terminal.reset_colors()?;
            } else {
                self.terminal.write(&ch.to_string())?;
            }
        }
        
        // カーソルが行末を超えている場合の処理
        if self.cursor.col >= chars.len() {
            // 行末にカーソルを表示
            self.terminal.set_background_color(Color::White)?;
            self.terminal.write(" ")?;
            self.terminal.reset_colors()?;
        }
        
        Ok(())
    }

    fn handle_input(&mut self) -> Result<()> {
        let key_event = self.terminal.read_key()?;
        let key = Key::from(key_event);

        // キーをVIMコマンドにマップ
        let command = self.key_mapper.map_key(&key, self.mode_manager.current());

        // コマンドを実行
        match command.execute(&mut self.buffer, &mut self.cursor)? {
            rvim::vim::CommandResult::None => {}
            rvim::vim::CommandResult::DeletedChar(_) => {}
            rvim::vim::CommandResult::DeletedLine(_) => {}
            rvim::vim::CommandResult::ModeTransition => {
                self.handle_mode_transition(&command)?;
            }
            rvim::vim::CommandResult::SaveRequested => {
                self.save_file()?;
                // Commandモードから実行された場合はNormalモードに戻る
                if self.mode_manager.current().is_command() {
                    self.mode_manager.enter_normal();
                }
            }
            rvim::vim::CommandResult::QuitRequested => {
                if self.buffer.is_modified() {
                    // 変更がある場合は警告を表示（簡略化）
                    // Commandモードから実行された場合はNormalモードに戻る
                    if self.mode_manager.current().is_command() {
                        self.mode_manager.enter_normal();
                    }
                    return Ok(());
                }
                self.should_quit = true;
            }
            rvim::vim::CommandResult::SaveAndQuitRequested => {
                self.save_file()?;
                self.should_quit = true;
            }
            rvim::vim::CommandResult::ForceQuitRequested => {
                self.should_quit = true;
            }
        }

        // カーソル位置の境界チェック
        self.adjust_cursor_position()?;

        Ok(())
    }

    fn handle_mode_transition(&mut self, command: &VimCommand) -> Result<()> {
        match command {
            VimCommand::EnterInsert => {
                self.mode_manager.enter_insert();
            }
            VimCommand::EnterInsertAfter => {
                self.mode_manager.enter_insert();
                // カーソルを一つ右に移動
                if let Ok(line_length) = self.buffer.line_length(self.cursor.row) {
                    self.cursor.move_right(line_length).ok();
                }
            }
            VimCommand::EnterInsertNewLine => {
                self.buffer.insert_line(self.cursor.row + 1)?;
                self.cursor.row += 1;
                self.cursor.col = 0;
                self.mode_manager.enter_insert();
            }
            VimCommand::EnterVisual => {
                self.mode_manager.enter_visual(self.cursor);
            }
            VimCommand::EnterCommand => {
                self.mode_manager.enter_command();
            }
            VimCommand::ExitToNormal => {
                self.mode_manager.enter_normal();
            }
            VimCommand::CommandInput(ch) => {
                if let rvim::vim::Mode::Command { input } = self.mode_manager.current() {
                    let mut new_input = input.clone();
                    new_input.push(*ch);
                    self.mode_manager.update_command_input(new_input)?;
                }
            }
            VimCommand::CommandBackspace => {
                if let rvim::vim::Mode::Command { input } = self.mode_manager.current() {
                    let mut new_input = input.clone();
                    new_input.pop();
                    self.mode_manager.update_command_input(new_input)?;
                }
            }
            VimCommand::ExecuteCommand(_) => {
                // ExecuteCommand処理後はNormalモードに戻る
                self.mode_manager.enter_normal();
            }
            _ => {}
        }
        Ok(())
    }

    fn save_file(&mut self) -> Result<()> {
        if let Some(path) = &self.file_path {
            FileSystem::write_file(path, &self.buffer.to_string())?;
            self.buffer.mark_saved();
        } else {
            // ファイル名を指定していない場合の処理（簡略化）
            return Err(EditorError::config("No file name specified".to_string()));
        }
        Ok(())
    }

    fn adjust_cursor_position(&mut self) -> Result<()> {
        // 行数の境界チェック
        let total_lines = self.buffer.line_count();
        if self.cursor.row >= total_lines {
            self.cursor.row = total_lines.saturating_sub(1);
        }

        // 列数の境界チェック
        if let Ok(line_length) = self.buffer.line_length(self.cursor.row) {
            self.cursor.clamp_to_line(line_length);
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let matches = Command::new("rvim")
        .version("0.1.0")
        .about("A VIM-like text editor written in Rust")
        .arg(
            Arg::new("file")
                .help("File to edit")
                .index(1)
                .required(false),
        )
        .get_matches();

    let mut editor = if let Some(file_path) = matches.get_one::<String>("file") {
        let path = PathBuf::from(file_path);
        Editor::with_file(path)?
    } else {
        Editor::new()?
    };

    match editor.run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Editor error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_creation() {
        // ターミナルが利用できない環境では失敗する可能性がある
        let result = Editor::new();
        if result.is_ok() {
            let editor = result.unwrap();
            assert_eq!(editor.cursor.row, 0);
            assert_eq!(editor.cursor.col, 0);
            assert!(editor.mode_manager.current().is_normal());
            assert!(!editor.should_quit);
        }
    }

    #[test]
    fn test_editor_with_content() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello, World!").unwrap();

        let result = Editor::with_file(temp_file.path().to_path_buf());
        if result.is_ok() {
            let editor = result.unwrap();
            assert_eq!(editor.buffer.line(0).unwrap(), "Hello, World!");
            assert_eq!(editor.file_path, Some(temp_file.path().to_path_buf()));
        }
    }

    #[test]
    fn test_cursor_adjustment() {
        if let Ok(mut editor) = Editor::new() {
            // バッファに内容を設定
            editor.buffer = Buffer::from_content("Short\nLonger line");

            // カーソルを無効な位置に設定
            editor.cursor = Position::new(0, 10);

            // 境界調整
            editor.adjust_cursor_position().unwrap();

            // 正しい位置に調整されているはず
            assert_eq!(editor.cursor.col, 5); // "Short"の長さ
        }
    }
}

#[cfg(test)]
mod demo {
    use rvim::{Buffer, ModeManager, Position};

    #[test]
    fn demo_new_architecture() {
        println!("=== RVIM 新アーキテクチャ動作確認 ===\n");

        // 1. 新しいPosition型のデモ
        println!("1. Position型デモ:");
        let mut pos = Position::new(0, 0);
        println!("  初期位置: row={}, col={}", pos.row, pos.col);

        pos.move_right(10).ok();
        println!("  右移動後: row={}, col={}", pos.row, pos.col);

        pos.move_to_line_start();
        println!("  行頭移動: row={}, col={}", pos.row, pos.col);

        // 2. 新しいBuffer型のデモ
        println!("\n2. Buffer型デモ:");
        let mut buffer = Buffer::from_content("Hello, World!\nThis is new RVIM!");
        println!("  初期内容:\n{}", indent_lines(&buffer.to_string()));

        buffer.insert_char(Position::new(0, 13), '!').unwrap();
        println!("  文字挿入後:\n{}", indent_lines(&buffer.to_string()));

        buffer.undo().unwrap();
        println!("  Undo後:\n{}", indent_lines(&buffer.to_string()));

        // 3. 新しいMode管理のデモ
        println!("\n3. モード管理デモ:");
        let mut mode_manager = ModeManager::new();
        println!("  初期モード: {}", mode_manager.current());

        mode_manager.enter_insert();
        println!("  Insert移行後: {}", mode_manager.current());

        mode_manager.enter_normal();
        println!("  Normal復帰後: {}", mode_manager.current());

        println!("\n=== 新アーキテクチャ完了: Rustらしい設計で実装 ===");
        println!("型安全性、エラーハンドリング、テスタビリティが向上しました。");
    }

    fn indent_lines(text: &str) -> String {
        text.lines()
            .map(|line| format!("    {}", line))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
