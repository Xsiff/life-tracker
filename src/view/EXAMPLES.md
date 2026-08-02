# view — examples

Concrete `State` → rendered-output pairs for the current matrix frontend. Each
example shows the relevant slice of `State` and the ASCII the view produces from
it. Colors cannot render in markdown, so the examples describe where category,
focus, and "now" highlights appear.

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
        2026-08-02: 00..06 Sleep, 07 Health, 08 Travel,
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
│            │00.00│01.00│02.00│03.00│04.00│ ... │22.00│23.00│                                  │
│────────────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼──────────────────────────────────│
│ **July 2026**                                                                                  │
│════════════╪═════╪═════╪═════╪═════╪═════╪═════╪═════╪═════╪══════════════════════════════════│
│ 31.07.2026 │ 6   │ 6*  │ ·   │ ·   │ ·   │ ... │ ·   │ ·   │                                  │
│────────────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼──────────────────────────────────│
│ **August 2026**                                                                                │
│════════════╪═════╪═════╪═════╪═════╪═════╪═════╪═════╪═════╪══════════════════════════════════│
│ 01.08.2026 │ ·   │ ·   │ ·   │ ·   │ ·   │ ... │ ·   │ ·   │                                  │
│────────────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼──────────────────────────────────│
│ 02.08.2026 │ 0   │ 0   │ 0   │ 0   │ 0   │ ... │ ·   │ ·   │                                  │
│────────────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼──────────────────────────────────│
│ 03.08.2026 │ 0   │ 0   │ 0   │ 0   │ ·   │ ... │ ·   │ ·   │                                  │
│────────────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┼──────────────────────────────────│
│ ...                                                                                            │
│ 0=Sleep  1=Health  2=Friends/Family  3=Romantic  4=Work                                       │
│ 5=Waste  6=Travel  7=Hobbies/Skills  8=Relaxation  9=Other                                    │
├────────────────────────────────────────────────────────────────────────────────────────────────┤
│ now 02.08.2026 · 13:47   Focus: 02.08.2026 13.00 Work *                                        │
│ ←↑↓→ move  ⏎ set  x clear  n note  v view  q quit                                               │
└────────────────────────────────────────────────────────────────────────────────────────────────┘
```

Legend:
- Each occupied cell shows the category digit in its category color.
- A noted cell appends `*`, e.g. `4*`.
- The focused cell gets the selection highlight.
- The real current `(date, hour)` cell gets the "now" highlight when it is not
  also the focused cell.
- Vertical separators, per-row `─` rules, and stronger month `═` separators are
  part of the intended appearance, not incidental spacing.

## 2. Category picker overlay

State: base matrix view, plus

```rust
overlay: Some(Overlay::CategoryPicker {
    date: 2026-08-02,
    hour: 13,
    selected: Category::Sleep,
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
        │ [+] add note               │
        │ ⏎ confirm   Esc cancel     │
        └────────────────────────────┘
```

Legend:
- Each category row uses the category color.
- The selected row keeps its category color but adds the focus highlight.
- `[+] add note` opens the note editor for the same `(date, hour)` slot.

## 3. Note editor overlay

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
        │ on API keys.               │
        │                            │
        │                            │
        │ ⏎ save   Esc cancel        │
        └────────────────────────────┘
```

Legend:
- The title reflects the target slot.
- The draft is shown verbatim and is not persisted until save.

## 4. Detail day view

The code still supports `ViewMode::Day` as a per-day rendering surface.

```text
┌ Sun, Aug 2 2026 ────────────────────────────────────────┐
│ Focused day: 02.08.2026                                 │
│ 00.00 Sleep                                             │
│ 01.00 Sleep                                             │
│ ...                                                     │
│ 13.00 Work *◀                                           │
│ ...                                                     │
├─────────────────────────────────────────────────────────┤
│ now 02.08.2026 · 13:47   Focus: 02.08.2026 13.00 Work * │
│ ↑↓ move  ⏎ set  x clear  n note  v view  Esc back       │
└─────────────────────────────────────────────────────────┘
```
