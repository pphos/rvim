# RVIM - Rust VIM-like Editor

## プロジェクト概要

Rust で VIM ライクなキーバインディングを持つターミナルベースのテキストエディタ。
型安全性、エラーハンドリング、テスタビリティを重視した Rust らしい設計で実装。

## プロジェクト名

**RVIM** (Rust VIM-like Editor)

## 対象プラットフォーム

- macOS (Primary target)

## 基本要件

### 1. テキスト編集機能 (CRUD)

#### Create (作成)

- 新規ファイルの作成
- 空のバッファでエディタを起動
- テキスト挿入・追加機能

#### Read (読み込み)

- 既存ファイルの読み込み
- ファイル内容の表示
- 大きなファイルの効率的な読み込み

#### Update (更新)

- テキストの編集・変更
- 行の挿入・削除
- 文字の挿入・削除・置換
- Undo/Redo 機能

#### Delete (削除)

- 文字・単語・行の削除
- 選択範囲の削除
- ファイル全体のクリア

### 2. VIM バインディングサポート

#### モード管理

- **Normal Mode**: ナビゲーションとコマンド実行
- **Insert Mode**: テキスト挿入
- **Visual Mode**: テキスト選択
- **Command Mode**: コマンド入力

#### 基本ナビゲーション (Normal Mode)

- `h`, `j`, `k`, `l`: カーソル移動
- `w`, `b`: 単語移動
- `0`, `$`: 行の始端・終端
- `gg`, `G`: ファイルの始端・終端
- `Ctrl+f`, `Ctrl+b`: ページスクロール

#### 編集コマンド (Normal Mode)

- `i`, `a`: Insert Mode への移行
- `o`, `O`: 新しい行の作成と Insert Mode
- `x`, `dd`: 削除コマンド
- `yy`, `p`: コピー・ペースト
- `u`, `Ctrl+r`: Undo・Redo

#### Insert Mode

- 通常の文字入力
- `Esc`: Normal Mode への復帰
- `Backspace`, `Delete`: 文字削除

#### Command Mode

- `:w`: ファイル保存
- `:q`: エディタ終了
- `:wq`: 保存して終了
- `:q!`: 強制終了
- コマンド入力時の画面下部表示
- Backspace による入力文字削除

## 技術仕様

### アーキテクチャ（Rust らしい設計）

```
┌─────────────────────────────────────┐
│              main.rs                │ ← エントリーポイント
│         (Editor struct)             │
├─────────────────────────────────────┤
│  rvim crate (lib.rs)                │ ← パブリックAPI
├─────────────────┬───────────────────┤
│   editor/       │     vim/          │ ← コア機能
│   - buffer.rs   │   - mode.rs       │
│   - cursor.rs   │   - command.rs    │
│                 │   - keymap.rs     │
├─────────────────┼───────────────────┤
│        io/      │     error.rs      │ ← インフラ＋エラー
│   - fs.rs       │                   │
│   - terminal.rs │                   │
└─────────────────┴───────────────────┘
```

### 依存関係

- **Terminal 操作**: `crossterm` 0.29
  - カーソルハイライト（背景色変更）
  - リアルタイム画面更新
- **コマンドライン引数**: `clap` 4.5
- **エラーハンドリング**: `anyhow` 1.0, `thiserror` 2.0
- **テスト**: `tempfile` 3.20, `assert_matches` 1.5, `pretty_assertions` 1.4
- **モック**: `mockall` 0.13
- **ベンチマーク**: `criterion` 0.7

### 型安全なデータ構造設計

#### Editor (main.rs)

```rust
struct Editor {
    buffer: Buffer,
    cursor: Position,
    mode_manager: ModeManager,
    key_mapper: KeyMapper,
    terminal: Terminal,
    file_path: Option<PathBuf>,
    should_quit: bool,
}
```

#### Buffer (editor/buffer.rs)

```rust
pub struct Buffer {
    lines: Vec<String>,
    file_path: Option<PathBuf>,
    modified: bool,
    undo_stack: Vec<Action>,
    redo_stack: Vec<Action>,
}
```

