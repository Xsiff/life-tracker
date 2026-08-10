# view

Renders the UI from `&State` to the terminal in ASCII (ratatui). **Stateless**
and **read-only**: it never mutates `State`, never reads the keyboard, and never
talks to storage. `ratatui` is used here and nowhere else.

Public verb: `view::render(frame: &mut ratatui::Frame, state: &State)`.

See `EXAMPLES.md` for concrete `State` → rendered-output pairs.

## Render flow

`render` draws the base matrix from `state`, then layers `state.overlay` on top
if present:

1. Draw the base matrix view from `calendar_view.rs`.
2. Draw the persistent status bar (`status_bar.rs`) across every view.
3. If `state.overlay.is_some()`, draw the overlay popup anchored to the focused
   cell over the base (`category_picker.rs`, `help_popup.rs`, or
   `note_editor.rs`); the base stays visible underneath.

Each base view reads the shared `Cursor` for selection and never keeps its own
copy.

## Modules

- **`mod.rs`** — the `render()` entry point + overlay handling + preview
  scenes for real terminal inspection.
- **`calendar_view.rs`** — the main matrix: columns are `00.00..23.00`, rows are
  sequential dates, and month headers split the timeline into readable blocks.
  Month labels are rendered as bold text in the terminal, not wrapped in
  literal markdown markers.
  Each populated hour cell shows the category digit in its category color; noted
  cells add `*`. The date column shows the date with an abbreviated weekday
  (e.g., `02.08.2026 Sat`), with a single `*` appended after the weekday when the
  day has a day-level note (e.g., `02.08.2026 Sat*│`). The date column is
  focusable and represents whole-day actions when `cursor.hour` is `None`. The
  date column receives the selection highlight only when the day itself is
  focused (`hour = None`); when any hour within the day is focused (`hour =
  Some(_)`), the date column stays unhighlighted. The matrix uses visible
  vertical separators per hour column and horizontal rules between rows so it
  reads like a table. Ordinary date rows use `─`, and month headers are
  followed by a stronger `═` separator while the final day of a month also
  transitions directly to a strong `═` separator before the next month header.
  The visible date/hour window is derived from the actual terminal size and
  centered around the focused cell.
- **`category_picker.rs`** — popup list driven by `NoteTarget`. For an hour
  target it shows one row per `Category` in its own color plus action rows for
  `[+] add note`, `[x] delete note`, and `[x] delete activity`. For a day
  target it renders only `[+] add note` and `[x] delete note`. The selected row
  is highlighted either way. The popup is positioned next to the focused cell
  when space allows, then flips or clamps to stay visible.
- **`help_popup.rs`** — read-only popup opened by `?`. It lists every category
  with its digit and a short description. `Esc` or `Enter` closes it.
- **`note_editor.rs`** — popup text box showing the `NoteEditor` draft + text
  cursor; title reflects the `NoteTarget` (day or hour). The current cursor
  position is rendered as a raw block `█` with no extra highlight or reverse
  video. The block occupies a full character cell so the cursor reads as a
  conventional terminal caret. `Shift+Enter` inserts a line break when the
  terminal reports it distinctly, `Ctrl+J` is the reliable fallback, and plain
  `Enter` still saves. Arrow keys move the text cursor within the draft
  (`Left`/`Right` by character, `Up`/`Down` between lines with column clamping).
  `Option/Alt+Left` and `Option/Alt+Right` jump by word/chunk. `Option/Alt+Delete`
  and `Option/Alt+Backspace` delete the previous word/chunk. Like the picker,
  it anchors to the active cell instead of centering on the screen. The
  note text area scrolls to keep the current cursor line in view when the draft
  grows taller than the popup. The separator above the helper line spans the
  full inner popup width so the editor reads as one contained window. Draft
  text wraps to the popup's inner width so long unbroken input never overflows
  the window.
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
  `Focus: 02.08.2026 13.00 Work *` or `Focus: 02.08.2026 Day *`.
- **Header mirroring.** When a matrix cell is focused, its matching hour header
  and date header also get a lighter blue highlight. The left date column uses
  the same lighter treatment as the hour header row, so the two axes stay in
  sync. When the current hour is visible, its matching header labels use a
  lighter yellow highlight. The cell itself keeps the stronger selection/now
  treatment.
- **Note markers.** A cell with an hour-level note is flagged with `*` inside that cell. A day with a day-level note shows a single `*` appended after the weekday abbreviation in the date column (e.g., `02.08.2026 Sat*│`). Hour slots for that day do **not** show `*` unless they individually have hour-level notes.
- **Legend palette.** The matrix view reserves a dedicated right-side pane for a
  boxed subtable showing every category number and color mapping so the numeric
  cells remain readable.
- **Category help.** The `?` popup is the longer-form companion to the palette
  pane: it explains what each category means in plain language.
- **Grid separators.** Column dividers are part of the view contract now; if
  the matrix is changed, keep the date column, hour columns, row lines, and the
  month transition rule visually distinct. Month headers should be followed by a
  strong `═` separator, and the last day of a month should also end on a strong
  `═` separator instead of a weak `─` rule first.
- **Viewport following focus.** The matrix scrolls both vertically and
  horizontally around the focused cell. If the terminal is too small to show all
  dates or all 24 hours, the focused date/hour stays within the visible window.

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
