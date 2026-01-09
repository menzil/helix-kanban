use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

/// 渲染首次运行欢迎对话框
pub fn render(f: &mut Frame, area: Rect, config: &crate::config::Config) {
    // 渲染半透明背景遮罩
    render_backdrop(f, area);

    // 创建居中的弹窗区域
    let popup_area = centered_rect(70, 60, area);

    // 清空弹窗区域
    f.render_widget(Clear, popup_area);

    // 弹窗外框
    let block = Block::default()
        .title(" 🎉 欢迎使用 Kanban！ ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(Color::Black));

    f.render_widget(block.clone(), popup_area);

    let inner = block.inner(popup_area);

    // 构建欢迎信息内容
    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "感谢您使用 Kanban！",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("已自动为您检测并配置以下工具："),
        Line::from(""),
        Line::from(vec![
            Span::styled("  编辑器:         ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &config.editor,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Markdown 预览: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                &config.markdown_viewer,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "配置文件位置:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{}", crate::config::get_config_path().display()),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "如需修改配置，请使用以下命令:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  kanban config editor <命令>",
                Style::default().fg(Color::Green),
            ),
            Span::styled("       # 设置编辑器", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled(
                "  kanban config viewer <命令>",
                Style::default().fg(Color::Green),
            ),
            Span::styled("       # 设置预览器", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("  kanban config show", Style::default().fg(Color::Green)),
            Span::styled(
                "                # 查看当前配置",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "快捷键提示:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ?", Style::default().fg(Color::Yellow)),
            Span::raw("        查看完整帮助"),
        ]),
        Line::from(vec![
            Span::styled("  Space", Style::default().fg(Color::Yellow)),
            Span::raw("    打开命令菜单"),
        ]),
        Line::from(vec![
            Span::styled("  :q", Style::default().fg(Color::Yellow)),
            Span::raw("       退出应用"),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "按任意键开始使用",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
    ];

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, inner);
}

/// 渲染半透明背景遮罩
fn render_backdrop(f: &mut Frame, area: Rect) {
    let block = Block::default().style(Style::default().bg(Color::Rgb(0, 0, 0)));
    f.render_widget(block, area);
}

/// 创建一个居中的矩形区域
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
