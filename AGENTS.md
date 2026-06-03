# Repository Guidelines

## Project Structure & Module Organization

This repository is `helix-kanban`, a Rust 2024 terminal kanban app distributed as the `hxk` binary. Source code lives in `src/`: `main.rs` is the entry point, `cli.rs` handles command-line behavior, `app.rs` and `state.rs` hold runtime orchestration, and feature areas are grouped under `fs/`, `models/`, `input/`, and `ui/`. MCP integration is in `src/mcp.rs`. Documentation is kept in root Markdown files and `docs/`; packaging files live in `homebrew/` and `homebrew-tap/`. The README screenshot asset is `screenshoot.png`.

## Build, Test, and Development Commands

- `cargo run` starts the development build of the TUI.
- `cargo run --release` runs the optimized binary.
- `cargo build` checks that the project compiles during development.
- `cargo build --release` creates the release binary.
- `cargo test` runs the Rust unit test suite.
- `cargo fmt --all -- --check` verifies rustfmt formatting, as CI does.
- `cargo clippy -- -D warnings` runs lint checks with warnings treated as failures.
- `./test-mcp.sh` smoke-tests the installed `hxk mcp` server and requires `hxk` and `jq` on `PATH`.

## Coding Style & Naming Conventions

Use standard Rust formatting via `cargo fmt`. Keep modules focused by domain and follow nearby patterns before adding abstractions. File and module names use `snake_case`; Rust types use `PascalCase`; functions, variables, and tests use `snake_case`. Keep CLI and MCP tool names stable unless intentionally changed and documented.

## Testing Guidelines

Most tests are inline `#[cfg(test)]` modules inside `src/`, especially in filesystem and input modules. Add tests near the code they cover and name them after the behavior verified, for example `parses_task_with_metadata`. Use `tempfile` for filesystem tests instead of user directories. Run `cargo test` before opening a PR; run `./test-mcp.sh` when changing MCP behavior.

## Commit & Pull Request Guidelines

Recent history mostly uses Conventional Commit prefixes such as `feat:`, `fix:`, `docs:`, and `chore:`. Follow that style with concise summaries, for example `fix: preserve task order after status rename`. Pull requests should describe the user-visible change, list verification commands, link relevant issues, and include screenshots or terminal recordings for TUI changes. Ensure CI-equivalent checks pass: tests, rustfmt, clippy, and release build.

## Security & Configuration Tips

Do not commit local kanban data, logs, or generated build output. User configuration and state live under `~/.kanban/`; local project boards may live in `.kanban/`. Treat task Markdown as user data, and avoid adding diagnostics that print task contents unless necessary for explicit debugging.
