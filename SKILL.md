---
name: life-tracker
description: Maintain and extend the life-tracker Rust TUI while preserving its calendar, day, category, and SQLite architecture.
---

# Life Tracker

- Keep the two-level Calendar → Day (24 hours) model and category-driven activities.
- Keep `ViewMode` separate from modal `Overlay`; UI modules render from `&App` and do not mutate state.
- Attempt SQLite mutations before committing in-memory changes; report errors without losing prior state.
- Preserve local-time “now” behavior, sparse day storage, and category discriminants `0..9`.
- After changes, run `cargo fmt --check`, `cargo check`, and `cargo test`.
