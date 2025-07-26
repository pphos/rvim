use crate::editor::cursor::Position;
use crate::error::{EditorError, Result};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Action {
    pub action_type: ActionType,
    pub position: Position,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    Insert,
    Delete,
    InsertLine,
    DeleteLine,
}

#[derive(Debug, Clone)]
pub struct Buffer {
    lines: Vec<String>,
    file_path: Option<PathBuf>,
    modified: bool,
    undo_stack: Vec<Action>,
    redo_stack: Vec<Action>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            file_path: None,
            modified: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn from_content(content: &str) -> Self {
        let lines = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|line| line.to_string()).collect()
        };

        Self {
            lines,
            file_path: None,
            modified: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn with_file_path(mut self, path: PathBuf) -> Self {
        self.file_path = Some(path);
        self
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn line(&self, index: usize) -> Result<&str> {
        self.lines
            .get(index)
            .map(|s| s.as_str())
            .ok_or_else(|| EditorError::out_of_bounds(index, 0))
    }

    pub fn line_length(&self, index: usize) -> Result<usize> {
        self.lines
            .get(index)
            .map(|line| line.len())
            .ok_or_else(|| EditorError::out_of_bounds(index, 0))
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    pub fn insert_char(&mut self, pos: Position, ch: char) -> Result<()> {
        if pos.row >= self.lines.len() {
            return Err(EditorError::out_of_bounds(pos.row, pos.col));
        }

        let line = &mut self.lines[pos.row];
        if pos.col > line.len() {
            return Err(EditorError::out_of_bounds(pos.row, pos.col));
        }

        let action = Action {
            action_type: ActionType::Insert,
            position: pos,
            content: ch.to_string(),
        };

        line.insert(pos.col, ch);
        self.modified = true;
        self.push_action(action);

        Ok(())
    }

    pub fn delete_char(&mut self, pos: Position) -> Result<Option<char>> {
        if pos.row >= self.lines.len() {
            return Err(EditorError::out_of_bounds(pos.row, pos.col));
        }

        let line = &mut self.lines[pos.row];
        if pos.col >= line.len() {
            return Ok(None);
        }

        let deleted_char = line.remove(pos.col);
        let action = Action {
            action_type: ActionType::Delete,
            position: pos,
            content: deleted_char.to_string(),
        };

        self.modified = true;
        self.push_action(action);

        Ok(Some(deleted_char))
    }

    pub fn insert_line(&mut self, row: usize) -> Result<()> {
        if row > self.lines.len() {
            return Err(EditorError::out_of_bounds(row, 0));
        }

        self.lines.insert(row, String::new());
        self.modified = true;

        let action = Action {
            action_type: ActionType::InsertLine,
            position: Position::new(row, 0),
            content: String::new(),
        };
        self.push_action(action);

        Ok(())
    }

    pub fn delete_line(&mut self, row: usize) -> Result<Option<String>> {
        if row >= self.lines.len() {
            return Err(EditorError::out_of_bounds(row, 0));
        }

        if self.lines.len() == 1 {
            let content = self.lines[0].clone();
            self.lines[0].clear();
            self.modified = true;

            let action = Action {
                action_type: ActionType::DeleteLine,
                position: Position::new(row, 0),
                content: content.clone(),
            };
            self.push_action(action);

            return Ok(Some(content));
        }

        let deleted_line = self.lines.remove(row);
        self.modified = true;

        let action = Action {
            action_type: ActionType::DeleteLine,
            position: Position::new(row, 0),
            content: deleted_line.clone(),
        };
        self.push_action(action);

        Ok(Some(deleted_line))
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self) -> Result<()> {
        let action = self.undo_stack.pop().ok_or(EditorError::EmptyUndoStack)?;

        match action.action_type {
            ActionType::Insert => {
                let pos = action.position;
                if pos.row < self.lines.len() && pos.col < self.lines[pos.row].len() {
                    self.lines[pos.row].remove(pos.col);
                }
            }
            ActionType::Delete => {
                let pos = action.position;
                if pos.row < self.lines.len() && pos.col <= self.lines[pos.row].len() {
                    if let Some(ch) = action.content.chars().next() {
                        self.lines[pos.row].insert(pos.col, ch);
                    }
                }
            }
            ActionType::InsertLine => {
                let pos = action.position;
                if pos.row < self.lines.len() {
                    self.lines.remove(pos.row);
                }
            }
            ActionType::DeleteLine => {
                let pos = action.position;
                if pos.row <= self.lines.len() {
                    self.lines.insert(pos.row, action.content.clone());
                }
            }
        }

        self.redo_stack.push(action);
        self.modified = !self.undo_stack.is_empty();
        Ok(())
    }

    pub fn redo(&mut self) -> Result<()> {
        let action = self.redo_stack.pop().ok_or(EditorError::EmptyRedoStack)?;

        match action.action_type {
            ActionType::Insert => {
                let pos = action.position;
                if pos.row < self.lines.len() && pos.col <= self.lines[pos.row].len() {
                    if let Some(ch) = action.content.chars().next() {
                        self.lines[pos.row].insert(pos.col, ch);
                    }
                }
            }
            ActionType::Delete => {
                let pos = action.position;
                if pos.row < self.lines.len() && pos.col < self.lines[pos.row].len() {
                    self.lines[pos.row].remove(pos.col);
                }
            }
            ActionType::InsertLine => {
                let pos = action.position;
                if pos.row <= self.lines.len() {
                    self.lines.insert(pos.row, action.content.clone());
                }
            }
            ActionType::DeleteLine => {
                let pos = action.position;
                if pos.row < self.lines.len() {
                    self.lines.remove(pos.row);
                }
            }
        }

        self.undo_stack.push(action);
        self.modified = true;
        Ok(())
    }

    pub fn mark_saved(&mut self) {
        self.modified = false;
    }

    fn push_action(&mut self, action: Action) {
        self.undo_stack.push(action);
        self.redo_stack.clear();
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buffer = Buffer::new();
        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.line(0).unwrap(), "");
        assert!(!buffer.is_modified());
        assert!(buffer.file_path().is_none());
    }

    #[test]
    fn test_from_content_empty() {
        let buffer = Buffer::from_content("");
        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.line(0).unwrap(), "");
    }

    #[test]
    fn test_from_content_single_line() {
        let buffer = Buffer::from_content("Hello, world!");
        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.line(0).unwrap(), "Hello, world!");
    }

    #[test]
    fn test_from_content_multiple_lines() {
        let buffer = Buffer::from_content("Line 1\nLine 2\nLine 3");
        assert_eq!(buffer.line_count(), 3);
        assert_eq!(buffer.line(0).unwrap(), "Line 1");
        assert_eq!(buffer.line(1).unwrap(), "Line 2");
        assert_eq!(buffer.line(2).unwrap(), "Line 3");
    }

    #[test]
    fn test_with_file_path() {
        let path = PathBuf::from("/path/to/file.txt");
        let buffer = Buffer::new().with_file_path(path.clone());
        assert_eq!(buffer.file_path(), Some(&path));
    }

    #[test]
    fn test_insert_char() {
        let mut buffer = Buffer::from_content("Hello");
        let pos = Position::new(0, 5);

        buffer.insert_char(pos, '!').unwrap();
        assert_eq!(buffer.line(0).unwrap(), "Hello!");
        assert!(buffer.is_modified());
    }

    #[test]
    fn test_insert_char_middle() {
        let mut buffer = Buffer::from_content("Hello");
        let pos = Position::new(0, 2);

        buffer.insert_char(pos, 'X').unwrap();
        assert_eq!(buffer.line(0).unwrap(), "HeXllo");
    }

    #[test]
    fn test_insert_char_out_of_bounds() {
        let mut buffer = Buffer::from_content("Hello");
        let pos = Position::new(1, 0);

        let result = buffer.insert_char(pos, 'X');
        assert!(result.is_err());
        assert!(!buffer.is_modified());
    }

    #[test]
    fn test_delete_char() {
        let mut buffer = Buffer::from_content("Hello");
        let pos = Position::new(0, 1);

        let deleted = buffer.delete_char(pos).unwrap();
        assert_eq!(deleted, Some('e'));
        assert_eq!(buffer.line(0).unwrap(), "Hllo");
        assert!(buffer.is_modified());
    }

    #[test]
    fn test_delete_char_at_end() {
        let mut buffer = Buffer::from_content("Hello");
        let pos = Position::new(0, 5);

        let deleted = buffer.delete_char(pos).unwrap();
        assert_eq!(deleted, None);
        assert_eq!(buffer.line(0).unwrap(), "Hello");
    }

    #[test]
    fn test_line_length() {
        let buffer = Buffer::from_content("Hello\nWorld!");
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.line_length(0).unwrap(), 5);
        assert_eq!(buffer.line_length(1).unwrap(), 6);

        // 空行も含む場合のテスト（Rustのlines()は末尾改行で空行を作らない）
        let buffer2 = Buffer::from_content("Hello\nWorld!\n");
        assert_eq!(buffer2.line_count(), 2); // lines()は末尾改行を無視
        assert_eq!(buffer2.line_length(0).unwrap(), 5);
        assert_eq!(buffer2.line_length(1).unwrap(), 6);

        // 明示的に空行を含む場合
        let buffer3 = Buffer::from_content("Hello\n\nWorld!");
        assert_eq!(buffer3.line_count(), 3);
        assert_eq!(buffer3.line_length(0).unwrap(), 5);
        assert_eq!(buffer3.line_length(1).unwrap(), 0); // 空行
        assert_eq!(buffer3.line_length(2).unwrap(), 6);
    }

    #[test]
    fn test_line_length_out_of_bounds() {
        let buffer = Buffer::from_content("Hello");
        let result = buffer.line_length(999);
        assert!(result.is_err());
    }

    #[test]
    fn test_undo_insert_char() {
        let mut buffer = Buffer::from_content("Hello");
        let pos = Position::new(0, 5);

        buffer.insert_char(pos, '!').unwrap();
        assert_eq!(buffer.line(0).unwrap(), "Hello!");

        buffer.undo().unwrap();
        assert_eq!(buffer.line(0).unwrap(), "Hello");
        assert!(!buffer.is_modified());
    }

    #[test]
    fn test_undo_delete_char() {
        let mut buffer = Buffer::from_content("Hello");
        let pos = Position::new(0, 1);

        buffer.delete_char(pos).unwrap();
        assert_eq!(buffer.line(0).unwrap(), "Hllo");

        buffer.undo().unwrap();
        assert_eq!(buffer.line(0).unwrap(), "Hello");
        assert!(!buffer.is_modified());
    }

    #[test]
    fn test_redo_after_undo() {
        let mut buffer = Buffer::from_content("Hello");
        let pos = Position::new(0, 5);

        buffer.insert_char(pos, '!').unwrap();
        buffer.undo().unwrap();
        buffer.redo().unwrap();

        assert_eq!(buffer.line(0).unwrap(), "Hello!");
        assert!(buffer.is_modified());
    }

    #[test]
    fn test_can_undo_redo() {
        let mut buffer = Buffer::new();

        assert!(!buffer.can_undo());
        assert!(!buffer.can_redo());

        let pos = Position::new(0, 0);
        buffer.insert_char(pos, 'X').unwrap();
        assert!(buffer.can_undo());
        assert!(!buffer.can_redo());

        buffer.undo().unwrap();
        assert!(!buffer.can_undo());
        assert!(buffer.can_redo());
    }

    #[test]
    fn test_empty_undo_stack() {
        let mut buffer = Buffer::new();
        let result = buffer.undo();
        assert!(matches!(result, Err(EditorError::EmptyUndoStack)));
    }

    #[test]
    fn test_empty_redo_stack() {
        let mut buffer = Buffer::new();
        let result = buffer.redo();
        assert!(matches!(result, Err(EditorError::EmptyRedoStack)));
    }
}

#[cfg(test)]
pub mod test_helpers {
    use super::*;

    pub struct BufferBuilder {
        content: String,
        path: Option<PathBuf>,
    }

    impl BufferBuilder {
        pub fn new() -> Self {
            Self {
                content: String::new(),
                path: None,
            }
        }

        pub fn with_content(content: &str) -> Self {
            Self {
                content: content.to_string(),
                path: None,
            }
        }

        pub fn with_path<P: Into<PathBuf>>(mut self, path: P) -> Self {
            self.path = Some(path.into());
            self
        }

        pub fn build(self) -> Buffer {
            let mut buffer = Buffer::from_content(&self.content);
            if let Some(path) = self.path {
                buffer = buffer.with_file_path(path);
            }
            buffer
        }
    }

    impl Default for BufferBuilder {
        fn default() -> Self {
            Self::new()
        }
    }

    #[test]
    fn test_buffer_builder() {
        let buffer = BufferBuilder::with_content("test content")
            .with_path("/test/path.txt")
            .build();

        assert_eq!(buffer.line(0).unwrap(), "test content");
        assert_eq!(buffer.file_path(), Some(&PathBuf::from("/test/path.txt")));
    }
}
