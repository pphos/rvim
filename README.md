# RVIM - Rust VIM-like Editor

Rust で VIM ライクなキーバインディングを持つターミナルベースのテキストエディタ by Claude Code

## 🎯 主要機能

### ✅ 実装済み機能

**基本 VIM コマンド:**

- `h,j,k,l` - カーソル移動
- `w,b` - 単語移動
- `0,$` - 行の開始・終端移動
- `gg,G` - ファイルの開始・終端移動
- `i,a,o,O` - Insert モード移行
- `v` - Visual モード
- `:w` - ファイル保存
- `:q` - 終了
- `:wq` - 保存して終了
- `:q!` - 強制終了
- `u` - Undo
- `Ctrl+r` - Redo
- `x` - 文字削除
- `dd` - 行削除
- `Backspace` - Insert モードでの文字削除・行結合

**エディタ機能:**

- ✅ **コマンドライン表示** - `:` コマンド入力時に画面下部で内容表示
- ✅ **カーソル位置ハイライト表示** - 現在のカーソル位置が白背景で強調表示
- ✅ **Insert モードでの文字削除改善** - Backspace で正しく削除・行結合
- ✅ **マルチライン編集** - 改行・行削除・行結合
- ✅ **Undo/Redo** - 完全な操作履歴管理
- ✅ **ファイル I/O** - 読み込み・保存・新規作成
- ✅ **モード表示** - ステータスラインにモード表示

## 🚀 使用方法

### インストール・実行

```bash
# 新規ファイル
cargo run

# 既存ファイル編集
cargo run test.txt

# ヘルプ表示
cargo run -- --help
```

### 基本操作

1. **Normal Mode (初期状態):**

   - `h,j,k,l` でカーソル移動
   - `i` で Insert Mode に移行
   - `v` で Visual Mode に移行
   - `:` で Command Mode に移行

2. **Insert Mode:**

   - 通常の文字入力
   - `Backspace` で文字削除・行結合
   - `Enter` で改行
   - `Esc` で Normal Mode に復帰

3. **Command Mode:**
   - `:` キーでコマンドモードに入り、画面下部にコマンドが表示
   - `w` - ファイル保存
   - `q` - 終了（未保存の場合は警告）
   - `wq` - 保存して終了
   - `q!` - 強制終了（未保存でも終了）
   - `Backspace` - コマンド文字削除
   - `Enter` - コマンド実行
   - `Esc` で Normal Mode に復帰

## 🏗️ アーキテクチャ

### Rust-idiomatic設計 + TDD実践

```
┌─────────────────┐
│   Terminal UI   │ ← crossterm
├─────────────────┤
│  VIM Bindings   │ ← vim/keymap.rs, vim/command.rs
├─────────────────┤
│   Editor Core   │ ← editor/buffer.rs, editor/cursor.rs
├─────────────────┤
│       I/O       │ ← io/fs.rs, io/terminal.rs
└─────────────────┘
```

### 主要コンポーネント

- **Editor Core**: `buffer.rs`, `cursor.rs` - テキスト編集の核心機能
- **VIM Layer**: `mode.rs`, `command.rs`, `keymap.rs` - VIMライクな操作
- **I/O Layer**: `fs.rs`, `terminal.rs` - ファイル・ターミナル操作
- **Error Handling**: `error.rs` - 統一エラーハンドリング

## 📊 品質指標

- **テスト数**: 119 テスト全て成功 ✅ (115 + 4 main tests)
- **テスト駆動開発**: Red-Green-Refactor サイクル完全実践 ✅
- **Rust-idiomatic設計**: 型安全性とエラーハンドリング ✅
- **モジュール構造**: 機能別の明確な責任分離 ✅

## 🔧 技術スタック

- **言語**: Rust 2024 Edition
- **ターミナル**: crossterm 0.29
- **CLI**: clap 4.5
- **エラーハンドリング**: anyhow 1.0, thiserror 2.0
- **テスト**: assert_matches, pretty_assertions, tempfile
- **モック**: mockall 0.13
- **ベンチマーク**: criterion 0.7

## 🐛 修正された問題

### ✅ Issue: 文字入力後に削除できない

- **原因**: Insert Mode での Backspace 処理が不完全
- **修正**: `VimCommand::Backspace`を追加し、行結合も対応

### ✅ Issue: カーソル位置が分からない

- **原因**: ターミナルカーソルが見づらい
- **修正**: カーソル位置のハイライト表示を実装

### ✅ Issue: コマンドライン機能の実装

- **要求**: `:` コマンド入力時に画面下部でコマンド内容を表示
- **実装**: CommandLine 状態管理とリアルタイム表示機能を追加

## 🎯 開発方針

1. **TDD 実践** - 全機能で Red-Green-Refactor サイクル
2. **Clean Architecture** - 依存関係逆転による疎結合設計
3. **型安全性** - Rust の型システムを活用
4. **テスト可能性** - mock を活用した単体テスト

---

**RVIM** - シンプルで実用的な VIM ライクエディタ 🦀
