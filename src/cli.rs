use anyhow::Result;
use std::env;

/// 处理 CLI 命令
/// 返回 true 表示应该继续进入 TUI，false 表示已处理完毕应该退出
pub fn handle_cli() -> Result<bool> {
    let args: Vec<String> = env::args().collect();

    // 如果没有参数，进入 TUI 模式
    if args.len() < 2 {
        return Ok(true);
    }

    // 处理 CLI 命令
    match args[1].as_str() {
        "init" => {
            cli_init()?;
            Ok(false)
        }
        "create" => {
            if args.len() < 3 {
                eprintln!("用法: hxk create <project-name>");
                std::process::exit(1);
            }
            cli_create(&args[2])?;
            Ok(false)
        }
        "list" => {
            cli_list()?;
            Ok(false)
        }
        "add" => {
            if args.len() < 3 {
                eprintln!("用法: hxk add <task-title>");
                std::process::exit(1);
            }
            cli_add(&args[2..])?;
            Ok(false)
        }
        "config" => {
            if args.len() < 3 {
                crate::config::show_config()?;
            } else {
                match args[2].as_str() {
                    "show" => crate::config::show_config()?,
                    "editor" => {
                        if args.len() < 4 {
                            eprintln!("用法: hxk config editor <命令>");
                            std::process::exit(1);
                        }
                        crate::config::set_editor(args[3..].join(" "))?;
                    }
                    "viewer" => {
                        if args.len() < 4 {
                            eprintln!("用法: hxk config viewer <命令>");
                            std::process::exit(1);
                        }
                        crate::config::set_viewer(args[3..].join(" "))?;
                    }
                    _ => {
                        eprintln!("未知的配置选项: {}", args[2]);
                        eprintln!("可用选项: show, editor, viewer");
                        std::process::exit(1);
                    }
                }
            }
            Ok(false)
        }
        "--help" | "-h" => {
            print_help();
            Ok(false)
        }
        "--version" | "-V" | "-v" => {
            print_version();
            Ok(false)
        }
        _ => {
            eprintln!("未知命令: {}", args[1]);
            eprintln!("使用 'hxk --help' 查看帮助");
            std::process::exit(1);
        }
    }
}

/// 初始化本地看板（已废弃，保留以兼容旧版本）
fn cli_init() -> Result<()> {
    println!("提示: 'kanban init' 命令已废弃");
    println!("本地项目会自动创建在 .kanban/ 目录");
    println!("\n使用 'kanban create <name>' 创建本地看板");

    Ok(())
}

/// 创建本地项目
fn cli_create(name: &str) -> Result<()> {
    use crate::fs;

    // 直接创建 .kanban 目录
    match fs::create_local_project(name) {
        Ok(path) => {
            println!("✓ 已创建本地看板");
            println!("  项目名: {}", name);
            println!("  位置: {}", path.display());
            Ok(())
        }
        Err(e) => {
            eprintln!("错误: {}", e);
            std::process::exit(1);
        }
    }
}

/// 列出所有项目
fn cli_list() -> Result<()> {
    use crate::fs;

    println!("全局项目 (~/.kanban/projects):");
    println!("{}", "=".repeat(40));

    let global_projects = fs::list_project_dirs()?;
    if global_projects.is_empty() {
        println!("  (无)");
    } else {
        for (i, path) in global_projects.iter().enumerate() {
            if let Some(name) = path.file_name() {
                println!("  {}. [G] {}", i + 1, name.to_string_lossy());
            }
        }
    }

    println!("\n本地项目 (当前目录 .kanban):");
    println!("{}", "=".repeat(40));

    let local_projects = fs::list_local_project_dirs()?;
    if local_projects.is_empty() {
        println!("  (无)");
        println!("\n提示: 使用 'kanban create <name>' 创建本地看板");
    } else {
        for path in local_projects.iter() {
            // 读取项目名称
            if let Ok(config) = fs::load_project_config(path) {
                println!("  1. [L] {}", config.name);
            }
        }
    }

    Ok(())
}

/// 快速添加任务
fn cli_add(args: &[String]) -> Result<()> {
    use crate::fs;

    let title = args.join(" ");

    // 查找第一个本地项目
    let local_projects = fs::list_local_project_dirs()?;

    let project_path = if let Some(first) = local_projects.first() {
        first.clone()
    } else {
        eprintln!("错误: 当前目录没有本地项目");
        eprintln!("使用 'kanban create <name>' 创建项目");
        std::process::exit(1);
    };

    // 获取下一个任务 ID
    let todo_dir = project_path.join("todo");
    let task_id = fs::get_next_task_id(&todo_dir).map_err(|e| anyhow::anyhow!(e))?;

    // 创建任务文件
    let task_path = todo_dir.join(format!("{:03}.md", task_id));

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let content = format!(
        "# {}\n\ncreated: {}\npriority: medium\n\n",
        title, timestamp
    );

    std::fs::write(&task_path, content)?;

    if let Some(project_name) = project_path.file_name() {
        println!("✓ 已添加任务到项目 '{}':", project_name.to_string_lossy());
        println!("  {}", title);
    }

    Ok(())
}

/// 打印帮助信息
fn print_help() {
    println!("Helix Kanban (hxk) - 终端看板工具\n");
    println!("用法:");
    println!("  hxk                     启动 TUI 界面");
    println!("  hxk create <名称>        在当前目录创建 .kanban/ 看板");
    println!("  hxk list                列出所有项目");
    println!("  hxk add <标题>           快速添加任务");
    println!("  hxk config [选项]        配置编辑器和预览器");
    println!("  hxk --help              显示此帮助信息");
    println!("  hxk --version           显示版本信息\n");
    println!("配置命令:");
    println!("  hxk config              查看当前配置");
    println!("  hxk config show         查看当前配置");
    println!("  hxk config editor <命令>   设置编辑器");
    println!("  hxk config viewer <命令>   设置 Markdown 预览器\n");
    println!("项目类型:");
    println!("  [G] 全局项目       存储在 ~/.kanban/projects/项目名/");
    println!("  [L] 本地项目       存储在当前目录 .kanban/\n");
    println!("示例:");
    println!("  hxk create 我的项目            # 创建 ./.kanban/ 目录");
    println!("  hxk add 实现新功能             # 添加任务到本地看板");
    println!("  hxk list                      # 列出全局和本地项目");
    println!("  hxk config editor nvim        # 设置编辑器为 neovim");
    println!("  hxk config viewer glow        # 设置预览器为 glow");
}

/// 打印版本信息
fn print_version() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const NAME: &str = env!("CARGO_PKG_NAME");
    println!("{} {}", NAME, VERSION);
}
