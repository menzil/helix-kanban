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

    // Sort by order (not ID)
    tasks.sort_by_key(|t| t.order);

    Ok(tasks)
}

/// Load a single task from a markdown file
pub fn load_task(path: &Path, status: &str) -> Result<Task, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;

    let parsed = parse_task_md(&content)?;

    // 从元数据读取ID（必需）
    let id = parsed
        .metadata
        .get("id")
        .and_then(|s| s.parse::<u32>().ok())
        .or_else(|| {
            // 如果没有ID，尝试从旧文件名格式解析（向后兼容）
            path.file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| {
                    // 尝试纯数字文件名（001.md）
                    if let Ok(id) = s.parse::<u32>() {
                        return Some(id);
                    }
                    // 尝试数字前缀（001-checkout-flow.md）
                    s.split('-')
                        .next()
                        .and_then(|prefix| prefix.parse::<u32>().ok())
                })
        })
        .ok_or_else(|| format!("Task file missing 'id' field and filename is not numeric: {:?}", path))?;

    // 从元数据读取order（可选，默认为id * 1000）
    let order = parsed
        .metadata
        .get("order")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or_else(|| (id as i32) * 1000);  // 默认值：兼容旧任务

    let created = parsed
        .metadata
        .get("created")
        .cloned()
        .unwrap_or_else(|| "0".to_string());

    let priority = parsed.metadata.get("priority").cloned();

    // Parse tags from comma-separated string
    let tags = parsed
        .metadata
        .get("tags")
        .map(|s| {
            s.split(',')
                .map(|tag| tag.trim().to_string())
                .filter(|tag| !tag.is_empty())
                .collect()
        })
        .unwrap_or_else(Vec::new);

    Ok(Task {
        id,
        order,
        title: parsed.title,
        content: parsed.content,
        created,
        priority,
        status: status.to_string(),
        tags,
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

    // 生成文件名
    let filename = if task.file_path.exists() && task.file_path.parent() == Some(status_dir.as_path()) {
        // 任务已存在且在同一目录，保持原文件名（避免不必要的重命名）
        task.file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| "Invalid file path".to_string())?
            .to_string()
    } else {
        // 新任务或跨目录移动，生成新文件名
        generate_task_filename(&task.title, &status_dir)
    };

    let file_path = status_dir.join(&filename);

    // 构建元数据
    let mut metadata = HashMap::new();
    metadata.insert("id".to_string(), task.id.to_string());
    metadata.insert("order".to_string(), task.order.to_string());
    metadata.insert("created".to_string(), task.created.clone());
    if let Some(priority) = &task.priority {
        metadata.insert("priority".to_string(), priority.clone());
    }
    // Save tags as comma-separated string
    if !task.tags.is_empty() {
        metadata.insert("tags".to_string(), task.tags.join(", "));
    }

    let content = generate_task_md(&task.title, &metadata, &task.content);

    // 如果文件路径变了（例如重命名），删除旧文件
    if task.file_path.exists() && task.file_path != file_path {
        let _ = fs::remove_file(&task.file_path);
    }

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

/// Slugify a string for use as a filename
/// Example: "Checkout Flow!" -> "checkout-flow"
fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0'  // 标记要删除的字符
            }
        })
        .filter(|&c| c != '\0')
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .take(50)  // 限制长度
        .collect()
}

/// Generate a unique filename for a task
/// Returns something like "checkout-flow.md" or "checkout-flow-2.md" if conflict
fn generate_task_filename(title: &str, status_dir: &Path) -> String {
    let base_slug = slugify(title);

    if base_slug.is_empty() {
        // 如果slug为空（例如纯中文标题），使用时间戳
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        return format!("task-{}.md", timestamp);
    }

    let mut filename = format!("{}.md", base_slug);
    let mut counter = 2;

    // 检查冲突
    while status_dir.join(&filename).exists() {
        filename = format!("{}-{}.md", base_slug, counter);
        counter += 1;
    }

    filename
}

/// Get the maximum order value in a status directory
pub fn get_max_order_in_status(project_path: &Path, status: &str) -> Result<i32, String> {
    let status_dir = project_path.join(status);

    if !status_dir.exists() {
        return Ok(-1000);  // 返回一个小于0的值，便于第一个任务order=0
    }

    let tasks = load_tasks_from_dir(&status_dir, status)?;

    Ok(tasks.iter().map(|t| t.order).max().unwrap_or(-1000))
}
