pub(super) fn insert_char(draft: &mut String, cursor: &mut usize, c: char) {
    draft.insert(*cursor, c);
    *cursor += c.len_utf8();
}

pub(super) fn erase_char(draft: &mut String, cursor: &mut usize) {
    if *cursor == 0 {
        return;
    }

    let new_cursor = draft[..*cursor].char_indices().last().map(|(idx, _)| idx).unwrap_or(0);
    draft.drain(new_cursor..*cursor);
    *cursor = new_cursor;
}

pub(super) fn erase_word(draft: &mut String, cursor: &mut usize) {
    if *cursor == 0 {
        return;
    }

    let mut pos = *cursor;
    while let Some((start, ch)) = prev_char_at(draft, pos) {
        if !ch.is_whitespace() {
            break;
        }
        pos = start;
        if pos == 0 {
            draft.drain(0..*cursor);
            *cursor = 0;
            return;
        }
    }

    while let Some((start, ch)) = prev_char_at(draft, pos) {
        if ch.is_whitespace() {
            break;
        }
        pos = start;
        if pos == 0 {
            break;
        }
    }

    draft.drain(pos..*cursor);
    *cursor = pos;
}

pub(super) fn move_note_cursor_left(draft: &str, cursor: &mut usize) {
    if *cursor == 0 {
        return;
    }

    let new_cursor = draft[..*cursor].char_indices().last().map(|(idx, _)| idx).unwrap_or(0);
    *cursor = new_cursor;
}

pub(super) fn move_note_cursor_right(draft: &str, cursor: &mut usize) {
    if *cursor >= draft.len() {
        return;
    }

    let next = draft[*cursor..].chars().next().map(|ch| *cursor + ch.len_utf8()).unwrap_or(*cursor);
    *cursor = next;
}

pub(super) fn move_note_cursor_word_left(draft: &str, cursor: &mut usize) {
    if *cursor == 0 {
        return;
    }

    let mut pos = *cursor;
    let mut saw_word = false;

    while let Some((start, ch)) = prev_char_at(draft, pos) {
        if ch.is_whitespace() {
            if saw_word {
                *cursor = pos;
                return;
            }
            pos = start;
            if pos == 0 {
                *cursor = 0;
                return;
            }
            continue;
        }

        saw_word = true;
        pos = start;
        if pos == 0 {
            *cursor = 0;
            return;
        }
    }

    *cursor = pos;
}

pub(super) fn move_note_cursor_word_right(draft: &str, cursor: &mut usize) {
    let mut pos = *cursor;
    if pos >= draft.len() {
        return;
    }

    while let Some((start, ch)) = next_char_at(draft, pos) {
        pos = start + ch.len_utf8();
        if ch.is_whitespace() {
            break;
        }
        if pos >= draft.len() {
            *cursor = draft.len();
            return;
        }
    }

    while let Some((start, ch)) = next_char_at(draft, pos) {
        if !ch.is_whitespace() {
            *cursor = start;
            return;
        }
        pos = start + ch.len_utf8();
        if pos >= draft.len() {
            *cursor = draft.len();
            return;
        }
    }

    *cursor = pos;
}

pub(super) fn move_note_cursor_vertical(draft: &str, cursor: &mut usize, delta: i8) {
    if delta == 0 {
        return;
    }

    let chars: Vec<char> = draft.chars().collect();
    let cursor_char_idx = byte_to_char_index(draft, *cursor);
    let (line_idx, col_idx) = char_index_to_line_col(&chars, cursor_char_idx);
    let target_line =
        if delta.is_negative() { line_idx.checked_sub(1) } else { line_idx.checked_add(1) };

    let Some(target_line) = target_line else {
        return;
    };

    let Some(target_char_idx) = line_col_to_char_index(&chars, target_line, col_idx) else {
        return;
    };

    *cursor = char_to_byte_index(&chars, target_char_idx);
}

pub(super) fn scroll_note_cursor_vertical(draft: &str, cursor: &mut usize, delta: i32) {
    for _ in 0..delta.unsigned_abs() {
        move_note_cursor_vertical(draft, cursor, if delta < 0 { -1 } else { 1 });
    }
}