#### Position (editor/cursor.rs)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}
```

## パフォーマンス要件

- 最大 10MB までのファイルを快適に編集可能
- キー入力に対する応答時間 < 50ms
- メモリ使用量の効率化

## ユーザビリティ要件

- VIM ユーザーが直感的に使用可能
- 明確なモード表示
- **カーソル位置のハイライト表示**: カーソル位置を白背景でハイライト
- **リアルタイムコマンド表示**: コマンド入力時の画面下部への即座表示
- エラーメッセージの適切な表示
- ファイル保存状態の視覚的フィードバック

## 開発フェーズ（TDD 重視）✅ 完了

### Phase 1: コアドメイン設計 ✅

- [x] **テストファースト**: Position の移動ロジック (editor/cursor.rs)
- [x] **テストファースト**: Buffer の基本操作 (editor/buffer.rs)
- [x] **テストファースト**: Mode の状態遷移 (vim/mode.rs)
- [x] 統一エラーハンドリング (error.rs)

### Phase 2: 基本編集機能 ✅

- [x] **テストファースト**: 文字挿入・削除機能
- [x] **テストファースト**: 行操作（挿入・削除）
- [x] **テストファースト**: Undo/Redo メカニズム
- [x] ファイル I/O 実装 (io/fs.rs)

### Phase 3: VIM バインディング ✅

- [x] **テストファースト**: キーバインディング解析 (vim/keymap.rs)
- [x] **テストファースト**: コマンド実行エンジン (vim/command.rs)
- [x] **テストファースト**: モード遷移ロジック

### Phase 4: UI 統合 ✅

- [x] ターミナル操作実装 (io/terminal.rs)
- [x] 統合テストの作成
- [x] メインエディタロジック統合 (main.rs)

### Phase 5: デグレード修正・機能向上 ✅

- [x] **カーソルハイライト表示**: カーソル位置の視覚的フィードバック機能を復旧
  - `render_line_with_cursor_highlight`関数実装
  - crossterm の背景色変更機能を使用
  - 行末超過時の適切なカーソル表示
- [x] **Backspace機能修正**: 正しいBackspace動作（カーソル左の文字削除）を実装
  - `VimCommand::DeleteCharBackward`の追加
  - 行頭での前行結合処理の実装
  - カーソル移動と文字削除の連携
- [x] **Commandモード表示**: `:command`入力時の画面下部表示機能を復旧
  - `CommandInput`と`CommandBackspace`コマンドの追加
  - ModeManagerとの状態同期
  - リアルタイム入力表示
- [x] **コマンド解析機能**: `:q`, `:w`, `:wq`, `:q!`コマンドの解析・実行機能を実装
  - `ExecuteCommand(String)`による解析機能
  - 文字列パターンマッチングでのコマンド判定
  - 実行後の自動Normalモード復帰

### 完了基準達成状況

1. **全てのテストが Green** ✅ (115 テスト成功)
2. **Rust らしい設計** ✅ (型安全性、エラーハンドリング)
3. **リファクタリング完了** ✅ (Clean Architecture → Rust-idiomatic へ)
4. **ドキュメント更新** ✅
5. **UI機能復旧** ✅ (カーソルハイライト、コマンド表示)
6. **VIMコマンド対応** ✅ (基本的なファイル操作コマンド)

## 開発アプローチ（t-wada 推奨手法）

### テスト駆動開発（TDD）

**Red-Green-Refactor サイクルを厳守**

1. **Red**: 失敗するテストを最初に書く
2. **Green**: テストを通すための最小限のコードを書く
3. **Refactor**: コードを改善し、テストが通ることを確認

### 依存関係の逆転（DIP）

```rust
// 抽象に依存し、具象に依存しない
trait FileSystem {
    fn read_file(&self, path: &Path) -> Result<String, Error>;
    fn write_file(&self, path: &Path, content: &str) -> Result<(), Error>;
}

trait Terminal {
    fn read_key(&self) -> Result<Key, Error>;
    fn write(&self, content: &str) -> Result<(), Error>;
    fn clear(&self) -> Result<(), Error>;
}

// EditorCoreは抽象に依存
struct EditorCore<F: FileSystem, T: Terminal> {
    file_system: F,
    terminal: T,
    state: EditorState,
}
```

### テスト戦略

#### 1. 単体テスト（Fast）

- **ピュアな関数のテスト**: 副作用のないロジック
- **モックを使用**: 外部依存をモック化
- **境界値テスト**: エッジケースの検証

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        TestFileSystem {}
        impl FileSystem for TestFileSystem {
            fn read_file(&self, path: &Path) -> Result<String, Error>;
            fn write_file(&self, path: &Path, content: &str) -> Result<(), Error>;
        }
    }

    #[test]
    fn test_cursor_movement_right() {
        // Given
        let mut cursor = CursorPosition { row: 0, col: 0 };
        let line_length = 10;

        // When
        cursor.move_right(line_length);

        // Then
        assert_eq!(cursor.col, 1);
    }
}
```

