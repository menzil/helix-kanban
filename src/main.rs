use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

mod app;
mod cli;
mod config;
mod core;
mod fs;
mod input;
mod models;
mod state;
mod ui;

use app::App;

/// 从临时文件创建新任务
fn handle_new_task_from_file(app: &mut App, temp_file_path: &str) -> Result<()> {
    use crate::models::Task;
    use crate::ui::layout::SplitNode;

    // 读取临时文件内容
    let content = std::fs::read_to_string(temp_file_path)?;

    // 如果文件为空或只有空白，不创建任务
    if content.trim().is_empty() {
        return Ok(());
    }

    // 解析第一行作为任务标题
    let title = content.lines()
        .next()
        .unwrap_or("未命名任务")
        .trim_start_matches('#')
        .trim()
        .to_string();

    // 如果标题为空或是默认模板标题，不创建任务
    if title.is_empty() || title == "任务标题" {
        return Ok(());
    }

    // 获取当前项目名称
    let project_name = if let Some(SplitNode::Leaf { project_id, .. }) =
        app.split_tree.find_pane(app.focused_pane) {
        if let Some(name) = project_id {
            name.clone()
        } else {
            anyhow::bail!("当前面板没有项目");
        }
    } else {
        anyhow::bail!("找不到当前面板");
    };

    // 获取项目路径
    let project_path = if let Some(project) = app.projects.iter().find(|p| &p.name == &project_name) {
        project.path.clone()
    } else {
        anyhow::bail!("在项目列表中找不到项目");
    };

    // 获取下一个任务 ID
    let next_id = crate::fs::get_next_task_id(&project_path)
        .map_err(|e| anyhow::anyhow!(e))?;

    // 获取当前选中的列作为初始状态
    let column = app.selected_column.get(&app.focused_pane).copied().unwrap_or(0);
    let status = app.get_status_name_by_column(column)
        .unwrap_or_else(|| "todo".to_string());

    // 创建任务
    let mut task = Task::new(next_id, title.clone(), status.clone());

    // 保存完整的文件内容到任务文件
    let task_dir = project_path.join(&status);
    std::fs::create_dir_all(&task_dir)?;
    let task_file = task_dir.join(format!("{:03}.md", next_id));
    std::fs::write(&task_file, &content)?;

    // 更新任务的文件路径
    task.file_path = task_file;

    // 重新加载项目以确保任务列表是最新的
    match crate::fs::load_project(&project_path) {
        Ok(updated_project) => {
            if let Some(project) = app.projects.iter_mut().find(|p| p.name == project_name) {
                *project = updated_project;

                // 找到新任务在当前列的索引（应该是最后一个）
                let new_task_idx = project.tasks.iter()
                    .filter(|t| t.status == status)
                    .count()
                    .saturating_sub(1);

                // 自动选中新创建的任务
                app.selected_task_index.insert(app.focused_pane, new_task_idx);
            }
        }
        Err(e) => {
            anyhow::bail!("重新加载项目失败: {}", e);
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    // 处理 CLI 命令
    let should_run_tui = cli::handle_cli()?;

    // 如果 CLI 命令已处理，直接退出
    if !should_run_tui {
        return Ok(());
    }

    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 确保全局 AI 配置文件存在
    let _ = fs::ensure_global_ai_config();

    // 创建应用
    let mut app = App::new()?;

    // 运行应用
    let res = run_app(&mut terminal, &mut app);

    // 恢复终端
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

/// 暂停终端（用于调用外部编辑器）
pub fn suspend_terminal<B>(terminal: &mut Terminal<B>) -> Result<()>
where
    B: ratatui::backend::Backend + std::io::Write,
{
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

/// 恢复终端（从外部编辑器返回）
pub fn resume_terminal<B>(terminal: &mut Terminal<B>) -> Result<()>
where
    B: ratatui::backend::Backend + std::io::Write,
{
    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal.clear()?;
    Ok(())
}

fn run_app<B>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()>
where
    B: ratatui::backend::Backend + std::io::Write,
{
    loop {
        // 清除过期的通知
        app.clear_expired_notification();

        terminal.draw(|f| ui::render(f, app))?;

        // 检查是否需要打开外部编辑器
        if let Some(file_path) = app.pending_editor_file.take() {
            let is_new_task = app.is_new_task_file;
            app.is_new_task_file = false;  // 重置标志

            suspend_terminal(terminal)?;

            // 调用外部编辑器
            if let Err(e) = open_external_editor(&file_path, &app.config.editor) {
                app.show_notification(
                    format!("打开编辑器失败: {}", e),
                    app::NotificationLevel::Error
                );
            }

            resume_terminal(terminal)?;

            if is_new_task {
                // 处理新任务创建
                if let Err(e) = handle_new_task_from_file(app, &file_path) {
                    app.show_notification(
                        format!("创建任务失败: {}", e),
                        app::NotificationLevel::Error
                    );
                }
                // 删除临时文件
                let _ = std::fs::remove_file(&file_path);
            } else {
                // 重新加载项目以获取最新的任务数据（编辑现有任务）
                if let Err(e) = app.reload_current_project() {
                    app.show_notification(
                        format!("重新加载项目失败: {}", e),
                        app::NotificationLevel::Error
                    );
                }
            }
        }

        // 检查是否需要打开外部预览
        if let Some(file_path) = app.pending_preview_file.take() {
            suspend_terminal(terminal)?;

            // 调用外部预览工具
            if let Err(e) = open_external_previewer(&file_path, &app.config.markdown_viewer) {
                app.show_notification(
                    format!("打开预览工具失败: {}", e),
                    app::NotificationLevel::Error
                );
            }

            resume_terminal(terminal)?;
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if !app.handle_key(key) {
                    // 退出前保存状态
                    let state = state::extract_state(app);
                    if let Err(e) = state::save_state(&state) {
                        eprintln!("保存状态失败: {}", e);
                    }
                    return Ok(()); // 退出应用
                }
            }
        }
    }
}

/// 调用外部编辑器打开文件
fn open_external_editor(file_path: &str, editor_cmd: &str) -> Result<()> {
    // 解析编辑器命令（可能包含参数）
    let parts: Vec<&str> = editor_cmd.split_whitespace().collect();
    let (editor, args) = if parts.is_empty() {
        ("vim", vec![])
    } else {
        (parts[0], parts[1..].to_vec())
    };

    let mut cmd = std::process::Command::new(editor);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.arg(file_path);

    let status = cmd.status()?;

    if !status.success() {
        anyhow::bail!("编辑器退出异常: {}", status);
    }

    Ok(())
}

/// 调用外部预览工具查看文件
fn open_external_previewer(file_path: &str, viewer_cmd: &str) -> Result<()> {
    // 解析预览器命令（可能包含参数）
    let parts: Vec<&str> = viewer_cmd.split_whitespace().collect();
    let (viewer, args) = if parts.is_empty() {
        ("cat", vec![])
    } else {
        (parts[0], parts[1..].to_vec())
    };

    let mut cmd = std::process::Command::new(viewer);
    for arg in args {
        cmd.arg(arg);
    }
    cmd.arg(file_path);

    let status = cmd.status()?;

    if !status.success() {
        anyhow::bail!("预览工具退出异常: {}", status);
    }

    // 等待用户按任意键继续
    println!("\n按任意键返回...");
    std::io::stdin().read_line(&mut String::new())?;

    Ok(())
}
