# life-tracker

## Development tooling

The repository uses `uv` to manage the Python-based pre-commit executable; the
application itself remains Rust-based.

```sh
uv sync --dev
make pre-commit-install
make pre-commit
```
