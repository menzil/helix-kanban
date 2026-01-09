use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::fs::parser::{generate_task_md, parse_task_md};
use crate::models::Task;

/// Load all tasks from a status directory (supports both legacy and metadata formats)
pub fn load_tasks_from_dir(dir: &Path, status: &str) -> Result<Vec<Task>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    // 获取项目根目录
    let project_path = dir.parent().ok_or("Invalid directory path")?;

    // 检查是否存在 tasks.toml（新格式）
    if project_path.join("tasks.toml").exists() {
        // 使用新的元数据格式加载
        return load_tasks_from_metadata(project_path, status);
    }

    // 否则使用旧格式（从 markdown 文件直接解析）
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

/// Save a task to a markdown file (supports both legacy and metadata formats)
pub fn save_task(project_path: &Path, task: &Task) -> Result<PathBuf, String> {
    let status_dir = project_path.join(&task.status);

    if !status_dir.exists() {
        fs::create_dir_all(&status_dir).map_err(|e| e.to_string())?;
    }

    let tasks_toml = project_path.join("tasks.toml");

    if tasks_toml.exists() {
        // 新格式：保存元数据到 tasks.toml，内容到 .md 文件
        save_task_metadata_format(project_path, task)
    } else {
        // 旧格式：保存完整数据到 .md 文件
        save_task_legacy_format(project_path, task)
    }
}

