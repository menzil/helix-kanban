use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::fs::parser::{
    generate_task_md, generate_toml_frontmatter, parse_task_md, parse_toml_frontmatter_with_recovery,
};
use crate::models::task::TaskFrontmatter;
use crate::models::Task;

/// 检测目录是否使用 frontmatter 格式
fn detect_frontmatter_format(dir: &Path) -> bool {
    if let Ok(entries) = fs::read_dir(dir) {
        let mut has_files = false;
        let mut all_frontmatter = true;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                has_files = true;
                if let Ok(content) = fs::read_to_string(&path) {
                    if !content.trim_start().starts_with("+++") {
                        all_frontmatter = false;
                        break;
                    }
                }
            }
        }

        // 如果目录为空或没有 md 文件，返回 true（新项目使用 frontmatter）
        // 如果有文件，只有当所有文件都是 frontmatter 格式时才返回 true
        !has_files || all_frontmatter
    } else {
        false
    }
}

/// Load all tasks from a status directory (supports legacy, metadata, and frontmatter formats)
pub fn load_tasks_from_dir(dir: &Path, status: &str) -> Result<Vec<Task>, String> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    // 检测 frontmatter 格式
    if detect_frontmatter_format(dir) {
        return load_tasks_from_frontmatter(dir, status);
    }

    // 否则使用旧格式（从 markdown 文件直接解析）
    let mut tasks = Vec::new();

    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("md")
            && let Ok(task) = load_task(&path, status) {
                tasks.push(task);
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
            path.file_stem().and_then(|s| s.to_str()).and_then(|s| {
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
        .ok_or_else(|| {
            format!(
                "Task file missing 'id' field and filename is not numeric: {:?}",
                path
            )
        })?;

    // 从元数据读取order（可选，默认为id * 1000）
    let order = parsed
        .metadata
        .get("order")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or_else(|| (id as i32) * 1000); // 默认值：兼容旧任务

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

    // 检查是否使用新格式（tasks.toml）
    let tasks_toml = project_path.join("tasks.toml");
    if tasks_toml.exists() {
        // 新格式：直接从 tasks.toml 读取所有任务的 ID
        let metadata_map = load_tasks_metadata(project_path)?;
        for (id_str, _) in metadata_map {
            if let Ok(id) = id_str.parse::<u32>() {
                if id > max_id {
                    max_id = id;
                }
            }
        }
    } else {
        // 旧格式：扫描所有状态目录
        for entry in fs::read_dir(project_path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();

            if path.is_dir() && !path.file_name().unwrap().to_str().unwrap().starts_with('.') {
                // 获取目录名作为 status
                let status = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                if let Ok(tasks) = load_tasks_from_dir(&path, &status) {
                    for task in tasks {
                        if task.id > max_id {
                            max_id = task.id;
                        }
                    }
                }
            }
        }
    }

    Ok(max_id + 1)
}

/// Save a task to a markdown file (supports legacy and frontmatter formats)
pub fn save_task(project_path: &Path, task: &Task) -> Result<PathBuf, String> {
    let status_dir = project_path.join(&task.status);

    if !status_dir.exists() {
        fs::create_dir_all(&status_dir).map_err(|e| e.to_string())?;
    }

    // 检查是否是新任务（文件不存在）
    let task_file = status_dir.join(format!("{}.md", task.id));
    let is_new_task = !task_file.exists();

    if detect_frontmatter_format(&status_dir) || is_new_project(project_path) || is_new_task {
        // frontmatter 格式：元数据和内容都在 .md 文件中
        save_task_frontmatter_format(project_path, task)
    } else {
        // 旧格式：保存完整数据到 .md 文件
        save_task_legacy_format(project_path, task)
    }
}

/// 检查是否是新项目（没有任何任务文件）
fn is_new_project(project_path: &Path) -> bool {
    // 读取项目配置获取所有状态
    let config_path = project_path.join(".kanban.toml");
    if !config_path.exists() {
        return true;
    }

    let config_content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return true,
    };

    let project_config: crate::models::ProjectConfig = match toml::from_str(&config_content) {
        Ok(c) => c,
        Err(_) => return true,
    };

    // 检查所有状态目录是否都为空
    for status in &project_config.statuses.order {
        let status_dir = project_path.join(status);
        if status_dir.exists() {
            if let Ok(entries) = fs::read_dir(&status_dir) {
                for entry in entries.flatten() {
                    if entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
                        return false;
                    }
                }
            }
        }
    }

    true
}

