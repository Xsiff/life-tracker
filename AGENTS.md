# life-tracker

## What we're building

Terminal (TUI) life-tracker in Rust (`ratatui` + `crossterm`). Two-level UI:

1. **Calendar view (main screen)** — a grid where rows are days of the week (Mon…Sun) and columns are weeks. Each cell is one day; the user navigates and selects a day cell to drill in.
2. **Day view** — opens on a selected day cell. Shows 24 cells, one per hour of the day. The user fills each hour cell with an activity drawn from a fixed set of categories.

**Domain model:**
- **Category** — a fixed set of 10 activity types (Sleep, Health, Friends/Family, Romantic, Work, Waste, Travel, Hobbies/Skills, Relaxation, Other). Categories are the palette for filling hours.
- Each hour cell holds one categorized activity (a category plus optional specifics).

Keep this two-level calendar → day(24 hours) structure and the category-driven fill model in mind; it's the core of the app and not obvious from filenames once code lands.

## Dependencies

- `ratatui` — TUI rendering (widgets, layout).
- `crossterm` — terminal backend (raw mode, events, alt-screen).
- `rusqlite` (with the `bundled` feature) — SQLite persistence; no system SQLite required.
- `chrono` — date/time math (weeks, days, ISO week numbers); also drives the "now" indicator.
- `directories` — resolve the per-OS data dir for the DB file.
- `anyhow` — error handling.

## File & folder layout

```
life-tracker/
├── Cargo.toml
├── src/
│   ├── main.rs              # entry: terminal setup/teardown, run loop
│   ├── app.rs               # App state, ViewMode + Overlay, cursor, mode transitions
│   ├── event.rs             # crossterm event polling -> AppEvent
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── category.rs      # Category definitions (fixed palette ~10)
│   │   ├── activity.rs      # Activity = category + optional note
│   │   └── calendar.rs      # Day, Week, hour-slot model + indexing
│   ├── ui/
│   │   ├── mod.rs           # top-level draw() dispatch by ViewMode + Overlay
│   │   ├── calendar_view.rs # weeks x days grid rendering
│   │   ├── day_view.rs      # 24-hour cell rendering
│   │   ├── category_picker.rs # popup/list to choose a category
│   │   ├── note_editor.rs   # popup text editor for day/hour notes
│   │   ├── status_bar.rs    # "now" indicator + current focus line
│   │   └── theme.rs         # category colors, styles
│   └── storage/
│       ├── mod.rs
│       └── sqlite_store.rs  # open DB, load/save (date, hour) rows + day notes
```

**Rationale (not obvious from filenames):**
- `domain/` is pure logic (no ratatui/crossterm) so it is unit-testable without a terminal.
- `ui/` renders from `&App` and never mutates state; `app.rs` owns all mutation. Keeps draw/update separated.
- `storage/` is isolated behind a small interface so the SQLite backend can change without touching UI.

## Data model

```rust
// Discriminants match the picker's number keys (0..9).
enum Category {
    Sleep = 0,      // hours sleeping
    Health,         // self-care, workouts, etc.
    FriendsFamily,  // time with friends or family
    Romantic,       // time with partner
    Work,           // time working
    Waste,          // time spent doing nothing
    Travel,         // time travelling
    HobbiesSkills,  // hobbies, improving skills, productive activity
    Relaxation,     // chill activities: gaming, films, etc.
    Other,          // anything else
}

struct Activity {
    category: Category,
    note: Option<String>,   // optional per-hour note
}

struct Day {
    date: NaiveDate,
    hours: [Option<Activity>; 24],  // index = hour 0..23
    note: Option<String>,           // optional day-level note
}
```

In-memory state is a `BTreeMap<NaiveDate, Day>` — sparse, only days with data are held.

A day with only a day-level note still counts as data and must be present in the
map. Empty days with no activities and no note should not be materialized.

**Current time ("now") is not stored** — it is read from the system clock (`chrono::Local::now()`) on each draw. The app derives today's date and the current hour to highlight the "now" cell and drive the status bar (see below). Do not persist it.

