#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Scroll(i32),
    MoveWordLeft,
    MoveWordRight,
    DeleteWord,
    Confirm,
    InsertNewline,
    Cancel,
    CycleView,
    Digit(u8),
    Char(char),
    Erase,
    Tick,
}
#[cfg(test)]
mod tests {
    use super::Action;
    use std::collections::HashSet;

    #[test]
    fn payload_variants_preserve_values() {
        assert_eq!(Action::Digit(0), Action::Digit(0));
        assert_eq!(Action::Digit(9), Action::Digit(9));
        assert_eq!(Action::Char('q'), Action::Char('q'));
        assert_eq!(Action::Char('N'), Action::Char('N'));
        assert_eq!(Action::Scroll(3), Action::Scroll(3));
        assert_ne!(Action::Digit(1), Action::Digit(2));
        assert_ne!(Action::Char('n'), Action::Char('N'));
        assert_ne!(Action::Scroll(1), Action::Scroll(2));
    }

    #[test]
    fn action_values_are_hashable_and_distinguish_variants() {
        let mut actions = HashSet::new();

        actions.insert(Action::MoveLeft);
        actions.insert(Action::MoveRight);
        actions.insert(Action::MoveUp);
        actions.insert(Action::MoveDown);
        actions.insert(Action::Scroll(2));
        actions.insert(Action::MoveWordLeft);
        actions.insert(Action::MoveWordRight);
        actions.insert(Action::DeleteWord);
        actions.insert(Action::Confirm);
        actions.insert(Action::InsertNewline);
        actions.insert(Action::Cancel);
        actions.insert(Action::CycleView);
        actions.insert(Action::Digit(3));
        actions.insert(Action::Char('x'));
        actions.insert(Action::Erase);
        actions.insert(Action::Tick);

        assert_eq!(actions.len(), 16);
        assert!(actions.contains(&Action::Digit(3)));
        assert!(actions.contains(&Action::Char('x')));
        assert!(actions.contains(&Action::Scroll(2)));
        assert!(!actions.contains(&Action::Digit(4)));
        assert!(!actions.contains(&Action::Char('q')));
    }

    #[test]
    fn copied_actions_match_originals() {
        let action = Action::CycleView;
        let copied = action;

        assert_eq!(action, copied);

        let payload = Action::Char('v');
        let copied_payload = payload;

        assert_eq!(payload, copied_payload);

        let scroll = Action::Scroll(-2);
        let copied_scroll = scroll;

        assert_eq!(scroll, copied_scroll);
    }
}
