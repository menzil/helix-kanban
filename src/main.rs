use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
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
pub fn suspend_terminal<B: ratatui::backend::Backend + std::io::Write>(terminal: &mut Terminal<B>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

/// 恢复终端（从外部编辑器返回）
pub fn resume_terminal<B: ratatui::backend::Backend + std::io::Write>(terminal: &mut Terminal<B>) -> Result<()> {
    enable_raw_mode()?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    terminal.clear()?;
    Ok(())
}

fn run_app<B: ratatui::backend::Backend + std::io::Write>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        // 检查是否需要打开外部编辑器
        if let Some(file_path) = app.pending_editor_file.take() {
            suspend_terminal(terminal)?;

            // 调用外部编辑器
            if let Err(e) = open_external_editor(&file_path, &app.config.editor) {
                eprintln!("打开编辑器失败: {}", e);
            }

            resume_terminal(terminal)?;

            // 重新加载项目以获取最新的任务数据
            if let Err(e) = app.reload_current_project() {
                eprintln!("重新加载项目失败: {}", e);
            }
        }

        // 检查是否需要打开外部预览
        if let Some(file_path) = app.pending_preview_file.take() {
            suspend_terminal(terminal)?;

            // 调用外部预览工具
            if let Err(e) = open_external_previewer(&file_path, &app.config.markdown_viewer) {
                eprintln!("打开预览工具失败: {}", e);
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

    Ok(())
}