/// 保存任务（旧格式：元数据+内容都在 markdown 文件中）
fn save_task_legacy_format(project_path: &Path, task: &Task) -> Result<PathBuf, String> {
    let status_dir = project_path.join(&task.status);

    // 生成文件名：使用任务ID
    let filename =
        if task.file_path.exists() && task.file_path.parent() == Some(status_dir.as_path()) {
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

/// 保存任务（frontmatter 格式：元数据和内容都在 .md 文件中）
fn save_task_frontmatter_format(project_path: &Path, task: &Task) -> Result<PathBuf, String> {
    let status_dir = project_path.join(&task.status);
    let file_path = status_dir.join(format!("{}.md", task.id));

    // 构建 frontmatter
    let frontmatter = TaskFrontmatter::from(task);

    // 生成 frontmatter 格式内容
    let content = generate_toml_frontmatter(&frontmatter, &task.title, &task.content);

    // 如果旧文件存在且路径不同，删除旧文件
    if task.file_path.exists() && task.file_path != file_path {
        let _ = fs::remove_file(&task.file_path);
    }

    fs::write(&file_path, content).map_err(|e| e.to_string())?;

    Ok(file_path)
}

/// Move a task to a different status
pub fn move_task(project_path: &Path, task: &Task, new_status: &str) -> Result<PathBuf, String> {
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

    // 如果使用 metadata-separated 格式（tasks.toml），需要更新元数据中的 status
    let tasks_toml = project_path.join("tasks.toml");
    if tasks_toml.exists() {
        let mut metadata_map = load_tasks_metadata(project_path)?;
        if let Some(metadata) = metadata_map.get_mut(&task.id.to_string()) {
            metadata.status = new_status.to_string();
            save_tasks_metadata(project_path, &metadata_map)?;
        }
    }
    // frontmatter 格式不需要额外操作，status 从目录名推断

    Ok(new_path)
}

/// Delete a task (removes file and metadata if using metadata-separated format)
pub fn delete_task(project_path: &Path, task: &Task) -> Result<(), String> {
    // 1. 删除文件
    fs::remove_file(&task.file_path).map_err(|e| e.to_string())?;

    // 2. 如果使用 metadata-separated 格式，同步删除 tasks.toml 中的元数据
    // frontmatter 格式不需要额外操作
    let tasks_toml = project_path.join("tasks.toml");
    if tasks_toml.exists() {
        let mut metadata_map = load_tasks_metadata(project_path)?;
        if metadata_map.remove(&task.id.to_string()).is_some() {
            save_tasks_metadata(project_path, &metadata_map)?;
        }
    }

    Ok(())
}

/// Get the maximum order value in a status directory
pub fn get_max_order_in_status(project_path: &Path, status: &str) -> Result<i32, String> {
    let status_dir = project_path.join(status);

    if !status_dir.exists() {
        return Ok(-1000); // 返回一个小于0的值，便于第一个任务order=0
    }

    let tasks = load_tasks_from_dir(&status_dir, status)?;

    Ok(tasks.iter().map(|t| t.order).max().unwrap_or(-1000))
}

/// 加载任务元数据文件（tasks.toml）
pub fn load_tasks_metadata(
    project_path: &Path,
) -> Result<HashMap<String, crate::models::TaskMetadata>, String> {
    let tasks_toml = project_path.join("tasks.toml");

    if !tasks_toml.exists() {
        return Ok(HashMap::new());
    }

    let content =
        fs::read_to_string(&tasks_toml).map_err(|e| format!("Failed to read tasks.toml: {}", e))?;

    // 处理空文件或只有 [tasks] 的旧格式
    let trimmed = content.trim();
    if trimmed.is_empty() || trimmed == "[tasks]" {
        return Ok(HashMap::new());
    }

    let config: crate::models::TasksConfig =
        toml::from_str(&content).map_err(|e| format!("Failed to parse tasks.toml: {}", e))?;

    Ok(config.tasks)
}

/// 保存任务元数据到 tasks.toml
pub fn save_tasks_metadata(
    project_path: &Path,
    metadata: &HashMap<String, crate::models::TaskMetadata>,
) -> Result<(), String> {
    let tasks_toml = project_path.join("tasks.toml");

    let config = crate::models::TasksConfig {
        tasks: metadata.clone(),
    };

    let content = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize tasks.toml: {}", e))?;

    fs::write(&tasks_toml, content).map_err(|e| format!("Failed to write tasks.toml: {}", e))?;

    Ok(())
}

/// 获取任务的完整内容（用于外部编辑器和复制）
/// 支持两种格式：frontmatter、legacy
pub fn get_task_full_content(task: &Task) -> Result<String, String> {
    // frontmatter 格式或旧格式：直接读取文件（文件已包含完整内容）
    fs::read_to_string(&task.file_path).map_err(|e| e.to_string())
}

/// 自动迁移项目从旧格式到新格式
///
/// 从 frontmatter 格式加载任务
fn load_tasks_from_frontmatter(dir: &Path, status: &str) -> Result<Vec<Task>, String> {
    let mut tasks = Vec::new();

    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // 使用带容错的解析器
        let parsed = match parse_toml_frontmatter_with_recovery(&content, &path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("任务解析错误 {}: {}", path.display(), e);
                continue;
            }
        };

        tasks.push(Task {
            id: parsed.frontmatter.id,
            order: parsed.frontmatter.order,
            title: parsed.title,
            content: parsed.content,
            created: parsed.frontmatter.created,
            priority: parsed.frontmatter.priority,
            status: status.to_string(),
            tags: parsed.frontmatter.tags,
            file_path: path,
        });
    }

    // 按 order 排序
    tasks.sort_by_key(|t| t.order);

    Ok(tasks)
}