## View modes

The app supports switching between multiple views (the calendar and day views are the first two; more can follow, e.g. week/agenda/stats). Model this with an explicit `ViewMode` so rendering and key handling stay view-agnostic:

```rust
enum ViewMode { Calendar, Day /* , Week, Agenda, Stats, ... */ }
```

Design constraints so new views drop in cleanly:
- `app.rs` holds the active `ViewMode` and a shared selection/cursor (current date + optional hour) that every view interprets in its own way; views must not keep their own private "current date" copies that can drift.
- `ui/mod.rs` dispatches `draw()` by `ViewMode`; each view is a self-contained module that renders from `&App`.
- Notes, the "now" indicator, and the category palette are cross-view concerns — keep them in shared modules (`status_bar.rs`, `theme.rs`, note editor) rather than baking them into a single view.
- A single key (e.g. `Tab` / `v`) cycles `ViewMode`; keep the binding in one place so all views share it.
- Switching to `Day` through the view-cycle key keeps the shared date selection
  and selects the current local hour when no hour is selected. Switching back to
  `Calendar` clears the hour portion of the shared selection. Opening a day with
  `Enter` also selects the current local hour only when the selected date is
  today; otherwise it selects hour `0`.

Only the Calendar and Day views are mocked up below for now, but keep the design suitable for additional views.

## Category colors

Each category has a fixed color, defined once in `ui/theme.rs` (`fn color(category: Category) -> ratatui::style::Color`). This is the single source of truth — the calendar cells, day-view hour cells, and category picker all pull from it, so colors stay consistent across every screen. Empty hours render dim/gray.

| # | Category       | Color              | ratatui `Color`  |
|---|----------------|--------------------|------------------|
| 0 | Sleep          | deep blue          | `Indexed(19)`    |
| 1 | Health         | cyan               | `Cyan`           |
| 2 | Friends/Family | green              | `Green`          |
| 3 | Romantic       | pink               | `Indexed(211)`   |
| 4 | Work           | metallic dark gray | `Indexed(240)`   |
| 5 | Waste          | red                | `Red`            |
| 6 | Travel         | gray               | `Indexed(244)`   |
| 7 | Hobbies/Skills | orange             | `Indexed(208)`   |
| 8 | Relaxation     | purple             | `Indexed(93)`    |
| 9 | Other          | yellow             | `Yellow`         |

In the **calendar view**, each day cell's fill blocks are colored by that day's dominant category (or a per-hour gradient); in the **day view**, each filled hour's label and marker use its category color; in the **picker**, each category row is shown in its own color. Prefer 256-color indices over raw RGB for broad terminal support; degrade gracefully on terminals without color.

## Persistence (SQLite via `rusqlite`)

Single DB file (`life-tracker.db`) in the `directories::ProjectDirs` data dir.

Schema — one row per filled hour, plus one row per day-level note:

```sql
CREATE TABLE IF NOT EXISTS activities (
    date     TEXT    NOT NULL,   -- ISO date, e.g. 2026-08-02
    hour     INTEGER NOT NULL,   -- 0..23
    category TEXT    NOT NULL,
    note     TEXT,               -- optional per-hour note
    PRIMARY KEY (date, hour)
);

CREATE TABLE IF NOT EXISTS day_notes (
    date     TEXT    NOT NULL,   -- ISO date
    note     TEXT    NOT NULL,   -- day-level note
    PRIMARY KEY (date)
);
```

- **Load on startup:** `SELECT * FROM activities` and `SELECT * FROM day_notes` → build the `BTreeMap<NaiveDate, Day>`.
- **Set an hour:** `INSERT OR REPLACE` the single `(date, hour)` row (category + optional per-hour note).
- **Clear an hour:** `DELETE` the single `(date, hour)` row.
- **Set/clear a day note:** `INSERT OR REPLACE` / `DELETE` the single `day_notes(date)` row.
- Writes are per-mutation (one row), so no bulk save/flush step is needed.

