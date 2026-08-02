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

`State` holds: the base `ViewMode`; the shared `Cursor` (date + optional hour);
an optional `Overlay` (`CategoryPicker` or `NoteEditor`); the sparse
`BTreeMap<NaiveDate, Day>`; `last_error`; and `quit`. Field-level meaning lives
in `domain/AGENTS.md`; the controller is just its sole owner/mutator.

- **Sparse map:** materialize a `Day` only when it gains an activity or a note;
  drop it when it becomes empty again. A day with only a day-level note still
  counts as data.
- **"Now" is never stored** — the view reads the clock at render time. `Tick`
  only signals that a redraw is due; the controller stores no timestamp.

## State-transition rules

- `Calendar --OpenDay--> Day --OpenPicker--> CategoryPicker --Confirm--> Day`.
  `Back`/`Cancel` (Esc) unwinds one level: overlay → base view, or Day → Calendar.
- `OpenNote` opens `NoteEditor` for the current target (day note from Calendar,
  hour note from Day). `NoteSave` commits the draft; `Cancel` discards it. Saving
  an empty note clears the corresponding note.
- Calendar navigation shows a fixed five-week window centered on the selected
  week. `MoveLeft`/`MoveRight` shift the window (no hard date boundary);
  `MoveUp`/`MoveDown` change weekday within the selected week.
- `CycleView` to `Day` keeps the date and selects the current local hour when no
  hour is set; back to `Calendar` clears the hour. `OpenDay` selects the current
  local hour only when the date is today, else hour `0`.
- `SetCategory` on a filled hour replaces the category and preserves its note.
  `ClearHour` removes both the activity and its note.
- `Quit` sets `quit = true`; `should_quit()` reports it to the main loop.

## Boundaries

- Never renders and never reads the keyboard — it only consumes `Action` and
  exposes `&State`.
- Holds the only `Store`; validation of loaded rows is storage's job, but the
  controller decides how a surfaced error appears (via `last_error`).
