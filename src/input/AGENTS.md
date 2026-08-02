# input

Reads the keyboard (and a periodic tick) and converts each raw event into a
domain `Action`, which it hands to the controller. Stateless — it keeps no
selection, view, or overlay state of its own. It owns the **single** key→`Action`
table so every view shares one binding source.

Public verb: `input::next_action(timeout: Duration) -> anyhow::Result<Option<Action>>`.
It blocks up to `timeout`; on timeout it emits `Action::Tick` so the clock/"now"
indicator stays live even without keypresses. It never touches `State`.

`Action` is expressed purely in domain terms — no `crossterm::KeyEvent` leaks
past this module. The controller interprets each `Action` per the active
`ViewMode` + `Overlay`; the same physical key can therefore map to one `Action`
here and mean different things depending on context (e.g. arrows → `MoveUp` in
both Day view and the picker).

## Key → Action table

The **Screen** column is the context in which the key is pressed. `input` emits
the same `Action` regardless of context where the table repeats it; the
controller resolves meaning.

| Screen   | Key            | `Action`                          |
|----------|----------------|-----------------------------------|
| Calendar | ← / →          | `MoveLeft` / `MoveRight`          |
| Calendar | ↑ / ↓          | `MoveUp` / `MoveDown`             |
| Calendar | Enter          | `OpenDay`                         |
| Calendar | N              | `OpenNote` (day-level note)       |
| Calendar | v / Tab        | `CycleView`                       |
| Calendar | q              | `Quit`                            |
| Day      | ↑ / ↓          | `MoveUp` / `MoveDown`             |
| Day      | Enter          | `OpenPicker`                      |
| Day      | x              | `ClearHour`                       |
| Day      | n              | `OpenNote` (hour-level note)      |
| Day      | v / Tab        | `CycleView`                       |
| Day      | Esc            | `Back`                            |
| Picker   | 0–9            | `SetCategory(c)` (digit → discriminant) |
| Picker   | ↑ / ↓          | `MoveUp` / `MoveDown`             |
| Picker   | Enter          | `Confirm`                         |
| Picker   | Esc            | `Cancel`                          |
| Note     | printable char | `NoteInput(c)`                    |
| Note     | Backspace (⌫)  | `NoteBackspace`                   |
| Note     | Enter          | `NoteSave`                        |
| Note     | Esc            | `Cancel`                          |
| (any)    | tick timeout   | `Tick`                            |

Notes:
- Picker digits `0–9` map directly to the `Category` discriminant, so shortcuts
  never drift from the palette order.
- `Esc` is `Back` on a base view (Day → Calendar) but `Cancel` inside an overlay;
  `input` emits the literal key mapping shown, and the controller applies the
  transition. Where both meanings are needed, prefer emitting `Back` on base
  views and `Cancel` in overlays as the table specifies.
- Anything not in the table is ignored (no `Action` produced).
