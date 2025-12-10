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
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(ratatui::widgets::BorderType::Rounded);

    f.render_widget(title_block, area);
}

/// 渲染预览内容
fn render_content(f: &mut Frame, area: Rect, app: &App) {
    // 简单地按行分割内容
    let lines: Vec<Line> = app
        .preview_content
        .lines()
        .skip(app.preview_scroll as usize)
        .map(|line| {
            // 简单的 markdown 高亮
            if line.starts_with("# ") {
                // 一级标题
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if line.starts_with("## ") || line.starts_with("### ") {
                // 二级/三级标题
                Line::from(Span::styled(
                    line,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
            } else if line.starts_with("- ") || line.starts_with("* ") {
                // 列表
                Line::from(Span::styled(line, Style::default().fg(Color::Green)))
            } else if line.starts_with("```") {
                // 代码块标记
                Line::from(Span::styled(line, Style::default().fg(Color::Magenta)))
            } else {
                // 普通文本
                Line::from(Span::raw(line))
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        )
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

/// 渲染状态栏
fn render_statusbar(f: &mut Frame, area: Rect) {
    let help_text = Line::from(vec![
        Span::raw("  "),
        Span::styled("j/k", Style::default().fg(Color::Cyan)),
        Span::raw(" 滚动  "),
        Span::styled("ESC", Style::default().fg(Color::Cyan)),
        Span::raw(" 返回  "),
    ]);

    let paragraph = Paragraph::new(help_text)
        .alignment(Alignment::Left)
        .style(Style::default().bg(Color::Rgb(40, 44, 52)));

    f.render_widget(paragraph, area);
}