fn byte_to_char_index(draft: &str, byte_idx: usize) -> usize {
    draft[..byte_idx.min(draft.len())].chars().count()
}

fn prev_char_at(draft: &str, idx: usize) -> Option<(usize, char)> {
    if idx == 0 {
        return None;
    }

    draft[..idx].char_indices().last()
}

fn next_char_at(draft: &str, idx: usize) -> Option<(usize, char)> {
    if idx >= draft.len() {
        return None;
    }

    draft[idx..].char_indices().next().map(|(rel_idx, ch)| (idx + rel_idx, ch))
}

fn char_index_to_line_col(chars: &[char], char_idx: usize) -> (usize, usize) {
    let mut line = 0usize;
    let mut col = 0usize;

    for (idx, ch) in chars.iter().enumerate() {
        if idx == char_idx {
            return (line, col);
        }
        if *ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    (line, col)
}

fn line_col_to_char_index(chars: &[char], target_line: usize, target_col: usize) -> Option<usize> {
    let mut line = 0usize;
    let mut col = 0usize;

    for (idx, ch) in chars.iter().enumerate() {
        if line == target_line && col == target_col {
            return Some(idx);
        }

        if *ch == '\n' {
            if line == target_line {
                return Some(idx);
            }
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }

    if line == target_line && col >= target_col {
        Some(chars.len())
    } else {
        None
    }
}

fn char_to_byte_index(chars: &[char], char_idx: usize) -> usize {
    chars.iter().take(char_idx).map(|ch| ch.len_utf8()).sum()
}

#[cfg(test)]
mod tests {
    use super::{
        erase_word, insert_char, move_note_cursor_left, move_note_cursor_right,
        move_note_cursor_vertical, move_note_cursor_word_left, move_note_cursor_word_right,
    };

    #[test]
    fn insert_char_supports_newlines() {
        let mut draft = String::from("abc");
        let mut cursor = 1usize;

        insert_char(&mut draft, &mut cursor, '\n');

        assert_eq!(draft, "a\nbc");
        assert_eq!(cursor, 2);
    }

    #[test]
    fn note_cursor_moves_by_character_left_and_right() {
        let draft = "abçd";
        let mut cursor = draft.len();

        move_note_cursor_left(draft, &mut cursor);
        assert_eq!(cursor, "abç".len());

        move_note_cursor_left(draft, &mut cursor);
        assert_eq!(cursor, "ab".len());

        move_note_cursor_right(draft, &mut cursor);
        assert_eq!(cursor, "abç".len());
    }

    #[test]
    fn note_cursor_moves_between_lines_with_column_clamping() {
        let draft = "ab\ncde\nf";

        let mut cursor = "ab\ncd".len();
        move_note_cursor_vertical(draft, &mut cursor, -1);
        assert_eq!(cursor, "ab".len());

        let mut cursor = 1usize;
        move_note_cursor_vertical(draft, &mut cursor, 1);
        assert_eq!(cursor, "ab\nc".len());
    }

    #[test]
    fn note_cursor_moves_by_word_left_and_right() {
        let draft = "alpha, beta  gamma";

        let mut cursor = draft.len();
        move_note_cursor_word_left(draft, &mut cursor);
        assert_eq!(cursor, "alpha, beta  ".len());

        move_note_cursor_word_left(draft, &mut cursor);
        assert_eq!(cursor, "alpha, ".len());

        let mut cursor = 0usize;
        move_note_cursor_word_right(draft, &mut cursor);
        assert_eq!(cursor, "alpha, ".len());

        move_note_cursor_word_right(draft, &mut cursor);
        assert_eq!(cursor, "alpha, beta  ".len());
    }

    #[test]
    fn delete_word_removes_previous_chunk_and_leaves_cursor_at_boundary() {
        let mut draft = String::from("alpha, beta  gamma");
        let mut cursor = draft.len();

        erase_word(&mut draft, &mut cursor);
        assert_eq!(draft, "alpha, beta  ");
        assert_eq!(cursor, "alpha, beta  ".len());

        erase_word(&mut draft, &mut cursor);
        assert_eq!(draft, "alpha, ");
        assert_eq!(cursor, "alpha, ".len());
    }
}
