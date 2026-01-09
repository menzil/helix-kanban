use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{Project, ProjectConfig, ProjectType, Status, StatusConfig};
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
    // 0. 自动迁移到新格式（如果需要）
    let _ = super::task::auto_migrate_project_to_new_format(project_path);

    // 1. 扫描实际存在的目录
    let actual_dirs = scan_status_directories(project_path)?;

    // 2. 加载或创建配置
    let mut config = load_project_config(project_path)?;

    // 3. 同步配置：移除不存在的、添加新发现的
    let config_updated = sync_status_config(&mut config, &actual_dirs);

    // 4. 如果配置有更新，保存回文件
    if config_updated {
        save_project_config(project_path, &config)?;
    }

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

/// 扫描项目目录下的所有状态目录
/// 返回目录名列表，按字母顺序排序
fn scan_status_directories(project_path: &Path) -> Result<Vec<String>, String> {
    if !project_path.exists() {
        return Ok(Vec::new());
    }

    let mut dirs = Vec::new();

    for entry in fs::read_dir(project_path)
        .map_err(|e| format!("Failed to read project directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        // 只处理目录，排除以 . 开头的目录
        if path.is_dir() {
            if let Some(name) = path.file_name() {
                let name_str = name.to_string_lossy().to_string();
                if !name_str.starts_with('.') {
                    dirs.push(name_str);
                }
            }
        }
    }

    // 按字母顺序排序
    dirs.sort();

    Ok(dirs)
}

/// 同步配置文件中的状态定义
/// 返回是否有更新
fn sync_status_config(config: &mut ProjectConfig, actual_dirs: &[String]) -> bool {
    let mut updated = false;

    // 1. 移除 order 中不存在的目录
    let original_len = config.statuses.order.len();
    config.statuses.order.retain(|s| actual_dirs.contains(s));
    if config.statuses.order.len() != original_len {
        updated = true;
    }

    // 2. 同时清理 statuses map 中不存在的条目
    let keys_to_remove: Vec<String> = config
        .statuses
        .statuses
        .keys()
        .filter(|k| !actual_dirs.contains(k))
        .cloned()
        .collect();

    for key in keys_to_remove {
        config.statuses.statuses.remove(&key);
        updated = true;
    }

    // 3. 添加新发现的目录到 order 末尾
    for dir in actual_dirs {
        if !config.statuses.order.contains(dir) {
            config.statuses.order.push(dir.clone());
            config.statuses.statuses.insert(
                dir.clone(),
                StatusConfig {
                    display: capitalize_first(dir),
                },
            );
            updated = true;
        }
    }

    updated
}

/// 首字母大写
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let mut result = first.to_uppercase().to_string();
            result.push_str(&chars.as_str());
            result
        }
    }
}

