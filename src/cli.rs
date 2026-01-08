use anyhow::Result;
use std::env;
use std::path::PathBuf;
use crate::fs;
use crate::models::{Task, ProjectType};

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
        // 新的结构化命令
        "project" => {
            if let Err(e) = handle_project_command(&args[1..]) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            Ok(false)
        }
        "task" => {
            if let Err(e) = handle_task_command(&args[1..]) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            Ok(false)
        }
        "status" => {
            if let Err(e) = handle_status_command(&args[1..]) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            Ok(false)
        }
        // 向后兼容的旧命令
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

// ============================================================================
// Project Commands
// ============================================================================

fn handle_project_command(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        print_project_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "list" => project_list(),
        "info" => {
            if args.len() < 3 {
                return Err("Missing project name\nUsage: hxk project info <name>".to_string());
            }
            project_info(&args[2])
        }
        "create" => {
            if args.len() < 3 {
                return Err("Missing project name\nUsage: hxk project create <name> [--local]".to_string());
            }
            let is_local = args.get(3).map(|s| s.as_str()) == Some("--local");
            project_create(&args[2], is_local)
        }
        "help" | "--help" | "-h" => {
            print_project_usage();
            Ok(())
        }
        cmd => Err(format!("Unknown project command: {}\nRun 'hxk project help' for usage", cmd)),
    }
}

fn print_project_usage() {
    println!("Kanban Project Commands

USAGE:
    hxk project <SUBCOMMAND>

SUBCOMMANDS:
    list              List all projects
    info <name>       Show project information
    create <name>     Create a new global project
    create <name> --local  Create a new local project

EXAMPLES:
    hxk project list
    hxk project info myproject
    hxk project create newproject
    hxk project create localproject --local");
}

fn project_list() -> Result<(), String> {
    let projects = fs::load_all_projects().map_err(|e| e.to_string())?;

    if projects.is_empty() {
        println!("No projects found.");
        return Ok(());
    }

    println!("TYPE  NAME                             PATH");
    println!("----  ------------------------------  ------------------------------------");

    for project in projects {
        let type_marker = match project.project_type {
            ProjectType::Global => "[G]",
            ProjectType::Local => "[L]",
        };

        let path = project.path.to_string_lossy();
        println!("{:<4}  {:<30}  {}", type_marker, project.name, path);
    }

    Ok(())
}

fn project_info(name: &str) -> Result<(), String> {
    let projects = fs::load_all_projects().map_err(|e| e.to_string())?;
    let project = projects.iter()
        .find(|p| p.name == name)
        .ok_or_else(|| format!("Project '{}' not found", name))?;

    println!("Project: {}", project.name);
    println!("Type: {}", match project.project_type {
        ProjectType::Global => "Global (~/.kanban/projects)",
        ProjectType::Local => "Local (.kanban)",
    });
    println!("Path: {}", project.path.to_string_lossy());
    println!("\nStatuses:");
    println!("  NAME                DISPLAY");
    println!("  ------------------  --------------------");

    for status in &project.statuses {
        println!("  {:<18}  {}", status.name, status.display);
    }

    println!("\nTasks: {}", project.tasks.len());

    Ok(())
}

fn project_create(name: &str, is_local: bool) -> Result<(), String> {
    let path = if is_local {
        fs::create_local_project(name)?
    } else {
        fs::create_project(name)?
    };

    println!("Created {} project: {}",
        if is_local { "local" } else { "global" },
        path.to_string_lossy()
    );

    Ok(())
}

// ============================================================================
// Task Commands
// ============================================================================

fn handle_task_command(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        print_task_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "list" => {
            if args.len() < 3 {
                return Err("Missing project name\nUsage: hxk task list <project> [--status <status>]".to_string());
            }
            let status = parse_flag(&args[3..], "--status");
            task_list(&args[2], status)
        }
        "show" => {
            if args.len() < 4 {
                return Err("Missing arguments\nUsage: hxk task show <project> <task-id>".to_string());
            }
            let task_id: u32 = args[3].parse()
                .map_err(|_| "Invalid task ID (must be a number)".to_string())?;
            task_show(&args[2], task_id)
        }
        "create" => {
            if args.len() < 3 {
                return Err("Missing project name\nUsage: hxk task create <project> --status <status> --title <title> [--content <content>]".to_string());
            }
            let status = parse_flag(&args[3..], "--status")
                .ok_or("Missing --status flag".to_string())?;
            let title = parse_flag(&args[3..], "--title")
                .ok_or("Missing --title flag".to_string())?;
            let content = parse_flag(&args[3..], "--content");
            task_create(&args[2], &status, &title, content)
        }
        "update" => {
            if args.len() < 4 {
                return Err("Missing arguments\nUsage: hxk task update <project> <task-id> [--title <title>] [--content <content>] [--priority <priority>]".to_string());
            }
            let task_id: u32 = args[3].parse()
                .map_err(|_| "Invalid task ID (must be a number)".to_string())?;
            let title = parse_flag(&args[4..], "--title");
            let content = parse_flag(&args[4..], "--content");
            let priority = parse_flag(&args[4..], "--priority");
            task_update(&args[2], task_id, title, content, priority)
        }
        "move" => {
            if args.len() < 4 {
                return Err("Missing arguments\nUsage: hxk task move <project> <task-id> --to <status>".to_string());
            }
            let task_id: u32 = args[3].parse()
                .map_err(|_| "Invalid task ID (must be a number)".to_string())?;
            let to_status = parse_flag(&args[4..], "--to")
                .ok_or("Missing --to flag".to_string())?;
            task_move(&args[2], task_id, &to_status)
        }
        "delete" => {
            if args.len() < 4 {
                return Err("Missing arguments\nUsage: hxk task delete <project> <task-id>".to_string());
            }
            let task_id: u32 = args[3].parse()
                .map_err(|_| "Invalid task ID (must be a number)".to_string())?;
            task_delete(&args[2], task_id)
        }
        "help" | "--help" | "-h" => {
            print_task_usage();
            Ok(())
        }
        cmd => Err(format!("Unknown task command: {}\nRun 'hxk task help' for usage", cmd)),
    }
}

