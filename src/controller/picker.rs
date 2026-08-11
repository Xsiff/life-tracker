use crate::domain::{Activity, Category};

use super::{CategoryPickerSelection, NoteTarget, State};

pub(super) fn picker_default_selection(
    state: &State,
    target: &NoteTarget,
) -> CategoryPickerSelection {
    match *target {
        NoteTarget::Day { .. } => CategoryPickerSelection::AddNote,
        NoteTarget::Hour { date, hour } => {
            match state.activity(date, hour).and_then(Activity::category) {
                Some(category) => CategoryPickerSelection::Category(category),
                None => CategoryPickerSelection::AddNote,
            }
        }
    }
}

pub(super) fn move_picker_up(
    target: &NoteTarget,
    selected: CategoryPickerSelection,
) -> CategoryPickerSelection {
    move_picker_by(target, selected, -1)
}

pub(super) fn move_picker_down(
    target: &NoteTarget,
    selected: CategoryPickerSelection,
) -> CategoryPickerSelection {
    move_picker_by(target, selected, 1)
}

fn move_picker_by(
    target: &NoteTarget,
    selected: CategoryPickerSelection,
    delta: isize,
) -> CategoryPickerSelection {
    let current = picker_selection_index(target, selected);
    let max = picker_selection_count(target).saturating_sub(1) as isize;
    let next = (current as isize + delta).clamp(0, max) as usize;
    picker_selection_at(target, next)
}

pub(super) fn scroll_picker(
    target: &NoteTarget,
    mut selected: CategoryPickerSelection,
    delta: i32,
) -> CategoryPickerSelection {
    for _ in 0..delta.unsigned_abs() {
        selected = if delta < 0 {
            move_picker_up(target, selected)
        } else {
            move_picker_down(target, selected)
        };
    }
    selected
}

fn picker_selection_count(target: &NoteTarget) -> usize {
    match target {
        NoteTarget::Day { .. } => 2,
        NoteTarget::Hour { .. } => Category::ALL.len() + 3,
    }
}

fn picker_selection_index(target: &NoteTarget, selected: CategoryPickerSelection) -> usize {
    match (target, selected) {
        (NoteTarget::Day { .. }, CategoryPickerSelection::AddNote) => 0,
        (NoteTarget::Day { .. }, CategoryPickerSelection::DeleteNote) => 1,
        (NoteTarget::Day { .. }, _) => 0,
        (NoteTarget::Hour { .. }, CategoryPickerSelection::Category(category)) => {
            usize::from(category.as_u8())
        }
        (NoteTarget::Hour { .. }, CategoryPickerSelection::AddNote) => Category::ALL.len(),
        (NoteTarget::Hour { .. }, CategoryPickerSelection::DeleteNote) => Category::ALL.len() + 1,
        (NoteTarget::Hour { .. }, CategoryPickerSelection::DeleteActivity) => {
            Category::ALL.len() + 2
        }
    }
}

fn picker_selection_at(target: &NoteTarget, index: usize) -> CategoryPickerSelection {
    match target {
        NoteTarget::Day { .. } => match index {
            0 => CategoryPickerSelection::AddNote,
            _ => CategoryPickerSelection::DeleteNote,
        },
        NoteTarget::Hour { .. } => {
            if index < Category::ALL.len() {
                CategoryPickerSelection::Category(Category::ALL[index])
            } else {
                match index - Category::ALL.len() {
                    0 => CategoryPickerSelection::AddNote,
                    1 => CategoryPickerSelection::DeleteNote,
                    _ => CategoryPickerSelection::DeleteActivity,
                }
            }
        }
    }
}
