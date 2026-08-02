# life-tracker

## What we're building

Terminal (TUI) life-tracker in Rust (`ratatui` + `crossterm`). The primary UI is
one month-grouped matrix:

1. **Matrix view (main screen)** — columns are hours `00.00..23.00`; rows are
   sequential dates. Rows are grouped under month headers (`August 2026`,
   `September 2026`, ...). Each cell is one `(date, hour)` slot, and the table
   uses visible grid separators so dates and hours read as a real spreadsheet.
   Ordinary date rows are separated by horizontal `─` rules; month boundaries use
   stronger `═` separators. The visible viewport follows the focused cell both
   vertically and horizontally when the terminal cannot show every date/hour at
   once.
2. **Popup editing** — pressing enter on a focused cell opens a popup for that
   slot. The popup lets the user choose a category by number/color, with a final
   `[+] add note` action that routes into the note editor for the same hour.
   Notes are edited in a text popup and noted cells show `*` in the matrix. A
   boxed palette pane explains the number-to-category mapping.

**Domain model:**
- **Category** — a fixed set of 10 activity types (Sleep, Health, Friends/Family, Romantic, Work, Waste, Travel, Hobbies/Skills, Relaxation, Other). Categories are the palette for filling hours.
- Each hour cell holds one categorized activity (a category plus optional specifics).

Keep this single matrix + popup editing structure and the category-driven fill
model in mind; it's the core of the app and not obvious from filenames once code
lands.

## Architecture (unidirectional data flow)

The program is a strict one-way loop (MVU / Elm-style). Data flows in a single
direction and there is exactly one owner of live state (the controller). Every
module has one job and one public verb.

```
 keyboard ──▶ input ──▶ Action ──▶ controller ──▶ State ──▶ view ──▶ terminal (ASCII)
                                      ▲   │
                                      │   ▼
                                    storage ◀──▶ disk
```

- **`input`** — reads the keyboard (and a periodic tick), converts raw key/tick
  events into a domain `Action`, and hands it to the controller. Stateless.
  See [`src/input/AGENTS.md`](src/input/AGENTS.md).
- **`controller`** — the brain. Owns the live `State`. On startup it asks
  `storage` to load persisted data and converts it into the initial `State`.
  On each `Action` it decides the next `State`, persisting through `storage`
  when the action mutates stored data. Stateful; the single source of truth.
  See [`src/controller/AGENTS.md`](src/controller/AGENTS.md).
- **`storage`** — loads persisted data from disk into domain types, and writes
  domain mutations back to disk. Talks only to the controller and the disk;
  never to `input` or `view`. See [`src/storage/AGENTS.md`](src/storage/AGENTS.md).
- **`view`** — reads the `State` from the controller and renders the UI to the
  terminal in ASCII. Stateless; never mutates `State`.
  See [`src/view/AGENTS.md`](src/view/AGENTS.md) and
  [`src/view/EXAMPLES.md`](src/view/EXAMPLES.md).
- **`domain`** — the protocol layer: only the cross-module contract types
  (`Action`, `Category`, `Activity`, `Day`) plus pure logic on them. Types owned
  by a single module (`State`, `Cursor`, `ViewMode`, `Overlay`, `NoteTarget` in
  `controller`; colors in `view`) live with that module, not here. Every module
  imports `domain`; `domain` imports nothing from them.
  See [`src/domain/AGENTS.md`](src/domain/AGENTS.md).

**Invariants (keep these true or the design leaks):**
- One-way only: `view` never mutates `State`; `storage` never touches `view`;
  all coordination passes through the `controller`.
- `controller` is the only owner/mutator of `State` and the only holder of the
  `Store` handle.
- `domain` stays free of `ratatui`, `crossterm`, and `rusqlite` types. `Action`
  carries neutral IR-style variants (`Confirm`, `Digit(u8)`, `Char(char)`, …),
  not `KeyEvent`; the controller resolves each into an effect by state.
- **Persist-then-commit:** for any action that mutates stored data, the
  controller attempts the `storage` write first and only updates in-memory
  `State` if the write succeeds; on error it keeps the old state and surfaces a
  readable error into `State`.

## Per-module docs

Each folder carries its own `AGENTS.md` with the details for that module; this
root file is the overview and the glue between them.

| Module | Doc | Covers |
|--------|-----|--------|
| `domain` | [`src/domain/AGENTS.md`](src/domain/AGENTS.md) | meaning + rules of the cross-module protocol types (`Action`, `Category`, `Activity`, `Day`) |
| `input` | [`src/input/AGENTS.md`](src/input/AGENTS.md) | the two-stage key → `InputIR` → `Action` mapping |
| `controller` | [`src/controller/AGENTS.md`](src/controller/AGENTS.md) | update flow, the `State` it owns, and all state-transition rules |
| `storage` | [`src/storage/AGENTS.md`](src/storage/AGENTS.md) | SQLite schema, the `Store` interface, load/validate, and I/O with the controller |
| `view` | [`src/view/AGENTS.md`](src/view/AGENTS.md) · [`src/view/EXAMPLES.md`](src/view/EXAMPLES.md) | render flow + shared concerns; worked `State` → output examples |

## Dependencies

- `ratatui` — TUI rendering (widgets, layout). `view` only.
- `crossterm` — terminal backend (raw mode, events, alt-screen). `input` + `main`.
- `rusqlite` (with the `bundled` feature) — SQLite persistence; no system SQLite required. `storage` only.
- `chrono` — date/time math (weeks, days, ISO week numbers); also drives the "now" indicator.
- `directories` — resolve the per-OS data dir for the DB file. `storage` only.
- `anyhow` — error handling.

