# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A terminal-based kanban board application built with Rust and rxtui.

**Key Features:**
- File-based storage (Markdown files + TOML config)
- Multiple project support
- Interactive TUI with keyboard navigation
- Helix-style keybindings

## Tech Stack

- **Language**: Rust (edition 2021)
- **UI Framework**: rxtui (local path: `./rxtui/rxtui`)
- **Data Storage**: File system (Markdown + TOML)
- **Dependencies**: Minimal - only `rxtui`, `serde`, and `toml`

## Project Structure

```
src/
├── main.rs              # Entry point, main app component
├── app.rs               # Application state and messages
├── fs/
│   ├── mod.rs
│   ├── project.rs       # Project file operations
│   ├── task.rs          # Task file operations
│   └── parser.rs        # Simple MD parser (no external deps)
├── models/
│   ├── mod.rs
│   ├── project.rs       # Project data model
│   ├── status.rs        # Status data model
│   └── task.rs          # Task data model
├── ui/
│   ├── mod.rs
│   ├── list.rs          # Project list view
│   ├── board.rs         # Kanban board view
│   ├── grid.rs          # Grid view (planned)
│   └── components.rs    # Shared components
└── input/
    ├── mod.rs
    └── keybindings.rs   # Helix-style keybindings (planned)
```

## Data Storage Format

**Directory Structure:**
```
~/.kanban/
└── projects/
    └── project-name/
        ├── .kanban.toml    # Project config
        ├── todo/
        │   └── 001.md      # Task files
        ├── doing/
        │   └── 002.md
        └── done/
            └── 003.md
```

**Project Config** (`.kanban.toml`):
```toml
name = "Project Name"
created = "timestamp"

[statuses]
order = ["todo", "doing", "done"]

[statuses.todo]
display = "Todo"

[statuses.doing]
display = "Doing"

[statuses.done]
display = "Done"
```

**Task File** (`001.md`):
```markdown
# Task Title

created: timestamp
priority: high

Task description content here.
```

## Development Commands

```bash
# Build the project
cargo build

# Run the application (requires real terminal)
cargo run

# Run in release mode
cargo run --release

# Or use the convenience script
./run.sh
```

## Key Implementation Notes

### rxtui Usage

- Uses local rxtui from `./rxtui/rxtui`
- Component-based architecture with `#[derive(Component)]`
- State management via `#[update]` and `#[view]` macros
- State initialization through `Default` trait
- Global keybindings with `@key_global` and `@char_global`

### Color Scheme

Using Nord-inspired palette:
- Background: `#2E3440`
- Cards: `#3B4252`
- Borders: `#4C566A`
- Primary: `#88C0D0`
- Info: `#81A1C1`
- Success: `#A3BE8C`
- Warning: `#EBCB8B`
- Error: `#BF616A`
- Text: `#ECEFF4`, `#D8DEE9`

### Keybindings

**Project List:**
- `j`/`k` or `↑`/`↓`: Navigate
- `Enter`: Open project
- `q` or `ESC`: Quit

**Board View:**
- `ESC`: Back to project list
- `h`/`l`: Navigate columns (planned)
- `a`: Add task (planned)

## Current Status

**Completed:**
- ✅ File system storage layer
- ✅ Markdown parser (no external deps)
- ✅ Data models (Project, Status, Task)
- ✅ Project list view with navigation
- ✅ Kanban board view with columns
- ✅ Basic keyboard navigation

**TODO:**
- Task creation/editing
- Task movement between statuses
- Project creation
- Grid view for multiple projects
- Full Helix-style keybindings
- Task deletion
- Priority editing

## Testing

A demo project is auto-created at `~/.kanban/projects/demo-project` with sample tasks for testing.
