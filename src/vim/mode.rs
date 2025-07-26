use crate::editor::Position;
use crate::error::{EditorError, Result};

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Visual {
        start: Position,
    },
    Command {
        input: String,
    },
}

impl Mode {
    pub fn is_normal(&self) -> bool {
        matches!(self, Mode::Normal)
    }

    pub fn is_insert(&self) -> bool {
        matches!(self, Mode::Insert)
    }

    pub fn is_visual(&self) -> bool {
        matches!(self, Mode::Visual { .. })
    }

    pub fn is_command(&self) -> bool {
        matches!(self, Mode::Command { .. })
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Normal => write!(f, "NORMAL"),
            Mode::Insert => write!(f, "INSERT"),
            Mode::Visual { .. } => write!(f, "VISUAL"),
            Mode::Command { .. } => write!(f, "COMMAND"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModeManager {
    current: Mode,
    previous: Option<Mode>,
}

impl ModeManager {
    pub fn new() -> Self {
        Self {
            current: Mode::Normal,
            previous: None,
        }
    }

    pub fn current(&self) -> &Mode {
        &self.current
    }

    pub fn previous(&self) -> Option<&Mode> {
        self.previous.as_ref()
    }

    pub fn enter_insert(&mut self) {
        self.transition_to(Mode::Insert);
    }

    pub fn enter_visual(&mut self, start: Position) {
        self.transition_to(Mode::Visual { start });
    }

    pub fn enter_command(&mut self) {
        self.transition_to(Mode::Command {
            input: String::new(),
        });
    }

    pub fn enter_normal(&mut self) {
        self.transition_to(Mode::Normal);
    }

    pub fn update_command_input(&mut self, input: String) -> Result<()> {
        match &mut self.current {
            Mode::Command { input: cmd_input } => {
                *cmd_input = input;
                Ok(())
            }
            _ => Err(EditorError::invalid_mode_transition(
                self.current.to_string(),
                "Command input update".to_string(),
            )),
        }
    }

    pub fn can_transition_to(&self, target: &Mode) -> bool {
        match (&self.current, target) {
            // Normal can transition to any mode
            (Mode::Normal, _) => true,
            // Insert can only transition to Normal
            (Mode::Insert, Mode::Normal) => true,
            // Visual can transition to Normal or Insert
            (Mode::Visual { .. }, Mode::Normal | Mode::Insert) => true,
            // Command can only transition to Normal
            (Mode::Command { .. }, Mode::Normal) => true,
            // All other transitions are invalid
            _ => false,
        }
    }

    pub fn transition_to(&mut self, target: Mode) {
        if self.can_transition_to(&target) {
            self.previous = Some(self.current.clone());
            self.current = target;
        }
    }

    pub fn try_transition_to(&mut self, target: Mode) -> Result<()> {
        if self.can_transition_to(&target) {
            self.transition_to(target);
            Ok(())
        } else {
            Err(EditorError::invalid_mode_transition(
                self.current.to_string(),
                target.to_string(),
            ))
        }
    }
}

impl Default for ModeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_display() {
        assert_eq!(Mode::Normal.to_string(), "NORMAL");
        assert_eq!(Mode::Insert.to_string(), "INSERT");
        assert_eq!(
            Mode::Visual {
                start: Position::new(0, 0)
            }
            .to_string(),
            "VISUAL"
        );
        assert_eq!(
            Mode::Command {
                input: "test".to_string()
            }
            .to_string(),
            "COMMAND"
        );
    }

    #[test]
    fn test_mode_predicates() {
        let normal = Mode::Normal;
        assert!(normal.is_normal());
        assert!(!normal.is_insert());
        assert!(!normal.is_visual());
        assert!(!normal.is_command());

        let insert = Mode::Insert;
        assert!(!insert.is_normal());
        assert!(insert.is_insert());
        assert!(!insert.is_visual());
        assert!(!insert.is_command());

        let visual = Mode::Visual {
            start: Position::new(0, 0),
        };
        assert!(!visual.is_normal());
        assert!(!visual.is_insert());
        assert!(visual.is_visual());
        assert!(!visual.is_command());

        let command = Mode::Command {
            input: String::new(),
        };
        assert!(!command.is_normal());
        assert!(!command.is_insert());
        assert!(!command.is_visual());
        assert!(command.is_command());
    }

    #[test]
    fn test_mode_manager_new() {
        let manager = ModeManager::new();
        assert!(manager.current().is_normal());
        assert!(manager.previous().is_none());
    }

    #[test]
    fn test_mode_manager_default() {
        let manager = ModeManager::default();
        assert!(manager.current().is_normal());
        assert!(manager.previous().is_none());
    }

    #[test]
    fn test_normal_to_insert_transition() {
        let mut manager = ModeManager::new();

        manager.enter_insert();
        assert!(manager.current().is_insert());
        assert!(manager.previous().unwrap().is_normal());
    }

    #[test]
    fn test_normal_to_visual_transition() {
        let mut manager = ModeManager::new();
        let start_pos = Position::new(5, 10);

        manager.enter_visual(start_pos);
        assert!(manager.current().is_visual());
        if let Mode::Visual { start } = manager.current() {
            assert_eq!(*start, start_pos);
        } else {
            panic!("Expected Visual mode");
        }
    }

    #[test]
    fn test_normal_to_command_transition() {
        let mut manager = ModeManager::new();

        manager.enter_command();
        assert!(manager.current().is_command());
        if let Mode::Command { input } = manager.current() {
            assert!(input.is_empty());
        } else {
            panic!("Expected Command mode");
        }
    }

    #[test]
    fn test_insert_to_normal_transition() {
        let mut manager = ModeManager::new();
        manager.enter_insert();

        manager.enter_normal();
        assert!(manager.current().is_normal());
        assert!(manager.previous().unwrap().is_insert());
    }

    #[test]
    fn test_can_transition_from_normal() {
        let manager = ModeManager::new();

        assert!(manager.can_transition_to(&Mode::Insert));
        assert!(manager.can_transition_to(&Mode::Visual {
            start: Position::new(0, 0)
        }));
        assert!(manager.can_transition_to(&Mode::Command {
            input: String::new()
        }));
        assert!(manager.can_transition_to(&Mode::Normal));
    }

    #[test]
    fn test_can_transition_from_insert() {
        let mut manager = ModeManager::new();
        manager.enter_insert();

        assert!(manager.can_transition_to(&Mode::Normal));
        assert!(!manager.can_transition_to(&Mode::Insert));
        assert!(!manager.can_transition_to(&Mode::Visual {
            start: Position::new(0, 0)
        }));
        assert!(!manager.can_transition_to(&Mode::Command {
            input: String::new()
        }));
    }

    #[test]
    fn test_can_transition_from_visual() {
        let mut manager = ModeManager::new();
        manager.enter_visual(Position::new(0, 0));

        assert!(manager.can_transition_to(&Mode::Normal));
        assert!(manager.can_transition_to(&Mode::Insert));
        assert!(!manager.can_transition_to(&Mode::Visual {
            start: Position::new(1, 1)
        }));
        assert!(!manager.can_transition_to(&Mode::Command {
            input: String::new()
        }));
    }

    #[test]
    fn test_can_transition_from_command() {
        let mut manager = ModeManager::new();
        manager.enter_command();

        assert!(manager.can_transition_to(&Mode::Normal));
        assert!(!manager.can_transition_to(&Mode::Insert));
        assert!(!manager.can_transition_to(&Mode::Visual {
            start: Position::new(0, 0)
        }));
        assert!(!manager.can_transition_to(&Mode::Command {
            input: String::new()
        }));
    }

    #[test]
    fn test_try_transition_to_success() {
        let mut manager = ModeManager::new();

        let result = manager.try_transition_to(Mode::Insert);
        assert!(result.is_ok());
        assert!(manager.current().is_insert());
    }

    #[test]
    fn test_try_transition_to_failure() {
        let mut manager = ModeManager::new();
        manager.enter_insert();

        let result = manager.try_transition_to(Mode::Visual {
            start: Position::new(0, 0),
        });
        assert!(result.is_err());
        assert!(manager.current().is_insert()); // 状態は変更されない
    }

    #[test]
    fn test_update_command_input() {
        let mut manager = ModeManager::new();
        manager.enter_command();

        let result = manager.update_command_input("w".to_string());
        assert!(result.is_ok());

        if let Mode::Command { input } = manager.current() {
            assert_eq!(input, "w");
        } else {
            panic!("Expected Command mode");
        }
    }

    #[test]
    fn test_update_command_input_wrong_mode() {
        let mut manager = ModeManager::new(); // Normal mode

        let result = manager.update_command_input("w".to_string());
        assert!(result.is_err());
    }
}
