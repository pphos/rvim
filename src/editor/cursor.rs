use crate::error::{EditorError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    pub fn origin() -> Self {
        Self { row: 0, col: 0 }
    }

    pub fn move_right(&mut self, line_length: usize) -> Result<()> {
        if self.col < line_length {
            self.col += 1;
            Ok(())
        } else {
            Err(EditorError::out_of_bounds(self.row, self.col + 1))
        }
    }

    pub fn move_left(&mut self) -> Result<()> {
        if self.col > 0 {
            self.col -= 1;
            Ok(())
        } else {
            Err(EditorError::out_of_bounds(self.row, 0))
        }
    }

    pub fn move_down(&mut self, total_lines: usize) -> Result<()> {
        if self.row + 1 < total_lines {
            self.row += 1;
            Ok(())
        } else {
            Err(EditorError::out_of_bounds(self.row + 1, self.col))
        }
    }

    pub fn move_up(&mut self) -> Result<()> {
        if self.row > 0 {
            self.row -= 1;
            Ok(())
        } else {
            Err(EditorError::out_of_bounds(0, self.col))
        }
    }

    pub fn move_to_line_start(&mut self) {
        self.col = 0;
    }

    pub fn move_to_line_end(&mut self, line_length: usize) {
        self.col = line_length;
    }

    pub fn move_to_buffer_start(&mut self) {
        self.row = 0;
        self.col = 0;
    }

    pub fn move_to_buffer_end(&mut self, total_lines: usize, last_line_length: usize) {
        if total_lines > 0 {
            self.row = total_lines - 1;
            self.col = last_line_length;
        } else {
            self.row = 0;
            self.col = 0;
        }
    }

    pub fn clamp_to_line(&mut self, line_length: usize) {
        if self.col > line_length {
            self.col = line_length;
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::origin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_new() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.row, 5);
        assert_eq!(pos.col, 10);
    }

    #[test]
    fn test_position_origin() {
        let pos = Position::origin();
        assert_eq!(pos.row, 0);
        assert_eq!(pos.col, 0);
    }

    #[test]
    fn test_position_default() {
        let pos = Position::default();
        assert_eq!(pos, Position::origin());
    }

    #[test]
    fn test_move_right_success() {
        let mut pos = Position::new(0, 5);
        assert!(pos.move_right(10).is_ok());
        assert_eq!(pos.col, 6);
    }

    #[test]
    fn test_move_right_at_line_end() {
        let mut pos = Position::new(0, 10);
        let result = pos.move_right(10);
        assert!(result.is_err());
        assert_eq!(pos.col, 10); // 位置は変更されない
    }

    #[test]
    fn test_move_left_success() {
        let mut pos = Position::new(0, 5);
        assert!(pos.move_left().is_ok());
        assert_eq!(pos.col, 4);
    }

    #[test]
    fn test_move_left_at_line_start() {
        let mut pos = Position::new(0, 0);
        let result = pos.move_left();
        assert!(result.is_err());
        assert_eq!(pos.col, 0); // 位置は変更されない
    }

    #[test]
    fn test_move_down_success() {
        let mut pos = Position::new(0, 5);
        assert!(pos.move_down(10).is_ok());
        assert_eq!(pos.row, 1);
    }

    #[test]
    fn test_move_down_at_buffer_end() {
        let mut pos = Position::new(9, 5);
        let result = pos.move_down(10);
        assert!(result.is_err());
        assert_eq!(pos.row, 9); // 位置は変更されない
    }

    #[test]
    fn test_move_up_success() {
        let mut pos = Position::new(5, 10);
        assert!(pos.move_up().is_ok());
        assert_eq!(pos.row, 4);
    }

    #[test]
    fn test_move_up_at_buffer_start() {
        let mut pos = Position::new(0, 5);
        let result = pos.move_up();
        assert!(result.is_err());
        assert_eq!(pos.row, 0); // 位置は変更されない
    }

    #[test]
    fn test_move_to_line_start() {
        let mut pos = Position::new(5, 10);
        pos.move_to_line_start();
        assert_eq!(pos.col, 0);
        assert_eq!(pos.row, 5);
    }

    #[test]
    fn test_move_to_line_end() {
        let mut pos = Position::new(5, 5);
        pos.move_to_line_end(20);
        assert_eq!(pos.col, 20);
        assert_eq!(pos.row, 5);
    }

    #[test]
    fn test_move_to_buffer_start() {
        let mut pos = Position::new(10, 15);
        pos.move_to_buffer_start();
        assert_eq!(pos.row, 0);
        assert_eq!(pos.col, 0);
    }

    #[test]
    fn test_move_to_buffer_end() {
        let mut pos = Position::new(0, 0);
        pos.move_to_buffer_end(10, 25);
        assert_eq!(pos.row, 9);
        assert_eq!(pos.col, 25);
    }

    #[test]
    fn test_move_to_buffer_end_empty_buffer() {
        let mut pos = Position::new(5, 10);
        pos.move_to_buffer_end(0, 0);
        assert_eq!(pos.row, 0);
        assert_eq!(pos.col, 0);
    }

    #[test]
    fn test_clamp_to_line() {
        let mut pos = Position::new(5, 25);
        pos.clamp_to_line(10);
        assert_eq!(pos.col, 10);
        assert_eq!(pos.row, 5);
    }

    #[test]
    fn test_clamp_to_line_no_change() {
        let mut pos = Position::new(5, 5);
        pos.clamp_to_line(10);
        assert_eq!(pos.col, 5);
        assert_eq!(pos.row, 5);
    }
}
