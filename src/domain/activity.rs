use super::note::normalize_note;
use super::Category;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Activity {
    category: Option<Category>,
    note: Option<String>,
}

impl Activity {
    pub fn new(category: Category) -> Self {
        Self { category: Some(category), note: None }
    }

    pub fn with_note(category: Category, note: impl Into<String>) -> Self {
        Self { category: Some(category), note: normalize_note(note.into()) }
    }

    pub fn note_only(note: impl Into<String>) -> Self {
        Self { category: None, note: normalize_note(note.into()) }
    }

    pub const fn category(&self) -> Option<Category> {
        self.category
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    pub fn has_category(&self) -> bool {
        self.category.is_some()
    }

    pub fn has_note(&self) -> bool {
        self.note.is_some()
    }

    pub fn set_category(&mut self, category: Category) {
        self.category = Some(category);
    }

    pub fn clear_category(&mut self) {
        self.category = None;
    }

    pub fn set_note(&mut self, note: impl Into<String>) {
        self.note = normalize_note(note.into());
    }

    pub fn clear_note(&mut self) {
        self.note = None;
    }

    pub fn is_empty(&self) -> bool {
        self.category.is_none() && self.note.is_none()
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

    #[test]
    fn note_only_activity_has_no_category() {
        let activity = Activity::note_only("blocked on review");
        assert_eq!(activity.category(), None);
        assert_eq!(activity.note(), Some("blocked on review"));
    }
}
