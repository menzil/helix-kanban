use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

use crate::fs;
use crate::models::{ProjectType, Task};

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
struct TaskInfo {
    id: String,
    order: String,
    title: String,
    priority: String,
    status: String,
    tags: String,
}

/// Start MCP server on stdio
pub fn start_mcp_server() -> Result<(), String> {
    eprintln!("Helix Kanban MCP server v1.0.0 (integrated) running on stdio");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => {
                let response = handle_request(request);
                let response_json = serde_json::to_string(&response).map_err(|e| e.to_string())?;
                writeln!(stdout, "{}", response_json).map_err(|e| e.to_string())?;
                stdout.flush().map_err(|e| e.to_string())?;
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
                let response_json = serde_json::to_string(&error_response).map_err(|e| e.to_string())?;
                writeln!(stdout, "{}", response_json).map_err(|e| e.to_string())?;
                stdout.flush().map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

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
                        message: e,
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

fn handle_tool_call(name: &str, arguments: &Value) -> Result<Value, String> {
    match name {
        "kanban_list_projects" => {
            let projects = fs::load_all_projects().map_err(|e| e.to_string())?;
            let numbered_projects: Vec<Project> = projects
                .iter()
                .enumerate()
                .map(|(i, p)| Project {
                    index: i + 1,
                    project_type: match p.project_type {
                        ProjectType::Global => "global".to_string(),
                        ProjectType::Local => "local".to_string(),
                    },
                    name: p.name.clone(),
                })
                .collect();
            Ok(serde_json::to_value(numbered_projects).unwrap())
        }

        "kanban_list_tasks" => {
            let project_name = arguments["project"]
                .as_str()
                .ok_or("Missing project parameter")?;
            let status_filter = arguments["status"].as_str();

            let project_path = find_project_path(project_name)?;
            let project = fs::load_project(&project_path)?;

            let tasks: Vec<&Task> = if let Some(status) = status_filter {
                project
                    .tasks
                    .iter()
                    .filter(|t| t.status == status)
                    .collect()
            } else {
                project.tasks.iter().collect()
            };

            let task_infos: Vec<TaskInfo> = tasks
                .iter()
                .map(|t| TaskInfo {
                    id: t.id.to_string(),
                    order: t.order.to_string(),
                    title: t.title.clone(),
                    priority: t.priority.as_deref().unwrap_or("-").to_string(),
                    status: t.status.clone(),
                    tags: if t.tags.is_empty() {
                        "-".to_string()
                    } else {
                        t.tags.join(", ")
                    },
                })
                .collect();

            Ok(serde_json::to_value(task_infos).unwrap())
        }

        "kanban_show_task" => {
            let project_name = arguments["project"]
                .as_str()
                .ok_or("Missing project parameter")?;
            let task_id: u32 = arguments["task_id"]
                .as_str()
                .ok_or("Missing task_id parameter")?
                .parse()
                .map_err(|_| "Invalid task_id")?;

            let project_path = find_project_path(project_name)?;
            let project = fs::load_project(&project_path)?;

            let task = project
                .tasks
                .iter()
                .find(|t| t.id == task_id)
                .ok_or_else(|| format!("Task {} not found", task_id))?;

            Ok(json!({
                "id": task.id,
                "title": task.title,
                "status": task.status,
                "order": task.order,
                "priority": task.priority.as_deref().unwrap_or("-"),
                "tags": task.tags.join(", "),
                "created": task.created,
                "content": task.content
            }))
        }

        "kanban_create_task" => {
            let project_name = arguments["project"]
                .as_str()
                .ok_or("Missing project parameter")?;
            let title = arguments["title"]
                .as_str()
                .ok_or("Missing title parameter")?;
            let status = arguments["status"].as_str().unwrap_or("todo");

            let project_path = find_project_path(project_name)?;
            let next_id = fs::get_next_task_id(&project_path)?;
            let max_order = fs::get_max_order_in_status(&project_path, status)?;
            let new_order = max_order + 1000;

            let mut task = Task::new(next_id, title.to_string(), status.to_string());
            task.order = new_order;

            if let Some(priority) = arguments["priority"].as_str() {
                task.priority = Some(priority.to_string());
            }

            if let Some(tags) = arguments["tags"].as_str() {
                task.tags = tags.split(',').map(|s| s.trim().to_string()).collect();
            }

            let file_path = fs::save_task(&project_path, &task)?;

            Ok(json!({
                "message": format!("Created task #{} in status '{}'", task.id, status),
                "task_id": task.id,
                "file": file_path.to_string_lossy()
            }))
        }

        "kanban_update_task" => {
            let project_name = arguments["project"]
                .as_str()
                .ok_or("Missing project parameter")?;
            let task_id: u32 = arguments["task_id"]
                .as_str()
                .ok_or("Missing task_id parameter")?
                .parse()
                .map_err(|_| "Invalid task_id")?;

            let project_path = find_project_path(project_name)?;
            let mut project = fs::load_project(&project_path)?;

            let task = project
                .tasks
                .iter_mut()
                .find(|t| t.id == task_id)
                .ok_or_else(|| format!("Task {} not found", task_id))?;

            if let Some(title) = arguments["title"].as_str() {
                task.title = title.to_string();
            }

            if let Some(priority) = arguments["priority"].as_str() {
                task.priority = if priority == "none" || priority.is_empty() {
                    None
                } else {
                    Some(priority.to_string())
                };
            }

            if let Some(tags) = arguments["tags"].as_str() {
                task.tags = tags.split(',').map(|s| s.trim().to_string()).collect();
            }

            fs::save_task(&project_path, task)?;

            Ok(json!({
                "message": format!("Updated task #{}", task_id)
            }))
        }

        "kanban_move_task" => {
            let project_name = arguments["project"]
                .as_str()
                .ok_or("Missing project parameter")?;
            let task_id: u32 = arguments["task_id"]
                .as_str()
                .ok_or("Missing task_id parameter")?
                .parse()
                .map_err(|_| "Invalid task_id")?;
            let new_status = arguments["to"]
                .as_str()
                .ok_or("Missing to parameter")?;

            let project_path = find_project_path(project_name)?;
            let mut project = fs::load_project(&project_path)?;

            let task = project
                .tasks
                .iter_mut()
                .find(|t| t.id == task_id)
                .ok_or_else(|| format!("Task {} not found", task_id))?;

            let old_status = task.status.clone();
            task.status = new_status.to_string();

            let new_path = fs::move_task(&project_path, task, new_status)?;
            task.file_path = new_path;

            Ok(json!({
                "message": format!("Moved task #{} from '{}' to '{}'", task_id, old_status, new_status)
            }))
        }

        "kanban_delete_task" => {
            let project_name = arguments["project"]
                .as_str()
                .ok_or("Missing project parameter")?;
            let task_id: u32 = arguments["task_id"]
                .as_str()
                .ok_or("Missing task_id parameter")?
                .parse()
                .map_err(|_| "Invalid task_id")?;

            let project_path = find_project_path(project_name)?;
            let project = fs::load_project(&project_path)?;

            let task = project
                .tasks
                .iter()
                .find(|t| t.id == task_id)
                .ok_or_else(|| format!("Task {} not found", task_id))?;

            fs::delete_task(&project_path, task)?;

            Ok(json!({
                "message": format!("Deleted task #{}", task_id)
            }))
        }

        "kanban_create_project" => {
            let name = arguments["name"]
                .as_str()
                .ok_or("Missing name parameter")?;
            let is_local = arguments["local"].as_bool().unwrap_or(false);

            let path = if is_local {
                fs::create_local_project(name)?
            } else {
                fs::create_project(name)?
            };

            Ok(json!({
                "message": format!("Created {} project: {}",
                    if is_local { "local" } else { "global" },
                    path.to_string_lossy())
            }))
        }

        "kanban_list_statuses" => {
            let project_name = arguments["project"]
                .as_str()
                .ok_or("Missing project parameter")?;

            let project_path = find_project_path(project_name)?;
            let project = fs::load_project(&project_path)?;

            let statuses: Vec<Value> = project
                .statuses
                .iter()
                .map(|s| {
                    let task_count = project.tasks.iter().filter(|t| t.status == s.name).count();
                    json!({
                        "name": s.name,
                        "display": s.display,
                        "tasks": task_count
                    })
                })
                .collect();

            Ok(json!(statuses))
        }

        "kanban_batch_create_tasks" => {
            let project_name = arguments["project"]
                .as_str()
                .ok_or("Missing project parameter")?;
            let tasks = arguments["tasks"]
                .as_array()
                .ok_or("Missing tasks array")?;

            let project_path = find_project_path(project_name)?;
            let mut results = Vec::new();
            let mut errors = Vec::new();

            for task_data in tasks {
                let title = task_data["title"].as_str().unwrap_or("");
                let status = task_data["status"].as_str().unwrap_or("todo");

                match create_single_task(&project_path, title, status, task_data) {
                    Ok(task_id) => results.push(json!({
                        "title": title,
                        "status": "created",
                        "task_id": task_id
                    })),
                    Err(e) => errors.push(json!({
                        "title": title,
                        "error": e
                    })),
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

        _ => Err(format!("Unknown tool: {}", name)),
    }
}

fn create_single_task(
    project_path: &std::path::Path,
    title: &str,
    status: &str,
    task_data: &Value,
) -> Result<u32, String> {
    let next_id = fs::get_next_task_id(project_path)?;
    let max_order = fs::get_max_order_in_status(project_path, status)?;
    let new_order = max_order + 1000;

    let mut task = Task::new(next_id, title.to_string(), status.to_string());
    task.order = new_order;

    if let Some(priority) = task_data["priority"].as_str() {
        task.priority = Some(priority.to_string());
    }

    if let Some(tags) = task_data["tags"].as_str() {
        task.tags = tags.split(',').map(|s| s.trim().to_string()).collect();
    }

    fs::save_task(project_path, &task)?;

    Ok(task.id)
}

fn find_project_path(project_name: &str) -> Result<std::path::PathBuf, String> {
    let projects = fs::load_all_projects().map_err(|e| e.to_string())?;
    projects
        .iter()
        .find(|p| p.name == project_name)
        .map(|p| p.path.clone())
        .ok_or_else(|| format!("Project '{}' not found", project_name))
}

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
