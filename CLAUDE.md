# RVIM - Rust VIM-like Editor

## プロジェクト概要

Rust で VIM ライクなキーバインディングを持つターミナルベースのテキストエディタを開発する。

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

## 技術仕様

### アーキテクチャ

```
┌─────────────────┐
│   Terminal UI   │ ← crossterm/termion
├─────────────────┤
│   Editor Core   │ ← 編集ロジック
├─────────────────┤
│   Buffer Mgmt   │ ← テキストバッファ管理
├─────────────────┤
│   File I/O      │ ← ファイル操作
└─────────────────┘
```

### 依存関係候補

- **Terminal 操作**: `crossterm` または `termion`
- **コマンドライン引数**: `clap`
- **ファイル I/O**: 標準ライブラリ + trait 抽象化
- **エラーハンドリング**: `anyhow` または `thiserror`
- **テスト**: `mockall` (モック), `tempfile` (一時ファイル)
- **アサーション**: `assert_matches`, `pretty_assertions`

### データ構造設計

#### EditorState

```rust
struct EditorState {
    mode: EditorMode,
    buffer: TextBuffer,
    cursor: CursorPosition,
    viewport: Viewport,
    command_history: Vec<Command>,
}
```

#### TextBuffer

```rust
struct TextBuffer {
    lines: Vec<String>,
    file_path: Option<PathBuf>,
    modified: bool,
    undo_stack: Vec<Action>,
    redo_stack: Vec<Action>,
}
```

## パフォーマンス要件

- 最大 10MB までのファイルを快適に編集可能
- キー入力に対する応答時間 < 50ms
- メモリ使用量の効率化

## ユーザビリティ要件

- VIM ユーザーが直感的に使用可能
- 明確なモード表示
- エラーメッセージの適切な表示
- ファイル保存状態の視覚的フィードバック

## 開発フェーズ（TDD 重視）

### Phase 1: コアドメイン設計

- [ ] **テストファースト**: CursorPosition の移動ロジック
- [ ] **テストファースト**: TextBuffer の基本操作
- [ ] **テストファースト**: EditorMode の状態遷移
- [ ] 抽象化レイヤー（traits）の定義

### Phase 2: 基本編集機能

- [ ] **テストファースト**: 文字挿入・削除機能
- [ ] **テストファースト**: 行操作（挿入・削除）
- [ ] **テストファースト**: Undo/Redo メカニズム
- [ ] ファイル I/O 抽象化の実装

### Phase 3: VIM バインディング

- [ ] **テストファースト**: キーバインディング解析
- [ ] **テストファースト**: コマンド実行エンジン
- [ ] **テストファースト**: モード遷移ロジック

### Phase 4: UI 統合

- [ ] ターミナル抽象化の実装
- [ ] 統合テストの作成
- [ ] E2E テストシナリオの実装

### 各フェーズの完了基準

1. **全てのテストが Green**
2. **コードカバレッジ > 85%**
3. **リファクタリング完了**
4. **ドキュメント更新**

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
├── src/
│   ├── main.rs                    # エントリーポイント（薄いレイヤー）
│   ├── lib.rs                     # ライブラリクレート
│   ├── domain/                    # ドメインロジック（純粋関数）
│   │   ├── mod.rs
│   │   ├── cursor.rs             # カーソル操作
│   │   ├── text_buffer.rs        # テキストバッファ
│   │   ├── editor_mode.rs        # モード管理
│   │   └── commands.rs           # コマンド定義
│   ├── application/               # アプリケーションサービス
│   │   ├── mod.rs
│   │   ├── editor_service.rs     # エディタ操作の協調
│   │   └── vim_bindings.rs       # VIMキーバインディング
│   ├── infrastructure/            # 外部依存の実装
│   │   ├── mod.rs
│   │   ├── file_system.rs        # ファイルI/O実装
│   │   └── terminal.rs           # ターミナル操作実装
│   └── ports/                     # インターフェース定義
│       ├── mod.rs
│       ├── file_system.rs        # ファイルI/O trait
│       └── terminal.rs           # ターミナル trait
├── tests/
│   ├── integration/               # 統合テスト
│   │   ├── mod.rs
│   │   └── editor_integration.rs
│   ├── e2e/                       # E2Eテスト
│   │   ├── mod.rs
│   │   └── user_scenarios.rs
│   └── common/                    # テスト共通機能
│       ├── mod.rs
│       └── test_helpers.rs
└── benches/                       # パフォーマンステスト
    └── editor_benchmarks.rs
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
