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
    // 判断是否是任务输入（需要更大的输入框）
    let is_task_input = title.contains("任务");

    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(ratatui::widgets::BorderType::Rounded);

    let inner = block.inner(area);
    f.render_widget(block, area);

    // 分割内部区域 - 任务输入使用更大的输入框
    let chunks = if is_task_input {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),  // 提示文本
                Constraint::Min(10),    // 大输入框（多行）
                Constraint::Length(3),  // 帮助文本
            ])
            .split(inner)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // 提示文本
                Constraint::Length(3), // 普通输入框
                Constraint::Length(2), // 帮助文本
            ])
            .split(inner)
    };

    // 提示文本
    let prompt_text = if is_task_input {
        Paragraph::new(format!("{}\n（支持多行输入，包含任务的所有内容）", prompt))
            .style(Style::default().fg(Color::Gray))
    } else {
        Paragraph::new(prompt).style(Style::default().fg(Color::Gray))
    };
    f.render_widget(prompt_text, chunks[0]);

    // 输入框
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .border_type(ratatui::widgets::BorderType::Rounded);

    let input_inner = input_block.inner(chunks[1]);
    f.render_widget(input_block, chunks[1]);

    // 渲染输入内容和光标（支持多行）
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

    let input_text = Paragraph::new(input_with_cursor)
        .wrap(Wrap { trim: false });  // 支持自动换行
    f.render_widget(input_text, input_inner);

    // 帮助文本
    let help_text = if is_task_input {
        "Enter: 确认 | Ctrl+Enter: 换行 | ESC: 取消"
    } else {
        "Enter: 确认 | ESC: 取消"
    };

    let help = Paragraph::new(help_text)
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
        .title(format!("  {}  ", title))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(76, 86, 106)))  // Nord border color
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(Color::Rgb(46, 52, 64)));  // Nord background

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // 列表
            Constraint::Length(1), // 帮助
        ])
        .split(inner);

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

    // 列表项 - 支持多行显示，添加分隔线
    let list_items: Vec<ListItem> = filtered_items
        .iter()
        .flat_map(|(idx, item)| {
            let is_selected = *idx == selected;

            // 分割成多行
            let lines: Vec<&str> = item.lines().collect();
            let main_line = lines.get(0).unwrap_or(&"");
            let sub_line = lines.get(1);

            let mut content_lines = vec![];

            // 第一项上方添加空行（顶部间距）
            if *idx == 0 {
                content_lines.push(Line::from(""));
            }

            if is_selected {
                // 选中项：蓝色背景，带序号
                let sequence_num = format!("{}", *idx + 1);

                content_lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        sequence_num,
                        Style::default()
                            .fg(Color::White)
                            .bg(Color::Rgb(94, 129, 172))  // 蓝色序号标记
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        *main_line,
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled("✓", Style::default().fg(Color::Rgb(163, 190, 140))),  // 绿色勾
                    Span::raw("  "),
                    Span::styled(
                        "Enter",
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Rgb(136, 192, 208)),
                    ),
                ]));
            } else {
                // 未选中项：正常显示
                content_lines.push(Line::from(format!("      {}", main_line)));
            }

            // 添加子行（路径）
            if let Some(sub) = sub_line {
                let sub_style = if is_selected {
                    Style::default().fg(Color::Rgb(216, 222, 233))
                } else {
                    Style::default().fg(Color::Rgb(129, 161, 193))
                };
                content_lines.push(Line::from(vec![
                    Span::styled(*sub, sub_style),
                ]));
            }

            // 添加分隔线（除了最后一项）
            if *idx < filtered_items.len() - 1 {
                content_lines.push(Line::from(vec![
                    Span::styled(
                        "  ────────────────────────────────────────────────────────",
                        Style::default().fg(Color::Rgb(76, 86, 106)),  // 灰色分隔线
                    ),
                ]));
            }

            let style = if is_selected {
                Style::default()
                    .bg(Color::Rgb(59, 66, 82))  // Nord 深蓝背景
            } else {
                Style::default()
            };

            vec![ListItem::new(content_lines).style(style)]
        })
        .collect();

    let list = List::new(list_items);
    f.render_widget(list, chunks[0]);

    // 帮助文本 - 右上角显示总数
    let help_text = format!("↑↓ 导航   Enter 切换选择   Esc 关闭   最多选择 1 个项目");
    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Rgb(129, 161, 193)))  // Nord frost color
        .alignment(Alignment::Center);
    f.render_widget(help_paragraph, chunks[1]);

    // 右上角显示计数
    let count_text = format!("{}/{}", filtered_items.len(), items.len());
    let count_area = Rect {
        x: area.x + area.width.saturating_sub(count_text.len() as u16 + 3),
        y: area.y,
        width: count_text.len() as u16 + 2,
        height: 1,
    };
    let count_paragraph = Paragraph::new(count_text)
        .style(Style::default().fg(Color::Rgb(129, 161, 193)));
    f.render_widget(count_paragraph, count_area);
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
        .border_style(Style::default().fg(Color::Yellow))
        .border_type(ratatui::widgets::BorderType::Rounded);

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
