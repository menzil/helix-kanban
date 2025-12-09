use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// 对话框类型
#[derive(Debug, Clone, PartialEq)]
pub enum DialogType {
    /// 输入对话框（用于创建项目、任务等）
    Input {
        title: String,
        prompt: String,
        value: String,
        cursor_pos: usize,
    },
    /// 选择对话框（用于选择项目等）
    Select {
        title: String,
        items: Vec<String>,
        selected: usize,
        filter: String,
    },
    /// 确认对话框
    Confirm {
        title: String,
        message: String,
        yes_selected: bool,
    },
}

/// 渲染居中的对话框
pub fn render_dialog(f: &mut Frame, dialog: &DialogType) {
    // 渲染半透明背景遮罩
    render_backdrop(f, f.area());

    let area = centered_rect(60, 50, f.area());

    // 清空对话框区域
    f.render_widget(Clear, area);

    match dialog {
        DialogType::Input {
            title,
            prompt,
            value,
            cursor_pos,
        } => render_input_dialog(f, area, title, prompt, value, *cursor_pos),
        DialogType::Select {
            title,
            items,
            selected,
            filter,
        } => render_select_dialog(f, area, title, items, *selected, filter),
        DialogType::Confirm {
            title,
            message,
            yes_selected,
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
    value: &str,
    cursor_pos: usize,
) {
    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // 分割内部区域
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // 提示文本
            Constraint::Length(3), // 输入框
            Constraint::Length(2), // 帮助文本
        ])
        .split(inner);

    // 提示文本
    let prompt_text = Paragraph::new(prompt).style(Style::default().fg(Color::Gray));
    f.render_widget(prompt_text, chunks[0]);

    // 输入框
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let input_inner = input_block.inner(chunks[1]);
    f.render_widget(input_block, chunks[1]);

    // 渲染输入内容和光标
    let chars: Vec<char> = value.chars().collect();
    let input_with_cursor = if cursor_pos >= chars.len() {
        // 光标在末尾
        format!("{}_", value)
    } else {
        // 光标在中间
        let mut display_chars = chars.clone();
        display_chars.insert(cursor_pos, '|');
        display_chars.into_iter().collect()
    };

    let input_text = Paragraph::new(input_with_cursor);
    f.render_widget(input_text, input_inner);

    // 帮助文本
    let help = Paragraph::new("Enter: 确认 | ESC: 取消")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
}

/// 渲染选择对话框
fn render_select_dialog(
    f: &mut Frame,
    area: Rect,
    title: &str,
    items: &[String],
    selected: usize,
    filter: &str,
) {
    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // 搜索框
            Constraint::Min(0),    // 列表
            Constraint::Length(1), // 帮助
        ])
        .split(inner);

    // 搜索框
    let search_block = Block::default()
        .title(" 搜索 ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let search_inner = search_block.inner(chunks[0]);
    f.render_widget(search_block, chunks[0]);
    let search_text = Paragraph::new(format!("{}_", filter));
    f.render_widget(search_text, search_inner);

    // 过滤项目列表
    let filtered_items: Vec<_> = if filter.is_empty() {
        items.iter().enumerate().collect()
    } else {
        items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.to_lowercase().contains(&filter.to_lowercase()))
            .collect()
    };

    // 列表项
    let list_items: Vec<ListItem> = filtered_items
        .iter()
        .map(|(idx, item)| {
            let is_selected = *idx == selected;
            let style = if is_selected {
                Style::default()
                    .bg(Color::Rgb(41, 98, 218))  // 蓝色高亮背景
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let content = if is_selected {
                Line::from(vec![
                    Span::raw("  ✓ "),
                    Span::styled(item.as_str(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::raw("  "),
                    Span::styled("Enter", Style::default().fg(Color::Cyan)),
                ])
            } else {
                Line::from(format!("    {}", item))
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(list_items);
    f.render_widget(list, chunks[1]);

    // 帮助文本
    let help = Paragraph::new("↑/↓: 选择 | Enter: 确认 | ESC: 取消")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[2]);
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
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

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
        .alignment(Alignment::Center);
    f.render_widget(message_text, chunks[0]);

    // 按钮区域
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[1]);

    // 是按钮
    let yes_style = if yes_selected {
        Style::default()
            .bg(Color::Green)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };
    let yes_button = Paragraph::new("[ 是 ]")
        .style(yes_style)
        .alignment(Alignment::Center);
    f.render_widget(yes_button, button_chunks[1]);

    // 否按钮
    let no_style = if !yes_selected {
        Style::default()
            .bg(Color::Red)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red)
    };
    let no_button = Paragraph::new("[ 否 ]")
        .style(no_style)
        .alignment(Alignment::Center);
    f.render_widget(no_button, button_chunks[2]);
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
