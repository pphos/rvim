use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EditorError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Position out of bounds: row {row}, col {col}")]
    OutOfBounds { row: usize, col: usize },

    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Invalid mode transition from {from} to {to}")]
    InvalidModeTransition { from: String, to: String },

    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    #[error("Undo stack is empty")]
    EmptyUndoStack,

    #[error("Redo stack is empty")]
    EmptyRedoStack,

    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, EditorError>;

impl EditorError {
    pub fn out_of_bounds(row: usize, col: usize) -> Self {
        Self::OutOfBounds { row, col }
    }

    pub fn invalid_command<S: Into<String>>(cmd: S) -> Self {
        Self::InvalidCommand(cmd.into())
    }

    pub fn invalid_mode_transition<S: Into<String>>(from: S, to: S) -> Self {
        Self::InvalidModeTransition {
            from: from.into(),
            to: to.into(),
        }
    }

    pub fn file_not_found<P: Into<PathBuf>>(path: P) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    pub fn permission_denied<P: Into<PathBuf>>(path: P) -> Self {
        Self::PermissionDenied { path: path.into() }
    }

    pub fn terminal<S: Into<String>>(msg: S) -> Self {
        Self::Terminal(msg.into())
    }

    pub fn parse<S: Into<String>>(msg: S) -> Self {
        Self::Parse(msg.into())
    }

    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config(msg.into())
    }
}

// Crosstermのエラーは std::io::Error を通して処理される

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_out_of_bounds_error() {
        let err = EditorError::out_of_bounds(10, 5);
        assert_eq!(err.to_string(), "Position out of bounds: row 10, col 5");
    }

    #[test]
    fn test_invalid_command_error() {
        let err = EditorError::invalid_command("invalid_cmd");
        assert_eq!(err.to_string(), "Invalid command: invalid_cmd");
    }

    #[test]
    fn test_file_not_found_error() {
        let path = Path::new("/nonexistent/file.txt");
        let err = EditorError::file_not_found(path);
        assert_eq!(err.to_string(), "File not found: /nonexistent/file.txt");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let editor_err: EditorError = io_err.into();
        assert!(matches!(editor_err, EditorError::Io(_)));
    }
}
