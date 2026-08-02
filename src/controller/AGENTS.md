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
it into the initial `State`: `ViewMode::Calendar`, a `Cursor` on today with no
hour, no overlay, no error, `quit = false`.

## Update flow

`update(action)` is a nearly-pure `(State, Action) -> State` transition:

1. **Routing.** If `state.overlay.is_some()`, the overlay interprets the action;
   otherwise the active `ViewMode` interprets it. The same `Action` means
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
- **`Cursor`** — the shared selection: a `date` plus an optional `hour` (`Some`
  in Day view, `None` in Calendar). Single source of selection; views never keep
  private copies that can drift.
- **`ViewMode`** — the base view (`Calendar`, `Day`, and future
  `Week`/`Agenda`/`Stats`). Always set.
- **`Overlay`** — optional modal state on top of the base view: `CategoryPicker`
  (date, hour, selected category) or `NoteEditor` (target, draft text, text
  cursor). `None` means the base view has focus.
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
- **Routing rule:** if `overlay.is_some()`, the overlay interprets the action;
  otherwise the active `ViewMode` does. Do not add a `Screen` enum alongside
  `ViewMode` — that would duplicate "which view am I in" and let them drift.

## State-transition rules

Each rule reads as *(context) + Action → effect*. The `Action` variants are the
neutral IR-style names (`Confirm`, `Cancel`, `Digit(n)`, `Char(c)`, moves,
`CycleView`, `Tick`); the controller resolves each into an effect by the current
`ViewMode` + `Overlay`.

- **`Confirm`** — in Calendar, opens the selected day (`Calendar → Day`); on a
  Day hour, opens the `CategoryPicker` overlay; in `CategoryPicker`, commits the
  highlighted category and returns to Day; in `NoteEditor`, saves the draft.
  Chain: `Calendar --Confirm--> Day --Confirm--> CategoryPicker --Confirm--> Day`.
- **`Cancel`** — in an overlay, discards it and returns to the underlying view;
  on Day (no overlay), returns to Calendar. Unwinds exactly one level.
- **`Char(c)`** — on a base view, letters are commands: `Char('n')`/`Char('N')`
  opens the `NoteEditor` for the current target (hour note from Day, day note
  from Calendar); `Char('x')` on a Day hour clears it (removes both the activity
  and its note); `Char('q')` quits (sets `quit = true`; `should_quit()` reports
  it to the main loop). Inside the `NoteEditor`, `Char(c)` inserts literal text.
- **`Erase`** — inside the `NoteEditor`, deletes the char before the text cursor;
  ignored elsewhere.
- **`Digit(n)`** — in `CategoryPicker`, selects category `n` (digit → discriminant);
  ignored elsewhere.
- **Moves** — Calendar shows a fixed five-week window centered on the selected
  week: `MoveLeft`/`MoveRight` shift the window (no hard date boundary),
  `MoveUp`/`MoveDown` change weekday within the selected week. In Day and in the
  picker, `MoveUp`/`MoveDown` move the selection.
- **`CycleView`** — to `Day` keeps the date and selects the current local hour
  when no hour is set; back to `Calendar` clears the hour. (`Confirm`-opening a
  day selects the current local hour only when the date is today, else hour `0`.)
- **Note save semantics** — saving an empty note clears the corresponding note.
- **`Tick`** — no state change; only bounds the input wait so the view can redraw
  the live clock.

## Boundaries

- Never renders and never reads the keyboard — it only consumes `Action` and
  exposes `&State`.
- Holds the only `Store`; validation of loaded rows is storage's job, but the
  controller decides how a surfaced error appears (via `last_error`).
