use crate::models::{Project, ProjectType};
use crate::ui::tags::tag_color;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use std::path::{Path, PathBuf};

use super::text_input::HelixTextArea;

const PROJECT_GRID_CARD_HEIGHT: u16 = 6;
const PROJECT_GRID_ORDER_STEP: i64 = 1000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectGridItem {
    pub name: String,
    pub project_type: ProjectType,
    pub path: PathBuf,
    pub path_label: String,
    pub task_count: usize,
    pub done_count: usize,
    pub is_current: bool,
    pub project_order: Option<i64>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectGridState {
    pub items: Vec<ProjectGridItem>,
    pub selected: usize,
    pub filter: String,
    pub columns: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectGridNavigation {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectGridOrderMove {
    Left,
    Right,
    Up,
    Down,
}

pub fn project_grid_state_from_projects(
    projects: &[Project],
    current_project_path: Option<&Path>,
) -> ProjectGridState {
    let mut items: Vec<ProjectGridItem> = projects
        .iter()
        .map(|project| project_grid_item_from_project(project, current_project_path))
        .collect();

    sort_project_grid_items(&mut items);
    let selected = items.iter().position(|item| item.is_current).unwrap_or(0);

    ProjectGridState {
        items,
        selected,
        filter: String::new(),
        columns: 1,
    }
}

pub fn project_grid_item_from_project(
    project: &Project,
    current_project_path: Option<&Path>,
) -> ProjectGridItem {
    ProjectGridItem {
        name: project.name.clone(),
        project_type: project.project_type,
        path: project.path.clone(),
        path_label: format_project_path_label(&project.path),
        task_count: project.tasks.len(),
        done_count: project_done_count(project),
        is_current: current_project_path == Some(project.path.as_path()),
        project_order: project.project_order,
        tags: project.tags.clone(),
    }
}

pub fn sort_project_grid_items(items: &mut [ProjectGridItem]) {
    items.sort_by(|left, right| {
        project_order_sort_key(left.project_order)
            .cmp(&project_order_sort_key(right.project_order))
            .then_with(|| {
                project_type_sort_key(left.project_type)
                    .cmp(&project_type_sort_key(right.project_type))
            })
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
            .then_with(|| left.path.cmp(&right.path))
    });
}

pub fn filter_project_grid_items(items: &[ProjectGridItem], filter: &str) -> Vec<usize> {
    let query = filter.trim().to_lowercase();
    if query.is_empty() {
        return (0..items.len()).collect();
    }

    items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            if project_grid_item_matches(item, &query) {
                Some(index)
            } else {
                None
            }
        })
        .collect()
}

pub fn navigate_project_grid(
    selected: usize,
    item_count: usize,
    columns: usize,
    direction: ProjectGridNavigation,
) -> usize {
    if item_count == 0 {
        return 0;
    }

    let safe_selected = selected.min(item_count - 1);
    let safe_columns = columns.max(1);

    match direction {
        ProjectGridNavigation::Left => safe_selected.saturating_sub(1),
        ProjectGridNavigation::Right => (safe_selected + 1).min(item_count - 1),
        ProjectGridNavigation::Up => safe_selected.saturating_sub(safe_columns),
        ProjectGridNavigation::Down => (safe_selected + safe_columns).min(item_count - 1),
    }
}

pub fn normalize_project_tags(tags_string: &str) -> Vec<String> {
    tags_string
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
        .map(ToString::to_string)
        .collect()
}

pub fn reassigned_project_grid_orders(items: &[ProjectGridItem]) -> Vec<ProjectGridItem> {
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let mut ordered_item = item.clone();
            ordered_item.project_order = Some((index as i64) * PROJECT_GRID_ORDER_STEP);
            ordered_item
        })
        .collect()
}

pub fn reordered_project_grid_state(
    state: &ProjectGridState,
    direction: ProjectGridOrderMove,
) -> Option<ProjectGridState> {
    let filtered_indices = filter_project_grid_items(&state.items, &state.filter);
    if filtered_indices.is_empty() || state.selected >= filtered_indices.len() {
        return None;
    }

    let target_selected = reorder_target_index(
        state.selected,
        filtered_indices.len(),
        state.columns,
        direction,
    )?;

    let selected_item_index = filtered_indices[state.selected];
    let target_item_index = filtered_indices[target_selected];
    let mut items = state.items.clone();
    items.swap(selected_item_index, target_item_index);

    Some(ProjectGridState {
        items: reassigned_project_grid_orders(&items),
        selected: target_selected,
        filter: state.filter.clone(),
        columns: state.columns,
    })
}

