# view

Renders the UI from `&State` to the terminal in ASCII (ratatui). **Stateless**
and **read-only**: it never mutates `State`, never reads the keyboard, and never
talks to storage. `ratatui` is used here and nowhere else.

Public verb: `view::render(frame: &mut ratatui::Frame, state: &State)`.

See `EXAMPLES.md` for concrete `State` → rendered-output pairs.

## Render flow

`render` dispatches by `state.view` (the base `ViewMode`), then layers
`state.overlay` on top if present:

1. Draw the base view for the active `ViewMode`. The current primary screen is a
   month-grouped day × hour matrix from `calendar_view.rs`; `day_view.rs`
   remains as a per-day fallback/detail renderer.
2. Draw the persistent status bar (`status_bar.rs`) across every view.
3. If `state.overlay.is_some()`, draw the overlay popup over the base
   (`category_picker.rs` or `note_editor.rs`); the base stays visible underneath.

Each base view reads the shared `Cursor` for selection and never keeps its own
copy.

## Modules

- **`mod.rs`** — the `render()` entry point + dispatch by `ViewMode` + `Overlay`
  + fixed preview scenes for real terminal inspection.
- **`calendar_view.rs`** — the main matrix: columns are `00.00..23.00`, rows are
  sequential dates, and month headers split the timeline into readable blocks.
  Each populated hour cell shows the category digit in its category color; noted
  cells add `*`. The matrix uses visible vertical separators per hour column and
  horizontal rules between rows so it reads like a table. Month boundaries are
  emphasized with a stronger `═` separator.
- **`day_view.rs`** — a per-day list of 24 hour slots. Useful as a detail view
  and test surface, but no longer the main frontend.
- **`category_picker.rs`** — popup list, one row per `Category` in its own color,
  plus a final `[+] add note` action. The selected row is highlighted.
- **`note_editor.rs`** — popup text box showing the `NoteEditor` draft + text
  cursor; title reflects the `NoteTarget` (day or hour).
- **`status_bar.rs`** — shared "now" indicator + current focused slot line.
- **`theme.rs`** — the single source of category colors:
  `fn color(category: Category) -> ratatui::style::Color`. Every screen pulls
  from it so colors stay consistent.

## Cross-view concerns

- **"Now" indicator.** Derived from one `chrono::Local::now()` reading per draw —
  never from `State`. The status bar shows live date/hour, and the matching
  matrix cell/hour gets a distinct highlight layered on top of the category
  color. The current hour is compared only against the local date; the model
  always exposes exactly 24 slots (`00`–`23`) even across DST.
- **Current focus.** The status bar shows the focused slot, e.g.
  `Focus: 02.08.2026 13.00 Work *`.
- **Note markers.** A saved hour note is flagged with `*` inside the cell.
- **Legend palette.** The matrix view ends with a palette showing every category
  number and color mapping so the numeric cells remain readable.
- **Grid separators.** Column dividers are part of the view contract now; if the
  matrix is changed, keep the date column, hour columns, row lines, and stronger
  month separators visually distinct.

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