/// 保存项目配置到 .kanban.toml
pub fn save_project_config(project_path: &Path, config: &ProjectConfig) -> Result<(), String> {
    let config_path = project_path.join(".kanban.toml");

    let content = toml::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    fs::write(&config_path, content)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
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

    // Create empty tasks.toml for new format (metadata separated)
    fs::write(project_dir.join("tasks.toml"), "[tasks]\n")
        .map_err(|e| format!("Failed to create tasks.toml: {}", e))?;

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

    // Create empty tasks.toml for new format (metadata separated)
    fs::write(project_dir.join("tasks.toml"), "[tasks]\n")
        .map_err(|e| format!("Failed to create tasks.toml: {}", e))?;

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
/// 删除项目（硬删除）- 直接删除项目目录
pub fn delete_project_by_path(project_path: &std::path::Path) -> Result<(), String> {
    if !project_path.exists() {
        return Err(format!("项目路径 '{}' 不存在", project_path.display()));
    }

    // 删除整个项目目录及其所有内容
    std::fs::remove_dir_all(project_path)
        .map_err(|e| format!("删除项目目录失败: {}", e))?;

    Ok(())
}

/// 删除项目（硬删除）- 根据项目名称和类型删除
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

/// 确保全局 AI 配置文件存在
/// 如果不存在则自动创建
pub fn ensure_global_ai_config() -> Result<(), String> {
    let ai_config_path = get_data_dir().join(".ai-config.json");

    // 如果已存在，不重复创建
    if ai_config_path.exists() {
        return Ok(());
    }

    let version = env!("CARGO_PKG_VERSION");

    let template = r##"{
  "project_type": "helix-kanban",
  "version": "VERSION",
  "description": "这是一个基于文件系统的看板项目，任务以 Markdown 文件形式存储",
  "important_notes": [
    "⚠️ 所有任务操作必须在看板目录中进行（即包含 .kanban.toml 的目录）",
    "⚠️ 看板路径可通过 hxk 应用中按 Space+p+i 获取",
    "⚠️ 不要在项目代码目录中直接操作任务文件"
  ],
  "how_to_find_kanban_dir": {
    "method_1": "在 hxk 应用中，按 Space → p → i 复制项目信息，包含看板路径",
    "method_2": "全局项目：~/.kanban/projects/项目名/",
    "method_3": "本地项目：项目代码目录/.kanban/",
    "example_global": "cd ~/.kanban/projects/匹达代驾",
    "example_local": "cd /path/to/your/project/.kanban"
  },
  "ai_instructions": {
    "create_task": {
      "description": "在看板目录的指定状态目录创建新任务",
      "command": "cd <看板目录> && 在 {status}/ 目录创建 {next_id}.md",
      "format": "# {title}\n\ncreated: {timestamp}\npriority: {priority}\n\n{description}",
      "example": "cd ~/.kanban/projects/myproject && echo '# 实现用户登录\\n\\ncreated: 2024-12-17\\npriority: high\\n\\n' > todo/005.md",
      "notes": [
        "任务编号从 001 开始，自动递增",
        "时间戳使用 ISO 8601 格式或 Unix 时间戳",
        "优先级: high, medium, low"
      ]
    },
    "move_task": {
      "description": "移动任务到另一个状态",
      "command": "cd <看板目录> && mv {from}/{file}.md {to}/",
      "example": "cd ~/.kanban/projects/myproject && mv todo/001.md doing/",
      "notes": [
        "保持文件名不变",
        "可用状态: todo, doing, done（以 .kanban.toml 为准）"
      ]
    },
    "list_tasks": {
      "description": "列出指定状态的所有任务",
      "command": "cd <看板目录> && ls {status}/*.md | xargs head -n 1",
      "example": "cd ~/.kanban/projects/myproject && ls todo/*.md",
      "notes": [
        "head -n 1 只显示标题（第一行）",
        "可以用 cat 查看完整内容"
      ]
    },
    "complete_task": {
      "description": "完成任务（移到 done）",
      "command": "cd <看板目录> && mv {from}/{file}.md done/",
      "example": "cd ~/.kanban/projects/myproject && mv doing/003.md done/"
    },
    "delete_task": {
      "description": "删除任务文件",
      "command": "cd <看板目录> && rm {status}/{file}.md",
      "example": "cd ~/.kanban/projects/myproject && rm todo/002.md",
      "warning": "删除操作不可恢复，请谨慎使用"
    }
  },
  "project_structure": {
    "root": "<看板目录>/",
    "config": ".kanban.toml",
    "statuses": [
      "todo/    - 待办任务",
      "doing/   - 进行中",
      "done/    - 已完成"
    ],
    "task_files": "{status}/001.md, 002.md, 003.md..."
  },
  "quick_commands": {
    "get_kanban_path": "在 hxk 中按 Space+p+i 复制看板路径",
    "cd_to_kanban": "cd <看板路径>",
    "add_task": "cd <看板路径> && 创建任务文件",
    "move_task": "cd <看板路径> && mv {from}/{file}.md {to}/",
    "list": "cd <看板路径> && ls {status}/*.md"
  },
  "task_format": {
    "header": "# 任务标题",
    "metadata": [
      "created: 2024-12-16T10:00:00+08:00",
      "priority: high|medium|low"
    ],
    "body": "任务的详细描述...",
    "example": "# 实现用户登录\\n\\ncreated: 2024-12-16T10:00:00+08:00\\npriority: high\\n\\n实现用户登录功能，支持邮箱和手机号登录。"
  },
  "tips": [
    "使用 'cat .kanban.toml' 查看项目配置和状态列表（在看板目录中）",
    "任务编号是文件名，如 001.md, 002.md",
    "可以直接编辑任务文件，应用会自动重新加载",
    "Y 键可以复制任务内容到剪贴板，方便分享给 AI",
    "Space+p+i 可以复制项目的看板路径和 AI 配置路径"
  ]
}"##;

    let config_content = template.replace("VERSION", version);
    std::fs::write(&ai_config_path, config_content)
        .map_err(|e| format!("创建 AI 配置文件失败: {}", e))?;

    Ok(())
}
