# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
