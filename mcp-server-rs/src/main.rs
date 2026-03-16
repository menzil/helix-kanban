use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::process::Command;

// MCP Protocol Types
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Debug, Serialize)]
struct ServerInfo {
    name: String,
    version: String,
}

#[derive(Debug, Serialize)]
struct ServerCapabilities {
    tools: HashMap<String, Value>,
}

#[derive(Debug, Serialize)]
struct Tool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

#[derive(Debug, Serialize)]
struct Project {
    index: usize,
    #[serde(rename = "type")]
    project_type: String,
    name: String,
}

#[derive(Debug, Serialize)]
struct Task {
    id: String,
    order: String,
    title: String,
    priority: String,
    status: String,
    tags: String,
}

// Execute hxk command
fn execute_hxk(args: &str) -> Result<String> {
    let output = Command::new("hxk")
        .args(args.split_whitespace())
        .output()
        .context("Failed to execute hxk command")?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Command failed: {}", stderr)
    }
}

// Parse project list output
fn parse_project_list(output: &str) -> Vec<Project> {
    let mut projects = Vec::new();
    let mut index = 1;

    for line in output.lines() {
        // Skip header lines
        if line.contains("TYPE")
            || line.contains("NAME")
            || line.contains("PATH")
            || line.contains("----")
            || line.trim().is_empty()
        {
            continue;
        }

        // Match format: [G]   project_name   /path/to/project
        if let Some(captures) = line.split_whitespace().collect::<Vec<_>>().get(0..2) {
            if let Some(type_marker) = captures[0].strip_prefix('[').and_then(|s| s.strip_suffix(']'))
            {
                let project_type = if type_marker == "G" { "global" } else { "local" };
                let name = captures[1].to_string();

                projects.push(Project {
                    index,
                    project_type: project_type.to_string(),
                    name,
                });
                index += 1;
            }
        }
    }

    projects
}

// Parse task list output
fn parse_task_list(output: &str) -> Vec<Task> {
    let mut tasks = Vec::new();
    let lines: Vec<&str> = output.lines().collect();

    if lines.len() < 2 {
        return tasks;
    }

    // Skip header line
    for line in lines.iter().skip(1) {
        if line.trim().is_empty() {
            continue;
        }

        // Split by multiple spaces (table columns)
        let parts: Vec<&str> = line.split("  ").filter(|s| !s.is_empty()).collect();

        if parts.len() >= 5 {
            tasks.push(Task {
                id: parts[0].trim().to_string(),
                order: parts.get(1).unwrap_or(&"").trim().to_string(),
                title: parts.get(2).unwrap_or(&"").trim().to_string(),
                priority: parts.get(3).unwrap_or(&"-").trim().to_string(),
                status: parts.get(4).unwrap_or(&"").trim().to_string(),
                tags: parts.get(5).unwrap_or(&"-").trim().to_string(),
            });
        }
    }

    tasks
}

