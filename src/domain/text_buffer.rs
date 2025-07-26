use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Action {
    pub action_type: ActionType,
    pub position: (usize, usize),
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    Insert,
    Delete,
    Replace,
}

#[derive(Debug, Clone)]
pub struct TextBuffer {
    lines: Vec<String>,
    file_path: Option<PathBuf>,
    modified: bool,
    undo_stack: Vec<Action>,
    redo_stack: Vec<Action>,
}

impl TextBuffer {
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

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn get_line(&self, index: usize) -> Option<&String> {
        self.lines.get(index)
    }

    pub fn get_line_length(&self, index: usize) -> usize {
        self.lines.get(index).map_or(0, |line| line.len())
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }

    pub fn file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    pub fn insert_char(&mut self, row: usize, col: usize, ch: char) -> Result<(), String> {
        if row >= self.lines.len() {
            return Err(format!("Row {} out of bounds", row));
        }

        let line = &mut self.lines[row];
        if col > line.len() {
            return Err(format!("Column {} out of bounds for row {}", col, row));
        }

        let action = Action {
            action_type: ActionType::Insert,
            position: (row, col),
            content: ch.to_string(),
        };

        line.insert(col, ch);
        self.modified = true;
        self.undo_stack.push(action);
        self.redo_stack.clear();

        Ok(())
    }

    pub fn delete_char(&mut self, row: usize, col: usize) -> Result<Option<char>, String> {
        if row >= self.lines.len() {
            return Err(format!("Row {} out of bounds", row));
        }

        let line = &mut self.lines[row];
        if col >= line.len() {
            return Ok(None);
        }

        let deleted_char = line.remove(col);
        let action = Action {
            action_type: ActionType::Delete,
            position: (row, col),
            content: deleted_char.to_string(),
        };

        self.modified = true;
        self.undo_stack.push(action);
        self.redo_stack.clear();

        Ok(Some(deleted_char))
    }

    pub fn insert_line(&mut self, row: usize) -> Result<(), String> {
        if row > self.lines.len() {
            return Err(format!("Row {} out of bounds", row));
        }

        self.lines.insert(row, String::new());
        self.modified = true;

        let action = Action {
            action_type: ActionType::Insert,
            position: (row, 0),
            content: "\n".to_string(),
        };
        self.undo_stack.push(action);
        self.redo_stack.clear();

        Ok(())
    }

    pub fn delete_line(&mut self, row: usize) -> Result<Option<String>, String> {
        if row >= self.lines.len() {
            return Err(format!("Row {} out of bounds", row));
        }

        if self.lines.len() == 1 {
            let content = self.lines[0].clone();
            self.lines[0].clear();
            self.modified = true;

            let action = Action {
                action_type: ActionType::Delete,
                position: (row, 0),
                content: content.clone(),
            };
            self.undo_stack.push(action);
            self.redo_stack.clear();

            return Ok(Some(content));
        }

        let deleted_line = self.lines.remove(row);
        self.modified = true;

        let action = Action {
            action_type: ActionType::Delete,
            position: (row, 0),
            content: deleted_line.clone(),
        };
        self.undo_stack.push(action);
        self.redo_stack.clear();

        Ok(Some(deleted_line))
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self) -> Result<(), String> {
        if let Some(action) = self.undo_stack.pop() {
            match action.action_type {
                ActionType::Insert => {
                    if action.content == "\n" {
                        if action.position.0 < self.lines.len() {
                            self.lines.remove(action.position.0);
                        }
                    } else {
                        let (row, col) = action.position;
                        if row < self.lines.len() && col < self.lines[row].len() {
                            self.lines[row].remove(col);
                        }
                    }
                }
                ActionType::Delete => {
                    if action.content.contains('\n') {
                        self.lines
                            .insert(action.position.0, action.content.replace('\n', ""));
                    } else {
                        let (row, col) = action.position;
                        if row < self.lines.len() && col <= self.lines[row].len() {
                            if let Some(ch) = action.content.chars().next() {
                                self.lines[row].insert(col, ch);
                            }
                        }
                    }
                }
                ActionType::Replace => {}
            }
            self.redo_stack.push(action);
            self.modified = !self.undo_stack.is_empty();
            Ok(())
        } else {
            Err("Nothing to undo".to_string())
        }
    }

