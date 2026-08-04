# view — examples

Concrete `State` → rendered-output pairs for the current matrix frontend. Each
example shows the relevant slice of `State` and the ASCII the view produces from
it. Colors cannot render in markdown, so the examples describe where category,
focus, and "now" highlights appear. The matrix is scrollable in both axes around
the focused cell; the examples below show only the visible viewport. Popup
overlays are anchored beside the focused cell and flip or clamp to stay on
screen, so the exact side can vary with terminal size.

The "now" indicator in every example assumes a live clock reading of
`02.08.2026 · 13:47` at render time (never taken from `State`).

## 1. Matrix view

State:

```rust
State {
    view: ViewMode::Calendar,
    cursor: Cursor { date: 2026-08-02, hour: Some(13) },
    overlay: None,
    days: {
        2026-07-31: some Travel hours,
        2026-08-01: empty,
        2026-08-02: day note "Daily journal", 00..06 Sleep, 07 Health, 08 Travel,
                    09..11 Work, 12 Health,
                    13 Work(note), 14 Work,
                    16 Relaxation, 17 HobbiesSkills,
        2026-08-03: some Sleep hours,
        ...
    },
    last_error: None,
    quit: false,
}
```

Output:

```text
┌ life-tracker ───────────────────────── Aug 2026 ───────────────────────────────────────────────┐
│              │10.00│11.00│12.00│13.00│14.00│15.00│16.00│17.00│┌ Palette ─────────────┐         │
│──────────────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼│0 = Sleep             │         │
│ **July 2026**                                                    │1 = Health            │         │
│══════════════╪═════╪═════╪═════╪═════╪═════╪═════╪═════╪═════╪│2 = Friends/Family    │         │
│ 31.07.2026 Wed  │ ·   │ ·   │ ·   │ ·   │ ·   │ ·   │ ·   │ ·   ││3 = Romantic          │         │
│──────────────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼│4 = Work              │         │
│ **August 2026**                                                  │5 = Waste             │         │
│══════════════╪═════╪═════╪═════╪═════╪═════╪═════╪═════╪═════╪│6 = Travel            │         │
│ 01.08.2026 Fri  │ ·   │ ·   │ ·   │ ·   │ ·   │ ·   │ ·   │ ·   ││7 = Hobbies/Skills    │         │
│──────────────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼│8 = Relaxation        │         │
│ 02.08.2026 Sun* │ 4   │ 4   │ 1   │ 4*  │ 4   │ ·   │ 8   │ 7   ││9 = Other             │         │
│──────────────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼└──────────────────────┘         │
│ 03.08.2026 Sun  │ ·   │ ·   │ ·   │ ·   │ ·   │ ·   │ ·   │ ·   │                                  │
│──────────────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼                                  │
├────────────────────────────────────────────────────────────────────────────────────────────────┤
│ now 02.08.2026 Sun · 13:47   Focus: 02.08.2026 Sun 13.00 Work *                                  │
│ ←↑↓→ move  ⏎ open  n note  x clear  q quit                                                      │
└────────────────────────────────────────────────────────────────────────────────────────────────┘
```

Legend:
- Each occupied cell shows the category digit in its category color.
- A noted cell appends `*`, e.g. `4*`. A day with a day-level note shows `*`
  appended after the weekday abbreviation in the date column (e.g.,
  `02.08.2026 Sun*│`), not on hour cells.
- The date column shows the date with an abbreviated weekday (e.g.,
  `02.08.2026 Sun`).
- The focused cell gets the selection highlight.
- The real current `(date, hour)` cell gets the "now" highlight when it is not
  also the focused cell.
- Vertical separators, per-row `─` rules, and stronger month `═` separators are
  part of the intended appearance, not incidental spacing.
- The visible hour columns shown above are only the current horizontal viewport;
  moving left/right shifts that hour window around the focused hour.
- The category picker and note editor are positioned next to the active cell
  rather than centered.
- **Date column highlight containment.** When the date column is focused
  (`cursor.hour = None`), the selection highlight is rendered only on the
  16-character date text and does **not** extend into the `│` separator. This
  keeps the highlight visually contained within the date column boundary.

## 1b. Matrix view - day focus

State:

```rust
State {
    cursor: Cursor { date: 2026-08-02, hour: None },
    overlay: None,
    days: {
        2026-08-02: day note "Daily journal",
                    08 Work, 13 Work(note),
    },
    last_error: None,
    quit: false,
}
```

Output:

