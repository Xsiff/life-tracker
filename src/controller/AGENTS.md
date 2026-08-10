# controller

The brain of the program and the **single source of truth**. It owns the live
`State` and the only `Store` handle. Nothing else mutates `State`; nothing else
talks to storage.

Public verbs:

```rust
pub struct Controller { /* state: State, store: Store */ }
impl Controller {
    pub fn new(store: Store) -> anyhow::Result<Self>; // load_all → initial State
    pub fn update(&mut self, action: Action) -> anyhow::Result<()>; // persist-then-commit
    pub fn state(&self) -> &State;   // read-only handle for the view
    pub fn should_quit(&self) -> bool;
}
```

## Startup

`new` asks `storage` for `load_all()` (a `BTreeMap<NaiveDate, Day>`) and converts
it into the initial `State`: `ViewMode::Calendar`, a `Cursor` on today and the
current local hour, no overlay, no error, `quit = false`.

## Update flow

`update(action)` is a nearly-pure `(State, Action) -> State` transition:

1. **Routing.** If `state.overlay.is_some()`, the overlay interprets the action;
   otherwise the base matrix screen interprets it. The same `Action` means
   different things per context.
2. **Persist-then-commit.** For any action that mutates stored data, call the
   matching `Store` method **first**. Only mutate in-memory `State` if it returns
   `Ok`. On `Err`, keep the previous state and set `state.last_error` (surfaced
   in the status bar). This keeps disk and memory from drifting.
3. Pure navigation/overlay actions (moves, view cycling, opening/closing popups)
   touch only in-memory `State` — no storage call.

## The State it owns

`State` and everything nested in it are **controller-owned** types — they are
not part of any cross-module protocol, so they are defined here, not in `domain`.
`view` only reads them through `&State`; nothing else touches them.

- **`State`** — the whole live model: the base `ViewMode`; the shared `Cursor`;
  an optional `Overlay`; the sparse `BTreeMap<NaiveDate, Day>` of days with data
  (`Day` itself is a `domain` protocol type — the controller/storage/view wire);
  a `last_error`; and a `quit` flag.
- **`Cursor`** — the shared selection: a `date` plus an optional `hour`.
  `hour = Some(h)` means an hour cell is focused; `hour = None` means the focus
  is on the date column for that row and actions apply to the whole day. Single
  source of selection; views never keep private copies that can drift.
- **`ViewMode`** — the base presentation mode. The active frontend currently
  uses only `Calendar`, which is the matrix screen. Keep it as a single source
  of base-screen identity even while only one mode is live.
- **`Overlay`** — optional modal state on top of the base view: `CategoryPicker`
  (a `NoteTarget` plus the selected picker row), `Help` (read-only category
  descriptions), or `NoteEditor` (target, draft text, text cursor). `None`
  means the base view has focus.
- **`CategoryPickerSelection`** — the focused row inside `CategoryPicker`:
  either a concrete `Category` or one of the trailing actions:
  `AddNote`, `DeleteNote`, or `DeleteActivity`.
- **`NoteTarget`** — what a note edit applies to: a whole `Day` or a single
  `Hour` (date + hour).

Field-level bodies live in `state.rs`; this doc is the meaning + rules.

- **Sparse map:** materialize a `Day` only when it gains an activity or a note;
  drop it when it becomes empty again. A day with only a day-level note still
  counts as data.
- **"Now" is never stored** — the view reads the clock at render time. `Tick`
  only signals that a redraw is due; the controller stores no timestamp.
- **`NoteEditor` draft:** the draft text + text cursor live in the overlay; the
  draft is not written to storage until a save. `Cancel` discards it and leaves
  the target, selection, and stored value unchanged.
  Arrow keys move the note cursor inside the editor: left/right move by
  character, and up/down move between lines while preserving the column when
  possible. `Option/Alt+←` and `Option/Alt+→` move by word/chunk inside the
  note editor and are ignored outside it. `Option/Alt+Delete` or
  `Option/Alt+Backspace` deletes the previous word/chunk inside the note
  editor and is ignored outside it.
