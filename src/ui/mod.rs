pub mod command_completion;
pub mod command_menu;
pub mod dialogs;
pub mod help;
mod kanban;
pub mod layout;
pub mod preview;
mod sidebar;
mod statusbar;
pub mod welcome;

use crate::app::App;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

/// 主渲染函数
pub fn render(f: &mut Frame, app: &App) {
    // 不显示状态栏，使用全屏布局
    // let main_chunks = Layout::default()
    //     .direction(Direction::Vertical)
    //     .constraints([
    //         Constraint::Min(0),      // 主内容区域
    //         Constraint::Length(1),  // 状态栏
    //     ])
    //     .split(f.area());

    // 渲染分屏内容（全屏）
    render_split_tree(f, f.area(), &app.split_tree, app);

    // 渲染状态栏（已注释）
    // statusbar::render(f, main_chunks[1], app);

    // 渲染对话框（如果有）
    if let Some(dialog) = &app.dialog {
        dialogs::render_dialog(f, dialog);
    }

    // 渲染帮助面板（如果处于帮助模式）
    if app.mode == crate::app::Mode::Help {
        help::render(f, f.area());
    }

    // 渲染预览面板（如果处于预览模式）
    if app.mode == crate::app::Mode::Preview {
        preview::render(f, f.area(), app);
    }

    // 渲染命令菜单（如果处于空格菜单模式）
    if app.mode == crate::app::Mode::SpaceMenu {
        command_menu::render(f, f.area(), app);
    }

    // 渲染命令补全（如果处于命令模式且有输入）
    if app.mode == crate::app::Mode::Command {
        command_completion::render(f, f.area(), app);
    }

    // 渲染欢迎对话框（如果是首次运行）
    if app.show_welcome_dialog {
        welcome::render(f, f.area(), &app.config);
    }

    // 渲染通知栏（如果有通知）
    if let Some(ref notification) = app.notification {
        render_notification(f, f.area(), notification);
    }
}

/// 渲染通知栏
fn render_notification(f: &mut Frame, area: ratatui::layout::Rect, notification: &crate::app::Notification) {
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, Borders, Paragraph};
    use crate::app::NotificationLevel;

    // 通知栏占据顶部 3 行
    let notification_area = ratatui::layout::Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 3,
    };

    // 根据级别选择颜色
    let (bg_color, fg_color, prefix) = match notification.level {
        NotificationLevel::Info => (Color::Blue, Color::White, "ℹ"),
        NotificationLevel::Success => (Color::Green, Color::White, "✓"),
        NotificationLevel::Warning => (Color::Yellow, Color::Black, "⚠"),
        NotificationLevel::Error => (Color::Red, Color::White, "✗"),
    };

    let content = Line::from(vec![
        Span::styled(format!(" {} ", prefix), Style::default().fg(fg_color).bg(bg_color).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(&notification.message, Style::default().fg(fg_color)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(bg_color))
        .style(Style::default().bg(bg_color));

    let paragraph = Paragraph::new(content).block(block);

    f.render_widget(paragraph, notification_area);
}

/// 递归渲染分屏树
fn render_split_tree(f: &mut Frame, area: ratatui::layout::Rect, node: &layout::SplitNode, app: &App) {
    use layout::SplitNode;

    match node {
        SplitNode::Leaf { project_id, id } => {
            let is_focused = *id == app.focused_pane;
            if let Some(pid) = project_id {
                if let Some(project) = app.projects.iter().find(|p| &p.name == pid) {
                    kanban::render(f, area, project, is_focused, app);
                } else {
                    render_empty_pane(f, area, "项目未找到", is_focused);
                }
            } else {
                render_empty_pane(f, area, "无项目 - 按 Space p o 打开项目", is_focused);
            }
        }
        SplitNode::Horizontal { left, right, ratio } => {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage((ratio * 100.0) as u16),
                    Constraint::Percentage(((1.0 - ratio) * 100.0) as u16),
                ])
                .split(area);

            render_split_tree(f, chunks[0], left, app);
            render_split_tree(f, chunks[1], right, app);
        }
        SplitNode::Vertical { top, bottom, ratio } => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage((ratio * 100.0) as u16),
                    Constraint::Percentage(((1.0 - ratio) * 100.0) as u16),
                ])
                .split(area);

            render_split_tree(f, chunks[0], top, app);
            render_split_tree(f, chunks[1], bottom, app);
        }
    }
}

/// 渲染空面板
fn render_empty_pane(f: &mut Frame, area: ratatui::layout::Rect, message: &str, is_focused: bool) {
    use ratatui::style::{Color, Style};
    use ratatui::widgets::{Block, Borders, Paragraph};

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .border_type(ratatui::widgets::BorderType::Rounded);

    let paragraph = Paragraph::new(message)
        .block(block)
        .style(Style::default().fg(Color::Gray));

    f.render_widget(paragraph, area);
}