pub fn reorder_target_index(
    selected: usize,
    item_count: usize,
    columns: usize,
    direction: ProjectGridOrderMove,
) -> Option<usize> {
    if item_count == 0 || selected >= item_count {
        return None;
    }

    let safe_columns = columns.max(1);

    match direction {
        ProjectGridOrderMove::Left if selected > 0 => Some(selected - 1),
        ProjectGridOrderMove::Right if selected + 1 < item_count => Some(selected + 1),
        ProjectGridOrderMove::Up if selected >= safe_columns => Some(selected - safe_columns),
        ProjectGridOrderMove::Down if selected + safe_columns < item_count => {
            Some(selected + safe_columns)
        }
        _ => None,
    }
}

pub fn update_project_grid_item_tags(
    state: &ProjectGridState,
    item_path: &Path,
    tags: Vec<String>,
) -> ProjectGridState {
    let items = state
        .items
        .iter()
        .map(|item| {
            let mut updated_item = item.clone();
            if updated_item.path == item_path {
                updated_item.tags = tags.clone();
            }
            updated_item
        })
        .collect();

    ProjectGridState {
        items,
        selected: state.selected,
        filter: state.filter.clone(),
        columns: state.columns,
    }
}

fn project_grid_item_matches(item: &ProjectGridItem, query: &str) -> bool {
    item.name.to_lowercase().contains(query)
        || item.path_label.to_lowercase().contains(query)
        || item.path.to_string_lossy().to_lowercase().contains(query)
        || item
            .tags
            .iter()
            .any(|tag| tag.to_lowercase().contains(query))
}

fn project_order_sort_key(project_order: Option<i64>) -> (u8, i64) {
    match project_order {
        Some(order) => (0, order),
        None => (1, 0),
    }
}

fn project_type_sort_key(project_type: ProjectType) -> u8 {
    match project_type {
        ProjectType::Global => 0,
        ProjectType::Local => 1,
    }
}

fn project_done_count(project: &Project) -> usize {
    project
        .statuses
        .last()
        .map(|status| {
            project
                .tasks
                .iter()
                .filter(|task| task.status == status.name)
                .count()
        })
        .unwrap_or(0)
}

fn format_project_path_label(path: &Path) -> String {
    let path_string = path.to_string_lossy().to_string();
    if let Ok(current_dir) = std::env::current_dir()
        && let Ok(stripped) = path.strip_prefix(&current_dir)
    {
        return stripped.display().to_string();
    }

    if let Ok(home_dir) = std::env::var("HOME") {
        let home_path = PathBuf::from(&home_dir);
        if let Ok(stripped) = path.strip_prefix(&home_path) {
            return format!("~/{}", stripped.display());
        }
    }

    path_string
}

/// 确认操作类型
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmAction {
    DeleteTask,
    DeleteProject,
    HideProject,
    DeleteStatus,
}

/// 对话框类型
pub enum DialogType {
    /// 输入对话框（用于创建项目、任务等）
    Input {
        title: String,
        prompt: String,
        textarea: Box<HelixTextArea>,
    },
    ProjectGrid {
        title: String,
        state: ProjectGridState,
    },
    ProjectTagsInput {
        title: String,
        prompt: String,
        textarea: Box<HelixTextArea>,
        project_path: PathBuf,
        project_name: String,
        grid_state: ProjectGridState,
    },
    /// 确认对话框
    Confirm {
        title: String,
        message: String,
        yes_selected: bool,
        action: ConfirmAction, // 添加操作类型
    },
}

