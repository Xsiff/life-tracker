# life-tracker

`life-tracker` is a terminal app for logging how your hours are actually spent.
It gives you a month-based timeline of your days, lets you mark each hour with
an activity category, and lets you attach notes when a plain category is not
enough.

## Demo

![life-tracker demo](docs/assets/life-tracker-demo.gif)

The demo shows the main flow of the app:

- moving around the hour grid
- assigning categories
- adding and editing notes
- using the popup-based interaction model

## What The App Is

`life-tracker` is a keyboard-driven TUI for tracking life hour by hour.

The main screen is a grid:

- rows are dates
- columns are hours `00` through `23`
- each cell represents one hour of one day

You can fill an hour with a category, leave a note on it, or do both. You can
also focus the date label itself and attach a note to the whole day.

The built-in categories are:

- `0` Sleep
- `1` Health
- `2` Friends/Family
- `3` Romantic
- `4` Work
- `5` Waste
- `6` Travel
- `7` Hobbies/Skills
- `8` Relaxation
- `9` Other

## What The App Is For

This app is for people who want a simple record of how their days unfold
without turning life tracking into a large system.

It is useful for:

- seeing where your time actually goes
- spotting repeated patterns across days and weeks
- keeping short context notes next to tracked hours
- building a personal record that can later be analyzed

The goal is not project management or calendar scheduling. The goal is to help
you look back and answer questions like:

- Where did my evenings go this week?
- How much time did I actually spend working?
- When did I feel off, stressed, tired, or productive?

## Installation

Private releases are distributed through GitHub Releases only. There is no
Homebrew tap, Cargo publish, or other public registry release.

### Install From A Release

The installer downloads the matching release archive and installs
`life-tracker` into `~/.local/bin` by default.

```bash
git clone git@xsiff:Xsiff/life-tracker.git
cd life-tracker
GH_TOKEN=your_github_token ./scripts/install_release.sh
```

For private repositories, the installer requires one of:

- `gh` authenticated locally
- `GH_TOKEN`
- `GITHUB_TOKEN`

If `~/.local/bin` is not on your `PATH`, add this once to `~/.zshrc`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Then reload your shell:

```bash
source ~/.zshrc
```

After installation, run the app from any terminal with:

```bash
life-tracker
```

### Manual Install

You can also install the binary manually from a GitHub Release archive:

```bash
tar -xzf life-tracker-v0.1.0-aarch64-apple-darwin.tar.gz
mkdir -p ~/.local/bin
mv life-tracker ~/.local/bin/life-tracker
chmod +x ~/.local/bin/life-tracker
```

### Build From Source

If you want to build locally:

```bash
git clone git@xsiff:Xsiff/life-tracker.git
cd life-tracker
cargo build --release
mkdir -p ~/.local/bin
cp target/release/life-tracker ~/.local/bin/life-tracker
chmod +x ~/.local/bin/life-tracker
```

## Basic Usage

Launch the app:

```bash
life-tracker
```

Then use the keyboard to move through the grid and press `Enter` on a focused
hour to open the action popup.

Core controls:

- `←` / `→` move across hours
- `↑` / `↓` move across dates
- `Enter` opens the popup for the focused hour or day
- `Esc` closes the current popup
- `q` quits the app

Notes and data are stored locally on your machine.
