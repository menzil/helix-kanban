# Helix Kanban MCP Server (Rust)

Rust implementation of the MCP server for helix-kanban.

## Features

- **Zero dependencies on external SDKs** - Pure Rust implementation of JSON-RPC protocol
- **Better performance** - No Node.js runtime overhead
- **Single binary** - Easy deployment, no npm install needed
- **Direct CLI integration** - Calls `hxk` command directly
- **Index numbering** - Projects are numbered for easy selection

## Build

```bash
cargo build --release
```

The binary will be at `target/release/helix-kanban-mcp`

## Install

```bash
cargo install --path .
```

## Usage

Add to your Claude Code MCP settings (`~/.claude/settings.json`):

```json
{
  "mcpServers": {
    "helix-kanban": {
      "command": "/path/to/helix-kanban-mcp"
    }
  }
}
```

Or use the full path to the binary:

```json
{
  "mcpServers": {
    "helix-kanban": {
      "command": "/Users/px/Documents/golden/kanban/mcp-server-rs/target/release/helix-kanban-mcp"
    }
  }
}
```

## Available Tools

Same as the JavaScript version:

- `kanban_list_projects` - List all projects with index numbers
- `kanban_list_tasks` - List tasks in a project
- `kanban_show_task` - Show task details
- `kanban_create_task` - Create a new task
- `kanban_update_task` - Update task properties
- `kanban_move_task` - Move task to different status
- `kanban_delete_task` - Delete a task
- `kanban_create_project` - Create new project
- `kanban_list_statuses` - List status columns
- `kanban_batch_create_tasks` - Bulk create tasks

## Comparison with JavaScript Version

| Feature | JavaScript | Rust |
|---------|-----------|------|
| Runtime | Node.js required | Standalone binary |
| Startup time | ~100ms | ~10ms |
| Memory usage | ~30MB | ~2MB |
| Dependencies | 200+ npm packages | 3 crates |
| Build time | npm install | cargo build |
| Distribution | npm package | Single binary |

## Development

```bash
# Run directly
cargo run

# Build release
cargo build --release

# Test with echo
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | cargo run
```