// Handle tool calls
fn handle_tool_call(name: &str, arguments: &Value) -> Result<Value> {
    match name {
        "kanban_list_projects" => {
            let output = execute_hxk("project list")?;
            let projects = parse_project_list(&output);
            Ok(serde_json::to_value(projects)?)
        }

        "kanban_list_tasks" => {
            let project = arguments["project"]
                .as_str()
                .context("Missing project parameter")?;
            let status = arguments["status"].as_str();

            let mut cmd = format!("task list {}", project);
            if let Some(s) = status {
                cmd.push_str(&format!(" --status {}", s));
            }

            let output = execute_hxk(&cmd)?;
            let tasks = parse_task_list(&output);
            Ok(serde_json::to_value(tasks)?)
        }

        "kanban_show_task" => {
            let project = arguments["project"]
                .as_str()
                .context("Missing project parameter")?;
            let task_id = arguments["task_id"]
                .as_str()
                .context("Missing task_id parameter")?;

            let cmd = format!("task show {} {}", project, task_id);
            let output = execute_hxk(&cmd)?;
            Ok(json!({ "content": output }))
        }

        "kanban_create_task" => {
            let project = arguments["project"]
                .as_str()
                .context("Missing project parameter")?;
            let title = arguments["title"]
                .as_str()
                .context("Missing title parameter")?;
            let status = arguments["status"].as_str().unwrap_or("todo");

            let mut cmd = format!("task create {} --status {} --title \"{}\"", project, status, title);

            if let Some(priority) = arguments["priority"].as_str() {
                cmd.push_str(&format!(" --priority {}", priority));
            }
            if let Some(tags) = arguments["tags"].as_str() {
                cmd.push_str(&format!(" --tags \"{}\"", tags));
            }

            let output = execute_hxk(&cmd)?;
            Ok(json!({ "message": output }))
        }

        "kanban_update_task" => {
            let project = arguments["project"]
                .as_str()
                .context("Missing project parameter")?;
            let task_id = arguments["task_id"]
                .as_str()
                .context("Missing task_id parameter")?;

            let mut cmd = format!("task update {} {}", project, task_id);

            if let Some(title) = arguments["title"].as_str() {
                cmd.push_str(&format!(" --title \"{}\"", title));
            }
            if let Some(priority) = arguments["priority"].as_str() {
                cmd.push_str(&format!(" --priority {}", priority));
            }
            if let Some(tags) = arguments["tags"].as_str() {
                cmd.push_str(&format!(" --tags \"{}\"", tags));
            }

            let output = execute_hxk(&cmd)?;
            Ok(json!({ "message": output }))
        }

        "kanban_move_task" => {
            let project = arguments["project"]
                .as_str()
                .context("Missing project parameter")?;
            let task_id = arguments["task_id"]
                .as_str()
                .context("Missing task_id parameter")?;
            let to = arguments["to"]
                .as_str()
                .context("Missing to parameter")?;

            let cmd = format!("task move {} {} --to {}", project, task_id, to);
            let output = execute_hxk(&cmd)?;
            Ok(json!({ "message": output }))
        }

        "kanban_delete_task" => {
            let project = arguments["project"]
                .as_str()
                .context("Missing project parameter")?;
            let task_id = arguments["task_id"]
                .as_str()
                .context("Missing task_id parameter")?;

            let cmd = format!("task delete {} {}", project, task_id);
            let output = execute_hxk(&cmd)?;
            Ok(json!({ "message": output }))
        }

        "kanban_create_project" => {
            let name = arguments["name"]
                .as_str()
                .context("Missing name parameter")?;
            let local = arguments["local"].as_bool().unwrap_or(false);

            let cmd = if local {
                format!("project create-local {}", name)
            } else {
                format!("project create {}", name)
            };

            let output = execute_hxk(&cmd)?;
            Ok(json!({ "message": output }))
        }

        "kanban_list_statuses" => {
            let project = arguments["project"]
                .as_str()
                .context("Missing project parameter")?;

            let cmd = format!("status list {}", project);
            let output = execute_hxk(&cmd)?;
            Ok(json!({ "statuses": output }))
        }

        "kanban_batch_create_tasks" => {
            let project = arguments["project"]
                .as_str()
                .context("Missing project parameter")?;
            let tasks = arguments["tasks"]
                .as_array()
                .context("Missing tasks array")?;

            let mut results = Vec::new();
            let mut errors = Vec::new();

            for task in tasks {
                let title = task["title"].as_str().unwrap_or("");
                let status = task["status"].as_str().unwrap_or("todo");

                let mut cmd = format!("task create {} --status {} --title \"{}\"", project, status, title);

                if let Some(priority) = task["priority"].as_str() {
                    cmd.push_str(&format!(" --priority {}", priority));
                }
                if let Some(tags) = task["tags"].as_str() {
                    cmd.push_str(&format!(" --tags \"{}\"", tags));
                }

                match execute_hxk(&cmd) {
                    Ok(output) => results.push(json!({ "title": title, "status": "created", "output": output })),
                    Err(e) => errors.push(json!({ "title": title, "error": e.to_string() })),
                }
            }

            Ok(json!({
                "total": tasks.len(),
                "created": results.len(),
                "failed": errors.len(),
                "results": results,
                "errors": errors
            }))
        }

        _ => anyhow::bail!("Unknown tool: {}", name),
    }
}

