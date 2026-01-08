# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A terminal-based kanban board application built with Rust and Ratatui.

**Key Features:**
- File-based storage with dual-mode support (legacy + metadata-separated)
- Multiple project support (global ~/.kanban + local .kanban)
- Interactive TUI with Vim-style keyboard navigation
- Split pane support for viewing multiple projects
- Comprehensive status management (create, rename, delete, reorder)
- Command menu system (Space key)
- Input method management (auto-switch for dialogs)
- Notification system

## Tech Stack

- **Language**: Rust (edition 2021)
- **UI Framework**: Ratatui (Terminal User Interface)
- **Data Storage**: File system (Markdown + TOML)
- **Dependencies**: Minimal - ratatui, serde, toml, crossterm, chrono, arboard (clipboard)

## Project Structure

```
src/
├── main.rs              # Entry point and main loop
├── app.rs               # Application state
├── fs/
│   ├── mod.rs
│   ├── project.rs       # Project CRUD operations
│   ├── task.rs          # Task operations (dual-mode support)
│   ├── status.rs        # Status management
│   └── parser.rs        # Markdown parser
├── models/
│   ├── mod.rs
│   ├── project.rs       # Project, ProjectConfig, TasksConfig
│   ├── status.rs        # Status model
│   └── task.rs          # Task, TaskMetadata
├── ui/
│   ├── mod.rs           # Main render logic
│   ├── board.rs         # Kanban board view
│   ├── dialogs.rs       # Input/Select/Confirm dialogs
│   ├── command_menu.rs  # Space command menu
│   ├── command_completion.rs  # Command mode completion
│   ├── layout.rs        # Split pane management
│   └── help.rs          # Help screen
├── input/
│   ├── mod.rs
│   ├── keyboard.rs      # Key event handling
│   └── commands.rs      # Command definitions
├── config.rs            # User configuration
├── state.rs             # Session state persistence
└── ime.rs               # Input method management
```

## Data Storage Format

### Directory Structure

**Global Projects:**
```
~/.kanban/
├── projects/
│   └── project-name/
│       ├── .kanban.toml    # Project config
│       ├── tasks.toml      # Task metadata (new format)
│       ├── todo/
│       │   └── 1.md        # Task content files
│       ├── doing/
│       └── done/
└── .ai-config.json         # AI interaction guide
```

**Local Projects:**
```
your-project/
└── .kanban/
    ├── .kanban.toml
    ├── tasks.toml          # Optional: new format
    ├── todo/
    ├── doing/
    └── done/
```

### Project Config (`.kanban.toml`)

```toml
name = "Project Name"
created = "1234567890"

[statuses]
order = ["todo", "doing", "done"]

[statuses.todo]
display = "Todo"

[statuses.doing]
display = "Doing"

[statuses.done]
display = "Done"
```

### Task Storage (Dual-Mode Support)

**New Format** (Metadata Separated):

`tasks.toml`:
```toml
[1]
id = 1
order = 1000
title = "Task Title"
status = "todo"
created = "1234567890"
priority = "high"
tags = ["feature", "urgent"]

[2]
id = 2
order = 2000
title = "Another Task"
status = "doing"
created = "1234567891"
```

`todo/1.md` (pure content):
```markdown
Task description content here.

## Subtasks
- [ ] Subtask 1
- [ ] Subtask 2
```

**Legacy Format** (Metadata + Content in Markdown):

`todo/001.md`:
```markdown
# Task Title

id: 1
order: 1000
created: 1234567890
priority: high
tags: feature, urgent

Task description content here.
```

**Note**: The system automatically detects which format to use based on the presence of `tasks.toml`. Both formats are fully supported for backward compatibility.

## Keybindings

### Normal Mode

**Navigation:**
- `j`/`k` or `↑`/`↓`: Move task selection up/down
- `h`/`l` or `←`/`→`: Move column selection left/right
- `H`/`L` (Shift): Move task to left/right column
- `J`/`K` (Shift): Move task up/down within column

**Project Management:**
- `n`: Create new local project
- `N` (Shift): Create new global project

**Task Operations:**
- `a`: Create new task (in dialog)
- `A` (Shift): Create new task in external editor
- `e`: Edit task title
- `E` (Shift): Edit task in external editor
- `v`: Preview task (TUI)
- `V` (Shift): Preview task externally
- `d`: Delete task
- `Y` (Shift): Copy task to clipboard
- `t`: Edit task tags

**Column Width:**
- `+`: Increase current column width
- `-`: Decrease current column width
- `=`: Reset all columns to equal width
- `m`: Maximize/restore current column

**Pane Management:**
- `Space w w`: Cycle through panes
- `Space w h/l/k/j`: Focus left/right/up/down pane
- `Space w v`: Split vertically
- `Space w s`: Split horizontally
- `Space w q`: Close current pane
- `Space w m`: Maximize/restore pane

**Other:**
- `:`: Enter command mode
- `?`: Show help
- `Space`: Open command menu
- `Esc`: Cancel/return to normal mode

### Command Menu (Space)

**Main Menu:**
- `f`: Quick switch project
- `p`: Project operations...
- `w`: Window operations...
- `t`: Task operations...
- `s`: Status management...
- `r`: Reload current project
- `R`: Reload all projects
- `?`: Show help
- `q`: Quit application

**Project Submenu (Space p):**
- `o`: Open/switch project
- `n`: New local project [L]
- `N`: New global project [G]
- `d`: Hide project (soft delete)
- `D`: Delete project files (hard delete)
- `r`: Rename project
- `i`: Copy project info to clipboard

