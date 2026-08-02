# view

Renders the UI from `&State` to the terminal in ASCII (ratatui). **Stateless**
and **read-only**: it never mutates `State`, never reads the keyboard, and never
talks to storage. `ratatui` is used here and nowhere else.

Public verb: `view::render(frame: &mut ratatui::Frame, state: &State)`.

See `EXAMPLES.md` for concrete `State` → rendered-output pairs.

## Render flow

`render` dispatches by `state.view` (the base `ViewMode`), then layers
`state.overlay` on top if present:

1. Draw the base view for the active `ViewMode` (`calendar_view.rs` or
   `day_view.rs`).
2. Draw the persistent status bar (`status_bar.rs`) across every view.
3. If `state.overlay.is_some()`, draw the overlay popup over the base
   (`category_picker.rs` or `note_editor.rs`); the base stays visible underneath.

Each base view reads the shared `Cursor` for selection and never keeps its own
copy.

## Modules

- **`mod.rs`** — the `render()` entry point + dispatch by `ViewMode` + `Overlay`.
- **`calendar_view.rs`** — weeks × days grid; fixed five-week window centered on
  the selected week. Each day cell's fill is colored by its `dominant_category`
  (from `domain::calendar`); empty tail blocks render dim.
- **`day_view.rs`** — 24 hour cells; each filled hour's label + marker use its
  category color; empty hours render dim.
- **`category_picker.rs`** — popup list, one row per `Category` in its own color;
  highlights the selected row.
- **`note_editor.rs`** — popup text box showing the `NoteEditor` draft + text
  cursor; title reflects the `NoteTarget` (day or hour).
- **`status_bar.rs`** — shared "now" indicator + current focus line (below).
- **`theme.rs`** — the single source of category colors:
  `fn color(category: Category) -> ratatui::style::Color`. Every screen pulls
  from it so colors stay consistent.

## Cross-view concerns

- **"Now" indicator.** Derived from one `chrono::Local::now()` reading per draw —
  never from `State`. The status bar shows live date/weekday/hour; the matching
  cell/hour gets a distinct marker (`●` or underline) layered on category color.
  The current hour is compared only against the local date; the model always
  exposes exactly 24 slots (`00`–`23`) even across DST.
- **Current focus.** The status bar shows what the cursor is on, e.g.
  `Focus: 13:00 Work` (Day) or `Focus: Sun 2 Aug (7h)` (Calendar).
- **Note markers.** A saved day/hour note is flagged with `*` in the parent view.

## Category colors

| # | Category       | ratatui `Color`  |
|---|----------------|------------------|
| 0 | Sleep          | `Indexed(19)`    |
| 1 | Health         | `Cyan`           |
| 2 | Friends/Family | `Green`          |
| 3 | Romantic       | `Indexed(211)`   |
| 4 | Work           | `Indexed(240)`   |
| 5 | Waste          | `Red`            |
| 6 | Travel         | `Indexed(244)`   |
| 7 | Hobbies/Skills | `Indexed(208)`   |
| 8 | Relaxation     | `Indexed(93)`    |
| 9 | Other          | `Yellow`         |

Prefer 256-color indices over raw RGB. Degrade gracefully: when the terminal
lacks the color depth, categories fall back to the nearest color, and selection,
now markers, and labels stay distinguishable via reverse/bold/underline even in
monochrome.