fn print_task_usage() {
    println!("Kanban Task Commands

USAGE:
    hxk task <SUBCOMMAND>

SUBCOMMANDS:
    list <project> [--status <status>]
        List tasks in a project

    show <project> <task-id>
        Show task details

    create <project> --status <status> --title <title> [--content <content>]
        Create a new task

    update <project> <task-id> [--title <title>] [--content <content>] [--priority <priority>]
        Update task properties

    move <project> <task-id> --to <status>
        Move task to a different status

    delete <project> <task-id>
        Delete a task

EXAMPLES:
    hxk task list myproject
    hxk task list myproject --status todo
    hxk task show myproject 42
    hxk task create myproject --status todo --title \"新任务\"
    hxk task update myproject 42 --priority high
    hxk task move myproject 42 --to doing
    hxk task delete myproject 42");
}

fn parse_flag<'a>(args: &'a [String], flag: &str) -> Option<String> {
    args.iter()
        .position(|s| s == flag)
        .and_then(|i| args.get(i + 1))
        .map(|s| s.clone())
}

fn find_project_path(project_name: &str) -> Result<PathBuf, String> {
    let projects = fs::load_all_projects().map_err(|e| e.to_string())?;
    projects.iter()
        .find(|p| p.name == project_name)
        .map(|p| p.path.clone())
        .ok_or_else(|| format!("Project '{}' not found", project_name))
}

fn task_list(project_name: &str, filter_status: Option<String>) -> Result<(), String> {
    let project_path = find_project_path(project_name)?;
    let project = fs::load_project(&project_path)?;

    let tasks: Vec<&Task> = if let Some(status) = filter_status {
        project.tasks.iter().filter(|t| t.status == status).collect()
    } else {
        project.tasks.iter().collect()
    };

    if tasks.is_empty() {
        println!("No tasks found.");
        return Ok(());
    }

    println!("ID    ORDER  TITLE                                PRIORITY  STATUS    TAGS");
    println!("----  -----  -----------------------------------  --------  --------  --------");

    for task in tasks {
        let priority = task.priority.as_deref().unwrap_or("-");
        let tags = task.tags.join(", ");
        let tags_display = if tags.is_empty() { "-".to_string() } else { tags };

        println!("{:<4}  {:<5}  {:<35}  {:<8}  {:<8}  {}",
            task.id,
            task.order,
            truncate(&task.title, 35),
            priority,
            truncate(&task.status, 8),
            truncate(&tags_display, 20)
        );
    }

    Ok(())
}

fn task_show(project_name: &str, task_id: u32) -> Result<(), String> {
    let project_path = find_project_path(project_name)?;
    let project = fs::load_project(&project_path)?;

    let task = project.tasks.iter()
        .find(|t| t.id == task_id)
        .ok_or_else(|| format!("Task {} not found", task_id))?;

    println!("Task #{}", task.id);
    println!("Title: {}", task.title);
    println!("Status: {}", task.status);
    println!("Order: {}", task.order);
    println!("Priority: {}", task.priority.as_deref().unwrap_or("-"));
    println!("Tags: {}", if task.tags.is_empty() { "-".to_string() } else { task.tags.join(", ") });
    println!("Created: {}", task.created);
    println!("File: {}", task.file_path.to_string_lossy());
    println!("\nContent:");
    println!("{}", task.content);

    Ok(())
}