#### 2. 統合テスト（Medium）

- **コンポーネント間の連携テスト**
- **実際のファイル I/O を使用**（テスト用一時ファイル）

#### 3. End-to-End テスト（Slow）

- **実際のターミナル操作をシミュレート**
- **ユーザーシナリオの検証**

### テストピラミッド

```
       /\
      /  \     E2E Tests (少数)
     /____\
    /      \   Integration Tests (中程度)
   /________\
  /          \ Unit Tests (大多数)
 /____________\
```

## ファイル構成

```
rvim/
├── Cargo.toml
├── README.md
├── CLAUDE.md                      # プロジェクト仕様書
├── DEVELOPMENT.md                 # 開発者ガイド
├── ARCHITECTURE.md                # アーキテクチャドキュメント
├── src/
│   ├── main.rs                    # エントリーポイント
│   ├── lib.rs                     # ライブラリクレート
│   ├── error.rs                   # エラー定義
│   ├── editor/                    # エディタコア
│   │   ├── mod.rs
│   │   ├── buffer.rs             # テキストバッファ管理
│   │   └── cursor.rs             # カーソル位置管理
│   ├── vim/                       # VIM機能
│   │   ├── mod.rs
│   │   ├── mode.rs               # モード管理（Normal/Insert/Visual/Command）
│   │   ├── command.rs            # VIMコマンド定義・実行
│   │   └── keymap.rs             # キーバインディング解析
│   └── io/                        # 入出力
│       ├── mod.rs
│       ├── fs.rs                 # ファイルシステム操作
│       └── terminal.rs           # ターミナル操作
├── tests/
│   ├── integration/               # 統合テスト
│   ├── e2e/                       # E2Eテスト
│   └── common/                    # テスト共通機能
└── target/                        # ビルド成果物
```

## 起動方法

```bash
# 新規ファイルで RVIM エディタを起動
rvim

# 既存ファイルを RVIM エディタで開く
rvim filename.txt
rvim /path/to/file.rs

# 複数ファイルを開く（将来の拡張機能）
rvim file1.txt file2.rs

# ヘルプ表示
rvim --help

# バージョン表示
rvim --version
```

### コマンドライン引数

- `<FILE>`: 編集するファイルのパス（オプション）
- `--help, -h`: ヘルプメッセージを表示
- `--version, -V`: バージョン情報を表示

## 成功基準

1. **テストカバレッジ > 85%**
2. **全てのテストが Green**
3. **TDD サイクルの完全実践**
4. **依存関係の逆転が適切に実装**
5. 基本的なテキスト編集が可能
6. 主要な VIM コマンドが動作
7. ファイルの保存・読み込みが安定動作
8. レスポンシブな UI
9. メモリリークなし

## 品質保証

- **静的解析**: clippy, rustfmt
- **メモリ安全性**: valgrind（必要に応じて）
- **パフォーマンステスト**: criterion.rs
- **コードレビュー**: 動作可能な最小の機能単位で実施

## 前提条件

コメントやコンソール中の返答は全て日本語で実施するがあります。

## 開発・コミット指針

### 機能単位での自動コミット

- **機能完了の定義**: テストが通り、型チェックが成功し、完全に動作する状態
- **自動コミット実行**: 各機能が完了次第、即座にコミットを実行
- **コミットメッセージ形式**: `[プレフィックス] 機能の簡潔な説明`
- **品質担保**: コミット前に必ずテスト実行とlintチェックを実施

#### コミット実行条件

1. **全テストが Green**: `cargo test`が成功
2. **型チェック通過**: `cargo check`が成功  
3. **Lint通過**: `cargo clippy`が成功
4. **フォーマット適用**: `cargo fmt`が適用済み
5. **機能が完全動作**: 追加・修正した機能が期待通りに動作

#### コミットタイミング

- 新機能追加完了時
- バグ修正完了時  
- リファクタリング完了時
- テスト追加・修正完了時
- ドキュメント更新完了時

**重要**: 中途半端な状態や動作しない状態でのコミットは禁止。必ず動作確認後にコミット実行。

#### コミットプレフィックスルール

- `[feat]` 新機能
- `[fix]` バグ修正  
- `[docs]` ドキュメントのみの変更
- `[style]` コードの意味に影響しない変更（空白、フォーマット、セミコロンなど）
- `[refactor]` バグ修正でも新機能でもないコード変更
- `[perf]` パフォーマンス改善のコード変更
- `[test]` テストの追加や既存テストの修正
- `[chore]` ビルドプロセスや補助ツール、ライブラリの変更