/// 渲染居中的对话框
pub fn render_dialog(f: &mut Frame, dialog: &mut DialogType) {
    // 渲染半透明背景遮罩
    render_backdrop(f, f.area());

    // 根据对话框类型和最大化状态决定大小
    let area = match dialog {
        DialogType::Input { textarea, .. } | DialogType::ProjectTagsInput { textarea, .. } => {
            if textarea.is_maximized() {
                // 最大化：占据 90% 的屏幕空间
                centered_rect(90, 90, f.area())
            } else {
                // 正常大小
                centered_rect(60, 50, f.area())
            }
        }
        DialogType::ProjectGrid { .. } => centered_rect(90, 80, f.area()),
        _ => centered_rect(60, 50, f.area()),
    };

    // 清空对话框区域
    f.render_widget(Clear, area);

    match dialog {
        DialogType::Input {
            title,
            prompt,
            textarea,
        } => render_input_dialog(f, area, title, prompt, textarea),
        DialogType::ProjectTagsInput {
            title,
            prompt,
            textarea,
            ..
        } => render_input_dialog(f, area, title, prompt, textarea),
        DialogType::ProjectGrid { title, state } => {
            render_project_grid_dialog(f, area, title, state)
        }
        DialogType::Confirm {
            title,
            message,
            yes_selected,
            ..
        } => render_confirm_dialog(f, area, title, message, *yes_selected),
    }
}

/// 渲染半透明背景遮罩
fn render_backdrop(f: &mut Frame, area: Rect) {
    let block = Block::default().style(Style::default().bg(Color::Rgb(0, 0, 0))); // 黑色背景
    f.render_widget(block, area);
}

/// 渲染输入对话框
fn render_input_dialog(
    f: &mut Frame,
    area: Rect,
    title: &str,
    prompt: &str,
    textarea: &mut HelixTextArea,
) {
    // 判断是否是任务输入（需要更大的输入框）
    let is_task_input = title.contains("任务");

    let block = Block::default()
        .title(format!("  {}  ", title))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(76, 86, 106))) // Nord border color
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(Color::Rgb(46, 52, 64))); // Nord background

    let inner = block.inner(area);
    f.render_widget(block, area);

    // 分割内部区域 - 任务输入使用更大的输入框
    let chunks = if is_task_input {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // 提示文本
                Constraint::Min(10),   // 大输入框（多行）
                Constraint::Length(2), // 模式指示器
            ])
            .split(inner)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // 提示文本
                Constraint::Length(5), // 普通输入框
                Constraint::Length(2), // 模式指示器
            ])
            .split(inner)
    };

    // 提示文本
    let prompt_text = if is_task_input {
        Paragraph::new(format!(
            "{}\n（Helix 模式编辑，Esc 切换模式，:w 或 Ctrl+S 提交）",
            prompt
        ))
        .style(Style::default().fg(Color::Rgb(129, 161, 193))) // Nord frost color
    } else {
        Paragraph::new(prompt).style(Style::default().fg(Color::Rgb(129, 161, 193)))
    };
    f.render_widget(prompt_text, chunks[0]);

    // 输入框 - 使用 HelixTextArea 渲染
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(136, 192, 208))) // Nord cyan
        .border_type(ratatui::widgets::BorderType::Rounded);

    let input_inner = input_block.inner(chunks[1]);
    f.render_widget(input_block, chunks[1]);

    // 渲染 TextArea
    textarea.render(f, input_inner);

    // 渲染模式指示器
    textarea.render_mode_indicator(f, chunks[2]);
}

fn render_project_grid_dialog(
    f: &mut Frame,
    area: Rect,
    title: &str,
    state: &mut ProjectGridState,
) {
    let block = Block::default()
        .title(format!("  {}  ", title))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(76, 86, 106)))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(Color::Rgb(46, 52, 64)));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(inner);

    render_project_grid_search(f, chunks[0], &state.filter);

    let filtered_indices = filter_project_grid_items(&state.items, &state.filter);
    if filtered_indices.is_empty() {
        state.selected = 0;
    } else {
        state.selected = state.selected.min(filtered_indices.len() - 1);
    }
    state.columns = project_grid_columns(chunks[1].width);

    render_project_grid_cards(f, chunks[1], state, &filtered_indices);
    render_project_grid_footer(f, chunks[2], filtered_indices.len(), state.items.len());

    let count_text = format!("{}/{}", filtered_indices.len(), state.items.len());
    let count_area = Rect {
        x: area.x + area.width.saturating_sub(count_text.len() as u16 + 3),
        y: area.y,
        width: count_text.len() as u16 + 2,
        height: 1,
    };
    let count_paragraph =
        Paragraph::new(count_text).style(Style::default().fg(Color::Rgb(129, 161, 193)));
    f.render_widget(count_paragraph, count_area);
}

