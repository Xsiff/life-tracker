# view — examples

Concrete `State` → rendered-output pairs. Each example shows the relevant slice
of `State` and the ASCII the view produces from it. Colors can't render in
markdown, so a legend follows each mockup (see `AGENTS.md` → Category colors).

The "now" indicator in every example assumes a live clock reading of
`Sun 2 Aug 2026 · 13:47` at render time (never taken from `State`).

## 1. Calendar view

State:

```rust
State {
    view: ViewMode::Calendar,
    cursor: Cursor { date: 2026-08-02 (Sun), hour: None },
    overlay: None,
    days: {
        // W31 has data on Mon–Fri and Sun; only non-empty days are present
        2026-07-27 (Mon): dominant Sleep,  ~2/3 filled,
        2026-07-28 (Tue): dominant Work,   fully filled,
        2026-07-29 (Wed): light fill,
        2026-07-30 (Thu): dominant Sleep,  ~2/3 filled,
        2026-07-31 (Fri): ~1/3 filled,
        2026-08-02 (Sun): 7h logged,
        // W32 Mon/Tue/Wed partially filled ...
    },
    last_error: None,
    quit: false,
}
```

Output:

```
┌ life-tracker ───────────────────────── Aug 2026 ┐
│        W31   W32   W33   W34   W35              │
│  Mon  [▓▓░] [▓░░] [   ] [   ] [   ]             │
│  Tue  [▓▓▓] [▓▓░] [   ] [   ] [   ]             │
│  Wed  [░░░] [▓░░] [   ] [   ] [   ]             │
│  Thu  [▓▓░] [   ] [   ] [   ] [   ]             │
│  Fri  [▓░░] [   ] [   ] [   ] [   ]             │
│  Sat  [   ] [   ] [   ] [   ] [   ]             │
│  Sun ●[▓▓░][   ] [   ] [   ] [   ]             │
├─────────────────────────────────────────────────┤
│ now Sun 2 Aug · 13:47   Focus: Sun 2 Aug (7h)   │
│ ←↑↓→ move  ⏎ open  N note  v view  q quit        │
└─────────────────────────────────────────────────┘
```

Legend:
- `Mon W31 [▓▓░]` → dominant Sleep `Indexed(19)`; empty tail dim gray.
- `Tue W31 [▓▓▓]` → dominant Work `Indexed(240)`, fully filled.
- `Wed W31 [░░░]` → has data but light fill, dim gray.
- `Sun ●[▓▓░]` → `●` marks today; `cursor` is here, so the cell also gets a
  reversed/bold highlight over its category color.

## 2. Day view

State:

```rust
State {
    view: ViewMode::Day,
    cursor: Cursor { date: 2026-08-02, hour: Some(13) },
    overlay: None,
    days: { 2026-08-02: Day {
        hours: [ 00..=06 Sleep, 07 Health, 08 Travel, 09..=11 Work,
                 12 Health, 13 Work (note: "Sprint planning…"),
                 14 Work, 16 Relaxation, 17 HobbiesSkills, ... ],
        note: None,
    } },
    ...
}
```

Output (cursor on hour 13, which has a note; now-hour is 13 but selection marker
`◀` wins on the focused row, `●` marks the now-hour otherwise):

```
┌ Sun, Aug 2 2026 ────────────────────────────────┐
│ 00 Sleep      06 Sleep      12 Health           │
│ 01 Sleep      07 Health     13 Work  *◀        │
│ 02 Sleep      08 Travel    ●14 Work             │
│ 03 Sleep      09 Work       15 Work             │
│ 04 Sleep      10 Work       16 Relaxation       │
│ 05 Sleep      11 Work       17 Hobbies/Skills   │
│                             ...                 │
├─────────────────────────────────────────────────┤
│ now Sun 2 Aug · 13:47   Focus: 13:00 Work *     │
│ ↑↓ move  ⏎ set  x clear  n note  v view  Esc back│
└─────────────────────────────────────────────────┘
```

Legend:
- Labels render in category color: `Sleep` `Indexed(19)`, `Health` `Cyan`,
  `Travel` `Indexed(244)`, `Work` `Indexed(240)`, `Relaxation` `Indexed(93)`,
  `Hobbies/Skills` `Indexed(208)`.
- `13 Work *◀` → focused hour: category color + reversed/bold + `◀`; `*` = note.
- `●14 Work` → `●` marks the current ("now") hour when it isn't the focused one.
- Empty hours render dim gray.

## 3. Category picker overlay

State: base `ViewMode::Day`, plus

```rust
overlay: Some(Overlay::CategoryPicker {
    date: 2026-08-02, hour: 13, selected: Category::Sleep,
})
```

Output (popup over the day view):

```
        ┌ Set activity — 13:00 ──────┐
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
        ├────────────────────────────┤
        │ ⏎ confirm   Esc cancel     │
        └────────────────────────────┘
```

Legend: each row in its own category color; `> 0 Sleep` (matching `selected`)
keeps its color with a reversed/bold cursor. Digits map to the discriminant.

## 4. Note editor overlay

State: base `ViewMode::Day`, plus

```rust
overlay: Some(Overlay::NoteEditor {
    target: NoteTarget::Hour { date: 2026-08-02, hour: 13 },
    draft: "Sprint planning, blocked\non API keys.".into(),
    cursor: 40,
})
```

Output:

```
        ┌ Note — 13:00 Work ─────────┐
        │ Sprint planning, blocked   │
        │ on API keys.               │
        │                            │
        │                            │
        ├────────────────────────────┤
        │ ⏎ save   Esc cancel        │
        └────────────────────────────┘
```

Legend: title reflects the `NoteTarget`; a day-level target renders a title like
`Note — Sun, Aug 2 2026`. The draft is shown verbatim with the text cursor; it
is not persisted until the save `Confirm` (Enter).

## 5. Error surfaced

State: any view with `last_error: Some("disk full: could not write hour")`. The
status bar shows the error in place of / alongside the focus line until the next
successful action clears it. In-memory `State` is otherwise unchanged (the failed
mutation was not committed).
