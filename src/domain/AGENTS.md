# domain

Shared, dependency-free vocabulary every other module speaks. `domain` imports
nothing from `input`, `controller`, `view`, or `storage`; they all import it.
It carries **no** `ratatui`, `crossterm`, or `rusqlite` types, so it is pure and
unit-testable without a terminal or DB.

This file defines *what each type means and the rules on it*, not full field-by-field
Rust definitions. Keep the concrete `struct`/`enum` bodies in the `.rs` files.

## Data types

- **`Category`** (`category.rs`) — the fixed activity palette. Ten variants with
  explicit discriminants `0..9` that match the picker's number keys, so picker
  order and shortcuts never drift. Noun-phrased. Colors are *not* defined here
  (that is `view/theme.rs`); only the identity/order of categories.

- **`Activity`** (`activity.rs`) — what fills a single hour: one `Category` plus
  an optional per-hour note.

- **`Day`** (`calendar.rs`) — one calendar date: its `NaiveDate`, its 24 hour
  slots (each an optional `Activity`, index = hour `0..23`), and an optional
  day-level note. Also home to `Week`, hour-slot indexing, week-window math, and
  the `dominant_category` helper (most-filled category; ties resolved by the
  lower discriminant).

- **`State`** (`state.rs`) — the whole live model owned by the controller: active
  `ViewMode`, the shared `Cursor`, an optional `Overlay`, the sparse
  `BTreeMap<NaiveDate, Day>` of days with data, a `last_error`, and a `quit` flag.
  See `controller/AGENTS.md` for ownership and mutation rules.

- **`Cursor`** (`state.rs`) — the shared selection: a `date` plus an optional
  `hour` (`Some` in Day view, `None` in Calendar). Single source of selection;
  views never keep private copies.

- **`ViewMode`** (`state.rs`) — the base view (`Calendar`, `Day`, and future
  `Week`/`Agenda`/`Stats`). Always set.

- **`Overlay`** (`state.rs`) — optional modal state on top of the base view:
  `CategoryPicker` (date, hour, selected category) or `NoteEditor` (target,
  draft text, text cursor). `None` means the base view has focus.

- **`NoteTarget`** (`state.rs`) — what a note edit applies to: a whole `Day` or a
  single `Hour` (date + hour).

- **`Action`** (`action.rs`) — the pivot contract from `input` to `controller`.
  Verb-phrased commands expressed purely in domain terms (e.g.
  `SetCategory(Category)`), never raw key events. See `input/AGENTS.md` for the
  full key→`Action` table.

## Invariants

- Sparse map: only days with activities or a note are materialized. A day with
  only a day-level note still counts as data. Empty days are never stored.
- "Now" is never a domain value — it is read from the system clock at render
  time, not persisted.
- Naming: types `PascalCase`, `Action` variants verb-phrased, data types
  noun-phrased, fields/methods `snake_case`, constants `SCREAMING_SNAKE_CASE`
  (`HOURS_PER_DAY`, `WINDOW_WEEKS`).