    pub fn redo(&mut self) -> Result<(), String> {
        if let Some(action) = self.redo_stack.pop() {
            match action.action_type {
                ActionType::Insert => {
                    if action.content == "\n" {
                        self.lines.insert(action.position.0, String::new());
                    } else {
                        let (row, col) = action.position;
                        if row < self.lines.len() && col <= self.lines[row].len() {
                            if let Some(ch) = action.content.chars().next() {
                                self.lines[row].insert(col, ch);
                            }
                        }
                    }
                }
                ActionType::Delete => {
                    if action.content.contains('\n') {
                        if action.position.0 < self.lines.len() {
                            self.lines.remove(action.position.0);
                        }
                    } else {
                        let (row, col) = action.position;
                        if row < self.lines.len() && col < self.lines[row].len() {
                            self.lines[row].remove(col);
                        }
                    }
                }
                ActionType::Replace => {}
            }
            self.undo_stack.push(action);
            self.modified = true;
            Ok(())
        } else {
            Err("Nothing to redo".to_string())
        }
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }

    pub fn mark_saved(&mut self) {
        self.modified = false;
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buffer = TextBuffer::new();
        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.get_line(0), Some(&String::new()));
        assert!(!buffer.is_modified());
        assert!(buffer.file_path().is_none());
    }

    #[test]
    fn test_from_content_empty() {
        let buffer = TextBuffer::from_content("");
        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.get_line(0), Some(&String::new()));
    }

    #[test]
    fn test_from_content_single_line() {
        let buffer = TextBuffer::from_content("Hello, world!");
        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.get_line(0), Some(&"Hello, world!".to_string()));
    }

    #[test]
    fn test_from_content_multiple_lines() {
        let buffer = TextBuffer::from_content("Line 1\nLine 2\nLine 3");
        assert_eq!(buffer.line_count(), 3);
        assert_eq!(buffer.get_line(0), Some(&"Line 1".to_string()));
        assert_eq!(buffer.get_line(1), Some(&"Line 2".to_string()));
        assert_eq!(buffer.get_line(2), Some(&"Line 3".to_string()));
    }

    #[test]
    fn test_with_file_path() {
        let path = PathBuf::from("/path/to/file.txt");
        let buffer = TextBuffer::new().with_file_path(path.clone());
        assert_eq!(buffer.file_path(), Some(&path));
    }

    #[test]
    fn test_insert_char() {
        let mut buffer = TextBuffer::from_content("Hello");

        buffer.insert_char(0, 5, '!').unwrap();
        assert_eq!(buffer.get_line(0), Some(&"Hello!".to_string()));
        assert!(buffer.is_modified());
    }

    #[test]
    fn test_insert_char_middle() {
        let mut buffer = TextBuffer::from_content("Hello");

        buffer.insert_char(0, 2, 'X').unwrap();
        assert_eq!(buffer.get_line(0), Some(&"HeXllo".to_string()));
    }

    #[test]
    fn test_insert_char_out_of_bounds() {
        let mut buffer = TextBuffer::from_content("Hello");

        let result = buffer.insert_char(1, 0, 'X');
        assert!(result.is_err());
        assert!(!buffer.is_modified());
    }

    #[test]
    fn test_delete_char() {
        let mut buffer = TextBuffer::from_content("Hello");

        let deleted = buffer.delete_char(0, 1).unwrap();
        assert_eq!(deleted, Some('e'));
        assert_eq!(buffer.get_line(0), Some(&"Hllo".to_string()));
        assert!(buffer.is_modified());
    }

    #[test]
    fn test_delete_char_at_end() {
        let mut buffer = TextBuffer::from_content("Hello");

        let deleted = buffer.delete_char(0, 5).unwrap();
        assert_eq!(deleted, None);
        assert_eq!(buffer.get_line(0), Some(&"Hello".to_string()));
    }

    #[test]
    fn test_insert_line() {
        let mut buffer = TextBuffer::from_content("Line 1");

        buffer.insert_line(1).unwrap();
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.get_line(0), Some(&"Line 1".to_string()));
        assert_eq!(buffer.get_line(1), Some(&String::new()));
        assert!(buffer.is_modified());
    }

    #[test]
    fn test_delete_line() {
        let mut buffer = TextBuffer::from_content("Line 1\nLine 2\nLine 3");

        let deleted = buffer.delete_line(1).unwrap();
        assert_eq!(deleted, Some("Line 2".to_string()));
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.get_line(0), Some(&"Line 1".to_string()));
        assert_eq!(buffer.get_line(1), Some(&"Line 3".to_string()));
        assert!(buffer.is_modified());
    }

    #[test]
    fn test_delete_last_line() {
        let mut buffer = TextBuffer::from_content("Only line");

        let deleted = buffer.delete_line(0).unwrap();
        assert_eq!(deleted, Some("Only line".to_string()));
        assert_eq!(buffer.line_count(), 1);
        assert_eq!(buffer.get_line(0), Some(&String::new()));
        assert!(buffer.is_modified());
    }

    #[test]
    fn test_undo_insert_char() {
        let mut buffer = TextBuffer::from_content("Hello");

        buffer.insert_char(0, 5, '!').unwrap();
        assert_eq!(buffer.get_line(0), Some(&"Hello!".to_string()));

        buffer.undo().unwrap();
        assert_eq!(buffer.get_line(0), Some(&"Hello".to_string()));
        assert!(!buffer.is_modified());
    }

    #[test]
    fn test_undo_delete_char() {
        let mut buffer = TextBuffer::from_content("Hello");

        buffer.delete_char(0, 1).unwrap();
        assert_eq!(buffer.get_line(0), Some(&"Hllo".to_string()));

        buffer.undo().unwrap();
        assert_eq!(buffer.get_line(0), Some(&"Hello".to_string()));
        assert!(!buffer.is_modified());
    }

    #[test]
    fn test_redo_after_undo() {
        let mut buffer = TextBuffer::from_content("Hello");

        buffer.insert_char(0, 5, '!').unwrap();
        buffer.undo().unwrap();
        buffer.redo().unwrap();

        assert_eq!(buffer.get_line(0), Some(&"Hello!".to_string()));
        assert!(buffer.is_modified());
    }

    #[test]
    fn test_can_undo_redo() {
        let mut buffer = TextBuffer::new();

        assert!(!buffer.can_undo());
        assert!(!buffer.can_redo());

        buffer.insert_char(0, 0, 'X').unwrap();
        assert!(buffer.can_undo());
        assert!(!buffer.can_redo());

        buffer.undo().unwrap();
        assert!(!buffer.can_undo());
        assert!(buffer.can_redo());
    }

    #[test]
    fn test_get_line_length() {
        let buffer = TextBuffer::from_content("Hello\nWorld!\n");
        assert_eq!(buffer.get_line_length(0), 5);
        assert_eq!(buffer.get_line_length(1), 6);
        assert_eq!(buffer.get_line_length(2), 0);
        assert_eq!(buffer.get_line_length(999), 0);
    }

    #[test]
    fn test_to_string() {
        let buffer = TextBuffer::from_content("Line 1\nLine 2\nLine 3");
        assert_eq!(buffer.to_string(), "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_mark_saved() {
        let mut buffer = TextBuffer::new();
        buffer.insert_char(0, 0, 'X').unwrap();
        assert!(buffer.is_modified());

        buffer.mark_saved();
        assert!(!buffer.is_modified());
    }
}
