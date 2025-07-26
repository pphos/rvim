#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorMode {
    Normal,
    Insert,
    Visual,
    Command,
}

impl EditorMode {
    pub fn is_normal(&self) -> bool {
        matches!(self, EditorMode::Normal)
    }

    pub fn is_insert(&self) -> bool {
        matches!(self, EditorMode::Insert)
    }

    pub fn is_visual(&self) -> bool {
        matches!(self, EditorMode::Visual)
    }

    pub fn is_command(&self) -> bool {
        matches!(self, EditorMode::Command)
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            EditorMode::Normal => "NORMAL",
            EditorMode::Insert => "INSERT",
            EditorMode::Visual => "VISUAL",
            EditorMode::Command => "COMMAND",
        }
    }
}

impl Default for EditorMode {
    fn default() -> Self {
        EditorMode::Normal
    }
}

#[derive(Debug, Clone)]
pub struct ModeTransition {
    pub from: EditorMode,
    pub to: EditorMode,
    pub trigger: TransitionTrigger,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransitionTrigger {
    Key(char),
    Escape,
    Command(String),
}

pub struct ModeManager {
    current_mode: EditorMode,
    previous_mode: Option<EditorMode>,
    transitions: Vec<ModeTransition>,
}

impl ModeManager {
    pub fn new() -> Self {
        let transitions = vec![
            // Normal to Insert
            ModeTransition {
                from: EditorMode::Normal,
                to: EditorMode::Insert,
                trigger: TransitionTrigger::Key('i'),
            },
            ModeTransition {
                from: EditorMode::Normal,
                to: EditorMode::Insert,
                trigger: TransitionTrigger::Key('a'),
            },
            ModeTransition {
                from: EditorMode::Normal,
                to: EditorMode::Insert,
                trigger: TransitionTrigger::Key('o'),
            },
            ModeTransition {
                from: EditorMode::Normal,
                to: EditorMode::Insert,
                trigger: TransitionTrigger::Key('O'),
            },
            // Normal to Visual
            ModeTransition {
                from: EditorMode::Normal,
                to: EditorMode::Visual,
                trigger: TransitionTrigger::Key('v'),
            },
            // Normal to Command
            ModeTransition {
                from: EditorMode::Normal,
                to: EditorMode::Command,
                trigger: TransitionTrigger::Key(':'),
            },
            // Back to Normal (from Insert)
            ModeTransition {
                from: EditorMode::Insert,
                to: EditorMode::Normal,
                trigger: TransitionTrigger::Escape,
            },
            // Back to Normal (from Visual)
            ModeTransition {
                from: EditorMode::Visual,
                to: EditorMode::Normal,
                trigger: TransitionTrigger::Escape,
            },
            // Back to Normal (from Command)
            ModeTransition {
                from: EditorMode::Command,
                to: EditorMode::Normal,
                trigger: TransitionTrigger::Escape,
            },
        ];

        Self {
            current_mode: EditorMode::Normal,
            previous_mode: None,
            transitions,
        }
    }

    pub fn current_mode(&self) -> &EditorMode {
        &self.current_mode
    }

    pub fn previous_mode(&self) -> Option<&EditorMode> {
        self.previous_mode.as_ref()
    }

    pub fn transition_with_key(&mut self, key: char) -> bool {
        for transition in &self.transitions {
            if transition.from == self.current_mode
                && transition.trigger == TransitionTrigger::Key(key)
            {
                self.previous_mode = Some(self.current_mode.clone());
                self.current_mode = transition.to.clone();
                return true;
            }
        }
        false
    }

    pub fn transition_with_escape(&mut self) -> bool {
        for transition in &self.transitions {
            if transition.from == self.current_mode
                && transition.trigger == TransitionTrigger::Escape
            {
                self.previous_mode = Some(self.current_mode.clone());
                self.current_mode = transition.to.clone();
                return true;
            }
        }
        false
    }

    pub fn transition_with_command(&mut self, command: &str) -> bool {
        for transition in &self.transitions {
            if transition.from == self.current_mode
                && transition.trigger == TransitionTrigger::Command(command.to_string())
            {
                self.previous_mode = Some(self.current_mode.clone());
                self.current_mode = transition.to.clone();
                return true;
            }
        }
        false
    }

    pub fn can_transition_with_key(&self, key: char) -> bool {
        self.transitions
            .iter()
            .any(|t| t.from == self.current_mode && t.trigger == TransitionTrigger::Key(key))
    }

    pub fn can_transition_with_escape(&self) -> bool {
        self.transitions
            .iter()
            .any(|t| t.from == self.current_mode && t.trigger == TransitionTrigger::Escape)
    }