- **Routing rule:** if `overlay.is_some()`, the overlay interprets the action;
  otherwise the active `ViewMode` does. Do not add a `Screen` enum alongside
  `ViewMode` — that would duplicate "which view am I in" and let them drift.

## State-transition rules

Each rule reads as *(context) + Action → effect*. The `Action` variants are the
neutral IR-style names (`Confirm`, `InsertNewline`, `Cancel`, `Digit(n)`,
`Char(c)`, moves, `CycleView`, `Tick`); the controller resolves each into an
effect by the current base state + `Overlay`.

- **`Confirm`** — on the base matrix view, opens the `CategoryPicker` overlay
  for the focused target. If focus is on an hour cell, the picker offers
  categories plus `AddNote`, `DeleteNote`, and `DeleteActivity`; if focus is on
  the date column, the picker exposes only `AddNote` and `DeleteNote`. In
  `CategoryPicker`, confirming a category saves that hour activity; confirming
  `AddNote` opens the note editor for the same target; confirming a delete row
  clears only that specific thing and otherwise no-ops when the target is
  already empty. `DeleteActivity` clears only the category and preserves a note
  if one exists; `DeleteNote` clears only the note and preserves a category if
  one exists. In `NoteEditor`, `Confirm` saves the draft.
- **`Cancel`** — in an overlay, discards it and returns to the underlying view.
  On the base view it is typically ignored or mapped to quit/back by the shell.
- **`Char(c)`** — on a base view, `Char('?')` opens the read-only category help
  popup; `Char('q')` quits (sets `quit = true`; `should_quit()` reports it to
  the main loop). Inside the `NoteEditor`, `Char(c)` inserts literal text.
- **`InsertNewline`** — inside the `NoteEditor`, inserts a line break at the
  current text cursor. This is the `Shift+Enter` path when the terminal reports
  it distinctly. Ignored elsewhere.
- **`Erase`** — inside the `NoteEditor`, deletes the char before the text cursor;
  ignored elsewhere.
- **`MoveLeft` / `MoveRight` / `MoveUp` / `MoveDown`** — inside the
  `NoteEditor`, move the text cursor through the draft instead of moving the
  matrix focus. Left/right move by character, up/down move between lines with
  column clamping. In the base matrix they still move the focused cell.
- **`MoveWordLeft` / `MoveWordRight`** — inside the `NoteEditor`, move the text
  cursor by word/chunk boundaries. The controller ignores them outside note
  mode.
- **`DeleteWord`** — inside the `NoteEditor`, deletes the word/chunk to the left
  of the text cursor. The controller ignores it outside note mode.
- **`Digit(n)`** — in `CategoryPicker`, selects category `n` (digit →
  discriminant) when `n` maps to a real category; it never targets the
  `AddNote` row. Inside the `NoteEditor`, it inserts the corresponding literal
  digit so number keys work the same as other text input. Ignored elsewhere.
- **`Help` overlay** — read-only. `Confirm` or `Cancel` closes it; other
  actions leave it open.
- **Moves** — in the base matrix, `MoveLeft`/`MoveRight` move horizontally
  across the row. `MoveLeft` from hour `00` lands on the date column for the
  same row; `MoveLeft` from the date column wraps to hour `23` on the previous
  date. `MoveRight` from the date column enters hour `00`. `MoveRight` from
  hour `23` advances to the next date’s date column. `MoveUp`/`MoveDown` move
  across dates while keeping either the focused hour or the day-column focus.
  The visible matrix window follows the focused date/hour, so moving beyond the
  currently visible slice scrolls the viewport. In the picker, vertical moves
  change the selected row.
- **`CycleView`** — currently unused by the frontend. Keep it neutral in the
  protocol, but do not rely on a second base screen existing.
- **Note save semantics** — saving an empty note clears the corresponding note.
  For an hour target, this clears only the note and preserves the category; if
  the hour had no category either, the slot disappears entirely.
- **`Tick`** — no state change; only bounds the input wait so the view can redraw
  the live clock.

## Boundaries

- Never renders and never reads the keyboard — it only consumes `Action` and
  exposes `&State`.
- Holds the only `Store`; validation of loaded rows is storage's job, but the
  controller decides how a surfaced error appears (via `last_error`).
