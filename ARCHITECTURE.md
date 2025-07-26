# RVIM アーキテクチャドキュメント

## 🏗️ アーキテクチャ概要

RVIMは **Clean Architecture** と **TDD (Test-Driven Development)** の原則に基づいて設計されたVIMライクエディタです。

### アーキテクチャ図

```
┌─────────────────────────────────────┐
│              Terminal UI            │ ← crossterm による実装
├─────────────────────────────────────┤
│            Application              │ ← アプリケーションサービス層
│  ┌───────────────┐ ┌─────────────────┐ │
│  │ EditorService │ │  VimBindings    │ │
│  └───────────────┘ └─────────────────┘ │
├─────────────────────────────────────┤
│              Domain                 │ ← ビジネスロジック層
│  ┌─────────────┐ ┌─────────────────┐  │
│  │ TextBuffer  │ │ CursorPosition  │  │
│  └─────────────┘ └─────────────────┘  │
│  ┌─────────────┐ ┌─────────────────┐  │
│  │ EditorMode  │ │  ModeManager    │  │
│  └─────────────┘ └─────────────────┘  │
├─────────────────────────────────────┤
│           Infrastructure            │ ← 外部依存実装層
│  ┌─────────────┐ ┌─────────────────┐  │
│  │ FileSystem  │ │ CrosstermTerm   │  │
│  └─────────────┘ └─────────────────┘  │
├─────────────────────────────────────┤
│              Ports                  │ ← インターフェース定義
│  ┌─────────────┐ ┌─────────────────┐  │
│  │FileSystemTr │ │  TerminalTrait  │  │
│  └─────────────┘ └─────────────────┘  │
└─────────────────────────────────────┘
```

## 📁 ディレクトリ構造

```
src/
├── main.rs                    # エントリーポイント
├── lib.rs                     # ライブラリクレート
├── domain/                    # ドメイン層（ビジネスロジック）
│   ├── mod.rs
│   ├── cursor.rs             # カーソル位置管理
│   ├── text_buffer.rs        # テキストバッファ管理
│   └── editor_mode.rs        # エディタモード管理
├── application/               # アプリケーション層
│   ├── mod.rs
│   ├── editor_service.rs     # エディタサービス
│   └── vim_bindings.rs       # VIMキーバインディング
├── infrastructure/            # インフラストラクチャ層
│   ├── mod.rs
│   ├── file_system.rs        # ファイルシステム実装
│   └── terminal.rs           # ターミナル実装
└── ports/                     # ポート（インターフェース）
    ├── mod.rs
    ├── file_system.rs        # ファイルシステムtrait
    └── terminal.rs           # ターミナルtrait
```

## 🎯 設計原則

### 1. 依存関係逆転の原則 (DIP)

- **高レベルモジュール**は**低レベルモジュール**に依存しない
- 両方とも**抽象**に依存する
- **抽象**は**詳細**に依存しない、**詳細**が**抽象**に依存する

```rust
// ❌ 悪い例: 具象に直接依存
struct EditorService {
    file_system: std::fs::File,
    terminal: crossterm::Terminal,
}

// ✅ 良い例: 抽象（trait）に依存
struct EditorService<F: FileSystem, T: Terminal> {
    file_system: F,
    terminal: T,
}
```

### 2. 単一責任の原則 (SRP)

各クラス・モジュールは**一つの変更理由**のみを持つ：

- `CursorPosition`: カーソル位置の管理のみ
- `TextBuffer`: テキストの編集操作のみ
- `EditorMode`: モード状態の管理のみ
- `VimBindings`: キーバインディングの解析のみ

### 3. オープン・クローズドの原則 (OCP)

新しい機能追加時にも既存コードを変更せずに拡張可能：

```rust
// 新しいコマンドの追加
pub enum VimCommand {
    MoveLeft,
    MoveRight,
    // 新しいコマンドを追加してもパーサーの変更は不要
    NewCommand,
}
```

## 🔄 データフロー

### 1. キー入力の処理フロー

```
[ユーザー入力] 
    ↓
[Terminal::read_key()] 
    ↓
[VimBindings::parse_key()] 
    ↓
[EditorService::execute_command()] 
    ↓
[Domain Objects] (Cursor, Buffer, Mode)
    ↓
[Terminal::render()]
```

### 2. ファイル操作フロー

```
[ユーザーコマンド :w]
    ↓
[VimBindings::parse_command()]
    ↓
[EditorService::execute_vim_command()]
    ↓
[FileSystem::write_file()]
    ↓
[ファイルシステム]
```

## 🧪 テスト戦略

### テストピラミッド

```
     🔺
    E2E        ← 統合テスト（少数）
   ─────
  Integration  ← コンポーネント間テスト（中程度）
 ───────────
Unit Tests     ← 単体テスト（多数）
```

### テストタイプ別の実装

#### 1. 単体テスト (Unit Tests)
- **対象**: Domain層の純粋関数
- **特徴**: 高速、依存なし、決定的
- **例**: `CursorPosition::move_right()`

```rust
#[test]
fn test_cursor_move_right() {
    let mut cursor = CursorPosition::new(0, 0);
    cursor.move_right(10);
    assert_eq!(cursor.col, 1);
}
```

#### 2. 統合テスト (Integration Tests)
- **対象**: アプリケーション層のサービス
- **特徴**: Mock使用、コンポーネント間連携テスト
- **例**: `EditorService` のファイル保存

```rust
#[test]
fn test_editor_service_save() {
    let mut mock_fs = MockFileSystem::new();
    mock_fs.expect_write_file()
           .times(1)
           .returning(|_, _| Ok(()));
    
    let mut service = EditorService::new(mock_fs, mock_terminal);
    service.execute_command(VimCommand::Save);
}
```

## 🚀 拡張ポイント

### 1. 新しいVIMコマンドの追加

1. `VimCommand` enumに新しいバリアントを追加
2. `VimBindings::parse_key()` にキーマッピング追加
3. `VimBindings::execute_command()` に実行ロジック追加
4. テストケース追加

### 2. 新しいファイル形式のサポート

1. `FileSystem` traitに新しいメソッド追加
2. `StdFileSystem` に実装追加
3. `MockFileSystem` にMock実装追加
4. テストケース追加

### 3. 新しいUI機能の追加

1. `Terminal` traitに新しいメソッド追加
2. `CrosstermTerminal` に実装追加
3. `MockTerminal` にMock実装追加
4. `EditorService` の描画ロジック更新

## 📊 依存関係マップ

```
main.rs
  ↓
EditorService ← FileSystem (trait)
  ↓            ↑ 
VimBindings    StdFileSystem (impl)
  ↓            
Domain Objects ← Terminal (trait)
               ↑
               CrosstermTerminal (impl)
```

## 🎯 設計決定の記録

### 1. なぜClean Architectureを採用したか？

- **テスタビリティ**: Mockを使った完全なテスト分離
- **保守性**: 変更の影響範囲を限定
- **拡張性**: 新機能追加時の既存コード変更を最小化

### 2. なぜTDDを採用したか？

- **品質保証**: テストファーストによる仕様の明確化
- **リファクタリング安全性**: テストによる品質保証
- **設計改善**: テスト可能な設計の強制

### 3. なぜRustを選んだか？

- **メモリ安全性**: ガベージコレクションなしでの安全なメモリ管理
- **パフォーマンス**: ネイティブコードレベルの高速動作
- **型安全性**: コンパイル時のバグ検出

---

**RVIM Architecture** - Clean, Testable, Extensible 🦀