    pub fn reset_to_normal(&mut self) {
        self.previous_mode = Some(self.current_mode.clone());
        self.current_mode = EditorMode::Normal;
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
    fn test_editor_mode_predicates() {
        assert!(EditorMode::Normal.is_normal());
        assert!(!EditorMode::Normal.is_insert());
        assert!(!EditorMode::Normal.is_visual());
        assert!(!EditorMode::Normal.is_command());

        assert!(EditorMode::Insert.is_insert());
        assert!(EditorMode::Visual.is_visual());
        assert!(EditorMode::Command.is_command());
    }

    #[test]
    fn test_editor_mode_to_string() {
        assert_eq!(EditorMode::Normal.to_string(), "NORMAL");
        assert_eq!(EditorMode::Insert.to_string(), "INSERT");
        assert_eq!(EditorMode::Visual.to_string(), "VISUAL");
        assert_eq!(EditorMode::Command.to_string(), "COMMAND");
    }

    #[test]
    fn test_editor_mode_default() {
        assert_eq!(EditorMode::default(), EditorMode::Normal);
    }

    #[test]
    fn test_mode_manager_new() {
        let manager = ModeManager::new();
        assert_eq!(manager.current_mode(), &EditorMode::Normal);
        assert!(manager.previous_mode().is_none());
    }

    #[test]
    fn test_normal_to_insert_transitions() {
        let mut manager = ModeManager::new();

        // Test 'i' key
        assert!(manager.transition_with_key('i'));
        assert_eq!(manager.current_mode(), &EditorMode::Insert);
        assert_eq!(manager.previous_mode(), Some(&EditorMode::Normal));

        // Reset to normal
        manager.reset_to_normal();

        // Test 'a' key
        assert!(manager.transition_with_key('a'));
        assert_eq!(manager.current_mode(), &EditorMode::Insert);

        // Reset to normal
        manager.reset_to_normal();

        // Test 'o' key
        assert!(manager.transition_with_key('o'));
        assert_eq!(manager.current_mode(), &EditorMode::Insert);

        // Reset to normal
        manager.reset_to_normal();

        // Test 'O' key
        assert!(manager.transition_with_key('O'));
        assert_eq!(manager.current_mode(), &EditorMode::Insert);
    }

    #[test]
    fn test_normal_to_visual_transition() {
        let mut manager = ModeManager::new();

        assert!(manager.transition_with_key('v'));
        assert_eq!(manager.current_mode(), &EditorMode::Visual);
        assert_eq!(manager.previous_mode(), Some(&EditorMode::Normal));
    }

    #[test]
    fn test_normal_to_command_transition() {
        let mut manager = ModeManager::new();

        assert!(manager.transition_with_key(':'));
        assert_eq!(manager.current_mode(), &EditorMode::Command);
        assert_eq!(manager.previous_mode(), Some(&EditorMode::Normal));
    }

    #[test]
    fn test_back_to_normal_with_escape() {
        let mut manager = ModeManager::new();

        // Insert to Normal
        manager.transition_with_key('i');
        assert!(manager.transition_with_escape());
        assert_eq!(manager.current_mode(), &EditorMode::Normal);

        // Visual to Normal
        manager.transition_with_key('v');
        assert!(manager.transition_with_escape());
        assert_eq!(manager.current_mode(), &EditorMode::Normal);

        // Command to Normal
        manager.transition_with_key(':');
        assert!(manager.transition_with_escape());
        assert_eq!(manager.current_mode(), &EditorMode::Normal);
    }

    #[test]
    fn test_invalid_transitions() {
        let mut manager = ModeManager::new();

        // Invalid key in Normal mode
        assert!(!manager.transition_with_key('z'));
        assert_eq!(manager.current_mode(), &EditorMode::Normal);

        // Escape in Normal mode (should not work)
        assert!(!manager.transition_with_escape());
        assert_eq!(manager.current_mode(), &EditorMode::Normal);

        // Move to Insert mode
        manager.transition_with_key('i');

        // Invalid key in Insert mode
        assert!(!manager.transition_with_key('v'));
        assert_eq!(manager.current_mode(), &EditorMode::Insert);
    }

    #[test]
    fn test_can_transition_predicates() {
        let manager = ModeManager::new();

        // In Normal mode
        assert!(manager.can_transition_with_key('i'));
        assert!(manager.can_transition_with_key('a'));
        assert!(manager.can_transition_with_key('o'));
        assert!(manager.can_transition_with_key('O'));
        assert!(manager.can_transition_with_key('v'));
        assert!(manager.can_transition_with_key(':'));
        assert!(!manager.can_transition_with_key('z'));
        assert!(!manager.can_transition_with_escape());
    }

    #[test]
    fn test_can_transition_from_insert() {
        let mut manager = ModeManager::new();
        manager.transition_with_key('i');

        // In Insert mode
        assert!(!manager.can_transition_with_key('i'));
        assert!(!manager.can_transition_with_key('v'));
        assert!(manager.can_transition_with_escape());
    }

    #[test]
    fn test_reset_to_normal() {
        let mut manager = ModeManager::new();

        // Move to Insert mode
        manager.transition_with_key('i');
        assert_eq!(manager.current_mode(), &EditorMode::Insert);

        // Reset to Normal
        manager.reset_to_normal();
        assert_eq!(manager.current_mode(), &EditorMode::Normal);
        assert_eq!(manager.previous_mode(), Some(&EditorMode::Insert));
    }

    #[test]
    fn test_mode_manager_default() {
        let manager = ModeManager::default();
        assert_eq!(manager.current_mode(), &EditorMode::Normal);
    }
}
