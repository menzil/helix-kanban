use crate::app::App;
use crate::models::Project;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
};

/// 根据标签名生成颜色（使用哈希）
fn tag_color(tag: &str) -> Color {
    let mut hash: u32 = 0;
    for byte in tag.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u32);
    }

    // 使用预定义的柔和颜色列表
    let colors = [
        Color::Rgb(100, 149, 237), // 蓝色
        Color::Rgb(144, 238, 144), // 绿色
        Color::Rgb(255, 182, 193), // 粉色
        Color::Rgb(255, 218, 185), // 橙色
        Color::Rgb(221, 160, 221), // 紫色
        Color::Rgb(173, 216, 230), // 浅蓝
        Color::Rgb(144, 238, 144), // 浅绿
        Color::Rgb(240, 230, 140), // 黄色
    ];

    colors[(hash as usize) % colors.len()]
}

/// 渲染看板视图
pub fn render(f: &mut Frame, area: Rect, project: &Project, is_focused: bool, app: &App) {
    let border_style = if is_focused {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // 计算任务统计
    let total_count = project.tasks.len();
    let done_count = project
        .tasks
        .iter()
        .filter(|t| {
            // 找到最后一个状态作为"已完成"状态
            if let Some(last_status) = project.statuses.last() {
                t.status == last_status.name
            } else {
                false
            }
        })
        .count();

    // 添加项目类型标记
    let project_type_label = match project.project_type {
        crate::models::ProjectType::Global => "[G]",
        crate::models::ProjectType::Local => "[L]",
    };

    let title = format!(
        " {} {} ({}/{}) ",
        project_type_label, project.name, done_count, total_count
    );

    let block = Block::default()
        .title(title)
        .title_alignment(ratatui::layout::Alignment::Center)
        .borders(Borders::ALL)
        .border_style(border_style)
        .border_type(ratatui::widgets::BorderType::Rounded);

    let inner = block.inner(area);
    f.render_widget(block, area);

    // 动态列布局：根据状态数量创建等宽列
    let num_columns = project.statuses.len();
    if num_columns == 0 {
        return;
    }

    // 获取当前项目的列宽配置
    let constraints: Vec<Constraint> =
        if let Some(widths) = app.config.column_widths.get(&project.name) {
            // 使用配置的宽度（确保列数匹配）
            if widths.len() == num_columns {
                widths.iter().map(|&w| Constraint::Percentage(w)).collect()
            } else {
                // 列数不匹配，使用默认等宽
                vec![Constraint::Fill(1); num_columns]
            }
        } else if let Some(Some(max_col)) = app.config.maximized_column.get(&project.name) {
            // 最大化模式：一列占 90%，其他列平分 10%
            (0..num_columns)
                .map(|i| {
                    if i == *max_col {
                        Constraint::Percentage(90)
                    } else {
                        let remaining = if num_columns > 1 {
                            10 / (num_columns - 1) as u16
                        } else {
                            0
                        };
                        Constraint::Percentage(remaining)
                    }
                })
                .collect()
        } else {
            // 默认等宽
            vec![Constraint::Fill(1); num_columns]
        };

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .flex(Flex::Start)
        .split(inner);

    // 渲染每一列
    for (col_idx, status) in project.statuses.iter().enumerate() {
        let tasks: Vec<_> = project
            .tasks
            .iter()
            .filter(|t| t.status == status.name)
            .collect();

        render_column(
            f,
            columns[col_idx],
            &status.display,
            &tasks,
            col_idx,
            app,
            is_focused,
            project,
        );
    }
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
    project: &Project,
) {
    let current_column = app
        .selected_column
        .get(&app.focused_pane)
        .copied()
        .unwrap_or(0);
    let is_column_focused = is_pane_focused && current_column == column_idx;

    // 简洁配色：聚焦=白色，非聚焦=灰色
    let (border_color, title_style) = if is_column_focused {
        (
            Color::White,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        (Color::DarkGray, Style::default().fg(Color::Gray))
    };

    let items: Vec<ListItem> = tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let selected_idx = app
                .selected_task_index
                .get(&app.focused_pane)
                .copied()
                .unwrap_or(0);
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
                Some("low") => Span::styled("● ", Style::default().fg(Color::Green)),
                _ => Span::raw("  "),
            };

            // 选中指示器
            let selection_indicator = if is_selected {
                Span::styled("▶ ", Style::default().fg(Color::White))
            } else {
                Span::raw("  ")
            };

            // 构建任务项内容
            let mut spans = vec![
                Span::raw(" "),
                selection_indicator,
                priority_indicator,
                Span::raw(&task.title),
            ];

            // 添加标签
            for tag in &task.tags {
                spans.push(Span::raw(" "));
                spans.push(Span::styled(
                    format!("[{}]", tag),
                    Style::default()
                        .fg(tag_color(tag))
                        .add_modifier(Modifier::BOLD),
                ));
            }

            spans.push(Span::raw(" "));

            // 任务项（紧凑布局，无额外间距）
            ListItem::new(Line::from(spans)).style(style)
        })
        .collect();

    // 列标题（调整后2秒内显示宽度百分比）
    let show_percentage = app
        .last_column_resize_time
        .map(|t| t.elapsed().as_secs() < 2)
        .unwrap_or(false);

    let title_with_count = if show_percentage {
        if let Some(widths) = app.config.column_widths.get(&project.name) {
            if column_idx < widths.len() {
                format!(" {} ({}) [{}%] ", title, tasks.len(), widths[column_idx])
            } else {
                format!(" {} ({}) ", title, tasks.len())
            }
        } else if let Some(Some(max_col)) = app.config.maximized_column.get(&project.name) {
            if column_idx == *max_col {
                format!(" {} ({}) [MAX] ", title, tasks.len())
            } else {
                format!(" {} ({}) ", title, tasks.len())
            }
        } else {
            format!(" {} ({}) ", title, tasks.len())
        }
    } else {
        format!(" {} ({}) ", title, tasks.len())
    };

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
