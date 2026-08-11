# storage

Owns all persistence. Talks **only** to the controller and the disk — never to
`input` or `view`. Loads persisted data from disk into domain types on startup,
and writes domain mutations back to disk per action. Isolated behind a small
interface (`Store`) so the SQLite backend can change without touching any other
module.

Backend: SQLite via `rusqlite` (with the `bundled` feature — no system SQLite
required). Single DB file `life-tracker.db` in the `directories::ProjectDirs`
data dir, currently resolved from `ProjectDirs::from("dev", "xsiff",
"life-tracker")`. `directories` and `rusqlite` are used here and nowhere else.

## Public interface

```rust
pub struct Store { /* conn */ }
impl Store {
    pub fn open() -> anyhow::Result<Self>;                 // resolve path, open, create tables
    pub fn load_all(&self) -> anyhow::Result<BTreeMap<NaiveDate, Day>>;
    pub fn set_hour(&self, date: NaiveDate, hour: u8, act: &Activity) -> anyhow::Result<()>;
    pub fn clear_hour(&self, date: NaiveDate, hour: u8) -> anyhow::Result<()>;
    pub fn set_day_note(&self, date: NaiveDate, note: &str) -> anyhow::Result<()>;
    pub fn clear_day_note(&self, date: NaiveDate) -> anyhow::Result<()>;
}
```

## Input / output

- **Input from disk → controller:** `load_all` reads every stored row and builds
  the sparse `BTreeMap<NaiveDate, Day>` the controller converts into initial
  `State`.
- **Input from controller → disk:** the `set_*`/`clear_*` verbs each write a
  single row. The controller calls these under its **persist-then-commit** rule
  (storage write first; in-memory mutation only on `Ok`). Storage just performs
  the write and returns `Result`; it does not know about `State`.
- Writes are per-mutation (one row), so there is no bulk save/flush step.

## Schema

```sql
CREATE TABLE IF NOT EXISTS activities (
    date     TEXT    NOT NULL,   -- ISO date, e.g. 2026-08-02
    hour     INTEGER NOT NULL,   -- 0..23
    category TEXT    NOT NULL,   -- empty string means "note-only hour"
    note     TEXT,               -- optional per-hour note
    PRIMARY KEY (date, hour)
);

CREATE TABLE IF NOT EXISTS day_notes (
    date     TEXT    NOT NULL,   -- ISO date
    note     TEXT    NOT NULL,   -- day-level note
    PRIMARY KEY (date)
);
```

- **`set_hour`** → `INSERT OR REPLACE` the `(date, hour)` row (optional category
  encoded as category text or `""`, plus optional note).
- **`clear_hour`** → `DELETE` the `(date, hour)` row.
- **`set_day_note`** → `INSERT OR REPLACE` the `day_notes(date)` row.
- **`clear_day_note`** → `DELETE` the `day_notes(date)` row.

## Validation on load

Loaded rows are validated, not silently coerced:
- dates must parse as ISO dates,
- `hour` must be in `0..=23`,
- category names must be known.

Invalid rows are reported and skipped rather than converted. A day present only
in `day_notes` still yields a materialized `Day` (note, no activities); fully
empty days are never produced.