fn render_project_grid_search(f: &mut Frame, area: Rect, filter: &str) {
    let search_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(136, 192, 208)))
        .border_type(ratatui::widgets::BorderType::Rounded);
    let search_inner = search_block.inner(area);
    f.render_widget(search_block, area);

    let search_text = if filter.is_empty() {
        "Search projects...".to_string()
    } else {
        format!("Search: {}", filter)
    };

    let search_style = if filter.is_empty() {
        Style::default().fg(Color::Rgb(129, 161, 193))
    } else {
        Style::default().fg(Color::Rgb(136, 192, 208))
    };

    f.render_widget(
        Paragraph::new(search_text).style(search_style),
        search_inner,
    );
}

fn render_project_grid_cards(
    f: &mut Frame,
    area: Rect,
    state: &ProjectGridState,
    filtered_indices: &[usize],
) {
    if area.height == 0 {
        return;
    }

    if filtered_indices.is_empty() {
        let empty = Paragraph::new("No projects match the current filter")
            .style(Style::default().fg(Color::Rgb(129, 161, 193)))
            .alignment(Alignment::Center);
        f.render_widget(empty, area);
        return;
    }

    let columns = project_grid_columns(area.width);
    let rows = (area.height / PROJECT_GRID_CARD_HEIGHT).max(1) as usize;
    let visible_slots = (columns * rows).max(1);
    let start = (state.selected / visible_slots) * visible_slots;
    let end = (start + visible_slots).min(filtered_indices.len());

    for (visible_index, item_index) in filtered_indices[start..end].iter().enumerate() {
        let filtered_index = start + visible_index;
        let row = visible_index / columns;
        let column = visible_index % columns;
        let card_area = project_grid_card_area(area, row, column, columns);
        if card_area.height == 0 || card_area.width == 0 {
            continue;
        }

        if let Some(item) = state.items.get(*item_index) {
            render_project_grid_card(f, card_area, item, filtered_index == state.selected);
        }
    }
}

fn render_project_grid_footer(
    f: &mut Frame,
    area: Rect,
    filtered_count: usize,
    total_count: usize,
) {
    let help_text = format!(
        "h/j/k/l select  Enter open  H/J/K/L move card  t tags  Esc close  [{}/{}]",
        filtered_count, total_count
    );
    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Rgb(129, 161, 193)))
        .alignment(Alignment::Center);
    f.render_widget(help_paragraph, area);
}

fn render_project_grid_card(f: &mut Frame, area: Rect, item: &ProjectGridItem, is_selected: bool) {
    let border_color = if is_selected {
        Color::Rgb(136, 192, 208)
    } else {
        Color::Rgb(76, 86, 106)
    };
    let card_style = if is_selected {
        Style::default().bg(Color::Rgb(59, 66, 82))
    } else {
        Style::default().bg(Color::Rgb(46, 52, 64))
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(card_style);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let type_marker = match item.project_type {
        ProjectType::Global => "[G]",
        ProjectType::Local => "[L]",
    };
    let current_marker = if item.is_current { " *" } else { "" };

    let mut lines = vec![
        Line::from(vec![
            Span::styled(
                type_marker,
                Style::default()
                    .fg(Color::Rgb(136, 192, 208))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                item.name.clone(),
                Style::default()
                    .fg(Color::Rgb(236, 239, 244))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                current_marker,
                Style::default()
                    .fg(Color::Rgb(163, 190, 140))
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![Span::styled(
            format!("Tasks: {}/{}", item.done_count, item.task_count),
            Style::default().fg(Color::Rgb(216, 222, 233)),
        )]),
        Line::from(vec![Span::styled(
            item.path_label.clone(),
            Style::default().fg(Color::Rgb(129, 161, 193)),
        )]),
    ];

    let tag_line = if item.tags.is_empty() {
        Line::from(vec![Span::styled(
            "Tags: []",
            Style::default().fg(Color::Rgb(76, 86, 106)),
        )])
    } else {
        let mut spans = vec![Span::styled(
            "Tags:",
            Style::default().fg(Color::Rgb(129, 161, 193)),
        )];
        for tag in &item.tags {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("[{}]", tag),
                Style::default()
                    .fg(tag_color(tag))
                    .add_modifier(Modifier::BOLD),
            ));
        }
        Line::from(spans)
    };

    lines.push(tag_line);

    let paragraph = Paragraph::new(lines)
        .style(card_style)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, inner);
}

