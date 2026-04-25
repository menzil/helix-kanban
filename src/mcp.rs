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
    eprintln!("hxk-mcp server v1.0.0 (integrated) running on stdio");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }

        match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(request) => {
                if let Some(response) = handle_request(request) {
                    let response_json = serde_json::to_string(&response).map_err(|e| e.to_string())?;
                    writeln!(stdout, "{}", response_json).map_err(|e| e.to_string())?;
                    stdout.flush().map_err(|e| e.to_string())?;
                }
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

fn handle_request(request: JsonRpcRequest) -> Option<JsonRpcResponse> {
    let id = request.id.clone();

    // Notifications have no id — do not send a response
    if id.is_none() {
        return None;
    }

    Some(match request.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "helix-kanban",
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
    })
}

fn handle_tool_call(name: &str, arguments: &Value) -> Result<Value, String> {
    match name {
        "helix-kanban_list_projects" => {
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

        "helix-kanban_list_tasks" => {
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

        "helix-kanban_show_task" => {
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

        "helix-kanban_create_task" => {
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

        "helix-kanban_update_task" => {
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

            if let Some(content) = arguments["content"].as_str() {
                task.content = content.to_string();
            }

            fs::save_task(&project_path, task)?;

            Ok(json!({
                "message": format!("Updated task #{}", task_id)
            }))
        }

        "helix-kanban_move_task" => {
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

        "helix-kanban_delete_task" => {
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

        "helix-kanban_create_project" => {
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

        "helix-kanban_list_statuses" => {
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

        "helix-kanban_batch_create_tasks" => {
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

        "helix-kanban_create_status" => {
            let project_name = arguments["project"]
                .as_str()
                .ok_or("Missing project parameter")?;
            let status_name = arguments["status"]
                .as_str()
                .ok_or("Missing status parameter")?;
            let display_name = arguments["display"]
                .as_str()
                .ok_or("Missing display parameter")?;

            let project_path = find_project_path(project_name)?;
            let project = fs::load_project(&project_path)?;

            // 验证状态名称
            fs::status::validate_status_name(status_name, &project.statuses)?;
            fs::status::validate_display_name(display_name)?;

            // 创建状态
            fs::status::create_status(&project_path, status_name, display_name)?;

            Ok(json!({
                "message": format!("Created status '{}' with display name '{}'", status_name, display_name)
            }))
        }

        "helix-kanban_rename_status" => {
            let project_name = arguments["project"]
                .as_str()
                .ok_or("Missing project parameter")?;
            let old_name = arguments["old_name"]
                .as_str()
                .ok_or("Missing old_name parameter")?;
            let new_name = arguments["new_name"]
                .as_str()
                .ok_or("Missing new_name parameter")?;
            let new_display = arguments["new_display"]
                .as_str()
                .ok_or("Missing new_display parameter")?;

            let project_path = find_project_path(project_name)?;

            // 验证新名称
            if old_name != new_name {
                let project = fs::load_project(&project_path)?;
                fs::status::validate_status_name(new_name, &project.statuses)?;
            }
            fs::status::validate_display_name(new_display)?;

            // 重命名状态
            fs::status::rename_status(&project_path, old_name, new_name, new_display)?;

            Ok(json!({
                "message": format!("Renamed status '{}' to '{}' ({})", old_name, new_name, new_display)
            }))
        }

        "helix-kanban_update_status_display" => {
            let project_name = arguments["project"]
                .as_str()
                .ok_or("Missing project parameter")?;
            let status_name = arguments["status"]
                .as_str()
                .ok_or("Missing status parameter")?;
            let new_display = arguments["display"]
                .as_str()
                .ok_or("Missing display parameter")?;

            let project_path = find_project_path(project_name)?;

            // 验证显示名称
            fs::status::validate_display_name(new_display)?;

            // 更新显示名称
            fs::status::update_status_display(&project_path, status_name, new_display)?;

            Ok(json!({
                "message": format!("Updated display name of '{}' to '{}'", status_name, new_display)
            }))
        }

        "helix-kanban_delete_status" => {
            let project_name = arguments["project"]
                .as_str()
                .ok_or("Missing project parameter")?;
            let status_name = arguments["status"]
                .as_str()
                .ok_or("Missing status parameter")?;
            let move_to = arguments["move_to"].as_str();

            let project_path = find_project_path(project_name)?;

            // 删除状态
            fs::status::delete_status(&project_path, status_name, move_to)?;

            let message = if let Some(target) = move_to {
                format!("Deleted status '{}' and moved tasks to '{}'", status_name, target)
            } else {
                format!("Deleted status '{}'", status_name)
            };

            Ok(json!({
                "message": message
            }))
        }

        "helix-kanban_move_status" => {
            let project_name = arguments["project"]
                .as_str()
                .ok_or("Missing project parameter")?;
            let status_name = arguments["status"]
                .as_str()
                .ok_or("Missing status parameter")?;
            let direction = arguments["direction"]
                .as_str()
                .ok_or("Missing direction parameter")?;

            let project_path = find_project_path(project_name)?;

            let dir_value = match direction {
                "left" => -1,
                "right" => 1,
                _ => return Err("Invalid direction, must be 'left' or 'right'".to_string()),
            };

            // 移动状态顺序
            fs::status::move_status_order(&project_path, status_name, dir_value)?;

            Ok(json!({
                "message": format!("Moved status '{}' to the {}", status_name, direction)
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
            name: "helix-kanban_list_projects".to_string(),
            description: "List all kanban projects (both global and local). Returns structured JSON with project names, types, and index numbers.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        },
        Tool {
            name: "helix-kanban_list_tasks".to_string(),
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
            name: "helix-kanban_show_task".to_string(),
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
            name: "helix-kanban_create_task".to_string(),
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
            name: "helix-kanban_update_task".to_string(),
            description: "Update task properties like title, priority, tags, or content.".to_string(),
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
                    },
                    "content": {
                        "type": "string",
                        "description": "New task content/description (markdown)"
                    }
                },
                "required": ["project", "task_id"]
            }),
        },
        Tool {
            name: "helix-kanban_move_task".to_string(),
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
            name: "helix-kanban_delete_task".to_string(),
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
            name: "helix-kanban_create_project".to_string(),
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
            name: "helix-kanban_list_statuses".to_string(),
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
            name: "helix-kanban_batch_create_tasks".to_string(),
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
        Tool {
            name: "helix-kanban_create_status".to_string(),
            description: "Create a new status column in a project.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project name"
                    },
                    "status": {
                        "type": "string",
                        "description": "Internal status name (alphanumeric, underscore, hyphen only)"
                    },
                    "display": {
                        "type": "string",
                        "description": "Display name for the status"
                    }
                },
                "required": ["project", "status", "display"]
            }),
        },
        Tool {
            name: "helix-kanban_rename_status".to_string(),
            description: "Rename a status column (both internal name and display name).".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project name"
                    },
                    "old_name": {
                        "type": "string",
                        "description": "Current internal status name"
                    },
                    "new_name": {
                        "type": "string",
                        "description": "New internal status name"
                    },
                    "new_display": {
                        "type": "string",
                        "description": "New display name"
                    }
                },
                "required": ["project", "old_name", "new_name", "new_display"]
            }),
        },
        Tool {
            name: "helix-kanban_update_status_display".to_string(),
            description: "Update the display name of a status without changing its internal name.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project name"
                    },
                    "status": {
                        "type": "string",
                        "description": "Internal status name"
                    },
                    "display": {
                        "type": "string",
                        "description": "New display name"
                    }
                },
                "required": ["project", "status", "display"]
            }),
        },
        Tool {
            name: "helix-kanban_delete_status".to_string(),
            description: "Delete a status column. Optionally move tasks to another status before deletion.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project name"
                    },
                    "status": {
                        "type": "string",
                        "description": "Status name to delete"
                    },
                    "move_to": {
                        "type": "string",
                        "description": "Optional: Target status to move tasks to before deletion"
                    }
                },
                "required": ["project", "status"]
            }),
        },
        Tool {
            name: "helix-kanban_move_status".to_string(),
            description: "Move a status column left or right in the board order.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "project": {
                        "type": "string",
                        "description": "Project name"
                    },
                    "status": {
                        "type": "string",
                        "description": "Status name to move"
                    },
                    "direction": {
                        "type": "string",
                        "description": "Direction to move: 'left' or 'right'",
                        "enum": ["left", "right"]
                    }
                },
                "required": ["project", "status", "direction"]
            }),
        },
    ]
}
