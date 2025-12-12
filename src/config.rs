/// 应用配置管理
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 外部编辑器命令（用于编辑任务）
    pub editor: String,
    /// Markdown 预览器命令
    pub markdown_viewer: String,
    /// 隐藏的全局项目列表（软删除）
    #[serde(default)]
    pub hidden_projects: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: detect_editor(),
            markdown_viewer: detect_markdown_viewer(),
            hidden_projects: Vec::new(),
        }
    }
}

/// 获取配置文件路径
/// Windows: %APPDATA%\kanban\config.toml
/// macOS: ~/Library/Application Support/kanban/config.toml
/// Linux: ~/.config/kanban/config.toml
pub fn get_config_path() -> PathBuf {
    let config_dir = directories::BaseDirs::new()
        .expect("Failed to get user directories")
        .config_dir()
        .to_path_buf();
    config_dir.join("kanban").join("config.toml")
}

/// 加载配置
pub fn load_config() -> Result<Config> {
    let config_path = get_config_path();

    if !config_path.exists() {
        // 配置文件不存在，返回默认配置
        return Ok(Config::default());
    }

    let content = std::fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&content)?;

    Ok(config)
}

/// 保存配置
pub fn save_config(config: &Config) -> Result<()> {
    let config_path = get_config_path();

    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = toml::to_string_pretty(config)?;
    std::fs::write(config_path, content)?;

    Ok(())
}

/// 隐藏项目（软删除 - 只对全局项目有效）
pub fn hide_project(config: &mut Config, project_name: &str) -> Result<()> {
    if !config.hidden_projects.contains(&project_name.to_string()) {
        config.hidden_projects.push(project_name.to_string());
        save_config(config)?;
    }
    Ok(())
}

/// 显示项目（取消隐藏）
pub fn unhide_project(config: &mut Config, project_name: &str) -> Result<()> {
    config.hidden_projects.retain(|p| p != project_name);
    save_config(config)?;
    Ok(())
}

/// 检查项目是否被隐藏
pub fn is_project_hidden(config: &Config, project_name: &str) -> bool {
    config.hidden_projects.contains(&project_name.to_string())
}

/// 检测系统编辑器
fn detect_editor() -> String {
    // 1. 检查环境变量
    if let Ok(editor) = std::env::var("VISUAL") {
        return editor;
    }
    if let Ok(editor) = std::env::var("EDITOR") {
        return editor;
    }

    // 2. 检查常见编辑器（按优先级）
    let common_editors = vec![
        "nvim",
        "vim",
        "nano",
        "emacs",
        "code", // VS Code
        "subl", // Sublime Text
    ];

    for editor in common_editors {
        if which(editor).is_ok() {
            return editor.to_string();
        }
    }

    // 3. 默认使用 vim
    "vim".to_string()
}

/// 检测 Markdown 预览器
fn detect_markdown_viewer() -> String {
    let os = std::env::consts::OS;

    match os {
        "macos" => {
            // macOS 上检查常见的 Markdown 预览器
            let viewers = vec![
                "open -a Marked\\ 2",     // Marked 2
                "open -a iA\\ Writer",    // iA Writer
                "open -a Typora",         // Typora
                "glow",                   // 终端预览器
                "open",                   // 默认应用
            ];

            for viewer in viewers {
                let cmd = viewer.split_whitespace().next().unwrap();
                if cmd == "open" || which(cmd).is_ok() {
                    return viewer.to_string();
                }
            }

            "open".to_string()
        }
        "linux" => {
            // Linux 上的选项
            let viewers = vec![
                "glow",        // 终端预览器
                "mdcat",       // 终端预览器
                "xdg-open",    // 默认应用
            ];

            for viewer in viewers {
                if which(viewer).is_ok() {
                    return viewer.to_string();
                }
            }

            "xdg-open".to_string()
        }
        _ => {
            // Windows 或其他系统
            "notepad".to_string()
        }
    }
}

/// 检查命令是否存在
fn which(cmd: &str) -> Result<PathBuf> {
    use std::process::Command;

    let output = Command::new("which")
        .arg(cmd)
        .output()?;

    if output.status.success() {
        let path = String::from_utf8(output.stdout)?
            .trim()
            .to_string();
        Ok(PathBuf::from(path))
    } else {
        Err(anyhow::anyhow!("Command not found: {}", cmd))
    }
}

/// 检查配置是否完整
pub fn is_config_complete(config: &Config) -> bool {
    !config.editor.is_empty() && !config.markdown_viewer.is_empty()
}

/// 首次运行检查和配置向导
/// 返回 (config, is_first_run)
pub fn check_first_run() -> Result<(Config, bool)> {
    let config_path = get_config_path();

    if !config_path.exists() {
        // 首次运行，创建默认配置
        let config = Config::default();
        save_config(&config)?;

        Ok((config, true))
    } else {
        Ok((load_config()?, false))
    }
}

/// 打印欢迎信息
fn print_welcome_message(config: &Config) {
    println!("🎉 欢迎使用 Kanban！");
    println!();
    println!("已自动检测并配置以下工具：");
    println!("  编辑器:         {}", config.editor);
    println!("  Markdown 预览: {}", config.markdown_viewer);
    println!();
    println!("配置文件位置: {}", get_config_path().display());
    println!();
    println!("如需修改配置，请使用以下命令：");
    println!("  kanban config editor <命令>       # 设置编辑器");
    println!("  kanban config viewer <命令>       # 设置预览器");
    println!("  kanban config show                # 查看当前配置");
    println!();
}

/// 更新编辑器配置
pub fn set_editor(editor: String) -> Result<()> {
    let mut config = load_config()?;
    config.editor = editor;
    save_config(&config)?;
    println!("✓ 编辑器已设置为: {}", config.editor);
    Ok(())
}

/// 更新 Markdown 预览器配置
pub fn set_viewer(viewer: String) -> Result<()> {
    let mut config = load_config()?;
    config.markdown_viewer = viewer;
    save_config(&config)?;
    println!("✓ Markdown 预览器已设置为: {}", config.markdown_viewer);
    Ok(())
}

/// 显示当前配置
pub fn show_config() -> Result<()> {
    let config = load_config()?;
    println!("当前配置:");
    println!("  编辑器:         {}", config.editor);
    println!("  Markdown 预览: {}", config.markdown_viewer);
    println!();
    println!("配置文件: {}", get_config_path().display());
    Ok(())
}
