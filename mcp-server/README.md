# Helix Kanban MCP Server

MCP (Model Context Protocol) server for Helix Kanban, enabling AI assistants to interact with your kanban boards.

## Features

- **Project Management**: List, create projects
- **Task Management**: Create, read, update, delete, and move tasks
- **Status Management**: List status columns
- **Path Resolution**: Get project file paths

## Installation

```bash
cd mcp-server
npm install
```

## Configuration

### For Claude Desktop

Add to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "helix-kanban": {
      "command": "node",
      "args": ["/Users/px/Documents/golden/kanban/mcp-server/index.js"]
    }
  }
}
```

### For Claude Code CLI

Add to your Claude Code settings (`~/.claude/settings.json`):

```json
{
  "mcpServers": {
    "helix-kanban": {
      "command": "node",
      "args": ["/Users/px/Documents/golden/kanban/mcp-server/index.js"]
    }
  }
}
```

## Available Tools

### kanban_list_projects
List all kanban projects (global and local).

**Parameters**: None

**Example**:
```json
{}
```

### kanban_list_tasks
List tasks in a project.

**Parameters**:
- `project` (required): Project name
- `status` (optional): Filter by status (todo, doing, done)

**Example**:
```json
{
  "project": "serhil_issue",
  "status": "todo"
}
```

### kanban_show_task
Show detailed task information.

**Parameters**:
- `project` (required): Project name
- `task_id` (required): Task ID

**Example**:
```json
{
  "project": "serhil_issue",
  "task_id": "1"
}
```

### kanban_create_task
Create a new task.

**Parameters**:
- `project` (required): Project name
- `title` (required): Task title
- `status` (optional): Initial status (default: "todo")
- `priority` (optional): Priority level (high, medium, low, none)
- `tags` (optional): Comma-separated tags

**Example**:
```json
{
  "project": "serhil_issue",
  "title": "Fix login bug",
  "status": "todo",
  "priority": "high",
  "tags": "bug,urgent"
}
```

### kanban_update_task
Update task properties.

**Parameters**:
- `project` (required): Project name
- `task_id` (required): Task ID
- `title` (optional): New title
- `priority` (optional): New priority
- `tags` (optional): New tags

**Example**:
```json
{
  "project": "serhil_issue",
  "task_id": "1",
  "priority": "high",
  "tags": "bug,critical"
}
```

### kanban_move_task
Move task to different status.

**Parameters**:
- `project` (required): Project name
- `task_id` (required): Task ID
- `to` (required): Target status

**Example**:
```json
{
  "project": "serhil_issue",
  "task_id": "1",
  "to": "doing"
}
```

### kanban_delete_task
Delete a task.

**Parameters**:
- `project` (required): Project name
- `task_id` (required): Task ID

**Example**:
```json
{
  "project": "serhil_issue",
  "task_id": "1"
}
```

### kanban_create_project
Create a new project.

**Parameters**:
- `name` (required): Project name
- `local` (optional): Create as local project (default: false)

**Example**:
```json
{
  "name": "my-new-project",
  "local": false
}
```

### kanban_list_statuses
List status columns in a project.

**Parameters**:
- `project` (required): Project name

**Example**:
```json
{
  "project": "serhil_issue"
}
```

### kanban_get_project_path
Get project directory path.

**Parameters**:
- `project` (required): Project name

**Example**:
```json
{
  "project": "serhil_issue"
}
```

## Testing

Test the MCP server manually:

```bash
# List projects
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"kanban_list_projects","arguments":{}}}' | node index.js

# List tasks
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"kanban_list_tasks","arguments":{"project":"serhil_issue"}}}' | node index.js
```

## Requirements

- Node.js 18+
- `hxk` command available in PATH
- Helix Kanban installed and configured

## Troubleshooting

### "hxk command not found"

Make sure Helix Kanban is installed and the `hxk` command is in your PATH:

```bash
which hxk
# Should output: /usr/local/bin/hxk or similar
```

### Permission denied

Make sure the index.js file is executable:

```bash
chmod +x index.js
```

## License

MIT OR Apache-2.0 (same as Helix Kanban)
