# RVIM アーキテクチャドキュメント

## 🏗️ アーキテクチャ概要

RVIMは **Rust-idiomatic設計** と **TDD (Test-Driven Development)** の原則に基づいて設計されたVIMライクエディタです。Clean Architectureの要素を取り入れつつ、Rustの特性を活かしたシンプルで実用的な設計を採用しています。

### アーキテクチャ図

```
┌─────────────────────────────────────┐
│              main.rs                │ ← エントリーポイント + Editor統合
├─────────────────────────────────────┤
│               vim/                  │ ← VIM機能層
│  ┌───────────────┐ ┌─────────────────┐ │
│  │   KeyMapper   │ │  CommandEngine  │ │
│  └───────────────┘ └─────────────────┘ │
│  ┌───────────────────────────────────┐ │
│  │        ModeManager                │ │
│  └───────────────────────────────────┘ │
├─────────────────────────────────────┤
│              editor/                │ ← エディタコア層
│  ┌─────────────┐ ┌─────────────────┐  │
│  │   Buffer    │ │    Position     │  │
│  └─────────────┘ └─────────────────┘  │
├─────────────────────────────────────┤
│               io/                   │ ← I/O層
│  ┌─────────────┐ ┌─────────────────┐  │
│  │ FileSystem  │ │   Terminal      │  │
│  └─────────────┘ └─────────────────┘  │
├─────────────────────────────────────┤
│             error.rs                │ ← 統一エラーハンドリング
└─────────────────────────────────────┘
```

## 📁 ディレクトリ構造

```
src/
├── main.rs                    # エントリーポイント
├── lib.rs                     # ライブラリクレート
├── error.rs                   # エラー定義
├── editor/                    # エディタコア層
│   ├── mod.rs
│   ├── buffer.rs             # テキストバッファ管理
│   └── cursor.rs             # カーソル位置管理
├── vim/                       # VIM機能層
│   ├── mod.rs
│   ├── mode.rs               # モード管理（Normal/Insert/Visual/Command）
│   ├── command.rs            # VIMコマンド定義・実行
│   └── keymap.rs             # キーバインディング解析
└── io/                        # I/O層
    ├── mod.rs
    ├── fs.rs                 # ファイルシステム操作
    └── terminal.rs           # ターミナル操作
```

## 🎯 設計原則

### 1. Rust-idiomatic設計

Rustの特性を活かした自然な設計を採用：

```rust
// 所有権を活用したメモリ安全性
pub struct Buffer {
    lines: Vec<String>,
    modified: bool,
    undo_stack: Vec<Action>,
}

// Copy traitによる軽量なPosition
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

// Resultによる明示的エラーハンドリング
pub fn save_to_file(&self, path: &Path) -> Result<(), RvimError>
```

### 2. 単一責任の原則 (SRP)

各モジュールは**一つの責任**のみを持つ：

- `editor/buffer.rs`: テキストバッファの管理のみ
- `editor/cursor.rs`: カーソル位置の計算のみ  
- `vim/mode.rs`: モード状態の管理のみ
- `vim/keymap.rs`: キーバインディングの解析のみ
- `vim/command.rs`: VIMコマンドの実行のみ

### 3. モジュール分離とテスタビリティ

機能ごとに明確に分離され、独立してテスト可能：

```rust
// 各モジュールが独立してテスト可能
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_buffer_operations() {
        let mut buffer = Buffer::new();
        buffer.insert_text(0, 0, "Hello");
        assert_eq!(buffer.get_line(0), Some("Hello"));
    }
}
```

## 🔄 データフロー

### 1. キー入力の処理フロー

```
[ユーザー入力] 
    ↓
[io/terminal.rs::read_key()] 
    ↓
[vim/keymap.rs::parse_key()] 
    ↓
[vim/command.rs::execute()] 
    ↓
[editor/] (buffer.rs, cursor.rs)
    ↓
[io/terminal.rs::render()]
```

### 2. ファイル操作フロー

```
[ユーザーコマンド :w]
    ↓
[vim/keymap.rs::parse_command()]
    ↓
[vim/command.rs::SaveFile]
    ↓
[io/fs.rs::write_file()]
    ↓
[ファイルシステム]
```

