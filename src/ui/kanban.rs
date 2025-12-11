use crate::app::App;
use crate::models::Project;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// 渲染看板视图
pub fn render(f: &mut Frame, area: Rect, project: &Project, is_focused: bool, app: &App) {
    let border_style = if is_focused {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // 计算任务统计
    let todo_count = project.tasks.iter().filter(|t| t.status == "todo").count();
    let doing_count = project.tasks.iter().filter(|t| t.status == "doing").count();
    let done_count = project.tasks.iter().filter(|t| t.status == "done").count();
    let total_count = project.tasks.len();

    let title = format!(" {} ({}/{}) ", project.name, done_count, total_count);

    let block = Block::default()
        .title(title)
        .title_alignment(ratatui::layout::Alignment::Center)
        .borders(Borders::ALL)
        .border_style(border_style)
        .border_type(ratatui::widgets::BorderType::Rounded);

    let inner = block.inner(area);
    f.render_widget(block, area);

    // 三列布局：待办 | 进行中 | 已完成
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(inner);

    // 按状态分类任务
    let todo_tasks: Vec<_> = project.tasks.iter().filter(|t| t.status == "todo").collect();
    let doing_tasks: Vec<_> = project.tasks.iter().filter(|t| t.status == "doing").collect();
    let done_tasks: Vec<_> = project.tasks.iter().filter(|t| t.status == "done").collect();

    // 渲染三列
    render_column(f, columns[0], "待办", &todo_tasks, 0, app, is_focused);
    render_column(f, columns[1], "进行中", &doing_tasks, 1, app, is_focused);
    render_column(f, columns[2], "已完成", &done_tasks, 2, app, is_focused);
}

/// 渲染单个列
fn render_column(
    f: &mut Frame,
    area: Rect,
    title: &str,
    tasks: &[&crate::models::Task],
    column_idx: usize,
    app: &App,
    is_pane_focused: bool,
) {
    let current_column = app.selected_column.get(&app.focused_pane).copied().unwrap_or(0);
    let is_column_focused = is_pane_focused && current_column == column_idx;

    // 简洁配色：聚焦=白色，非聚焦=灰色
    let (border_color, title_style) = if is_column_focused {
        (Color::White, Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
    } else {
        (Color::DarkGray, Style::default().fg(Color::Gray))
    };

    let items: Vec<ListItem> = tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let selected_idx = app.selected_task_index.get(&app.focused_pane).copied().unwrap_or(0);
            let is_selected = is_column_focused && i == selected_idx;

            // 只有选中的任务高亮，其他使用默认样式
            let style = if is_selected {
                Style::default()
                    .bg(Color::Rgb(41, 98, 218))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // 优先级指示器
            let priority_indicator = match task.priority.as_deref() {
                Some("high") => Span::styled("● ", Style::default().fg(Color::Red)),
                Some("medium") => Span::styled("● ", Style::default().fg(Color::Yellow)),
                Some("low") => Span::styled("● ", Style::default().fg(Color::Blue)),
                _ => Span::raw("  "),
            };

            // 选中指示器
            let selection_indicator = if is_selected {
                Span::styled("▶ ", Style::default().fg(Color::White))
            } else {
                Span::raw("  ")
            };

            // 任务项（紧凑布局，无额外间距）
            ListItem::new(Line::from(vec![
                Span::raw(" "),
                selection_indicator,
                priority_indicator,
                Span::raw(&task.title),
                Span::raw(" "),
            ]))
            .style(style)
        })
        .collect();

    // 列标题
    let title_with_count = format!(" {} ({}) ", title, tasks.len());

    let list = List::new(items).block(
        Block::default()
            .title(title_with_count)
            .title_alignment(ratatui::layout::Alignment::Center)
            .title_style(title_style)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .border_type(ratatui::widgets::BorderType::Rounded),
    );

    f.render_widget(list, area);
}
