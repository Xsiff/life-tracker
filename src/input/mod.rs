use std::time::Duration;
use std::{
    collections::VecDeque,
    sync::{Mutex, OnceLock},
};

use crossterm::event::{
    poll, read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEvent, MouseEventKind,
};

use crate::domain::Action;

static PENDING_ACTIONS: OnceLock<Mutex<VecDeque<Action>>> = OnceLock::new();
static PENDING_SCROLL: OnceLock<Mutex<i32>> = OnceLock::new();
#[cfg(test)]
static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
const MAX_SCROLL_CHUNK: i32 = 4;

pub fn next_action(timeout: Duration) -> anyhow::Result<Option<Action>> {
    let wait = if has_pending_input() { Duration::ZERO } else { timeout };

    if poll(wait)? {
        queue_event(read()?);
        return Ok(pop_pending_input());
    }

    Ok(pop_pending_input().or(Some(Action::Tick)))
}

fn action_for_key(key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Left if key.modifiers.contains(KeyModifiers::ALT) => Some(Action::MoveWordLeft),
        KeyCode::Right if key.modifiers.contains(KeyModifiers::ALT) => Some(Action::MoveWordRight),
        KeyCode::Delete if key.modifiers.contains(KeyModifiers::ALT) => Some(Action::DeleteWord),
        KeyCode::Backspace if key.modifiers.contains(KeyModifiers::ALT) => Some(Action::DeleteWord),
        KeyCode::Left => Some(Action::MoveLeft),
        KeyCode::Right => Some(Action::MoveRight),
        KeyCode::Up => Some(Action::MoveUp),
        KeyCode::Down => Some(Action::MoveDown),
        KeyCode::Enter => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                Some(Action::InsertNewline)
            } else {
                Some(Action::Confirm)
            }
        }
        KeyCode::Esc => Some(Action::Cancel),
        KeyCode::Tab => Some(Action::CycleView),
        KeyCode::Backspace => Some(Action::Erase),
        KeyCode::Char(c) if c.is_ascii_digit() => Some(Action::Digit(c as u8 - b'0')),
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                None
            } else {
                Some(Action::Char(c))
            }
        }
        _ => None,
    }
}

fn queue_event(event: Event) {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => {
            if let Some(action) = action_for_key(key) {
                push_pending_action(action);
            }
        }
        Event::Mouse(mouse) => queue_mouse(mouse),
        _ => {}
    }
}

fn queue_mouse(mouse: MouseEvent) {
    let Some(delta) = scroll_delta_for_mouse(mouse.kind) else {
        return;
    };

    push_scroll_delta(delta);

    while poll(Duration::ZERO).unwrap_or(false) {
        let Ok(event) = read() else {
            break;
        };
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                if let Some(action) = action_for_key(key) {
                    push_pending_action(action);
                }
            }
            Event::Mouse(next_mouse) => {
                if let Some(next_delta) = scroll_delta_for_mouse(next_mouse.kind) {
                    push_scroll_delta(next_delta);
                }
            }
            _ => {}
        }
    }
}

fn scroll_delta_for_mouse(kind: MouseEventKind) -> Option<i32> {
    match kind {
        MouseEventKind::ScrollUp => Some(-1),
        MouseEventKind::ScrollDown => Some(1),
        MouseEventKind::ScrollLeft => Some(-1),
        MouseEventKind::ScrollRight => Some(1),
        _ => None,
    }
}

fn push_pending_action(action: Action) {
    let queue = PENDING_ACTIONS.get_or_init(|| Mutex::new(VecDeque::new()));
    let mut queue = queue.lock().expect("pending action queue poisoned");
    queue.push_back(action);
}

fn pop_pending_action() -> Option<Action> {
    let queue = PENDING_ACTIONS.get_or_init(|| Mutex::new(VecDeque::new()));
    let mut queue = queue.lock().expect("pending action queue poisoned");
    queue.pop_front()
}

fn push_scroll_delta(delta: i32) {
    let pending = PENDING_SCROLL.get_or_init(|| Mutex::new(0));
    let mut pending = pending.lock().expect("pending scroll poisoned");

    if *pending == 0 || pending.signum() == delta.signum() {
        *pending += delta;
    } else {
        *pending = delta;
    }
}