## File & folder layout

```
life-tracker/
├── Cargo.toml
├── AGENTS.md                # this overview
├── src/
│   ├── main.rs              # app entry: terminal setup/teardown + main loop
│   │                        #   input::next_action → controller.update → view::render
│   ├── domain/              # protocol types shared across modules + pure logic
│   │   ├── AGENTS.md
│   │   ├── mod.rs
│   │   ├── category.rs      # Category (fixed palette, 0..9)
│   │   ├── activity.rs      # Activity = category + optional note
│   │   ├── calendar.rs      # Day, Week, hour-slot model + indexing, week-window math
│   │   └── action.rs        # Action (neutral IR-style input → controller contract)
│   ├── input/               # keyboard/tick → Action
│   │   ├── AGENTS.md
│   │   └── mod.rs
│   ├── controller/          # owns State; update(&mut self, Action); holds Store
│   │   ├── AGENTS.md
│   │   ├── mod.rs
│   │   └── state.rs         # State, Cursor, ViewMode, Overlay, NoteTarget, CategoryPickerSelection
│   ├── view/                # State → ASCII (ratatui); never mutates State
│   │   ├── AGENTS.md
│   │   ├── EXAMPLES.md
│   │   ├── mod.rs           # render() entry + preview scene helpers
│   │   ├── calendar_view.rs # month-grouped day x hour matrix + palette pane
│   │   ├── category_picker.rs # popup/list to choose a category or the add-note action
│   │   ├── note_editor.rs   # popup text editor for day/hour notes
│   │   ├── status_bar.rs    # "now" indicator + current focused slot line
│   │   └── theme.rs         # category colors, styles
│   └── storage/
│       ├── AGENTS.md
│       ├── mod.rs
│       └── sqlite_store.rs  # open DB, load/save (date, hour) rows + day notes
```

## Module interfaces (the cross-module contracts)

Each module exposes one public verb so the loop reads like a sentence:
`next_action → update → render`. Details live in each module's `AGENTS.md`.

```rust
// input — raw keyboard/tick events → domain Action
pub fn next_action(timeout: Duration) -> anyhow::Result<Option<Action>>;

// controller — owns State; the only mutator
pub struct Controller { /* state: State, store: Store */ }
impl Controller {
    pub fn new(store: Store) -> anyhow::Result<Self>; // load_all → initial State
    pub fn update(&mut self, action: Action) -> anyhow::Result<()>; // persist-then-commit
    pub fn state(&self) -> &State;                    // read-only for the view
    pub fn should_quit(&self) -> bool;
}

// view — State → ASCII; never mutates
pub fn render(frame: &mut ratatui::Frame, state: &State);

// storage — disk ↔ domain; talks only to the controller
pub struct Store { /* conn */ }
impl Store {
    pub fn open() -> anyhow::Result<Self>;
    pub fn load_all(&self) -> anyhow::Result<BTreeMap<NaiveDate, Day>>;
    pub fn set_hour(&self, date: NaiveDate, hour: u8, act: &Activity) -> anyhow::Result<()>;
    pub fn clear_hour(&self, date: NaiveDate, hour: u8) -> anyhow::Result<()>;
    pub fn set_day_note(&self, date: NaiveDate, note: &str) -> anyhow::Result<()>;
    pub fn clear_day_note(&self, date: NaiveDate) -> anyhow::Result<()>;
}
```

The **`Action`** enum is the pivot contract between `input` and `controller`. Its
variants are neutral, IR-style names for keystrokes (`MoveLeft`, `MoveRight`,
`MoveUp`, `MoveDown`, `Confirm`, `Cancel`, `CycleView`, `Digit(u8)`,
`Char(char)`, `Erase`, `Tick`), **not** pre-decided effects — the controller
resolves each into an effect (move the focused slot, open the category popup,
save note, quit, …) using the current base state + `Overlay`. It is defined in
`domain/action.rs`; the full key → `InputIR` → `Action` mapping lives in
[`src/input/AGENTS.md`](src/input/AGENTS.md).

## Naming conventions

- **Modules / files:** `snake_case` — `calendar_view.rs`, `sqlite_store.rs`.
- **Types (structs/enums/traits):** `PascalCase` — `State`, `Action`, `Category`, `Store`.
- **Enum variants:** `PascalCase`; **`Action` variants name the keystroke, not
  the effect** (`Confirm`, `Cancel`, `Digit`, `Char`), **noun-phrased for data**
  (`Day`, `Activity`).
- **Functions / methods / fields:** `snake_case` — `update`, `next_action`,
  `render`, `load_all`, `set_hour`, `dominant_category`.
- **Constants:** `SCREAMING_SNAKE_CASE` — `HOURS_PER_DAY`, `WINDOW_WEEKS`.
- **One public verb per module** — `input::next_action`, `controller::update`,
  `view::render`, `storage::{load_all, set_*, clear_*}`. A second public verb is
  a signal the module may be doing two jobs.

## Main loop

`main.rs`: enter raw mode + alt-screen → build `Store` and `Controller` →
`loop { view::render(&mut frame, controller.state()); if let Some(a) = input::next_action(tick)? { controller.update(a)? } }`
until `controller.should_quit()` → restore terminal on exit, including on panic
via a guard so the terminal is never left in raw mode. The draw step re-reads the
system clock each frame so the "now" indicator stays live; the `Tick` action
bounds how long `next_action` blocks so the clock refreshes even without input.