fn task_create(project_name: &str, status: &str, title: &str, content: Option<String>) -> Result<(), String> {
    let project_path = find_project_path(project_name)?;

    // Get next task ID
    let next_id = fs::get_next_task_id(&project_path)?;

    // Get max order for the status
    let max_order = fs::get_max_order_in_status(&project_path, status)?;
    let new_order = max_order + 1000;

    // Create task
    let mut task = Task::new(next_id, title.to_string(), status.to_string());
    task.order = new_order;
    task.content = content.unwrap_or_default();

    // Save task
    let file_path = fs::save_task(&project_path, &task)?;

    println!("Created task #{} in status '{}'", task.id, status);
    println!("File: {}", file_path.to_string_lossy());

    Ok(())
}

fn task_update(
    project_name: &str,
    task_id: u32,
    title: Option<String>,
    content: Option<String>,
    priority: Option<String>,
) -> Result<(), String> {
    let project_path = find_project_path(project_name)?;
    let mut project = fs::load_project(&project_path)?;

    let task = project.tasks.iter_mut()
        .find(|t| t.id == task_id)
        .ok_or_else(|| format!("Task {} not found", task_id))?;

    let mut updated = false;

    if let Some(new_title) = title {
        task.title = new_title;
        updated = true;
    }

    if let Some(new_content) = content {
        task.content = new_content;
        updated = true;
    }

    if let Some(new_priority) = priority {
        task.priority = if new_priority == "none" || new_priority.is_empty() {
            None
        } else {
            Some(new_priority)
        };
        updated = true;
    }

    if updated {
        fs::save_task(&project_path, task)?;
        println!("Updated task #{}", task_id);
    } else {
        println!("No changes made to task #{}", task_id);
    }

    Ok(())
}

fn task_move(project_name: &str, task_id: u32, new_status: &str) -> Result<(), String> {
    let project_path = find_project_path(project_name)?;
    let mut project = fs::load_project(&project_path)?;

    let task = project.tasks.iter_mut()
        .find(|t| t.id == task_id)
        .ok_or_else(|| format!("Task {} not found", task_id))?;

    let old_status = task.status.clone();
    task.status = new_status.to_string();

    // Move file and update task
    let new_path = fs::move_task(&project_path, task, new_status)?;
    task.file_path = new_path;

    println!("Moved task #{} from '{}' to '{}'", task_id, old_status, new_status);

    Ok(())
}

fn task_delete(project_name: &str, task_id: u32) -> Result<(), String> {
    let project_path = find_project_path(project_name)?;
    let project = fs::load_project(&project_path)?;

    let task = project.tasks.iter()
        .find(|t| t.id == task_id)
        .ok_or_else(|| format!("Task {} not found", task_id))?;

    fs::delete_task(task)?;

    println!("Deleted task #{}", task_id);

    Ok(())
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else {
        format!("{}...", s.chars().take(max_len - 3).collect::<String>())
    }
}

// ============================================================================
// Status Commands
// ============================================================================

fn handle_status_command(args: &[String]) -> Result<(), String> {
    if args.len() < 2 {
        print_status_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "list" => {
            if args.len() < 3 {
                return Err("Missing project name\nUsage: hxk status list <project>".to_string());
            }
            status_list(&args[2])
        }
        "create" => {
            if args.len() < 4 {
                return Err("Missing arguments\nUsage: hxk status create <project> <name> [--display <display-name>]".to_string());
            }
            let display = parse_flag(&args[4..], "--display")
                .unwrap_or_else(|| args[3].clone());
            status_create(&args[2], &args[3], &display)
        }
        "rename" => {
            if args.len() < 5 {
                return Err("Missing arguments\nUsage: hxk status rename <project> <old-name> <new-name>".to_string());
            }
            status_rename(&args[2], &args[3], &args[4])
        }
        "delete" => {
            if args.len() < 4 {
                return Err("Missing arguments\nUsage: hxk status delete <project> <name> [--move-to <target>]".to_string());
            }
            let move_to = parse_flag(&args[4..], "--move-to");
            status_delete(&args[2], &args[3], move_to.as_deref())
        }
        "help" | "--help" | "-h" => {
            print_status_usage();
            Ok(())
        }
        cmd => Err(format!("Unknown status command: {}\nRun 'hxk status help' for usage", cmd)),
    }
}