// Define available tools
fn get_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "kanban_list_projects".to_string(),
            description: "List all kanban projects (both global and local). Returns structured JSON with project names, types, and index numbers.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "kanban_list_tasks".to_string(),
            description: "List all tasks in a specific project. Returns structured JSON with task details (ID, title, status, priority, tags).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project name (e.g., 'serhil_issue')"
                    },
                    "status": {
                        "type": "string",
                        "description": "Optional: Filter by status (e.g., 'todo', 'doing', 'done')"
                    }
                },
                "required": ["project"]
            }),
        },
        Tool {
            name: "kanban_show_task".to_string(),
            description: "Show detailed information about a specific task, including full description and metadata.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project name"
                    },
                    "task_id": {
                        "type": "string",
                        "description": "Task ID (e.g., '1', '12')"
                    }
                },
                "required": ["project", "task_id"]
            }),
        },
        Tool {
            name: "kanban_create_task".to_string(),
            description: "Create a new task in a project. Returns the created task ID and file path.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project name"
                    },
                    "title": {
                        "type": "string",
                        "description": "Task title"
                    },
                    "status": {
                        "type": "string",
                        "description": "Task status (e.g., 'todo', 'doing', 'done')",
                        "default": "todo"
                    },
                    "priority": {
                        "type": "string",
                        "description": "Task priority: 'high', 'medium', 'low', or 'none'",
                        "enum": ["high", "medium", "low", "none"]
                    },
                    "tags": {
                        "type": "string",
                        "description": "Comma-separated tags (e.g., 'bug,urgent' or 'feature')"
                    }
                },
                "required": ["project", "title"]
            }),
        },
        Tool {
            name: "kanban_update_task".to_string(),
            description: "Update task properties like title, priority, tags, or status.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project name"
                    },
                    "task_id": {
                        "type": "string",
                        "description": "Task ID"
                    },
                    "title": {
                        "type": "string",
                        "description": "New task title"
                    },
                    "priority": {
                        "type": "string",
                        "description": "New priority",
                        "enum": ["high", "medium", "low", "none"]
                    },
                    "tags": {
                        "type": "string",
                        "description": "New comma-separated tags"
                    }
                },
                "required": ["project", "task_id"]
            }),
        },
        Tool {
            name: "kanban_move_task".to_string(),
            description: "Move a task to a different status column.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project name"
                    },
                    "task_id": {
                        "type": "string",
                        "description": "Task ID"
                    },
                    "to": {
                        "type": "string",
                        "description": "Target status (e.g., 'todo', 'doing', 'done')"
                    }
                },
                "required": ["project", "task_id", "to"]
            }),
        },
        Tool {
            name: "kanban_delete_task".to_string(),
            description: "Delete a task from a project.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project name"
                    },
                    "task_id": {
                        "type": "string",
                        "description": "Task ID"
                    }
                },
                "required": ["project", "task_id"]
            }),
        },
        Tool {
            name: "kanban_create_project".to_string(),
            description: "Create a new kanban project. Can be global (stored in ~/.kanban/projects/) or local (stored in current directory .kanban/).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Project name"
                    },
                    "local": {
                        "type": "boolean",
                        "description": "Create as local project (default: false for global)",
                        "default": false
                    }
                },
                "required": ["name"]
            }),
        },
        Tool {
            name: "kanban_list_statuses".to_string(),
            description: "List all status columns in a project (e.g., todo, doing, done).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project name"
                    }
                },
                "required": ["project"]
            }),
        },
        Tool {
            name: "kanban_batch_create_tasks".to_string(),
            description: "Create multiple tasks at once from a list. Useful for bulk task creation.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project name"
                    },
                    "tasks": {
                        "type": "array",
                        "description": "Array of tasks to create",
                        "items": {
                            "type": "object",
                            "properties": {
                                "title": { "type": "string", "description": "Task title" },
                                "status": {
                                    "type": "string",
                                    "description": "Task status",
                                    "default": "todo"
                                },
                                "priority": {
                                    "type": "string",
                                    "description": "Task priority",
                                    "enum": ["high", "medium", "low", "none"]
                                },
                                "tags": {
                                    "type": "string",
                                    "description": "Comma-separated tags"
                                }
                            },
                            "required": ["title"]
                        }
                    }
                },
                "required": ["project", "tasks"]
            }),
        },
    ]
}

// Handle JSON-RPC request
fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id.clone();

    match request.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "helix-kanban-mcp",
                    "version": "1.0.0"
                },
                "capabilities": {
                    "tools": {}
                }
            })),
            error: None,
        },

        "tools/list" => {
            let tools = get_tools();
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({ "tools": tools })),
                error: None,
            }
        }

        "tools/call" => {
            let params = request.params.unwrap_or(json!({}));
            let name = params["name"].as_str().unwrap_or("");
            let arguments = &params["arguments"];

            match handle_tool_call(name, arguments) {
                Ok(result) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!({
                        "content": [
                            {
                                "type": "text",
                                "text": serde_json::to_string_pretty(&result).unwrap_or_default()
                            }
                        ]
                    })),
                    error: None,
                },
                Err(e) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32000,
                        message: e.to_string(),
                        data: None,
                    }),
                },
            }
        }

        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        },
    }
}

fn main() -> Result<()> {
    eprintln!("Helix Kanban MCP server v1.0.0 (Rust) running on stdio");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => {
                let response = handle_request(request);
                let response_json = serde_json::to_string(&response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
            Err(e) => {
                eprintln!("Failed to parse request: {}", e);
                let error_response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                let response_json = serde_json::to_string(&error_response)?;
                writeln!(stdout, "{}", response_json)?;
                stdout.flush()?;
            }
        }
    }

    Ok(())
}
