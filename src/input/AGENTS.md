# input

Reads the keyboard (and a periodic tick) and turns each raw event into a domain
`Action` for the controller. Stateless — it keeps no selection, view, or overlay
state, and it does not know what any key "does". It only knows the physical
event and its intermediate representation.

Public verb: `input::next_action(timeout: Duration) -> anyhow::Result<Option<Action>>`.
It blocks up to `timeout`; on timeout it emits the `Tick` IR so the clock/"now"
indicator stays live even without keypresses. It never touches `State`. No
`crossterm::KeyEvent` leaks past this module.

## Two stages: key → IR → Action

Because the *meaning* of a key changes with state (Enter opens a day in Calendar
but confirms a picker in an overlay), `input` must not categorize keys by task.
Instead it maps each physical key to a small, stable **intermediate
representation (IR)** — a name for the *keystroke*, not its effect. The IR is
then trivially mapped to an `Action` variant. The controller resolves what that
`Action` means for the current `ViewMode` + `Overlay`.

```
physical key ──▶ InputIR ──▶ Action ──▶ (controller decides meaning by state)
```

The IR is intentionally effect-free: `Enter` is `Confirm` (an IR), never
"OpenDay". This keeps `input` stateless and keeps one physical key from needing
N task-specific branches.

### Key → IR

Each physical key maps to exactly one IR, regardless of state:

| Physical key      | `InputIR`      |
|-------------------|----------------|
| ←                 | `Left`         |
| →                 | `Right`        |
| ↑                 | `Up`           |
| ↓                 | `Down`         |
| Enter             | `Confirm`      |
| Esc               | `Cancel`       |
| Tab               | `Cycle`        |
| `0`–`9`           | `Digit(u8)`    |
| printable char    | `Char(char)`   |
| Backspace         | `Erase`        |
| (tick timeout)    | `Tick`         |

Notes:
- Letters that are also commands (`v`, `n`, `N`, `x`, `q`) arrive as
  `Char(c)` — `input` does not special-case them; the controller decides whether
  a `Char('q')` means quit (base view) or literal text (note editor). This is the
  whole point of the IR: no task categorization here.
- Anything with no mapping produces no IR (and thus no `Action`).

### IR → Action

The IR maps 1:1 to an `Action` variant. This layer is still effect-free naming;
the controller interprets the `Action` per state.

| `InputIR`     | `Action`             |
|---------------|----------------------|
| `Left`        | `MoveLeft`           |
| `Right`       | `MoveRight`          |
| `Up`          | `MoveUp`             |
| `Down`        | `MoveDown`           |
| `Confirm`     | `Confirm`            |
| `Cancel`      | `Cancel`             |
| `Cycle`       | `CycleView`          |
| `Digit(n)`    | `Digit(n)`           |
| `Char(c)`     | `Char(c)`            |
| `Erase`       | `Erase`              |
| `Tick`        | `Tick`               |

The controller then resolves context into an effect, e.g.:
- `Confirm` → open the selected day in Calendar, open the category picker on a
  Day hour, or commit inside an overlay.
- `Cancel` → step back one level on a base view (Day → Calendar), or discard
  inside an overlay.
- `Digit(n)` → pick category `n` in the picker, ignored elsewhere.
- `Char('n')`/`Char('x')`/`Char('q')` → note / clear / quit on a base view;
  literal text inside the note editor.

This means the `Action` enum carries neutral IR-style variants (`Confirm`,
`Cancel`, `Digit`, `Char`, `Erase`, moves, `CycleView`, `Tick`) rather than
pre-decided effects; see `domain/action.rs` and `controller/AGENTS.md` for how
each is interpreted.

Because `input` holds no key→command table, the letter-command bindings (`n`/`N`
note, `x` clear, `q` quit, `v` view) are **not** documented here — they are just
`Char(c)` at this layer. Their meaning lives with the code that owns it: the
state-transition rules in `controller/AGENTS.md`.