## Screens & state machine

Two orthogonal pieces of state, not one `Screen` enum:

- **`ViewMode`** (see above) is the base view — `Calendar`, `Day`, and future `Week`/`Agenda`/`Stats`. It is always set.
- **`Overlay`** is optional modal state layered on top of the active view. When `Some`, it captures input and renders as a popup; the base view stays visible underneath. `None` means the base view has focus.

```rust
struct App {
    view: ViewMode,          // base view, always present
    cursor: Cursor,          // shared selection: date + optional hour
    overlay: Option<Overlay>,// modal popup on top of the base view
    // ...
}

enum Overlay {
    CategoryPicker {
        date: NaiveDate,
        hour: u8,
        selected: Category,
    }, // set the activity for an hour
    NoteEditor {
        target: NoteTarget,
        draft: String,
        cursor: usize,
    }, // edit a day or hour note
}

enum NoteTarget {
    Day { date: NaiveDate },
    Hour { date: NaiveDate, hour: u8 },
}
```

`NoteEditor` must also retain its editable draft text and text-cursor state in
the app’s overlay state. The target identifies what will be changed; the draft
is not written to storage until `Enter` confirms it. `Esc` discards the draft
and leaves the target, selection, and stored value unchanged.

Do **not** model the base view as a `Screen` variant alongside the overlays — that duplicates "which view am I in" between `Screen` and `ViewMode` and lets them drift. The base view is `ViewMode`; overlays are `Option<Overlay>`. Input routing: if `overlay.is_some()`, the overlay handles keys; otherwise the active view does.

Transitions: `Calendar --Enter--> Day --Enter--> CategoryPicker --select--> Day`.
`Esc` from a picker or note editor cancels and returns to the underlying view;
`Esc` from Day returns to Calendar.
From either Calendar (day note) or Day (hour note), a note key opens `NoteEditor`; `Enter`/`Esc` saves/cancels back to the prior screen.

Calendar navigation displays a fixed five-week window centered on the selected
week. Moving left or right shifts that window as needed; navigation has no hard
date boundary. Moving up or down changes the weekday while preserving the
selected week.

Selecting a category for an already-filled hour replaces its category and
preserves the existing hour note. Clearing an hour removes both its activity
and its hour note. Saving an empty note clears the corresponding note.

Every storage mutation must be attempted before the in-memory mutation is
committed. If SQLite returns an error, the app keeps the previous in-memory
state and exposes a readable error in the status area or an error overlay.
Loaded rows must validate that dates parse as ISO dates, hours are in `0..=23`,
and category names are known; invalid rows are reported and skipped rather than
silently converted.

## Status bar ("now" indicator + current focus)

A persistent status bar (`ui/status_bar.rs`), rendered across all views, shows two things:

1. **Now** — the live system date, weekday, and hour (from `chrono::Local::now()`), e.g. `Sun 2 Aug · 13:47`. The matching cell is also highlighted in-view (the "now" cell/hour gets a distinct marker such as a `●` or an underline, layered on top of category color).
2. **Current focus** — what the selection is on and its activity/category, e.g. `Focus: 13:00 Work` in the day view, or `Focus: Sun 2 Aug (7h logged)` in the calendar view.

Keep this in a shared module so every view (current and future) renders the same indicator.

The calendar’s dominant category is the category with the greatest number of
filled hours. Ties are resolved by the lower category discriminant. A day with
no filled hours renders as empty, even if it has a note.

The now indicator is derived from one local-clock reading per draw. Stored dates
are local calendar dates, and the current hour is compared only with the local
date. Daylight-saving transitions may produce a repeated or missing local hour;
the model still exposes exactly 24 numbered slots (`00` through `23`).

## UI mockups

Markdown code blocks can't render real terminal color, so each mockup below is followed by a color legend mapping the visible cells to their runtime category colors (see Category colors).

### Calendar view (main screen)