fn print_status_usage() {
    println!("Kanban Status Commands

USAGE:
    hxk status <SUBCOMMAND>

SUBCOMMANDS:
    list <project>
        List all statuses in a project

    create <project> <name> [--display <display-name>]
        Create a new status column

    rename <project> <old-name> <new-name>
        Rename a status (both internal name and display name)

    delete <project> <name> [--move-to <target>]
        Delete a status (move tasks to target if specified)

EXAMPLES:
    hxk status list myproject
    hxk status create myproject review --display \"Review\"
    hxk status rename myproject todo backlog
    hxk status delete myproject archived --move-to done");
}

fn status_list(project_name: &str) -> Result<(), String> {
    let project_path = find_project_path(project_name)?;
    let project = fs::load_project(&project_path)?;

    if project.statuses.is_empty() {
        println!("No statuses found.");
        return Ok(());
    }

    println!("NAME                DISPLAY              TASKS");
    println!("------------------  -------------------  -----");

    for status in &project.statuses {
        let task_count = project.tasks.iter()
            .filter(|t| t.status == status.name)
            .count();

        println!("{:<18}  {:<19}  {}",
            status.name,
            status.display,
            task_count
        );
    }

    Ok(())
}

fn status_create(project_name: &str, name: &str, display: &str) -> Result<(), String> {
    let project_path = find_project_path(project_name)?;

    fs::status::create_status(&project_path, name, display)?;

    println!("Created status '{}' with display name '{}'", name, display);

    Ok(())
}

fn status_rename(project_name: &str, old_name: &str, new_name: &str) -> Result<(), String> {
    let project_path = find_project_path(project_name)?;

    fs::status::rename_status(&project_path, old_name, new_name, new_name)?;

    println!("Renamed status from '{}' to '{}'", old_name, new_name);

    Ok(())
}

fn status_delete(project_name: &str, name: &str, move_to: Option<&str>) -> Result<(), String> {
    let project_path = find_project_path(project_name)?;

    fs::status::delete_status(&project_path, name, move_to)?;

    if let Some(target) = move_to {
        println!("Deleted status '{}' and moved tasks to '{}'", name, target);
    } else {
        println!("Deleted status '{}'", name);
    }

    Ok(())
}

// ============================================================================
// Legacy Commands (Backward Compatibility)
// ============================================================================

/// 初始化本地看板（已废弃，保留以兼容旧版本）
fn cli_init() -> Result<()> {
    println!("提示: 'kanban init' 命令已废弃");
    println!("本地项目会自动创建在 .kanban/ 目录");
    println!("\n使用 'kanban create <name>' 创建本地看板");

    Ok(())
}

/// 创建本地项目
fn cli_create(name: &str) -> Result<()> {
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
    let next_id = fs::get_next_task_id(&project_path).map_err(|e| anyhow::anyhow!(e))?;

    // 获取 todo 状态的最大 order
    let max_order = fs::get_max_order_in_status(&project_path, "todo").map_err(|e| anyhow::anyhow!(e))?;
    let new_order = max_order + 1000;

    // 创建任务
    let mut task = Task::new(next_id, title.clone(), "todo".to_string());
    task.order = new_order;

    // 保存任务
    fs::save_task(&project_path, &task).map_err(|e| anyhow::anyhow!(e))?;

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
    println!("  hxk <命令> [参数]         运行 CLI 命令");
    println!("  hxk --help              显示此帮助信息");
    println!("  hxk --version           显示版本信息\n");

    println!("CLI 命令（AI 友好接口）:");
    println!("  project                项目管理命令");
    println!("  task                   任务管理命令");
    println!("  status                 状态列管理命令\n");

    println!("传统命令（向后兼容）:");
    println!("  create <名称>           在当前目录创建 .kanban/ 看板");
    println!("  list                   列出所有项目");
    println!("  add <标题>              快速添加任务");
    println!("  config [选项]           配置编辑器和预览器\n");

    println!("详细用法:");
    println!("  hxk project --help     查看项目管理命令");
    println!("  hxk task --help        查看任务管理命令");
    println!("  hxk status --help      查看状态管理命令\n");

    println!("项目类型:");
    println!("  [G] 全局项目       存储在 ~/.kanban/projects/项目名/");
    println!("  [L] 本地项目       存储在当前目录 .kanban/\n");

    println!("示例:");
    println!("  # 交互式 TUI");
    println!("  hxk\n");

    println!("  # CLI 命令（AI 友好）");
    println!("  hxk project list");
    println!("  hxk task list myproject --status todo");
    println!("  hxk task create myproject --status todo --title \"新任务\"");
    println!("  hxk status create myproject review --display \"Review\"\n");

    println!("  # 传统命令");
    println!("  hxk create 我的项目");
    println!("  hxk add 实现新功能");
    println!("  hxk config editor nvim\n");

    println!("AI 协作:");
    println!("  在 TUI 中按 Space+p+i 复制项目看板路径");
    println!("  AI 配置文件: ~/.kanban/.ai-config.json（自动生成）");
    println!("  CLI 命令提供简洁的文本输出，易于 AI 解析和操作");
}

/// 打印版本信息
fn print_version() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const NAME: &str = env!("CARGO_PKG_NAME");
    println!("{} {}", NAME, VERSION);
}
