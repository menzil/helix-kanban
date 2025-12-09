use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::fs::parser::{generate_task_md, parse_task_md};
use crate::models::Task;

/// Load all tasks from a status directory
pub fn load_tasks_from_dir(dir: &Path, status: &str) -> Result<Vec<Task>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut tasks = Vec::new();

    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Ok(task) = load_task(&path, status) {
                tasks.push(task);
            }
        }
    }

    // Sort by ID
    tasks.sort_by_key(|t| t.id);

    Ok(tasks)
}

/// Load a single task from a markdown file
pub fn load_task(path: &Path, status: &str) -> Result<Task, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;

    let parsed = parse_task_md(&content)?;

    // Extract ID from filename (e.g., "001.md" -> 1)
    let id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.parse::<u32>().ok())
        .ok_or_else(|| format!("Invalid task filename: {:?}", path))?;

    let created = parsed
        .metadata
        .get("created")
        .cloned()
        .unwrap_or_else(|| "0".to_string());

    let priority = parsed.metadata.get("priority").cloned();

    Ok(Task {
        id,
        title: parsed.title,
        content: parsed.content,
        created,
        priority,
        status: status.to_string(),
        file_path: path.to_path_buf(),
    })
}

/// Get the next available task ID for a project
pub fn get_next_task_id(project_path: &Path) -> Result<u32, String> {
    let mut max_id = 0;

    // Scan all status directories for tasks
    for entry in fs::read_dir(project_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.is_dir() && !path.file_name().unwrap().to_str().unwrap().starts_with('.') {
            if let Ok(tasks) = load_tasks_from_dir(&path, "") {
                for task in tasks {
                    if task.id > max_id {
                        max_id = task.id;
                    }
                }
            }
        }
    }

    Ok(max_id + 1)
}

/// Save a task to a markdown file
pub fn save_task(project_path: &Path, task: &Task) -> Result<PathBuf, String> {
    let status_dir = project_path.join(&task.status);

    if !status_dir.exists() {
        fs::create_dir_all(&status_dir).map_err(|e| e.to_string())?;
    }

    let filename = format!("{:03}.md", task.id);
    let file_path = status_dir.join(filename);

    let mut metadata = HashMap::new();
    metadata.insert("created".to_string(), task.created.clone());
    if let Some(priority) = &task.priority {
        metadata.insert("priority".to_string(), priority.clone());
    }

    let content = generate_task_md(&task.title, &metadata, &task.content);

    fs::write(&file_path, content).map_err(|e| e.to_string())?;

    Ok(file_path)
}

/// Move a task to a different status
pub fn move_task(
    project_path: &Path,
    task: &Task,
    new_status: &str,
) -> Result<PathBuf, String> {
    let old_path = &task.file_path;
    let new_dir = project_path.join(new_status);

    if !new_dir.exists() {
        fs::create_dir_all(&new_dir).map_err(|e| e.to_string())?;
    }

    let filename = old_path
        .file_name()
        .ok_or_else(|| "Invalid file path".to_string())?;
    let new_path = new_dir.join(filename);

    fs::rename(old_path, &new_path).map_err(|e| e.to_string())?;

    Ok(new_path)
}

/// Delete a task
pub fn delete_task(task: &Task) -> Result<(), String> {
    fs::remove_file(&task.file_path).map_err(|e| e.to_string())
}
