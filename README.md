# life-tracker

Terminal life tracker in Rust. It stores each day as a row and each hour as a
column, so you can log categories, attach notes, and edit entries directly from
a spreadsheet-like TUI.

## What It Does

- Shows a month-grouped matrix where rows are dates and columns are hours
  `00..23`.
- Lets you focus either an hour cell or the date label for whole-day actions.
- Opens a popup on `Enter` to set a category, add a note, delete a note, or
  delete an activity.
- Supports hour-level notes and day-level notes. Notes are marked with `*`.
- Stores data locally in SQLite. No account and no network are required.

Each hour slot can hold a category, a note, or both. Deleting a category does
not delete the note, and deleting a note does not delete the category.

## Current UI Model

The app no longer uses a calendar screen plus a separate day screen. The active
UI is a single matrix view with popup editing:

- The main screen is a scrolling date x hour table with month separators.
- The left date column is focusable and represents day-level actions.
- The popup editor is context-sensitive:
  - On an hour cell, it offers categories `0..9`, `add note`, `delete note`,
    and `delete activity`.
  - On a date label, it offers `add note` and `delete note`.
- The note editor supports a visible text cursor, `Backspace`, save on `Enter`,
  cancel on `Esc`, and newline insertion via `Shift+Enter` when available or
  `Ctrl+J` as the reliable fallback.

## Categories

The fixed category palette is:

`0` Sleep, `1` Health, `2` Friends/Family, `3` Romantic, `4` Work,
`5` Waste, `6` Travel, `7` Hobbies/Skills, `8` Relaxation, `9` Other

## Installation

Private releases are distributed through GitHub Releases only. There is no
Homebrew tap, Cargo publish, or other public registry release.

### Install From A GitHub Release

The install script downloads a release archive, extracts the binary, and places
`life-tracker` into `~/.local/bin` by default.

```bash
git clone git@xsiff:Xsiff/life-tracker.git
cd life-tracker
GH_TOKEN=your_github_token ./scripts/install_release.sh
```

For private repositories, the installer requires either `GH_TOKEN` or
`GITHUB_TOKEN`. If `gh` is installed and already authenticated, that is also
accepted.

### Manual Install

1. Download the matching release archive from GitHub Releases.
2. Extract it.
3. Move `life-tracker` into a directory on your `PATH`, for example
   `~/.local/bin`.

```bash
tar -xzf life-tracker-v0.1.0-aarch64-apple-darwin.tar.gz
mkdir -p ~/.local/bin
mv life-tracker ~/.local/bin/life-tracker
```

### Build From Source

SQLite is bundled through `rusqlite`, so no system SQLite install is needed.

```bash
git clone git@xsiff:Xsiff/life-tracker.git
cd life-tracker
cargo build --release
cp target/release/life-tracker ~/.local/bin/life-tracker
```

## Running

After installation, launch the app from any terminal with:

```bash
life-tracker
```

## Keybindings

### Matrix

| Key | Action |
|---|---|
| `←` / `→` | Move across the focused row |
| `↑` / `↓` | Move across dates |
| `Enter` | Open the popup for the focused target |
| `n` / `N` | Open note editor for the focused target |
| `x` / `X` | Clear the focused value |
| `q` / `Q` | Quit |
| `Tab` | Reserved `CycleView` action; currently no visible mode switch |

`MoveLeft` from hour `00` lands on the date label. `MoveRight` from the date
label enters hour `00`. `MoveRight` from hour `23` advances to the next date at
hour `00`.

### Category Popup

| Key | Action |
|---|---|
| `↑` / `↓` | Move selection |
| `0`-`9` | Jump to a category row on hour targets |
| `Enter` | Confirm selected row |
| `Esc` | Cancel |

### Note Editor

| Key | Action |
|---|---|
| text input | Insert text |
| `Backspace` | Erase one character before the cursor |
| `Enter` | Save |
| `Esc` | Cancel |
| `Shift+Enter` | Insert newline when the terminal reports it distinctly |
| `Ctrl+J` | Insert newline reliably |

## Data Storage

Data is stored in a local SQLite database in the platform data directory
resolved via `directories`.

- Hour entries are persisted per `(date, hour)`.
- Day notes are stored separately from hour entries.
- Writes happen immediately through the controller's persist-then-commit flow.

On macOS, the database is created under the user data directory returned by
`directories::ProjectDirs`, typically inside
`~/Library/Application Support/life-tracker/`.

## Development

The repository includes some Python-based tooling through `uv`, while the app
itself is Rust.

```bash
uv sync --dev
make pre-commit-install
make pre-commit
```

For the app itself:

```bash
cargo build
cargo test
cargo run
cargo clippy
```

For the detailed architecture and module contracts, see [AGENTS.md](AGENTS.md).

## Releasing

Versioned releases are published privately on GitHub Releases.

1. Update `version` in `Cargo.toml`.
2. Commit the version bump.
3. Create and push a tag such as `v0.1.0`.
4. GitHub Actions builds the release archives and attaches them to that GitHub
   Release.

The release workflow does not publish to Cargo, Homebrew, or any public package
registry.

## Merge Bot

This repo includes a simple branch watcher plus an agent prompt template:

- [`docs/merge-bot-agent.md`](/Users/mozsoy/life-tracker/docs/merge-bot-agent.md)
- [`scripts/merge_bot_watch.sh`](/Users/mozsoy/life-tracker/scripts/merge_bot_watch.sh)

The watcher polls local branches, filters them by name, and hands a merge prompt
to whatever agent command you configure.
Use shell-style glob patterns in `MERGE_BOT_WATCH_PATTERNS` if you want to
watch branch families like `frontend/*` or `action/*`.

Basic usage:

```bash
chmod +x scripts/merge_bot_watch.sh
MERGE_BOT_WATCH_PATTERNS='frontend action controller domain' \
MERGE_BOT_TARGET_BRANCH=integration \
MERGE_BOT_AGENT=codex \
MERGE_BOT_MODEL='gpt-5.4-mini' \
MERGE_BOT_EFFORT=medium \
MERGE_BOT_COMMAND='your-agent-cli --prompt-file -' \
./scripts/merge_bot_watch.sh
```


If you do not set `MERGE_BOT_COMMAND`, the watcher prints the generated prompt
instead of invoking an agent.