fn project_grid_columns(width: u16) -> usize {
    if width >= 100 {
        3
    } else if width >= 68 {
        2
    } else {
        1
    }
}

fn project_grid_card_area(area: Rect, row: usize, column: usize, columns: usize) -> Rect {
    let row_constraints = vec![Constraint::Length(PROJECT_GRID_CARD_HEIGHT); row + 1];
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(area);
    let Some(row_area) = rows.get(row).copied() else {
        return Rect::default();
    };

    let column_constraints = vec![Constraint::Fill(1); columns];
    let column_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(column_constraints)
        .split(row_area);

    column_areas.get(column).copied().unwrap_or_default()
}

/// 渲染确认对话框
fn render_confirm_dialog(
    f: &mut Frame,
    area: Rect,
    title: &str,
    message: &str,
    yes_selected: bool,
) {
    let block = Block::default()
        .title(format!("  {}  ", title))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(235, 203, 139))) // Nord yellow for warnings
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(Color::Rgb(46, 52, 64))); // Nord background

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // 消息
            Constraint::Length(3), // 按钮
        ])
        .split(inner);

    // 消息文本
    let message_text = Paragraph::new(message)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Rgb(216, 222, 233))); // Nord snow storm
    f.render_widget(message_text, chunks[0]);

    // 按钮区域 - 添加快捷键提示
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[1]);

    // 否按钮 (n) - 放在左侧
    let no_style = if !yes_selected {
        Style::default()
            .bg(Color::Rgb(191, 97, 106)) // Nord 柔和红色
            .fg(Color::Rgb(46, 52, 64)) // Nord 深色背景
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Rgb(191, 97, 106))
            .add_modifier(Modifier::DIM)
    };
    let no_button = Paragraph::new("[ n ] 否")
        .style(no_style)
        .alignment(Alignment::Center);
    f.render_widget(no_button, button_chunks[1]);

    // 是按钮 (y) - 放在右侧
    let yes_style = if yes_selected {
        Style::default()
            .bg(Color::Rgb(163, 190, 140)) // Nord 柔和绿色
            .fg(Color::Rgb(46, 52, 64)) // Nord 深色背景
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Rgb(163, 190, 140))
            .add_modifier(Modifier::DIM)
    };
    let yes_button = Paragraph::new("[ y ] 是")
        .style(yes_style)
        .alignment(Alignment::Center);
    f.render_widget(yes_button, button_chunks[2]);
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

#[cfg(test)]
mod tests {
    use super::{
        ProjectGridItem, ProjectGridNavigation, ProjectGridOrderMove, ProjectGridState,
        filter_project_grid_items, navigate_project_grid, normalize_project_tags,
        reassigned_project_grid_orders, reorder_target_index, reordered_project_grid_state,
        sort_project_grid_items,
    };
    use crate::models::ProjectType;
    use std::path::PathBuf;

    fn item(
        name: &str,
        project_type: ProjectType,
        path: &str,
        project_order: Option<i64>,
        tags: Vec<&str>,
    ) -> ProjectGridItem {
        ProjectGridItem {
            name: name.to_string(),
            project_type,
            path: PathBuf::from(path),
            path_label: path.to_string(),
            task_count: 0,
            done_count: 0,
            is_current: false,
            project_order,
            tags: tags.into_iter().map(ToString::to_string).collect(),
        }
    }

    fn state(items: Vec<ProjectGridItem>, selected: usize) -> ProjectGridState {
        ProjectGridState {
            items,
            selected,
            filter: String::new(),
            columns: 2,
        }
    }

    #[test]
    fn sorts_ordered_projects_before_unordered_with_stable_fallback() {
        let mut items = vec![
            item("zeta", ProjectType::Local, "/local/zeta", None, vec![]),
            item(
                "alpha",
                ProjectType::Local,
                "/local/alpha",
                Some(2000),
                vec![],
            ),
            item("beta", ProjectType::Global, "/global/beta", None, vec![]),
            item(
                "gamma",
                ProjectType::Global,
                "/global/gamma",
                Some(1000),
                vec![],
            ),
        ];

        sort_project_grid_items(&mut items);

        let names: Vec<&str> = items.iter().map(|item| item.name.as_str()).collect();
        assert_eq!(names, vec!["gamma", "alpha", "beta", "zeta"]);
    }

