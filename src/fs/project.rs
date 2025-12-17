use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{Project, ProjectConfig, ProjectType, Status};
use serde::{Deserialize, Serialize};

/// 本地项目索引结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LocalProjectsIndex {
    local_projects: Vec<String>,
}

impl Default for LocalProjectsIndex {
    fn default() -> Self {
        Self {
            local_projects: Vec::new(),
        }
    }
}

/// 获取本地项目索引文件路径
fn get_local_projects_index_path() -> PathBuf {
    get_data_dir().join("local_projects.json")
}

/// 加载本地项目索引
fn load_local_projects_index() -> LocalProjectsIndex {
    let index_path = get_local_projects_index_path();

    if !index_path.exists() {
        return LocalProjectsIndex::default();
    }

    match fs::read_to_string(&index_path) {
        Ok(content) => {
            serde_json::from_str(&content).unwrap_or_default()
        }
        Err(_) => LocalProjectsIndex::default(),
    }
}

/// 保存本地项目索引
fn save_local_projects_index(index: &LocalProjectsIndex) -> std::io::Result<()> {
    let index_path = get_local_projects_index_path();

    // 确保父目录存在
    if let Some(parent) = index_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let content = serde_json::to_string_pretty(index)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    fs::write(&index_path, content)
}

/// 添加本地项目到索引
pub fn add_local_project_to_index(project_path: &Path) -> std::io::Result<()> {
    let mut index = load_local_projects_index();
    let path_str = project_path.to_string_lossy().to_string();

    // 避免重复添加
    if !index.local_projects.contains(&path_str) {
        index.local_projects.push(path_str);
        save_local_projects_index(&index)?;
    }

    Ok(())
}

/// 从索引中移除无效路径，返回有效路径列表
fn clean_and_get_valid_paths() -> std::io::Result<Vec<PathBuf>> {
    let mut index = load_local_projects_index();
    let mut valid_paths = Vec::new();
    let mut changed = false;

    // 过滤出仍然存在的路径
    index.local_projects.retain(|path_str| {
        let path = PathBuf::from(path_str);
        let exists = path.exists() && path.join(".kanban.toml").exists();

        if exists {
            valid_paths.push(path);
        } else {
            changed = true; // 发现无效路径
        }

        exists
    });

    // 如果有路径被清理，保存更新后的索引
    if changed {
        save_local_projects_index(&index)?;
    }

    Ok(valid_paths)
}

/// Get the kanban data directory
/// All platforms: ~/.kanban
pub fn get_data_dir() -> PathBuf {
    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .expect("Failed to get home directory");
    PathBuf::from(home_dir).join(".kanban")
}

/// Get the projects directory (~/.kanban/projects)
pub fn get_projects_dir() -> PathBuf {
    get_data_dir().join("projects")
}

/// Get the local kanban directory (.kanban in current directory)
pub fn get_local_kanban_dir() -> PathBuf {
    std::env::current_dir()
        .expect("Failed to get current directory")
        .join(".kanban")
}

/// Initialize the kanban data directory structure
pub fn init_data_dir() -> std::io::Result<()> {
    let projects_dir = get_projects_dir();
    if !projects_dir.exists() {
        fs::create_dir_all(&projects_dir)?;
    }
    Ok(())
}

/// List all global project directories
pub fn list_project_dirs() -> std::io::Result<Vec<PathBuf>> {
    let projects_dir = get_projects_dir();

    if !projects_dir.exists() {
        return Ok(Vec::new());
    }

    let mut project_paths = Vec::new();

    for entry in fs::read_dir(projects_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Check if it has a .kanban.toml file
            if path.join(".kanban.toml").exists() {
                project_paths.push(path);
            }
        }
    }

    Ok(project_paths)
}

/// List all local project directories (.kanban if exists)
/// Returns: current directory's .kanban + all valid paths from index
pub fn list_local_project_dirs() -> std::io::Result<Vec<PathBuf>> {
    let mut all_paths = Vec::new();

    // 1. 优先添加当前目录的本地项目（如果存在）
    let local_kanban_dir = get_local_kanban_dir();
    if local_kanban_dir.exists() && local_kanban_dir.join(".kanban.toml").exists() {
        all_paths.push(local_kanban_dir.clone());

        // 同时确保当前目录在索引中（自动注册）
        let _ = add_local_project_to_index(&local_kanban_dir);
    }

    // 2. 从索引中加载其他本地项目（自动清理无效路径）
    let indexed_paths = clean_and_get_valid_paths()?;

    // 3. 合并路径（去重）
    for path in indexed_paths {
        // 避免重复添加当前目录
        if !all_paths.contains(&path) {
            all_paths.push(path);
        }
    }

    Ok(all_paths)
}

/// Load project configuration from .kanban.toml
pub fn load_project_config(project_path: &Path) -> Result<ProjectConfig, String> {
    let config_path = project_path.join(".kanban.toml");

    if !config_path.exists() {
        return Err(format!("Config file not found: {:?}", config_path));
    }

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    toml::from_str(&content).map_err(|e| format!("Failed to parse TOML: {}", e))
}

/// Load a project with all its tasks
pub fn load_project(project_path: &Path) -> Result<Project, String> {
    load_project_with_type(project_path, ProjectType::Global)
}

