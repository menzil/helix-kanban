use crate::app::App;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

/// 渲染预览界面
pub fn render(f: &mut Frame, area: Rect, app: &App) {
    // 渲染半透明背景遮罩
    render_backdrop(f, area);

    // 创建居中弹窗（80% 宽度，90% 高度）
    let popup_area = centered_rect(80, 90, area);

    // 清空弹窗区域
    f.render_widget(Clear, popup_area);

    // 分割区域：顶部标题栏 + 内容区 + 底部状态栏
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),       // 标题栏
            Constraint::Min(0),           // 内容区
            Constraint::Length(1),        // 状态栏
        ])
        .split(popup_area);

    // 渲染标题栏
    render_header(f, chunks[0]);

    // 渲染预览内容
    render_content(f, chunks[1], app);

    // 渲染状态栏
    render_statusbar(f, chunks[2]);
}

/// 渲染半透明背景遮罩
fn render_backdrop(f: &mut Frame, area: Rect) {
    let block = Block::default().style(Style::default().bg(Color::Rgb(0, 0, 0)));
    f.render_widget(block, area);
}

/// 创建居中矩形
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// 渲染标题栏
fn render_header(f: &mut Frame, area: Rect) {
    let title_block = Block::default()
        .title(" 任务预览 ")
        .title_style(
            Style::default()
                .fg(Color::Rgb(136, 192, 208))  // Nord cyan
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(136, 192, 208)))  // Nord cyan
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(Color::Rgb(46, 52, 64)));  // Nord background

    f.render_widget(title_block, area);
}

/// 渲染预览内容
fn render_content(f: &mut Frame, area: Rect, app: &App) {
    // 按行分割内容并进行 Markdown 高亮
    let lines: Vec<Line> = app
        .preview_content
        .lines()
        .skip(app.preview_scroll as usize)
        .map(|line| {
            let trimmed = line.trim_start();

            // 标题（需要严格匹配 # 后面有空格）
            if trimmed.starts_with("# ") {
                // 一级标题 - Nord yellow
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Rgb(235, 203, 139))  // Nord yellow
                        .add_modifier(Modifier::BOLD),
                ))
            } else if trimmed.starts_with("## ") {
                // 二级标题 - Nord frost
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Rgb(136, 192, 208))  // Nord frost
                        .add_modifier(Modifier::BOLD),
                ))
            } else if trimmed.starts_with("### ") || trimmed.starts_with("#### ") {
                // 三级/四级标题 - Nord light blue
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Rgb(129, 161, 193))  // Nord light blue
                        .add_modifier(Modifier::BOLD),
                ))
            } else if trimmed.starts_with("- [ ]") || trimmed.starts_with("* [ ]") {
                // 未完成任务列表 - Nord frost
                Line::from(Span::styled(
                    line,
                    Style::default().fg(Color::Rgb(136, 192, 208))
                ))
            } else if trimmed.starts_with("- [x]") || trimmed.starts_with("* [x]") {
                // 已完成任务列表 - Nord green
                Line::from(Span::styled(
                    line,
                    Style::default().fg(Color::Rgb(163, 190, 140))  // Nord green
                ))
            } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
                // 普通列表 - Nord green
                Line::from(Span::styled(
                    line,
                    Style::default().fg(Color::Rgb(163, 190, 140))
                ))
            } else if trimmed.starts_with("> ") {
                // 引用 - Nord purple (斜体)
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Rgb(180, 142, 173))  // Nord purple
                        .add_modifier(Modifier::ITALIC),
                ))
            } else if trimmed.starts_with("```") {
                // 代码块标记 - Nord orange
                Line::from(Span::styled(
                    line,
                    Style::default().fg(Color::Rgb(208, 135, 112))  // Nord orange
                ))
            } else if trimmed.starts_with("    ") || trimmed.starts_with("\t") {
                // 缩进代码块 - Nord snow storm (dim)
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Rgb(216, 222, 233))
                        .add_modifier(Modifier::DIM),
                ))
            } else {
                // 普通文本 - Nord snow storm
                Line::from(Span::styled(
                    line,
                    Style::default().fg(Color::Rgb(216, 222, 233))
                ))
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(76, 86, 106)))  // Nord comment
                .border_type(ratatui::widgets::BorderType::Rounded)
                .style(Style::default().bg(Color::Rgb(46, 52, 64))),  // Nord background
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// 渲染状态栏
fn render_statusbar(f: &mut Frame, area: Rect) {
    let help_text = Line::from(vec![
        Span::raw("  "),
        Span::styled("j/k", Style::default().fg(Color::Rgb(136, 192, 208))),  // Nord cyan
        Span::raw(" 滚动  "),
        Span::styled("ESC", Style::default().fg(Color::Rgb(136, 192, 208))),  // Nord cyan
        Span::raw(" 返回  "),
    ]);

    let paragraph = Paragraph::new(help_text)
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::Rgb(59, 66, 82)));  // Nord darker background

    f.render_widget(paragraph, area);
}