/// 从 metadata-separated 格式迁移到 frontmatter 格式
pub fn migrate_metadata_to_frontmatter(project_path: &Path) -> Result<(), String> {
    let tasks_toml = project_path.join("tasks.toml");

    if !tasks_toml.exists() {
        return Ok(());
    }

    // 加载元数据
    let metadata_map = load_tasks_metadata(project_path)?;

    if metadata_map.is_empty() {
        // 空的 tasks.toml，直接删除
        let _ = fs::remove_file(&tasks_toml);
        return Ok(());
    }

    // 遍历每个任务，将元数据合并到 .md 文件中
    for (id_str, metadata) in &metadata_map {
        let content_path = project_path
            .join(&metadata.status)
            .join(format!("{}.md", id_str));

        if !content_path.exists() {
            continue;
        }

        // 读取纯内容
        let content = fs::read_to_string(&content_path)
            .map_err(|e| format!("Failed to read content file: {}", e))?;

        // 构建 frontmatter
        let frontmatter = TaskFrontmatter {
            id: metadata.id,
            order: metadata.order,
            created: metadata.created.clone(),
            priority: metadata.priority.clone(),
            tags: metadata.tags.clone(),
        };

        // 生成 frontmatter 格式内容
        let new_content = generate_toml_frontmatter(&frontmatter, &metadata.title, &content);

        // 写回文件
        fs::write(&content_path, new_content)
            .map_err(|e| format!("Failed to write frontmatter file: {}", e))?;
    }

    // 删除 tasks.toml
    fs::remove_file(&tasks_toml).map_err(|e| format!("Failed to remove tasks.toml: {}", e))?;

    Ok(())
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

        // 创建有效的 tasks.toml（新格式标志）
        // 注意：空的 tasks.toml 会被自动迁移删除，所以需要创建一个有效的结构
        fs::write(project_path.join("tasks.toml"), "[tasks]\n").unwrap();

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

        // 先创建一个旧格式的任务文件，使项目不被识别为"新项目"
        let existing_task = r#"# Existing Task

id: 99
order: 99000
created: 1234567890

Existing content.
"#;
        fs::write(project_path.join("todo/99.md"), existing_task).unwrap();

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
    fn test_save_task_metadata_format_migrates_to_frontmatter() {
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

        // 验证 tasks.toml 已被删除（迁移到 frontmatter 格式）
        assert!(!project_path.join("tasks.toml").exists());

        // 验证内容文件使用 frontmatter 格式
        let content_path = project_path.join("todo/1.md");
        assert!(content_path.exists());
        let content = fs::read_to_string(&content_path).unwrap();
        assert!(content.starts_with("+++"));
        assert!(content.contains("id = 1"));
        assert!(content.contains("order = 1000"));
        assert!(content.contains("priority = \"high\""));
        assert!(content.contains("# New Task"));
        assert!(content.contains("Task content here."));
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

        // 创建任务（metadata-separated 格式）
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
        // 注意：load_tasks_from_dir 会触发自动迁移到 frontmatter 格式
        let tasks = load_tasks_from_dir(&project_path.join("todo"), "todo").unwrap();
        let task = &tasks[0];

        // 移动任务
        let result = move_task(project_path, task, "done");
        assert!(result.is_ok());

        // 验证文件已移动
        assert!(!project_path.join("todo/1.md").exists());
        assert!(project_path.join("done/1.md").exists());

        // 自动迁移后，tasks.toml 应该已被删除
        // 任务现在使用 frontmatter 格式存储
        assert!(!project_path.join("tasks.toml").exists());

        // 验证任务内容仍然正确（frontmatter 格式）
        let content = fs::read_to_string(project_path.join("done/1.md")).unwrap();
        assert!(content.starts_with("+++"));
        assert!(content.contains("id = 1"));
        assert!(content.contains("# Task to Move"));
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
        let result = delete_task(project_path, &task);
        assert!(result.is_ok());
        assert!(!project_path.join("todo/1.md").exists());
    }

    #[test]
    fn test_delete_task_with_frontmatter() {
        let temp_dir = setup_legacy_project();
        let project_path = temp_dir.path();

        // 创建 frontmatter 格式的任务
        let mut task = Task::new(1, "Test Task".to_string(), "todo".to_string());
        task.order = 1000;
        save_task(project_path, &task).unwrap();

        // 验证任务存在
        assert!(project_path.join("todo/1.md").exists());

        // 重新加载任务以获取正确的 file_path
        let tasks = load_tasks_from_dir(&project_path.join("todo"), "todo").unwrap();
        let task = &tasks[0];

        // 删除任务
        let result = delete_task(project_path, task);
        assert!(result.is_ok());

        // 验证文件已删除
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
    fn test_load_tasks_from_frontmatter_multiple() {
        let temp_dir = setup_legacy_project();
        let project_path = temp_dir.path();

        // 创建多个任务（会自动使用 frontmatter 格式）
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
        let todo_tasks = load_tasks_from_dir(&project_path.join("todo"), "todo").unwrap();
        assert_eq!(todo_tasks.len(), 2);
        // 按 order 排序
        assert_eq!(todo_tasks[0].id, 2); // order=1000
        assert_eq!(todo_tasks[1].id, 1); // order=2000

        // 加载 done 状态的任务
        let done_tasks = load_tasks_from_dir(&project_path.join("done"), "done").unwrap();
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

    #[test]
    fn test_load_tasks_metadata_empty_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_path = temp_dir.path();

        // 创建空的 tasks.toml
        fs::write(project_path.join("tasks.toml"), "").unwrap();

        // 应该返回空的 HashMap，而不是错误
        let result = load_tasks_metadata(project_path);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_load_tasks_metadata_old_tasks_section_format() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_path = temp_dir.path();

        // 创建旧格式的 tasks.toml（只有 [tasks] 节）
        fs::write(project_path.join("tasks.toml"), "[tasks]").unwrap();

        // 应该返回空的 HashMap，而不是解析错误
        let result = load_tasks_metadata(project_path);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_save_task_with_metadata_project_migrates() {
        let temp_dir = setup_metadata_project();
        let project_path = temp_dir.path();

        // 创建新任务
        let mut task = Task::new(99, "New Task".to_string(), "todo".to_string());
        task.order = 99000;
        task.priority = Some("high".to_string());

        // 保存任务（会触发迁移到 frontmatter 格式）
        save_task(project_path, &task).unwrap();

        // 验证 tasks.toml 已被删除
        assert!(!project_path.join("tasks.toml").exists());

        // 验证任务使用 frontmatter 格式
        let content = fs::read_to_string(project_path.join("todo/99.md")).unwrap();
        assert!(content.starts_with("+++"));
        assert!(content.contains("id = 99"));
        assert!(content.contains("priority = \"high\""));
    }

    #[test]
    fn test_move_task_with_frontmatter() {
        let temp_dir = setup_legacy_project();
        let project_path = temp_dir.path();

        // 先创建一个任务（frontmatter 格式）
        let mut task = Task::new(1, "Test Task".to_string(), "todo".to_string());
        task.order = 1000;
        save_task(project_path, &task).unwrap();

        // 重新加载任务
        let tasks = load_tasks_from_dir(&project_path.join("todo"), "todo").unwrap();
        assert!(!tasks.is_empty());
        let task = &tasks[0];
        let original_id = task.id;

        // 移动任务到 done 状态
        move_task(project_path, task, "done").unwrap();

        // 验证文件已移动
        assert!(project_path.join("done").join(format!("{}.md", original_id)).exists());
        assert!(!project_path.join("todo").join(format!("{}.md", original_id)).exists());

        // 验证任务仍然是 frontmatter 格式
        let content = fs::read_to_string(project_path.join("done/1.md")).unwrap();
        assert!(content.starts_with("+++"));
    }

    #[test]
    fn test_save_task_frontmatter_format() {
        let temp_dir = setup_legacy_project();
        let project_path = temp_dir.path();

        // 新项目（没有任何任务）应该使用 frontmatter 格式
        let task = Task {
            id: 1,
            order: 1000,
            title: "Frontmatter Task".to_string(),
            content: "Task content here.".to_string(),
            created: "1234567890".to_string(),
            priority: Some("high".to_string()),
            status: "todo".to_string(),
            tags: vec!["feature".to_string(), "urgent".to_string()],
            file_path: PathBuf::new(),
        };

        let result = save_task(project_path, &task);
        assert!(result.is_ok());

        let saved_path = result.unwrap();
        assert!(saved_path.exists());
        assert_eq!(saved_path, project_path.join("todo/1.md"));

        // 验证 frontmatter 格式
        let content = fs::read_to_string(&saved_path).unwrap();
        assert!(content.starts_with("+++"));
        assert!(content.contains("id = 1"));
        assert!(content.contains("order = 1000"));
        assert!(content.contains("created = \"1234567890\""));
        assert!(content.contains("priority = \"high\""));
        assert!(content.contains("tags = ["));
        assert!(content.contains("# Frontmatter Task"));
        assert!(content.contains("Task content here."));

        // 不应该创建 tasks.toml
        assert!(!project_path.join("tasks.toml").exists());
    }

    #[test]
    fn test_load_tasks_from_frontmatter() {
        let temp_dir = setup_legacy_project();
        let project_path = temp_dir.path();

        // 创建 frontmatter 格式的任务文件
        let task_content = r#"+++
id = 1
order = 1000
created = "1234567890"
priority = "high"
tags = ["feature", "urgent"]
+++

# Test Frontmatter Task

This is the task content.
"#;
        fs::write(project_path.join("todo/1.md"), task_content).unwrap();

        // 加载任务
        let tasks = load_tasks_from_dir(&project_path.join("todo"), "todo").unwrap();
        assert_eq!(tasks.len(), 1);

        let task = &tasks[0];
        assert_eq!(task.id, 1);
        assert_eq!(task.order, 1000);
        assert_eq!(task.title, "Test Frontmatter Task");
        assert_eq!(task.status, "todo");
        assert_eq!(task.priority, Some("high".to_string()));
        assert_eq!(task.tags, vec!["feature", "urgent"]);
        assert!(task.content.contains("This is the task content."));
    }

    #[test]
    fn test_frontmatter_recovery() {
        let temp_dir = setup_legacy_project();
        let project_path = temp_dir.path();

        // 创建损坏的 frontmatter 文件（缺少 id）
        let corrupted_content = r#"+++
order = 1000
created = "1234567890"
+++

# Recovered Task

Content here.
"#;
        fs::write(project_path.join("todo/42.md"), corrupted_content).unwrap();

        // 加载任务（应该从文件名恢复 id）
        let tasks = load_tasks_from_dir(&project_path.join("todo"), "todo").unwrap();
        assert_eq!(tasks.len(), 1);

        let task = &tasks[0];
        assert_eq!(task.id, 42); // 从文件名恢复
        assert_eq!(task.title, "Recovered Task");
    }
}