/// Load a project with all its tasks, specifying project type
pub fn load_project_with_type(project_path: &Path, project_type: ProjectType) -> Result<Project, String> {
    let config = load_project_config(project_path)?;

    let mut statuses = Vec::new();
    for status_name in &config.statuses.order {
        if let Some(status_config) = config.statuses.statuses.get(status_name) {
            statuses.push(Status::new(
                status_name.clone(),
                status_config.display.clone(),
            ));
        }
    }

    let mut project = Project::new(config.name.clone(), project_path.to_path_buf(), project_type);
    project.statuses = statuses;

    // Load tasks from all status directories
    for status in &project.statuses {
        let status_dir = project_path.join(&status.name);
        if status_dir.exists() {
            if let Ok(tasks) = super::task::load_tasks_from_dir(&status_dir, &status.name) {
                project.tasks.extend(tasks);
            }
        }
    }

    Ok(project)
}

/// Create a new project
pub fn create_project(name: &str) -> Result<PathBuf, String> {
    let project_dir = get_projects_dir().join(name);

    if project_dir.exists() {
        return Err(format!("Project '{}' already exists", name));
    }

    // Create project directory
    fs::create_dir_all(&project_dir)
        .map_err(|e| format!("Failed to create project directory: {}", e))?;

    // Create default statuses directories
    let default_statuses = vec!["todo", "doing", "done"];
    for status in &default_statuses {
        fs::create_dir_all(project_dir.join(status))
            .map_err(|e| format!("Failed to create status directory: {}", e))?;
    }

    // Create .kanban.toml with default configuration
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let config = format!(
        concat!(
            "name = \"{}\"\n",
            "created = \"{}\"\n",
            "\n",
            "[statuses]\n",
            "order = [\"todo\", \"doing\", \"done\"]\n",
            "\n",
            "[statuses.todo]\n",
            "display = \"Todo\"\n",
            "\n",
            "[statuses.doing]\n",
            "display = \"Doing\"\n",
            "\n",
            "[statuses.done]\n",
            "display = \"Done\"\n"
        ),
        name, timestamp
    );

    fs::write(project_dir.join(".kanban.toml"), config)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(project_dir)
}

/// Create a new local project in .kanban directory
pub fn create_local_project(name: &str) -> Result<PathBuf, String> {
    let project_dir = get_local_kanban_dir();

    // Check if .kanban already exists
    if project_dir.exists() {
        return Err("本地看板已存在，一个目录只能有一个本地项目".to_string());
    }

    // Create .kanban directory
    fs::create_dir_all(&project_dir)
        .map_err(|e| format!("Failed to create .kanban directory: {}", e))?;

    // Create default statuses directories
    let default_statuses = vec!["todo", "doing", "done"];
    for status in &default_statuses {
        fs::create_dir_all(project_dir.join(status))
            .map_err(|e| format!("Failed to create status directory: {}", e))?;
    }

    // Create .kanban.toml with default configuration
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let config = format!(
        concat!(
            "name = \"{}\"\n",
            "created = \"{}\"\n",
            "\n",
            "[statuses]\n",
            "order = [\"todo\", \"doing\", \"done\"]\n",
            "\n",
            "[statuses.todo]\n",
            "display = \"Todo\"\n",
            "\n",
            "[statuses.doing]\n",
            "display = \"Doing\"\n",
            "\n",
            "[statuses.done]\n",
            "display = \"Done\"\n"
        ),
        name, timestamp
    );

    fs::write(project_dir.join(".kanban.toml"), config)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    // 自动将新创建的本地项目添加到索引
    let _ = add_local_project_to_index(&project_dir);

    Ok(project_dir)
}

/// Rename a project
pub fn rename_project(old_name: &str, new_name: &str) -> Result<(), String> {
    let projects_dir = get_projects_dir();
    let old_path = projects_dir.join(old_name);
    let new_path = projects_dir.join(new_name);

    if !old_path.exists() {
        return Err(format!("Project '{}' does not exist", old_name));
    }

    if new_path.exists() {
        return Err(format!("Project '{}' already exists", new_name));
    }

    // Rename the directory
    fs::rename(&old_path, &new_path)
        .map_err(|e| format!("Failed to rename project directory: {}", e))?;

    // Update the name in .kanban.toml
    let config_path = new_path.join(".kanban.toml");
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    // Parse and update the config
    let mut config: ProjectConfig = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse TOML: {}", e))?;

    config.name = new_name.to_string();

    let new_content = toml::to_string(&config)
        .map_err(|e| format!("Failed to serialize TOML: {}", e))?;

    fs::write(config_path, new_content)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}

/// 删除项目（包括所有任务）
pub fn delete_project(project_name: &str, project_type: &ProjectType) -> Result<(), String> {
    let project_dir = match project_type {
        ProjectType::Global => get_projects_dir().join(project_name),
        ProjectType::Local => get_local_kanban_dir().join(project_name),
    };

    if !project_dir.exists() {
        return Err(format!("项目 '{}' 不存在", project_name));
    }

    // 删除整个项目目录及其所有内容
    std::fs::remove_dir_all(&project_dir)
        .map_err(|e| format!("删除项目目录失败: {}", e))?;

    Ok(())
}
