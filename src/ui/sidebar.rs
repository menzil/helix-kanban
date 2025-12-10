use crate::app::App;
use crate::models::ProjectType;
use crate::ui::layout::SplitNode;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

/// 渲染左侧项目侧边栏
pub fn render(f: &mut Frame, area: Rect, app: &App) {
    // 获取当前聚焦面板的项目名称
    let current_project = if let Some(SplitNode::Leaf { project_id, .. }) =
        app.split_tree.find_pane(app.focused_pane)
    {
        project_id.clone()
    } else {
        None
    };

    // 创建项目列表项
    let items: Vec<ListItem> = app
        .projects
        .iter()
        .map(|project| {
            let is_selected = current_project.as_ref() == Some(&project.name);

            // 计算任务数量
            let task_count = project.tasks.len();

            // 项目类型标签
            let type_tag = match project.project_type {
                ProjectType::Global => "[G]",
                ProjectType::Local => "[L]",
            };
            let tag_color = match project.project_type {
                ProjectType::Global => Color::Rgb(136, 192, 208), // 蓝色
                ProjectType::Local => Color::Rgb(163, 190, 140),  // 绿色
            };

            // 构建列表项内容
            let content = if is_selected {
                Line::from(vec![
                    Span::raw(" "),
                    Span::styled(
                        type_tag,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::styled(
                        &project.name,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        format!("{}", task_count),
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::raw(" "),
                    Span::styled(
                        type_tag,
                        Style::default()
                            .fg(tag_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::styled(&project.name, Style::default().fg(Color::Gray)),
                    Span::raw("  "),
                    Span::styled(
                        format!("{}", task_count),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            };

            let style = if is_selected {
                Style::default().bg(Color::Rgb(41, 98, 218)) // 蓝色高亮背景
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    // 创建列表块
    let list = List::new(items).block(
        Block::default()
            .title(" 项目列表 ")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .border_type(ratatui::widgets::BorderType::Rounded),
    );

    f.render_widget(list, area);
}
