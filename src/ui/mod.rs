pub mod command_completion;
pub mod command_menu;
pub mod dialogs;
pub mod help;
mod kanban;
pub mod layout;
pub mod preview;
mod sidebar;
mod statusbar;
pub mod text_input;
pub mod welcome;

use crate::app::App;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};

/// 主渲染函数
pub fn render(f: &mut Frame, app: &mut App) {
    // 克隆 split_tree 以避免借用冲突
    let split_tree = app.split_tree.clone();

    // 渲染分屏内容（全屏）
    render_split_tree(f, f.area(), &split_tree, app);

    // 渲染状态栏（已注释）
    // statusbar::render(f, main_chunks[1], app);

    // 渲染对话框（如果有）
    if let Some(dialog) = &mut app.dialog {
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

    // 渲染搜索条（如果处于搜索模式）
    if app.mode == crate::app::Mode::Search {
        render_search_bar(f, f.area(), app);
    }

    // 渲染命令菜单（如果处于空格菜单模式）
    if app.mode == crate::app::Mode::SpaceMenu {
        command_menu::render(f, f.area(), app);
    }

    // 渲染命令补全（如果处于命令模式且有输入） - 已注释
    // if app.mode == crate::app::Mode::Command {
    //     command_completion::render(f, f.area(), app);
    // }

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
fn render_notification(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    notification: &crate::app::Notification,
) {
    use crate::app::NotificationLevel;
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, Borders, Clear, Paragraph};

    // 通知栏占据底部 3 行
    let notification_height = 3;
    let notification_area = ratatui::layout::Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(notification_height),
        width: area.width,
        height: notification_height,
    };

    // 根据级别选择颜色
    let (bg_color, fg_color, prefix) = match notification.level {
        NotificationLevel::Info => (Color::Blue, Color::White, "ℹ"),
        NotificationLevel::Success => (Color::Green, Color::White, "✓"),
        NotificationLevel::Warning => (Color::Yellow, Color::Black, "⚠"),
        NotificationLevel::Error => (Color::Red, Color::White, "✗"),
    };

    // 先清除背景区域
    f.render_widget(Clear, notification_area);

    // 构建内容，所有 Span 都设置背景色
    let content = Line::from(vec![
        Span::styled(
            format!(" {} ", prefix),
            Style::default()
                .fg(fg_color)
                .bg(bg_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {} ", notification.message),
            Style::default().fg(fg_color).bg(bg_color),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(fg_color).bg(bg_color))
        .style(Style::default().bg(bg_color));

    let paragraph = Paragraph::new(content)
        .block(block)
        .style(Style::default().bg(bg_color));

    f.render_widget(paragraph, notification_area);
}

/// 渲染搜索条
fn render_search_bar(f: &mut Frame, area: ratatui::layout::Rect, app: &mut App) {
    use ratatui::layout::Rect;
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, Borders, Clear, Paragraph};

    let search_height = 3;
    let search_area = Rect {
        x: 0,
        y: area.height - search_height,
        width: area.width,
        height: search_height,
    };

    // 先清除搜索区域背景
    f.render_widget(Clear, search_area);

    // 实心背景
    let bg_color = Color::Rgb(46, 52, 64);
    let fg_color = Color::Rgb(236, 239, 244);
    let accent_color = Color::Rgb(136, 192, 208);

    let state = match &app.search_state {
        Some(s) => s,
        None => return,
    };

    // 构建搜索条内容
    let match_count = state.matches.len();
    let selected = state.selected;

    let status_text = if match_count == 0 {
        if state.query.is_empty() {
            "Type to search...".to_string()
        } else {
            "No matches".to_string()
        }
    } else {
        format!("{}/{}", selected + 1, match_count)
    };

    // 根据模式显示不同的提示
    let (tip1, tip2, tip3) = if state.selecting {
        ("j/k/h/l:nav", "enter:go", "esc:back")
    } else {
        ("enter:select", "esc:quit", "")
    };

    let mut spans = vec![
        Span::styled(" Search: ", Style::default().fg(accent_color).bg(bg_color)),
        Span::styled(
            &state.query,
            Style::default().fg(fg_color).bg(bg_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", Style::default().bg(bg_color)),
        Span::styled(&status_text, Style::default().fg(Color::Rgb(76, 86, 106)).bg(bg_color)),
        Span::styled(" matches  ", Style::default().fg(Color::Rgb(76, 86, 106)).bg(bg_color)),
        Span::styled(tip1, Style::default().fg(accent_color).bg(bg_color)),
    ];

    if !tip2.is_empty() {
        spans.push(Span::styled("  ", Style::default().bg(bg_color)));
        spans.push(Span::styled(tip2, Style::default().fg(accent_color).bg(bg_color)));
    }
    if !tip3.is_empty() {
        spans.push(Span::styled("  ", Style::default().bg(bg_color)));
        spans.push(Span::styled(tip3, Style::default().fg(accent_color).bg(bg_color)));
    }

    let content = Line::from(spans);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent_color).bg(bg_color))
        .style(Style::default().bg(bg_color));

    let paragraph = Paragraph::new(content)
        .block(block)
        .style(Style::default().bg(bg_color));

    f.render_widget(paragraph, search_area);
}

/// 递归渲染分屏树
fn render_split_tree(
    f: &mut Frame,
    area: ratatui::layout::Rect,
    node: &layout::SplitNode,
    app: &mut App,
) {
    use layout::SplitNode;

    match node {
        SplitNode::Leaf { project_id, id } => {
            let is_focused = *id == app.focused_pane;
            if let Some(pid) = project_id {
                // 克隆项目以避免借用冲突
                if let Some(project) = app.projects.iter().find(|p| &p.name == pid).cloned() {
                    kanban::render(f, area, &project, is_focused, app);
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