fn pop_pending_scroll_action() -> Option<Action> {
    let pending = PENDING_SCROLL.get_or_init(|| Mutex::new(0));
    let mut pending = pending.lock().expect("pending scroll poisoned");

    if *pending < 0 {
        let chunk = (*pending).abs().min(MAX_SCROLL_CHUNK);
        *pending += chunk;
        Some(Action::Scroll(-chunk))
    } else if *pending > 0 {
        let chunk = (*pending).min(MAX_SCROLL_CHUNK);
        *pending -= chunk;
        Some(Action::Scroll(chunk))
    } else {
        None
    }
}

fn pop_pending_input() -> Option<Action> {
    pop_pending_action().or_else(pop_pending_scroll_action)
}

fn has_pending_input() -> bool {
    let has_actions = {
        let queue = PENDING_ACTIONS.get_or_init(|| Mutex::new(VecDeque::new()));
        let queue = queue.lock().expect("pending action queue poisoned");
        !queue.is_empty()
    };
    if has_actions {
        return true;
    }

    let pending = PENDING_SCROLL.get_or_init(|| Mutex::new(0));
    let pending = pending.lock().expect("pending scroll poisoned");
    *pending != 0
}

#[cfg(test)]
fn clear_pending_actions() {
    let queue = PENDING_ACTIONS.get_or_init(|| Mutex::new(VecDeque::new()));
    let mut queue = queue.lock().expect("pending action queue poisoned");
    queue.clear();

    let pending = PENDING_SCROLL.get_or_init(|| Mutex::new(0));
    let mut pending = pending.lock().expect("pending scroll poisoned");
    *pending = 0;
}

#[cfg(test)]
fn lock_test_state() -> std::sync::MutexGuard<'static, ()> {
    let lock = TEST_LOCK.get_or_init(|| Mutex::new(()));
    lock.lock().expect("test state lock poisoned")
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEventKind};

    use super::{
        action_for_key, clear_pending_actions, has_pending_input, lock_test_state,
        pop_pending_input, push_scroll_delta, scroll_delta_for_mouse,
    };
    use crate::domain::Action;

    #[test]
    fn shift_enter_maps_to_insert_newline() {
        let _guard = lock_test_state();
        clear_pending_actions();
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT);

        assert_eq!(action_for_key(key), Some(Action::InsertNewline));
    }

    #[test]
    fn plain_enter_maps_to_confirm() {
        let _guard = lock_test_state();
        clear_pending_actions();
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);

        assert_eq!(action_for_key(key), Some(Action::Confirm));
    }

    #[test]
    fn mouse_scroll_up_maps_to_negative_delta() {
        let _guard = lock_test_state();
        clear_pending_actions();
        assert_eq!(scroll_delta_for_mouse(MouseEventKind::ScrollUp), Some(-1));
    }

    #[test]
    fn mouse_scroll_down_maps_to_positive_delta() {
        let _guard = lock_test_state();
        clear_pending_actions();
        assert_eq!(scroll_delta_for_mouse(MouseEventKind::ScrollDown), Some(1));
    }

    #[test]
    fn zero_scroll_delta_does_not_move() {
        let _guard = lock_test_state();
        clear_pending_actions();
        push_scroll_delta(0);
        assert_eq!(pop_pending_input(), None);
    }

    #[test]
    fn same_direction_scroll_accumulates() {
        let _guard = lock_test_state();
        clear_pending_actions();
        push_scroll_delta(6);

        assert_eq!(pop_pending_input(), Some(Action::Scroll(4)));
        assert_eq!(pop_pending_input(), Some(Action::Scroll(2)));
        assert_eq!(pop_pending_input(), None);
    }

    #[test]
    fn opposite_direction_scroll_replaces_backlog() {
        let _guard = lock_test_state();
        clear_pending_actions();
        push_scroll_delta(4);
        push_scroll_delta(-1);

        assert_eq!(pop_pending_input(), Some(Action::Scroll(-1)));
        assert_eq!(pop_pending_input(), None);
    }

    #[test]
    fn pending_keyboard_action_wins_over_scroll() {
        let _guard = lock_test_state();
        clear_pending_actions();
        push_scroll_delta(2);
        super::push_pending_action(Action::Confirm);

        assert!(has_pending_input());
        assert_eq!(pop_pending_input(), Some(Action::Confirm));
        assert_eq!(pop_pending_input(), Some(Action::Scroll(2)));
    }
}
