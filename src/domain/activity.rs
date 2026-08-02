use super::Category;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Activity {
    category: Category,
    note: Option<String>,
}

impl Activity {
    pub fn new(category: Category) -> Self {
        Self {
            category,
            note: None,
        }
    }

    pub fn with_note(category: Category, note: impl Into<String>) -> Self {
        Self {
            category,
            note: normalize_note(note.into()),
        }
    }

    pub const fn category(&self) -> Category {
        self.category
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    pub fn has_note(&self) -> bool {
        self.note.is_some()
    }

    pub fn set_note(&mut self, note: impl Into<String>) {
        self.note = normalize_note(note.into());
    }

    pub fn clear_note(&mut self) {
        self.note = None;
    }
}

fn normalize_note(note: String) -> Option<String> {
    let trimmed = note.trim();
    if trimmed.is_empty() {
        None
    } else if trimmed.len() == note.len() {
        Some(note)
    } else {
        Some(trimmed.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::Activity;
    use crate::domain::Category;

    #[test]
    fn empty_notes_are_not_stored() {
        let activity = Activity::with_note(Category::Work, "   ");
        assert_eq!(activity.note(), None);
    }

    #[test]
    fn notes_are_trimmed() {
        let mut activity = Activity::new(Category::Health);
        activity.set_note("  walk  ");
        assert_eq!(activity.note(), Some("walk"));
    }
}
