use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{Project, ProjectConfig, ProjectType, Status};

/// Get the kanban data directory
/// Windows: %APPDATA%\kanban
/// macOS: ~/Library/Application Support/kanban
/// Linux: ~/.local/share/kanban
pub fn get_data_dir() -> PathBuf {
    let data_dir = directories::BaseDirs::new()
        .expect("Failed to get user directories")
        .data_dir()
        .to_path_buf();
    data_dir.join("kanban")
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
pub fn list_local_project_dirs() -> std::io::Result<Vec<PathBuf>> {
    let local_kanban_dir = get_local_kanban_dir();

    if !local_kanban_dir.exists() {
        return Ok(Vec::new());
    }

    // 检查是否有 .kanban.toml 文件
    if local_kanban_dir.join(".kanban.toml").exists() {
        Ok(vec![local_kanban_dir])
    } else {
        Ok(Vec::new())
    }
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
