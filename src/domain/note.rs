pub(super) fn normalize_note(note: String) -> Option<String> {
    let trimmed = note.trim();
    if trimmed.is_empty() {
        None
    } else if trimmed.len() == note.len() {
        Some(note)
    } else {
        Some(trimmed.to_owned())
    }
}
