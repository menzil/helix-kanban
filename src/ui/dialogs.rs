use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::text_input::HelixTextArea;

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
        textarea: HelixTextArea,
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
        action: ConfirmAction,  // 添加操作类型
    },
}

/// 渲染居中的对话框
pub fn render_dialog(f: &mut Frame, dialog: &mut DialogType) {
    // 渲染半透明背景遮罩
    render_backdrop(f, f.area());

    // 根据对话框类型和最大化状态决定大小
    let area = match dialog {
        DialogType::Input { textarea, .. } => {
            if textarea.is_maximized() {
                // 最大化：占据 90% 的屏幕空间
                centered_rect(90, 90, f.area())
            } else {
                // 正常大小
                centered_rect(60, 50, f.area())
            }
        }
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
        .border_style(Style::default().fg(Color::Rgb(76, 86, 106)))  // Nord border color
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(Color::Rgb(46, 52, 64)));  // Nord background

    let inner = block.inner(area);
    f.render_widget(block, area);

    // 分割内部区域 - 任务输入使用更大的输入框
    let chunks = if is_task_input {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),  // 提示文本
                Constraint::Min(10),    // 大输入框（多行）
                Constraint::Length(2),  // 模式指示器
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
        Paragraph::new(format!("{}\n（Helix 模式编辑，Esc 切换模式，:w 或 Ctrl+S 提交）", prompt))
            .style(Style::default().fg(Color::Rgb(129, 161, 193)))  // Nord frost color
    } else {
        Paragraph::new(prompt).style(Style::default().fg(Color::Rgb(129, 161, 193)))
    };
    f.render_widget(prompt_text, chunks[0]);

    // 输入框 - 使用 HelixTextArea 渲染
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(136, 192, 208)))  // Nord cyan
        .border_type(ratatui::widgets::BorderType::Rounded);

    let input_inner = input_block.inner(chunks[1]);
    f.render_widget(input_block, chunks[1]);

    // 渲染 TextArea
    textarea.render(f, input_inner);

    // 渲染模式指示器
    textarea.render_mode_indicator(f, chunks[2]);
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
            Constraint::Length(3), // 搜索框
            Constraint::Min(0),    // 列表
            Constraint::Length(1), // 帮助
        ])
        .split(inner);

    // 渲染搜索框
    let search_text = if filter.is_empty() {
        "🔍 输入搜索...".to_string()
    } else {
        format!("🔍 {}", filter)
    };

    let search_style = if filter.is_empty() {
        Style::default().fg(Color::Rgb(129, 161, 193))  // 灰色提示
    } else {
        Style::default().fg(Color::Rgb(136, 192, 208))  // 高亮搜索文本
    };

    let search_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(136, 192, 208)))
        .border_type(ratatui::widgets::BorderType::Rounded);

    let search_inner = search_block.inner(chunks[0]);
    f.render_widget(search_block, chunks[0]);

    let search_paragraph = Paragraph::new(search_text)
        .style(search_style);
    f.render_widget(search_paragraph, search_inner);

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
        .enumerate()
        .flat_map(|(filtered_idx, (idx, item))| {
            let is_selected = filtered_idx == selected;

            // 分割成多行
            let lines: Vec<&str> = item.lines().collect();
            let main_line = lines.get(0).unwrap_or(&"");
            let sub_line = lines.get(1);

            let mut content_lines = vec![];

            // 第一项上方添加空行（顶部间距）
            if filtered_idx == 0 {
                content_lines.push(Line::from(""));
            }

            if is_selected {
                // 选中项：蓝色背景，带序号
                let sequence_num = format!("{}", filtered_idx + 1);

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

    // 创建 ListState 以支持滚动
    let mut list_state = ratatui::widgets::ListState::default();

    // 在过滤后的项目中找到当前选中项的索引
    let filtered_selected = filtered_items.iter()
        .position(|(idx, _)| *idx == selected)
        .unwrap_or(0);

    list_state.select(Some(filtered_selected));

    f.render_stateful_widget(list, chunks[1], &mut list_state);

    // 帮助文本 - 简化提示（搜索框已经在顶部显示）
    let help_text = format!("↑↓ 导航  Enter 确认  Esc 取消  [{}/{}]", filtered_items.len(), items.len());
    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Rgb(129, 161, 193)))  // Nord frost color
        .alignment(Alignment::Center);
    f.render_widget(help_paragraph, chunks[2]);

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
        .title(format!("  {}  ", title))
        .title_alignment(Alignment::Left)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(235, 203, 139)))  // Nord yellow for warnings
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(Color::Rgb(46, 52, 64)));  // Nord background

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
        .style(Style::default().fg(Color::Rgb(216, 222, 233)));  // Nord snow storm
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
            .bg(Color::Rgb(191, 97, 106))   // Nord 柔和红色
            .fg(Color::Rgb(46, 52, 64))      // Nord 深色背景
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
            .bg(Color::Rgb(163, 190, 140))  // Nord 柔和绿色
            .fg(Color::Rgb(46, 52, 64))      // Nord 深色背景
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