/// 保存任务（旧格式：元数据+内容都在 markdown 文件中）
fn save_task_legacy_format(project_path: &Path, task: &Task) -> Result<PathBuf, String> {
    let status_dir = project_path.join(&task.status);

    // 生成文件名：使用任务ID
    let filename = if task.file_path.exists() && task.file_path.parent() == Some(status_dir.as_path()) {
        // 任务已存在且在同一目录，保持原文件名（避免不必要的重命名）
        task.file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| "Invalid file path".to_string())?
            .to_string()
    } else {
        // 新任务或跨目录移动，使用任务ID作为文件名
        format!("{}.md", task.id)
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

/// 保存任务（新格式：元数据在 tasks.toml，内容在 .md 文件）
fn save_task_metadata_format(project_path: &Path, task: &Task) -> Result<PathBuf, String> {
    // 1. 加载现有元数据
    let mut metadata_map = load_tasks_metadata(project_path)?;

    // 2. 更新当前任务的元数据
    let task_metadata = crate::models::TaskMetadata::from(task);
    metadata_map.insert(task.id.to_string(), task_metadata);

    // 3. 保存元数据到 tasks.toml
    save_tasks_metadata(project_path, &metadata_map)?;

    // 4. 保存内容到 {status}/{id}.md
    let status_dir = project_path.join(&task.status);
    let content_path = status_dir.join(format!("{}.md", task.id));

    // 如果旧文件存在且路径不同，删除旧文件
    if task.file_path.exists() && task.file_path != content_path {
        let _ = fs::remove_file(&task.file_path);
    }

    fs::write(&content_path, &task.content).map_err(|e| e.to_string())?;

    Ok(content_path)
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

    // 移动文件
    fs::rename(old_path, &new_path).map_err(|e| e.to_string())?;

    // 如果使用新格式（tasks.toml），需要更新元数据中的 status
    let tasks_toml = project_path.join("tasks.toml");
    if tasks_toml.exists() {
        let mut metadata_map = load_tasks_metadata(project_path)?;
        if let Some(metadata) = metadata_map.get_mut(&task.id.to_string()) {
            metadata.status = new_status.to_string();
            save_tasks_metadata(project_path, &metadata_map)?;
        }
    }

    Ok(new_path)
}

/// Delete a task
pub fn delete_task(task: &Task) -> Result<(), String> {
    fs::remove_file(&task.file_path).map_err(|e| e.to_string())
}

/// Slugify a string for use as a filename
/// Example: "Checkout Flow!" -> "checkout-flow"
#[allow(dead_code)]
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
#[allow(dead_code)]
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

/// 加载任务元数据文件（tasks.toml）
pub fn load_tasks_metadata(project_path: &Path) -> Result<HashMap<String, crate::models::TaskMetadata>, String> {
    let tasks_toml = project_path.join("tasks.toml");

    if !tasks_toml.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(&tasks_toml).map_err(|e| format!("Failed to read tasks.toml: {}", e))?;

    let config: crate::models::TasksConfig = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse tasks.toml: {}", e))?;

    Ok(config.tasks)
}

/// 保存任务元数据到 tasks.toml
pub fn save_tasks_metadata(project_path: &Path, metadata: &HashMap<String, crate::models::TaskMetadata>) -> Result<(), String> {
    let tasks_toml = project_path.join("tasks.toml");

    let config = crate::models::TasksConfig {
        tasks: metadata.clone(),
    };

    let content = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize tasks.toml: {}", e))?;

    fs::write(&tasks_toml, content)
        .map_err(|e| format!("Failed to write tasks.toml: {}", e))?;

    Ok(())
}

/// 从元数据格式加载任务（新格式）
pub fn load_tasks_from_metadata(project_path: &Path, status: &str) -> Result<Vec<Task>, String> {
    // 1. 加载元数据
    let metadata_map = load_tasks_metadata(project_path)?;

    // 2. 过滤出指定状态的任务
    let mut tasks = Vec::new();

    for (id, metadata) in metadata_map {
        if metadata.status != status {
            continue;
        }

        // 3. 加载对应的内容文件
        let content_path = project_path.join(&status).join(format!("{}.md", id));

        if !content_path.exists() {
            // 内容文件缺失，跳过此任务（可选：记录警告）
            continue;
        }

        let content = fs::read_to_string(&content_path)
            .map_err(|e| format!("Failed to read content file {:?}: {}", content_path, e))?;

        // 4. 组装Task
        tasks.push(Task::from_metadata(metadata, content, content_path));
    }

    // 5. 按order排序
    tasks.sort_by_key(|t| t.order);

    Ok(tasks)
}

/// 自动迁移项目从旧格式到新格式
///
/// 此函数会：
/// 1. 检查项目是否使用旧格式（没有 tasks.toml）
/// 2. 如果是旧格式且有任务，则自动迁移
/// 3. 创建 tasks.toml 并提取所有任务的元数据
/// 4. 将任务内容文件改为纯内容（移除元数据）
pub fn auto_migrate_project_to_new_format(project_path: &Path) -> Result<bool, String> {
    let tasks_toml = project_path.join("tasks.toml");

    // 如果已经是新格式，不需要迁移
    if tasks_toml.exists() {
        return Ok(false);
    }

    // 读取项目配置以获取所有状态
    let config_path = project_path.join(".kanban.toml");
    if !config_path.exists() {
        return Err("Project config not found".to_string());
    }

    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let project_config: crate::models::ProjectConfig = toml::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    // 收集所有任务
    let mut all_tasks = Vec::new();
    let mut has_tasks = false;

    for status in &project_config.statuses.order {
        let status_dir = project_path.join(status);
        if !status_dir.exists() {
            continue;
        }

        // 使用旧格式加载任务
        for entry in fs::read_dir(&status_dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Ok(task) = load_task(&path, status) {
                    all_tasks.push(task);
                    has_tasks = true;
                }
            }
        }
    }

    // 如果没有任务，只创建空的 tasks.toml
    if !has_tasks {
        fs::write(&tasks_toml, "[tasks]\n")
            .map_err(|e| format!("Failed to create tasks.toml: {}", e))?;
        return Ok(true);
    }

    // 创建元数据映射
    let mut metadata_map = HashMap::new();

    for task in &all_tasks {
        let metadata = crate::models::TaskMetadata::from(task);
        metadata_map.insert(task.id.to_string(), metadata);
    }

    // 保存元数据到 tasks.toml
    save_tasks_metadata(project_path, &metadata_map)?;

    // 更新所有任务文件：移除元数据，只保留内容
    for task in &all_tasks {
        let new_path = project_path.join(&task.status).join(format!("{}.md", task.id));

        // 只保存纯内容
        fs::write(&new_path, &task.content)
            .map_err(|e| format!("Failed to write task content: {}", e))?;

        // 如果旧文件路径不同，删除旧文件
        if task.file_path != new_path && task.file_path.exists() {
            let _ = fs::remove_file(&task.file_path);
        }
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// 创建测试用的项目目录和配置（旧格式，无 tasks.toml）
    fn setup_legacy_project() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // 创建项目配置
        let config = r#"name = "Test Project"
created = "1234567890"

[statuses]
order = ["todo", "doing", "done"]

[statuses.todo]
display = "Todo"

[statuses.doing]
display = "Doing"

[statuses.done]
display = "Done"
"#;
        fs::write(project_path.join(".kanban.toml"), config).unwrap();

        // 创建状态目录
        fs::create_dir_all(project_path.join("todo")).unwrap();
        fs::create_dir_all(project_path.join("doing")).unwrap();
        fs::create_dir_all(project_path.join("done")).unwrap();

        temp_dir
    }

    /// 创建测试用的项目目录和配置（新格式，有 tasks.toml）
    fn setup_metadata_project() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path();

        // 创建项目配置
        let config = r#"name = "Test Project"
created = "1234567890"

[statuses]
order = ["todo", "doing", "done"]

[statuses.todo]
display = "Todo"

[statuses.doing]
display = "Doing"

[statuses.done]
display = "Done"
"#;
        fs::write(project_path.join(".kanban.toml"), config).unwrap();

        // 创建空的 tasks.toml（新格式标志）
        fs::write(project_path.join("tasks.toml"), "").unwrap();

        // 创建状态目录
        fs::create_dir_all(project_path.join("todo")).unwrap();
        fs::create_dir_all(project_path.join("doing")).unwrap();
        fs::create_dir_all(project_path.join("done")).unwrap();

        temp_dir
    }

    #[test]
    fn test_load_task_legacy_format() {
        let temp_dir = setup_legacy_project();
        let project_path = temp_dir.path();

        // 创建旧格式任务文件
        let task_content = r#"# Test Task

id: 1
order: 1000
created: 1234567890
priority: high
tags: bug, urgent

This is the task content.
"#;
        fs::write(project_path.join("todo/1.md"), task_content).unwrap();

        // 加载任务
        let task = load_task(&project_path.join("todo/1.md"), "todo").unwrap();

        assert_eq!(task.id, 1);
        assert_eq!(task.order, 1000);
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.status, "todo");
        assert_eq!(task.priority, Some("high".to_string()));
        assert_eq!(task.tags, vec!["bug", "urgent"]);
        assert!(task.content.contains("This is the task content."));
    }

    #[test]
    fn test_load_task_from_filename() {
        let temp_dir = setup_legacy_project();
        let project_path = temp_dir.path();

        // 创建没有 id 元数据的任务文件（从文件名解析 ID）
        let task_content = r#"# Task Without ID

order: 1000
created: 1234567890

Content here.
"#;
        fs::write(project_path.join("todo/42.md"), task_content).unwrap();

        let task = load_task(&project_path.join("todo/42.md"), "todo").unwrap();
        assert_eq!(task.id, 42);
    }

    #[test]
    fn test_load_tasks_from_dir_legacy() {
        let temp_dir = setup_legacy_project();
        let project_path = temp_dir.path();

        // 创建多个任务
        let task1 = r#"# Task 1

id: 1
order: 2000
created: 1234567890

Content 1.
"#;
        let task2 = r#"# Task 2

id: 2
order: 1000
created: 1234567891

Content 2.
"#;
        fs::write(project_path.join("todo/1.md"), task1).unwrap();
        fs::write(project_path.join("todo/2.md"), task2).unwrap();

        let tasks = load_tasks_from_dir(&project_path.join("todo"), "todo").unwrap();

        assert_eq!(tasks.len(), 2);
        // 应该按 order 排序，task2 (order=1000) 在前
        assert_eq!(tasks[0].id, 2);
        assert_eq!(tasks[1].id, 1);
    }

    #[test]
    fn test_load_tasks_from_dir_empty() {
        let temp_dir = setup_legacy_project();
        let project_path = temp_dir.path();

        let tasks = load_tasks_from_dir(&project_path.join("todo"), "todo").unwrap();
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_load_tasks_from_dir_nonexistent() {
        let temp_dir = setup_legacy_project();
        let project_path = temp_dir.path();

        let tasks = load_tasks_from_dir(&project_path.join("nonexistent"), "nonexistent").unwrap();
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_get_next_task_id() {
        let temp_dir = setup_legacy_project();
        let project_path = temp_dir.path();

        // 空项目，下一个 ID 应该是 1
        let next_id = get_next_task_id(project_path).unwrap();
        assert_eq!(next_id, 1);

        // 添加任务
        let task = r#"# Task

id: 5
order: 1000
created: 1234567890

Content.
"#;
        fs::write(project_path.join("todo/5.md"), task).unwrap();

        let next_id = get_next_task_id(project_path).unwrap();
        assert_eq!(next_id, 6);
    }

    #[test]
    fn test_save_task_legacy_format() {
        let temp_dir = setup_legacy_project();
        let project_path = temp_dir.path();

        let task = Task {
            id: 1,
            order: 1000,
            title: "New Task".to_string(),
            content: "Task content here.".to_string(),
            created: "1234567890".to_string(),
            priority: Some("medium".to_string()),
            status: "todo".to_string(),
            tags: vec!["feature".to_string()],
            file_path: PathBuf::new(),
        };

        let result = save_task(project_path, &task);
        assert!(result.is_ok());

        let saved_path = result.unwrap();
        assert!(saved_path.exists());
        assert_eq!(saved_path, project_path.join("todo/1.md"));

        // 验证内容
        let content = fs::read_to_string(&saved_path).unwrap();
        assert!(content.contains("# New Task"));
        assert!(content.contains("id: 1"));
        assert!(content.contains("priority: medium"));
        assert!(content.contains("tags: feature"));
    }

    #[test]
    fn test_save_task_metadata_format() {
        let temp_dir = setup_metadata_project();
        let project_path = temp_dir.path();

        let task = Task {
            id: 1,
            order: 1000,
            title: "New Task".to_string(),
            content: "Task content here.".to_string(),
            created: "1234567890".to_string(),
            priority: Some("high".to_string()),
            status: "todo".to_string(),
            tags: vec!["bug".to_string(), "urgent".to_string()],
            file_path: PathBuf::new(),
        };

        let result = save_task(project_path, &task);
        assert!(result.is_ok());

        // 验证内容文件只包含纯内容
        let content_path = project_path.join("todo/1.md");
        assert!(content_path.exists());
        let content = fs::read_to_string(&content_path).unwrap();
        assert_eq!(content, "Task content here.");

        // 验证元数据已保存到 tasks.toml
        let metadata = load_tasks_metadata(project_path).unwrap();
        assert!(metadata.contains_key("1"));
        let task_meta = metadata.get("1").unwrap();
        assert_eq!(task_meta.title, "New Task");
        assert_eq!(task_meta.status, "todo");
        assert_eq!(task_meta.priority, Some("high".to_string()));
        assert_eq!(task_meta.tags, vec!["bug", "urgent"]);
    }

    #[test]
    fn test_move_task_legacy_format() {
        let temp_dir = setup_legacy_project();
        let project_path = temp_dir.path();

        // 创建任务
        let task_content = r#"# Task to Move

id: 1
order: 1000
created: 1234567890

Content.
"#;
        fs::write(project_path.join("todo/1.md"), task_content).unwrap();

        let task = load_task(&project_path.join("todo/1.md"), "todo").unwrap();

        // 移动任务
        let result = move_task(project_path, &task, "doing");
        assert!(result.is_ok());

        // 验证文件已移动
        assert!(!project_path.join("todo/1.md").exists());
        assert!(project_path.join("doing/1.md").exists());
    }

    #[test]
    fn test_move_task_metadata_format() {
        let temp_dir = setup_metadata_project();
        let project_path = temp_dir.path();

        // 创建任务（新格式）
        let task = Task {
            id: 1,
            order: 1000,
            title: "Task to Move".to_string(),
            content: "Content.".to_string(),
            created: "1234567890".to_string(),
            priority: None,
            status: "todo".to_string(),
            tags: vec![],
            file_path: PathBuf::new(),
        };
        save_task(project_path, &task).unwrap();

        // 重新加载任务以获取正确的 file_path
        let tasks = load_tasks_from_dir(&project_path.join("todo"), "todo").unwrap();
        let task = &tasks[0];

        // 移动任务
        let result = move_task(project_path, task, "done");
        assert!(result.is_ok());

        // 验证文件已移动
        assert!(!project_path.join("todo/1.md").exists());
        assert!(project_path.join("done/1.md").exists());

        // 验证 tasks.toml 中的 status 已更新
        let metadata = load_tasks_metadata(project_path).unwrap();
        let task_meta = metadata.get("1").unwrap();
        assert_eq!(task_meta.status, "done");
    }

    #[test]
    fn test_delete_task() {
        let temp_dir = setup_legacy_project();
        let project_path = temp_dir.path();

        // 创建任务
        let task_content = "# Task\n\nid: 1\norder: 1000\ncreated: 0\n\nContent.";
        fs::write(project_path.join("todo/1.md"), task_content).unwrap();

        let task = load_task(&project_path.join("todo/1.md"), "todo").unwrap();

        // 删除任务
        let result = delete_task(&task);
        assert!(result.is_ok());
        assert!(!project_path.join("todo/1.md").exists());
    }

    #[test]
    fn test_get_max_order_in_status() {
        let temp_dir = setup_legacy_project();
        let project_path = temp_dir.path();

        // 空目录
        let max_order = get_max_order_in_status(project_path, "todo").unwrap();
        assert_eq!(max_order, -1000);

        // 添加任务
        let task1 = "# Task 1\n\nid: 1\norder: 500\ncreated: 0\n\nContent.";
        let task2 = "# Task 2\n\nid: 2\norder: 2000\ncreated: 0\n\nContent.";
        fs::write(project_path.join("todo/1.md"), task1).unwrap();
        fs::write(project_path.join("todo/2.md"), task2).unwrap();

        let max_order = get_max_order_in_status(project_path, "todo").unwrap();
        assert_eq!(max_order, 2000);
    }

    #[test]
    fn test_load_tasks_from_metadata() {
        let temp_dir = setup_metadata_project();
        let project_path = temp_dir.path();

        // 创建多个任务
        let task1 = Task {
            id: 1,
            order: 2000,
            title: "Task 1".to_string(),
            content: "Content 1.".to_string(),
            created: "1234567890".to_string(),
            priority: None,
            status: "todo".to_string(),
            tags: vec![],
            file_path: PathBuf::new(),
        };
        let task2 = Task {
            id: 2,
            order: 1000,
            title: "Task 2".to_string(),
            content: "Content 2.".to_string(),
            created: "1234567891".to_string(),
            priority: Some("high".to_string()),
            status: "todo".to_string(),
            tags: vec!["urgent".to_string()],
            file_path: PathBuf::new(),
        };
        let task3 = Task {
            id: 3,
            order: 3000,
            title: "Task 3".to_string(),
            content: "Content 3.".to_string(),
            created: "1234567892".to_string(),
            priority: None,
            status: "done".to_string(),
            tags: vec![],
            file_path: PathBuf::new(),
        };

        save_task(project_path, &task1).unwrap();
        save_task(project_path, &task2).unwrap();
        save_task(project_path, &task3).unwrap();

        // 加载 todo 状态的任务
        let todo_tasks = load_tasks_from_metadata(project_path, "todo").unwrap();
        assert_eq!(todo_tasks.len(), 2);
        // 按 order 排序
        assert_eq!(todo_tasks[0].id, 2); // order=1000
        assert_eq!(todo_tasks[1].id, 1); // order=2000

        // 加载 done 状态的任务
        let done_tasks = load_tasks_from_metadata(project_path, "done").unwrap();
        assert_eq!(done_tasks.len(), 1);
        assert_eq!(done_tasks[0].id, 3);
    }

    #[test]
    fn test_auto_migration() {
        // 创建临时测试目录
        let test_dir = std::env::temp_dir().join("kanban_test_migration");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        // 创建项目配置
        let config = r#"name = "Test Project"
created = "1234567890"

[statuses]
order = ["todo", "done"]

[statuses.todo]
display = "Todo"

[statuses.done]
display = "Done"
"#;
        fs::write(test_dir.join(".kanban.toml"), config).unwrap();

        // 创建状态目录
        fs::create_dir_all(test_dir.join("todo")).unwrap();
        fs::create_dir_all(test_dir.join("done")).unwrap();

        // 创建旧格式的任务文件
        let task1 = r#"# Test Task 1

id: 1
order: 1000
created: 1234567890
priority: high
tags: test, feature

This is task content.
"#;
        fs::write(test_dir.join("todo/1.md"), task1).unwrap();

        let task2 = r#"# Test Task 2

id: 2
order: 2000
created: 1234567891

Another task.
"#;
        fs::write(test_dir.join("done/2.md"), task2).unwrap();

        // 执行迁移
        let result = auto_migrate_project_to_new_format(&test_dir);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);

        // 验证 tasks.toml 已创建
        assert!(test_dir.join("tasks.toml").exists());

        // 验证元数据已提取
        let metadata = load_tasks_metadata(&test_dir).unwrap();
        assert_eq!(metadata.len(), 2);
        assert!(metadata.contains_key("1"));
        assert!(metadata.contains_key("2"));

        // 验证任务1的元数据
        let task1_meta = metadata.get("1").unwrap();
        assert_eq!(task1_meta.title, "Test Task 1");
        assert_eq!(task1_meta.status, "todo");
        assert_eq!(task1_meta.priority, Some("high".to_string()));
        assert_eq!(task1_meta.tags, vec!["test", "feature"]);

        // 验证内容文件已更新为纯内容
        let content1 = fs::read_to_string(test_dir.join("todo/1.md")).unwrap();
        assert!(!content1.contains("id:"));
        assert!(!content1.contains("order:"));
        assert!(content1.contains("This is task content."));

        // 清理
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    fn test_auto_migration_already_new_format() {
        let temp_dir = setup_metadata_project();
        let project_path = temp_dir.path();

        // 已经是新格式，不应该迁移
        let result = auto_migrate_project_to_new_format(project_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_auto_migration_empty_project() {
        let temp_dir = setup_legacy_project();
        let project_path = temp_dir.path();

        // 空项目迁移
        let result = auto_migrate_project_to_new_format(project_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);

        // 应该创建空的 tasks.toml
        assert!(project_path.join("tasks.toml").exists());
    }
}