### 3. エディタ状態管理フロー

```
[Editor State] (main.rs)
    ├── buffer: Buffer (editor/buffer.rs)
    ├── cursor: Position (editor/cursor.rs)  
    ├── mode: ModeManager (vim/mode.rs)
    ├── key_mapper: KeyMapper (vim/keymap.rs)
    └── terminal: Terminal (io/terminal.rs)
```

## 🧪 テスト戦略

### テストピラミッド

```
     🔺
    E2E        ← 実際のファイル操作テスト（少数）
   ─────
  Integration  ← モジュール間連携テスト（中程度）
 ───────────
Unit Tests     ← 各モジュールの単体テスト（多数）
```

### 実際の実装状況

#### 1. 単体テスト (Unit Tests) - 119テスト
- **対象**: 各モジュールの純粋関数
- **特徴**: 高速、独立、決定的
- **実装例**: `Position::move_right()`, `Buffer::insert_text()`

```rust
#[test]
fn test_cursor_move_right() {
    let position = Position::new(0, 0);
    let new_position = position.move_right(10);
    assert_eq!(new_position.col, 1);
}
```

#### 2. 統合テスト (Integration Tests) 
- **対象**: VIMコマンドと編集機能の連携
- **特徴**: 実際のファイル操作、コンポーネント間連携
- **実装例**: `vim/tests/`, `editor/tests/`

```rust
#[test]
fn test_vim_editing_integration() {
    let mut buffer = Buffer::new();
    let position = Position::new(0, 0);
    
    // VIMコマンドの統合テスト
    let result = execute_vim_command(&mut buffer, VimCommand::InsertChar('H'));
    assert!(result.is_ok());
}
```

## 🚀 拡張ポイント

### 1. 新しいVIMコマンドの追加

1. `vim/command.rs`の`VimCommand` enumに新しいバリアント追加
2. `vim/keymap.rs`にキーマッピング追加
3. `vim/command.rs`の`execute()`に実行ロジック追加
4. 対応するテストケース追加

### 2. 新しいファイル形式のサポート

1. `io/fs.rs`に新しいファイル操作関数追加
2. `editor/buffer.rs`に形式固有の読み込み・保存ロジック追加
3. `error.rs`に新しいエラータイプ追加（必要に応じて）
4. テストケース追加

### 3. 新しいUI機能の追加

1. `io/terminal.rs`に新しい描画メソッド追加
2. `main.rs`の描画ロジック更新
3. モード表示やステータスライン機能の拡張
4. ターミナル操作のテストケース追加

## 📊 依存関係マップ

```
main.rs (Editor)
  ├── editor/buffer.rs
  ├── editor/cursor.rs  
  ├── vim/mode.rs
  ├── vim/keymap.rs
  ├── vim/command.rs
  ├── io/terminal.rs
  ├── io/fs.rs
  └── error.rs

内部依存関係:
vim/command.rs → editor/ (Buffer, Position)
vim/keymap.rs → vim/command.rs  
main.rs → all modules
```

## 🎯 設計決定の記録

### 1. なぜRust-idiomatic設計を採用したか？

- **所有権システム**: Rustの特性を活かしたメモリ安全な設計
- **シンプルさ**: Clean Architectureの複雑さを避け、実用性を重視
- **テスタビリティ**: モジュール分離による優れたテスト可能性

### 2. なぜTDDを採用したか？

- **品質保証**: 119のテストによる高い品質保証
- **リファクタリング安全性**: テストによる安全なコード変更
- **仕様明確化**: テストファーストによる要件の明確化

### 3. なぜモジュール分離設計を選んだか？

- **責任分離**: 各モジュールが明確な責任を持つ
- **保守性**: 変更の影響範囲を限定
- **拡張性**: 新機能追加時の既存コード変更を最小化

### 4. なぜRust 2024 Editionを使用したか？

- **最新機能**: Rustの最新の言語機能を活用
- **パフォーマンス**: ネイティブコードレベルの高速動作
- **型安全性**: コンパイル時のバグ検出とメモリ安全性

---

**RVIM Architecture** - Clean, Testable, Extensible 🦀