    #[test]
    fn filters_by_name_path_and_tag() {
        let items = vec![
            item(
                "frontend",
                ProjectType::Global,
                "/work/client-app",
                None,
                vec!["urgent"],
            ),
            item(
                "backend",
                ProjectType::Local,
                "/work/api",
                None,
                vec!["slow"],
            ),
        ];

        assert_eq!(filter_project_grid_items(&items, "front"), vec![0]);
        assert_eq!(filter_project_grid_items(&items, "api"), vec![1]);
        assert_eq!(filter_project_grid_items(&items, "urgent"), vec![0]);
        assert_eq!(
            filter_project_grid_items(&items, "missing"),
            Vec::<usize>::new()
        );
    }

    #[test]
    fn grid_navigation_respects_boundaries() {
        assert_eq!(
            navigate_project_grid(0, 5, 2, ProjectGridNavigation::Left),
            0
        );
        assert_eq!(
            navigate_project_grid(0, 5, 2, ProjectGridNavigation::Right),
            1
        );
        assert_eq!(
            navigate_project_grid(1, 5, 2, ProjectGridNavigation::Down),
            3
        );
        assert_eq!(
            navigate_project_grid(4, 5, 2, ProjectGridNavigation::Down),
            4
        );
        assert_eq!(navigate_project_grid(1, 5, 2, ProjectGridNavigation::Up), 0);
    }

    #[test]
    fn normalizes_comma_separated_project_tags() {
        assert_eq!(
            normalize_project_tags(" urgent, client ,, release "),
            vec!["urgent", "client", "release"]
        );
    }

    #[test]
    fn reassigns_stable_unique_order_values() {
        let ordered = reassigned_project_grid_orders(&[
            item("a", ProjectType::Global, "/a", Some(50), vec![]),
            item("b", ProjectType::Global, "/b", None, vec![]),
            item("c", ProjectType::Global, "/c", Some(10), vec![]),
        ]);

        let orders: Vec<Option<i64>> = ordered.iter().map(|item| item.project_order).collect();
        assert_eq!(orders, vec![Some(0), Some(1000), Some(2000)]);
    }

    #[test]
    fn reorders_selected_item_down_one_grid_row() {
        let original = state(
            vec![
                item("a", ProjectType::Global, "/a", Some(0), vec![]),
                item("b", ProjectType::Global, "/b", Some(1000), vec![]),
                item("c", ProjectType::Global, "/c", Some(2000), vec![]),
                item("d", ProjectType::Global, "/d", Some(3000), vec![]),
            ],
            0,
        );

        let reordered =
            reordered_project_grid_state(&original, ProjectGridOrderMove::Down).unwrap();

        let names: Vec<&str> = reordered
            .items
            .iter()
            .map(|item| item.name.as_str())
            .collect();
        let orders: Vec<Option<i64>> = reordered
            .items
            .iter()
            .map(|item| item.project_order)
            .collect();
        assert_eq!(names, vec!["c", "b", "a", "d"]);
        assert_eq!(orders, vec![Some(0), Some(1000), Some(2000), Some(3000)]);
        assert_eq!(reordered.selected, 2);
    }

    #[test]
    fn reorder_targets_follow_grid_direction() {
        assert_eq!(
            reorder_target_index(3, 6, 2, ProjectGridOrderMove::Up),
            Some(1)
        );
        assert_eq!(
            reorder_target_index(1, 6, 2, ProjectGridOrderMove::Down),
            Some(3)
        );
        assert_eq!(
            reorder_target_index(2, 6, 2, ProjectGridOrderMove::Left),
            Some(1)
        );
        assert_eq!(
            reorder_target_index(2, 6, 2, ProjectGridOrderMove::Right),
            Some(3)
        );
    }

    #[test]
    fn does_not_reorder_past_grid_boundary() {
        let original = state(
            vec![item("a", ProjectType::Global, "/a", Some(0), vec![])],
            0,
        );

        assert!(reordered_project_grid_state(&original, ProjectGridOrderMove::Up).is_none());
        assert_eq!(
            reorder_target_index(0, 2, 2, ProjectGridOrderMove::Up),
            None
        );
        assert_eq!(
            reorder_target_index(0, 2, 2, ProjectGridOrderMove::Down),
            None
        );
    }
}
