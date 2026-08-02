# life-tracker

A terminal (TUI) life-tracker written in Rust. Log how you spend each hour of your day using a fixed palette of activity categories, and see your weeks at a glance in a calendar grid.

## Features

- **Calendar view** — a grid of weeks (columns) by days of the week (rows). Each cell summarizes how full that day is.
- **Day view** — drill into any day to see all 24 hours, one cell per hour.
- **Category-driven logging** — fill each hour with an activity drawn from a fixed set of 10 categories (Sleep, Health, Friends/Family, Romantic, Work, Waste, Travel, Hobbies/Skills, Relaxation, Other), with an optional note.
- **Color-coded at a glance** — every category has its own fixed color, used consistently across the calendar cells, day view, and category picker so you can read your week without labels.
- **Live "now" indicator** — a status bar shows the current date and hour and highlights the matching cell, alongside your current focus (the selected day/hour and its activity).
- **Notes on days and hours** — attach a free-text note to any hour or to a whole day via a popup editor; noted cells are flagged with a `*` marker.
- **Switchable views** — cycle between views (calendar and day views, with room for week/agenda/stats later) with a single key.
- **Local persistence** — everything is stored locally in a single SQLite database; no account or network required.

## Installation

Requires a Rust toolchain (edition 2021 or later). No system SQLite is needed — it is bundled.

```bash
git clone git@xsiff:Xsiff/life-tracker.git
cd life-tracker
cargo build --release
```

## Usage

```bash
cargo run --release
```

The app opens on the calendar view. Navigate to a day, open it, and fill in your hours.

### Keybindings

| Screen   | Key          | Action               |
|----------|--------------|----------------------|
| Calendar | arrows       | move selection       |
| Calendar | Enter        | open day             |
| Calendar | N            | edit day note        |
| Calendar | v / Tab      | switch view          |
| Calendar | q            | quit                 |
| Day      | ↑/↓          | move hour            |
| Day      | Enter        | open category picker |
| Day      | x            | clear hour           |
| Day      | n            | edit hour note       |
| Day      | v / Tab      | switch view          |
| Day      | Esc          | back to calendar     |
| Picker   | 0–9 / arrows | choose category      |
| Picker   | Enter / Esc  | confirm / cancel     |
| Note     | text / ⌫     | edit note text       |
| Note     | Enter / Esc  | save / cancel        |

## Data storage

Activities are saved to a single SQLite file (`life-tracker.db`) in your platform's data directory (resolved via [`directories`](https://crates.io/crates/directories)). Each filled hour is one row, day notes are stored separately, and changes are written immediately as you edit.

## Development

See [`AGENTS.md`](AGENTS.md) for the full system design: module layout, domain model, persistence schema, screen state machine, view modes, status bar, and UI mockups.

```bash
cargo build
cargo test
cargo run
cargo clippy
```

## License

See [`LICENSE`](LICENSE).