Rows = days of week, columns = weeks. Selected cell highlighted; the "now" day is marked with `●`; each cell's fill blocks are colored by the day's dominant category. A status bar at the bottom shows the live time and current focus.

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

Cell colors (dominant category → block color):
- `Mon W31 [▓▓░]` → Sleep `Indexed(19)` deep blue (dominant), empty tail dim gray.
- `Tue W31 [▓▓▓]` → Work `Indexed(240)` metallic dark gray (fully filled work day).
- `Wed W31 [░░░]` → dim gray (day has data but light fill).
- `Sun ●[▓▓░]` → the `●` marks today ("now"); selected cell also gets a reversed/bold highlight on top of its category color.

### Day view (24 hour cells)

One cell per hour. Filled hours show category color + label; empty hours are dim. Selected hour highlighted; the current hour is marked with `●`. Hours with a note show a `*` marker.

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

Hour-label colors (each label renders in its category color):
- `Sleep` → `Indexed(19)` deep blue · `Health` → `Cyan` · `Travel` → `Indexed(244)` gray.
- `Work` → `Indexed(240)` metallic dark gray · `Relaxation` → `Indexed(93)` purple · `Hobbies/Skills` → `Indexed(208)` orange.
- Selected hour (`13 Work *◀`) keeps its category color but adds a reversed/bold highlight + `◀` marker; `*` means it has a note.
- `●14 Work` → the `●` marks the current hour ("now").
- Empty hours render dim gray.

When the terminal does not support the requested color depth, category colors
degrade to the closest available terminal color. Selection, now markers, and
text labels must remain distinguishable through attributes such as reverse,
bold, or underline even in a monochrome terminal.

### Category picker (overlay)

Popup list over the day view; each row is shown in its own category color. Number keys or arrows select.

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

Row colors (each entry uses its own category color, matching the palette above):
- `0 Sleep` → `Indexed(19)` deep blue · `1 Health` → `Cyan` · `2 Friends/Family` → `Green` · `3 Romantic` → `Indexed(211)` pink · `4 Work` → `Indexed(240)` metallic dark gray.
- `5 Waste` → `Red` · `6 Travel` → `Indexed(244)` gray · `7 Hobbies/Skills` → `Indexed(208)` orange · `8 Relaxation` → `Indexed(93)` purple · `9 Other` → `Yellow`.
- The highlighted row (`> 0 Sleep`) keeps its category color with a reversed/bold cursor.
- Number keys map directly to the enum discriminant (0–9), so the picker order and shortcuts never drift.

### Note editor (overlay)

A popup text box for adding/editing a note, opened for either a whole day (from the calendar) or a single hour (from the day view). The title reflects the target. Enter saves, Esc cancels; existing text is pre-filled for editing.

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

- Day-level note uses the same editor with a title like `Note — Sun, Aug 2 2026`.
- A saved note is flagged in the parent view with a `*` marker (see day/calendar mockups).

## Keybindings

| Screen   | Key          | Action               |
|----------|--------------|----------------------|
| Calendar | arrows       | move selection       |
| Calendar | Enter        | open day             |
| Calendar | N            | edit day note        |
| Calendar | v / Tab      | switch view          |
| Calendar | q            | quit                 |
| Day      | ↑/↓          | move hour            |
| Day      | Enter        | open category picker  |
| Day      | x            | clear hour           |
| Day      | n            | edit hour note       |
| Day      | v / Tab      | switch view          |
| Day      | Esc          | back to calendar     |
| Picker   | 0–9 / arrows | choose category      |
| Picker   | Enter / Esc  | confirm / cancel     |
| Note     | text / ⌫     | edit note text       |
| Note     | Enter / Esc  | save / cancel        |

## Main loop

`main.rs`: enter raw mode + alt-screen → `loop { draw(&app); handle event → update app }` → restore terminal on exit, including on panic via a guard so the terminal is never left in raw mode. The draw step re-reads the system clock each frame so the "now" indicator stays live.