```text
┌ life-tracker ───────────────────────── Aug 2026 ───────────────────────────────────────────────┐
│              │08.00│09.00│10.00│11.00│12.00│13.00│14.00│15.00│┌ Palette ─────────────┐         │
│──────────────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼│0 = Sleep             │         │
│ **August 2026**                                                  │1 = Health            │         │
│══════════════╪═════╪═════╪═════╪═════╪═════╪═════╪═════╪═════╪│2 = Friends/Family    │         │
│ 02.08.2026 Sat*│ 4   │ ·   │ ·   │ ·   │ ·   │ 4*  │ ·   │ ·   ││3 = Romantic          │         │
│──────────────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼│4 = Work              │         │
│ 03.08.2026 Sun  │ ·   │ ·   │ ·   │ ·   │ ·   │ ·   │ ·   │ ·   ││5 = Waste             │         │
│──────────────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼│6 = Travel            │         │
├────────────────────────────────────────────────────────────────────────────────────────────────┤
│ now 02.08.2026 Sat · 13:47   Focus: 02.08.2026 Sat Day *                                      │
│ ←↑↓→ move  ⏎ open  n note  x clear  q quit                                                      │
└────────────────────────────────────────────────────────────────────────────────────────────────┘
```

Legend:
- The date column is highlighted when `cursor.hour` is `None` (day focus).
- The selection highlight is only on the 16-character date text and does not
  extend into the `│` separator or the adjacent hour cell.
- Day focus means actions apply to the whole day; pressing `Enter` opens the
  day-level note picker (`[+] add note`, `[x] delete note`).

## 2. Category picker overlay

State: base matrix view, plus

```rust
overlay: Some(Overlay::CategoryPicker {
    target: NoteTarget::Hour { date: 2026-08-02, hour: 13 },
    selected: CategoryPickerSelection::Category(Category::Sleep),
})
```

Output:

```text
        ┌ Set activity - 13.00 ──────┐
        │ > 0 Sleep                  │
        │   1 Health                 │
        │   2 Friends/Family         │
        │   3 Romantic               │
        │   4 Work                   │
        │   5 Waste                  │
        │   6 Travel                 │
        │   7 Hobbies/Skills         │
        │   8 Relaxation             │
        │   9 Other                  │
        │   [+] add note             │
        │   [x] delete note          │
        │   [x] delete activity      │
        │ ⏎ confirm   Esc cancel     │
        └────────────────────────────┘
```

Legend:
- Each category row uses the category color.
- The selected row keeps its category color but adds the focus highlight.
- `[+] add note` opens the note editor for the same `(date, hour)` slot.
- `[x] delete note` clears only the note for that hour, preserving its category.
- `[x] delete activity` clears only the category for that hour and preserves an
  existing note.
- The add-note row is part of the picker selection model, so it can be focused
  and confirmed just like the category and delete rows.
- The popup is anchored beside the focused cell and will move left or up if
  there is not enough room on the right or below.

## 3. Day note picker

State: base matrix view, with the date column focused, plus

```rust
State {
    cursor: Cursor { date: 2026-08-02, hour: None },
    overlay: Some(Overlay::CategoryPicker {
        target: NoteTarget::Day { date: 2026-08-02 },
        selected: CategoryPickerSelection::AddNote,
    }),
    ..
}
```

Output:

```text
        ┌ Day - 02.08.2026 ─────────┐
        │ > [+] add note            │
        │   [x] delete note         │
        │ ⏎ confirm   Esc cancel    │
        └───────────────────────────┘
```

Legend:
- Day focus means actions apply to the whole day, not an hour slot.
- Pressing `Enter` on a day focus opens this picker, which intentionally exposes
  only note actions.
- The popup is anchored beside the date cell and clamped to remain visible.

## 4. Note editor overlay

State: base matrix view, plus

```rust
overlay: Some(Overlay::NoteEditor {
    target: NoteTarget::Hour { date: 2026-08-02, hour: 13 },
    draft: "Sprint planning, blocked\non API keys.".into(),
    cursor: 40,
})
```

Output:

```text
        ┌ Note - 13:00 Work ─────────┐
        │ Sprint planning, blocked   │
        │ on API keys.|              │
        │                            │
        │                            │
        │ ⇧⏎ newline  ⏎ save         │
        │ Esc cancel                 │
        └────────────────────────────┘
```

Legend:
- The title reflects the target slot.
- The draft is shown verbatim and is not persisted until save.
- The visible `|` marker shows the current text cursor. If the stored cursor is
  past the end of the draft, it is clamped to the end of the text.
- `Shift+Enter` inserts a new line at the cursor when the terminal reports it
  distinctly; `Ctrl+J` is the reliable fallback. Plain `Enter` saves.
- The editor is anchored beside the focused cell instead of being centered.

The matrix is the only base screen now. Editing happens by focusing an hour cell
and opening one of the two popup overlays above.
