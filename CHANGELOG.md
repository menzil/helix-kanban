# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Status column quick positioning**: New keyboard shortcuts for moving status columns to edges
  - Press `Space s H` (Shift+H) to move current status column to the leftmost position
  - Press `Space s L` (Shift+L) to move current status column to the rightmost position
  - Complements existing `Space s h`/`Space s l` for incremental movement
- **Helix-style modal editing**: Integrated tui-textarea with Helix-inspired keybindings
  - Normal mode: Navigate with `hjkl`, `gg`/`G`, `w`/`b`/`e`, etc.
  - Insert mode: Press `i`/`a`/`o`/`O` to enter insert mode
  - Command mode: Press `:` for commands (`:w` to save, `:q` to cancel)
  - Visual feedback: Mode indicator shows current editing mode
  - New task creation and task editing now default to Normal mode
  - Other dialogs remain in Insert mode for quick input

### Changed
- **Removed automatic IME switching**: Simplified input handling by removing automatic input method switching
  - Removed `ime.rs` module and all IME-related code
  - Users now manually control their input method as needed
  - Reduces complexity and potential issues with input method management

### Fixed
- Fixed compilation errors with `HelixTextArea::new()` missing third parameter
- Cleaned up unused imports across the codebase
- Added `#[allow(dead_code)]` attributes for utility functions reserved for future use

## [0.2.20] - 2026-01-08

### Added
- **Column width adjustment**: Dynamically adjust status column widths with keyboard shortcuts
  - Press `+` to increase current column width (+5%)
  - Press `-` to decrease current column width (-5%)
  - Press `=` to reset all columns to equal width
  - Press `m` to toggle maximize/restore current column (90% vs equal distribution)
- Width percentage indicator in column headers (shows for 2 seconds after adjustment)
- Per-project column width persistence in `~/.kanban/config.toml`

### Changed
- Column widths are now configurable and saved per project
- UI automatically redistributes remaining width when adjusting a column
- Column headers temporarily show width percentage `[30%]` or `[MAX]` after adjustments

### Technical
- Added `column_widths` and `maximized_column` fields to Config struct
- Added `last_column_resize_time` field to App for tracking adjustment timing
- Implemented `adjust_column_width()`, `toggle_maximize_column()`, `reset_column_widths()`, and `normalize_widths()` helper functions
- Added 4 new Command variants: `IncreaseColumnWidth`, `DecreaseColumnWidth`, `ResetColumnWidths`, `ToggleMaximizeColumn`
- Enhanced UI rendering to support dynamic width constraints based on configuration

### How to Use
1. Navigate to any column with `h`/`l`
2. Press `+`/`-` to adjust width (10%-80% range)
3. Press `m` to maximize/restore the column
4. Press `=` to reset to equal width
5. Configuration is automatically saved and persists across restarts

## [0.2.19] - 2026-01-08

### Added
- **Dynamic status columns**: Support for custom number of status columns beyond the default three (todo/doing/done)
- Automatic directory scanning: Program automatically detects status directories in project folders
- Auto-sync configuration: Automatically updates `.kanban.toml` when directories are added or removed
- Flex layout support: Upgraded to ratatui 0.30.0 with equal-width column distribution

### Changed
- Status columns are now dynamically loaded from project directories
- UI layout now uses `Constraint::Fill(1)` with `Flex::Start` for equal-width columns
- All keyboard navigation (h/l, H/L) now adapts to dynamic column count
- Task movement logic now supports arbitrary number of status columns

### Technical
- Added `scan_status_directories()` function to detect status directories
- Added `sync_status_config()` function to maintain config-directory consistency
- Added `get_status_name_by_column()` and `get_status_count()` helper methods to App
- Removed all hardcoded 3-column limitations
- Updated Backend trait constraints for ratatui 0.30 compatibility

### How to Use
1. Create new status: `mkdir ~/.kanban/projects/myproject/review`
2. Reload project: Press `Space` → `r` in the app
3. Adjust order: Edit `order` array in `.kanban.toml`
4. Delete status: `rm -rf ~/.kanban/projects/myproject/review`

## [0.2.18] - Previous release

...
