# RVIM 開発者ガイド

## 🚀 開発環境セットアップ

### 必要な環境

```bash
# Rust toolchain (最新安定版推奨)
rustc 1.70.0+
cargo 1.70.0+

# 開発ツール
clippy    # 静的解析
rustfmt   # コードフォーマット
```

### プロジェクトセットアップ

```bash
# リポジトリクローン
git clone <repository-url>
cd rvim

# 依存関係インストール
cargo build

# テスト実行
cargo test

# 実行
cargo run
```

## 🧪 TDD開発フロー

### Red-Green-Refactor サイクル

#### 1. 🔴 Red: 失敗するテストを書く

```rust
#[test]
fn test_new_feature() {
    // Given
    let input = setup_test_data();
    
    // When
    let result = new_feature(input);
    
    // Then
    assert_eq!(result, expected_output);
}
```

#### 2. 🟢 Green: テストを通す最小実装

```rust
fn new_feature(input: Input) -> Output {
    // とりあえずテストが通る最小実装
    expected_output
}
```

#### 3. 🔄 Refactor: 実装を改善

```rust
fn new_feature(input: Input) -> Output {
    // より良い実装に改善
    // テストが保護してくれる
    improved_implementation(input)
}
```

## 📝 コーディング規約

### Rust コーディングスタイル

```rust
// ✅ 良い例
pub struct EditorState {
    cursor: CursorPosition,
    buffer: TextBuffer,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            cursor: CursorPosition::new(0, 0),
            buffer: TextBuffer::new(),
        }
    }
}

// ❌ 悪い例
pub struct editorstate {
    pub cursor: CursorPosition,
    pub buffer: TextBuffer,
}
```

### 命名規約

- **構造体**: `PascalCase` (例: `EditorService`)
- **関数・変数**: `snake_case` (例: `move_cursor`)
- **定数**: `SCREAMING_SNAKE_CASE` (例: `MAX_BUFFER_SIZE`)
- **trait**: `PascalCase` (例: `FileSystem`)

### エラーハンドリング

```rust
// ✅ Result型を使用
fn read_file(path: &Path) -> Result<String, Error> {
    std::fs::read_to_string(path)
        .context("Failed to read file")
}

// ❌ panic!を使用しない
fn read_file(path: &Path) -> String {
    std::fs::read_to_string(path)
        .expect("File should exist") // 避ける
}
```

## 🏗️ 新機能の実装手順

### 1. Domain層の実装

```bash
# 1. テストファイル作成
touch src/domain/new_feature.rs

# 2. テスト駆動で実装
cargo test new_feature_tests
```

### 2. Application層の統合

```rust
// VimCommandに新しいコマンド追加
pub enum VimCommand {
    // 既存コマンド...
    NewFeature,
}

// パーサーに追加
fn parse_normal_mode_key(&mut self, key_event: KeyEvent) -> VimCommand {
    match key_event {
        // 新しいキーバインディング
        KeyEvent { code: KeyCode::Char('n'), .. } => VimCommand::NewFeature,
        // 既存の処理...
    }
}
```

### 3. Infrastructure層の更新（必要な場合）

```rust
// 新しい外部依存が必要な場合
pub trait NewDependency {
    fn new_operation(&self) -> Result<()>;
}
```

## 🧪 テスト作成ガイド

### 単体テストの書き方

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_movement() {
        // Arrange (Given)
        let mut cursor = CursorPosition::new(0, 0);
        
        // Act (When)
        cursor.move_right(10);
        
        // Assert (Then)
        assert_eq!(cursor.col, 1);
    }
}
```

### Mockを使った統合テスト

```rust
#[test]
fn test_editor_service_integration() {
    // Mock作成
    let mut mock_fs = MockFileSystem::new();
    mock_fs.expect_read_file()
           .with(eq(PathBuf::from("test.txt")))
           .times(1)
           .returning(|_| Ok("content".to_string()));

    // テスト実行
    let service = EditorService::new(mock_fs, mock_terminal);
    let result = service.load_file("test.txt");
    
    assert!(result.is_ok());
}
```

### テスト命名規約

```rust
// パターン: test_[何を]_[どんな条件で]_[期待する結果]
#[test]
fn test_cursor_move_right_at_line_end_stays_at_end() { }

#[test]
fn test_buffer_insert_char_at_middle_shifts_content() { }

#[test]
fn test_editor_save_file_with_valid_path_succeeds() { }
```

## 🔍 デバッグ手法

### ログ出力

```rust
// 開発時のデバッグ出力
#[cfg(debug_assertions)]
eprintln!("Debug: cursor position = {:?}", cursor);

// トレースログ
log::trace!("Executing command: {:?}", command);
```

### テストでのデバッグ

```rust
#[test]
fn test_complex_scenario() {
    let state = setup_complex_state();
    
    // 中間状態の確認
    println!("State before: {:?}", state);
    
    let result = complex_operation(state);
    
    // 結果の詳細確認
    println!("Result: {:?}", result);
    assert_eq!(result.field, expected_value);
}
```

## 🔧 開発ツール

### Cargo コマンド

```bash
# 開発中によく使うコマンド
cargo check          # 高速コンパイルチェック
cargo test           # テスト実行
cargo test -- --nocapture  # printlnの出力を表示
cargo clippy         # 静的解析
cargo fmt            # コードフォーマット
cargo doc --open     # ドキュメント生成・表示

# パフォーマンステスト
cargo bench

# 特定のテストのみ実行
cargo test cursor_tests
```

### CI/CD設定

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - name: Run tests
      run: cargo test
    - name: Run clippy
      run: cargo clippy -- -D warnings
    - name: Check formatting
      run: cargo fmt -- --check
```

## 📊 品質管理

### コードカバレッジ

```bash
# カバレッジ測定
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### パフォーマンス測定

```rust
// benches/editor_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_cursor_movement(c: &mut Criterion) {
    c.bench_function("cursor move right", |b| {
        b.iter(|| {
            let mut cursor = CursorPosition::new(0, 0);
            cursor.move_right(black_box(100));
        })
    });
}

criterion_group!(benches, benchmark_cursor_movement);
criterion_main!(benches);
```

## 🐛 トラブルシューティング

### よくある問題と解決法

#### 1. 借用チェッカーエラー

```rust
// ❌ 問題のあるコード
let line = buffer.get_line(0);
buffer.insert_char(0, 0, 'x');  // エラー: 借用競合

// ✅ 解決方法
let line_content = buffer.get_line(0).cloned();
buffer.insert_char(0, 0, 'x');
```

#### 2. テストでのMockセットアップ

```rust
// ✅ 正しいMockセットアップ
mock_terminal.expect_write()
             .with(eq("expected text"))
             .times(1)
             .returning(|_| Ok(()));
```

#### 3. ターミナル関連のテスト

```rust
// ターミナル操作は実際の環境では実行しない
#[test]
fn test_terminal_operations() {
    if std::env::var("CI").is_ok() {
        // CI環境ではスキップ
        return;
    }
    
    // 実際のターミナルテスト
}
```

## 📚 参考資料

### Rust関連
- [The Rust Programming Language](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

### アーキテクチャ関連
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)

### TDD関連
- [Test Driven Development: By Example](https://www.kent-beck.com/tdd-by-example)

---

**Happy Coding!** 🦀✨