# domain

The **protocol layer**: only the types that define contracts *between* modules
live here. If a type is internal to a single module, it belongs in that module,
not here (e.g. `State`, `Cursor`, `ViewMode`, `Overlay`, `NoteTarget` live in
`controller`; category colors live in `view`).

`domain` imports nothing from `input`, `controller`, `view`, or `storage`; they
all import it. It carries **no** `ratatui`, `crossterm`, or `rusqlite` types, so
it is pure and unit-testable without a terminal or DB.

This file defines *what each type means and the rules on it*, not full
field-by-field Rust definitions. Keep the concrete `struct`/`enum` bodies in the
`.rs` files.

## Why these types are here (and others are not)

A type earns a place in `domain` only if more than one module must agree on it —
it decides a protocol:

- `Action` is the wire between `input` and `controller`.
- `Category`, `Activity`, `Day` are the wire between `controller` and `storage`
  (persisted shape) and are read by `view` (rendered shape in the matrix/popup UI).

Types that only one module owns are documented with that module:

- `State`, `Cursor`, `ViewMode`, `Overlay`, `NoteTarget` → owned/mutated solely
  by `controller`; `view` reads them through `&State`. See
  `controller/AGENTS.md`.
- Category → color mapping, styles → `view/theme.rs`. See `view/AGENTS.md`.

## Protocol types

- **`Action`** (`action.rs`) — the contract from `input` to `controller`.
  Neutral, IR-style variants that name the *keystroke*, not its effect
  (`MoveLeft/Right/Up/Down`, `Confirm`, `InsertNewline`, `Cancel`, `CycleView`, `Digit(u8)`,
  `Char(char)`, `Erase`, `Tick`), never raw key events. `input` produces it; the
  controller resolves each into an effect by state. See `input/AGENTS.md` for the
  key → `InputIR` → `Action` mapping and `controller/AGENTS.md` for interpretation.

- **`Category`** (`category.rs`) — the fixed activity palette shared by input
  selection, storage rows, and rendering. Ten variants with explicit
  discriminants `0..9`. Only the identity and order of categories live here; the
  color for each is a `view` concern, not a protocol concern.

- **`Activity`** (`activity.rs`) — what fills a single hour: one `Category` plus
  an optional per-hour note. This is the persisted/rendered unit crossing
  controller ↔ storage ↔ view.

- **`Day`** (`calendar.rs`) — one calendar date: its `NaiveDate`, its 24 hour
  slots (each an optional `Activity`, index = hour `0..23`), and an optional
  day-level note. The load/save unit between controller and storage. Also home
  to `Week`, hour-slot indexing, week-window math, and the `dominant_category`
  helper (most-filled category; ties resolved by the lower discriminant).

## Invariants

- Sparse map: only days with activities or a note are materialized. A day with
  only a day-level note still counts as data. Empty days are never stored.
- "Now" is never a domain value — it is read from the system clock at render
  time, not persisted.
- Naming: types `PascalCase`, `Action` variants name the keystroke (`Confirm`,
  `Digit`, `Char`) not the effect, data types noun-phrased, fields/methods
  `snake_case`, constants `SCREAMING_SNAKE_CASE` (`HOURS_PER_DAY`, `WINDOW_WEEKS`).