**Window Submenu (Space w):**
- `w`: Next window
- `v`: Split vertically (left/right)
- `s`: Split horizontally (top/bottom)
- `q`: Close current window
- `m`: Maximize/restore window
- `h/l/k/j`: Focus panes directionally

**Task Submenu (Space t):**
- `a`: New task
- `e`: Edit task title
- `E`: Edit in external editor
- `v`: Preview task (TUI)
- `V`: Preview externally
- `d`: Delete task
- `Y`: Copy to clipboard
- `h/m/l/n`: Set priority (high/medium/low/none)

**Status Submenu (Space s):**
- `a`: Create new status
- `r`: Rename status (internal name)
- `e`: Edit display name
- `h`: Move status column left
- `l`: Move status column right
- `d`: Delete status

### Command Mode (`:`)

- `:q` or `:quit`: Quit application
- `:reload`: Reload current project
- `:reload-all`: Reload all projects

### Dialog Mode

**Input Dialog:**
- `Enter`: Confirm
- `Ctrl+J`: New line (task input only)
- `Esc`: Cancel
- `←`/`→`: Move cursor
- `Home`/`End`: Jump to line start/end
- `Backspace`/`Delete`: Delete character

**Select Dialog:**
- `↑`/`↓` or `j`/`k`: Navigate
- `Enter`: Select
- `Esc`: Cancel
- Type to filter

**Confirm Dialog:**
- `←`/`→` or `h`/`l`: Toggle Yes/No
- `y`: Confirm Yes
- `n`: Cancel (No)
- `Enter`: Execute selection
- `Esc`: Cancel

## Key Implementation Notes

### Dual-Mode Task Storage

The system supports two storage formats:

1. **Metadata Format** (new): Metadata in `tasks.toml`, content in `{status}/{id}.md`
2. **Legacy Format** (old): Everything in markdown file with frontmatter

Detection logic in `src/fs/task.rs`:
```rust
pub fn load_tasks_from_dir(dir: &Path, status: &str) -> Result<Vec<Task>, String> {
    let project_path = dir.parent()?;

    if project_path.join("tasks.toml").exists() {
        load_tasks_from_metadata(project_path, status)  // New format
    } else {
        // Legacy format - parse from markdown
    }
}
```

### Status Management

Complete CRUD operations in `src/fs/status.rs`:
- Name validation: `[a-zA-Z0-9_-]`, starts with alphanumeric
- Display name validation: 1-50 characters
- Reserved names: (none currently)
- Minimum status count: 1 (cannot delete last status)

### Split Pane System

Recursive tree structure in `src/ui/layout.rs`:
```rust
pub enum SplitNode {
    Leaf { project_id: Option<String>, id: usize },
    Horizontal { left: Box<SplitNode>, right: Box<SplitNode>, ratio: f32 },
    Vertical { top: Box<SplitNode>, bottom: Box<SplitNode>, ratio: f32 },
}
```

### Command Menu

Hierarchical menu system with:
- Main menu (Space)
- 4 submenus (Project, Window, Task, Status)
- Keyboard navigation (j/k/↑/↓)
- Direct shortcut execution
- Visual selection highlighting

### Input Method Management

Auto-switches input method:
- English on app start
- User's preferred IME when entering dialogs
- English when exiting dialogs

Located in `src/ime.rs`, uses macOS `im-select` utility.

### Color Scheme

Nord-inspired palette:
- Background: `#2E3440` (dark blue-grey)
- Surface: `#3B4252`
- Border: `#4C566A`
- Primary: `#88C0D0` (cyan)
- Success: `#A3BE8C` (green)
- Warning: `#EBCB8B` (yellow)
- Error: `#BF616A` (red)
- Text: `#ECEFF4`, `#D8DEE9`

## Current Status

**Completed:**
- ✅ File system storage layer (dual-mode)
- ✅ Markdown parser
- ✅ Data models (Project, Status, Task, TaskMetadata)
- ✅ Project list and kanban board views
- ✅ Full keyboard navigation
- ✅ Task CRUD operations
- ✅ Task movement between statuses
- ✅ Project creation (local + global)
- ✅ Project management (rename, delete, hide)
- ✅ Split pane support
- ✅ Command menu system
- ✅ Status management (create, rename, delete, reorder)
- ✅ Column width adjustment
- ✅ Priority system
- ✅ Tag system
- ✅ External editor integration
- ✅ External preview integration
- ✅ Clipboard support
- ✅ Session state persistence
- ✅ Help system
- ✅ Notification system
- ✅ Input method management
- ✅ Metadata separation infrastructure

**In Progress:**
- 🔄 CLI API for AI interaction
- 🔄 Migration tool for legacy → metadata format

**Planned:**
- ⏳ Task search/filter
- ⏳ Task dependencies
- ⏳ Time tracking
- ⏳ Task archival

## Development Commands

```bash
# Build the project
cargo build

# Run the application
cargo run --release

# Install globally
cargo install --path . --locked

# Homebrew (if tap is set up)
brew install helix-kanban
```

## Testing

Demo projects are auto-created:
- `~/.kanban/projects/demo-project` (global)
- Current directory's `.kanban` (if present)

## AI Interaction Guide

The application includes a `.ai-config.json` file in `~/.kanban/` that provides:
- Project structure overview
- Task file format specification
- Command examples for AI tools
- Quick reference for common operations

Copy project info to clipboard: `Space` → `p` → `i`
