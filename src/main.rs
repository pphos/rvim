mod application;
mod domain;
mod infrastructure;
mod ports;

use anyhow::Result;
use clap::{Arg, Command};
use std::env;
use std::path::PathBuf;

use application::editor_service::EditorService;
use infrastructure::{file_system::StdFileSystem, terminal::CrosstermTerminal};

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

    let file_system = StdFileSystem::new();
    let terminal = CrosstermTerminal::new()?;

    let mut editor = if let Some(file_path) = matches.get_one::<String>("file") {
        let path = PathBuf::from(file_path);
        EditorService::with_file(file_system, terminal, path)?
    } else {
        EditorService::new(file_system, terminal)?
    };

    match editor.run() {
        Ok(_) => println!("Editor exited successfully"),
        Err(e) => {
            eprintln!("Editor error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(test)]
mod demo {
    use super::domain::{
        cursor::CursorPosition, editor_mode::ModeManager, text_buffer::TextBuffer,
    };

    #[test]
    fn demo_core_functionality() {
        println!("=== RVIM Phase 1 コアドメイン動作確認 ===\n");

        // 1. カーソル機能のデモ
        println!("1. カーソル機能デモ:");
        let mut cursor = CursorPosition::new(0, 0);
        println!("  初期位置: row={}, col={}", cursor.row, cursor.col);

        cursor.move_right(10);
        println!("  右移動後: row={}, col={}", cursor.row, cursor.col);

        cursor.move_down(5);
        println!("  下移動後: row={}, col={}", cursor.row, cursor.col);

        cursor.move_to_line_start();
        println!("  行頭移動: row={}, col={}", cursor.row, cursor.col);

        // 2. テキストバッファ機能のデモ
        println!("\n2. テキストバッファ機能デモ:");
        let mut buffer = TextBuffer::from_content("Hello, World!\nThis is RVIM!");
        println!("  初期内容:\n{}", indent_lines(&buffer.to_string()));

        buffer.insert_char(0, 13, '!').unwrap();
        println!("  文字挿入後:\n{}", indent_lines(&buffer.to_string()));

        buffer.insert_line(1).unwrap();
        buffer.insert_char(1, 0, 'N').unwrap();
        buffer.insert_char(1, 1, 'e').unwrap();
        buffer.insert_char(1, 2, 'w').unwrap();
        println!("  新しい行追加後:\n{}", indent_lines(&buffer.to_string()));

        println!("  Undo実行:");
        buffer.undo().unwrap();
        buffer.undo().unwrap();
        buffer.undo().unwrap();
        buffer.undo().unwrap();
        println!("  Undo後:\n{}", indent_lines(&buffer.to_string()));

        // 3. エディタモード機能のデモ
        println!("\n3. エディタモード機能デモ:");
        let mut mode_manager = ModeManager::new();
        println!("  初期モード: {}", mode_manager.current_mode().to_string());

        mode_manager.transition_with_key('i');
        println!("  'i'キー後: {}", mode_manager.current_mode().to_string());

        mode_manager.transition_with_escape();
        println!("  Escape後: {}", mode_manager.current_mode().to_string());

        mode_manager.transition_with_key('v');
        println!("  'v'キー後: {}", mode_manager.current_mode().to_string());

        mode_manager.transition_with_key(':');
        println!("  ':'キー後: {}", mode_manager.current_mode().to_string());

        println!("\n=== Phase 1 完了: 全てのコアドメインロジックが正常動作 ===");
        println!("次のフェーズでUI統合とVIMバインディングを実装します。");
    }

    fn indent_lines(text: &str) -> String {
        text.lines()
            .map(|line| format!("    {}", line))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
