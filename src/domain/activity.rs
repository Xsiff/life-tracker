use super::Category;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    pub category: Category,
    pub note: Option<String>,
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
            note: Some(note.into()),
        }
    }

    pub fn has_note(&self) -> bool {
        self.note.as_deref().is_some_and(|note| !note.is_empty())
    }
}
