#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorPosition {
    pub row: usize,
    pub col: usize,
}

impl CursorPosition {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    pub fn move_right(&mut self, line_length: usize) {
        if self.col < line_length {
            self.col += 1;
        }
    }

    pub fn move_left(&mut self) {
        if self.col > 0 {
            self.col -= 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.row > 0 {
            self.row -= 1;
        }
    }

    pub fn move_down(&mut self, buffer_height: usize) {
        if self.row < buffer_height.saturating_sub(1) {
            self.row += 1;
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

    pub fn move_to_buffer_end(&mut self, buffer_height: usize, last_line_length: usize) {
        if buffer_height > 0 {
            self.row = buffer_height.saturating_sub(1);
            self.col = last_line_length;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_new() {
        let cursor = CursorPosition::new(5, 10);
        assert_eq!(cursor.row, 5);
        assert_eq!(cursor.col, 10);
    }

    #[test]
    fn test_cursor_movement_right() {
        let mut cursor = CursorPosition::new(0, 0);
        let line_length = 10;

        cursor.move_right(line_length);

        assert_eq!(cursor.col, 1);
        assert_eq!(cursor.row, 0);
    }

    #[test]
    fn test_cursor_movement_right_at_line_end() {
        let mut cursor = CursorPosition::new(0, 10);
        let line_length = 10;

        cursor.move_right(line_length);

        assert_eq!(cursor.col, 10);
        assert_eq!(cursor.row, 0);
    }

    #[test]
    fn test_cursor_movement_left() {
        let mut cursor = CursorPosition::new(0, 5);

        cursor.move_left();

        assert_eq!(cursor.col, 4);
        assert_eq!(cursor.row, 0);
    }

    #[test]
    fn test_cursor_movement_left_at_line_start() {
        let mut cursor = CursorPosition::new(0, 0);

        cursor.move_left();

        assert_eq!(cursor.col, 0);
        assert_eq!(cursor.row, 0);
    }

    #[test]
    fn test_cursor_movement_up() {
        let mut cursor = CursorPosition::new(5, 3);

        cursor.move_up();

        assert_eq!(cursor.row, 4);
        assert_eq!(cursor.col, 3);
    }

    #[test]
    fn test_cursor_movement_up_at_buffer_start() {
        let mut cursor = CursorPosition::new(0, 3);

        cursor.move_up();

        assert_eq!(cursor.row, 0);
        assert_eq!(cursor.col, 3);
    }

    #[test]
    fn test_cursor_movement_down() {
        let mut cursor = CursorPosition::new(0, 3);
        let buffer_height = 10;

        cursor.move_down(buffer_height);

        assert_eq!(cursor.row, 1);
        assert_eq!(cursor.col, 3);
    }

    #[test]
    fn test_cursor_movement_down_at_buffer_end() {
        let mut cursor = CursorPosition::new(9, 3);
        let buffer_height = 10;

        cursor.move_down(buffer_height);

        assert_eq!(cursor.row, 9);
        assert_eq!(cursor.col, 3);
    }

    #[test]
    fn test_cursor_move_to_line_start() {
        let mut cursor = CursorPosition::new(5, 10);

        cursor.move_to_line_start();

        assert_eq!(cursor.row, 5);
        assert_eq!(cursor.col, 0);
    }

    #[test]
    fn test_cursor_move_to_line_end() {
        let mut cursor = CursorPosition::new(5, 3);
        let line_length = 15;

        cursor.move_to_line_end(line_length);

        assert_eq!(cursor.row, 5);
        assert_eq!(cursor.col, 15);
    }

    #[test]
    fn test_cursor_move_to_buffer_start() {
        let mut cursor = CursorPosition::new(10, 20);

        cursor.move_to_buffer_start();

        assert_eq!(cursor.row, 0);
        assert_eq!(cursor.col, 0);
    }

    #[test]
    fn test_cursor_move_to_buffer_end() {
        let mut cursor = CursorPosition::new(0, 0);
        let buffer_height = 10;
        let last_line_length = 25;

        cursor.move_to_buffer_end(buffer_height, last_line_length);

        assert_eq!(cursor.row, 9);
        assert_eq!(cursor.col, 25);
    }

    #[test]
    fn test_cursor_move_to_buffer_end_empty_buffer() {
        let mut cursor = CursorPosition::new(5, 10);
        let buffer_height = 0;
        let last_line_length = 0;

        cursor.move_to_buffer_end(buffer_height, last_line_length);

        assert_eq!(cursor.row, 5);
        assert_eq!(cursor.col, 10);
    }
}
