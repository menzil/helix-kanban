/// 命令补全 UI - 类似 Helix 的命令提示
use crate::app::App;
use crate::input::CommandDef;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

/// 渲染命令补全提示
pub fn render(f: &mut Frame, area: Rect, app: &mut App) {
    let input = &app.command_input;

    // 获取匹配的命令列表
    let matches = app.command_registry.find_matches(input);

    if matches.is_empty() {
        return;
    }

    // 确保有选中的索引，如果没有则默认选中第一项
    if app.completion_selected_index.is_none() || app.completion_selected_index >= Some(matches.len()) {
        app.completion_selected_index = Some(0);
    }

    // 计算布局：上方输入框，下方补全列表
    let total_height = area.height.min(20); // 最多20行
    let input_height = 2; // 输入框占2行
    let list_height = total_height - input_height;

    let input_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: input_height,
    };

    let completion_area = Rect {
        x: area.x,
        y: area.y + input_height,
        width: area.width,
        height: list_height,
    };

    // 渲染输入框
    render_command_input(f, input_area, input);

    // 渲染命令列表
    render_command_list(f, completion_area, &matches, app.completion_selected_index);

    // 如果只有一个匹配且是精确匹配，显示详细信息
    if matches.len() == 1 || (!input.is_empty() && matches.iter().any(|cmd| cmd.name == input || cmd.aliases.contains(&input.as_str()))) {
        let cmd = matches[0];
        render_command_detail(f, area, cmd);
    }
}

/// 渲染命令输入框
fn render_command_input(f: &mut Frame, area: Rect, input: &str) {
    // 清空区域
    f.render_widget(Clear, area);

    // 创建输入框
    let block = Block::default()
        .title(" 输入命令 ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(136, 192, 208)))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(Color::Rgb(46, 52, 64)));

    f.render_widget(block.clone(), area);

    let inner = block.inner(area);

    // 显示当前输入
    let input_text = format!(":{}", input);
    let line = Line::from(vec![
        Span::raw(input_text),
    ]);

    let paragraph = Paragraph::new(line)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left);

    f.render_widget(paragraph, inner);
}

/// 渲染命令列表（底部多列布局）
fn render_command_list(f: &mut Frame, area: Rect, commands: &[&CommandDef], selected_index: Option<usize>) {
    // 清空区域
    f.render_widget(Clear, area);

    // 使用深色背景
    let bg_block = Block::default()
        .style(Style::default().bg(Color::Rgb(30, 30, 30)));
    f.render_widget(bg_block, area);

    // 计算列数（每列最多20个字符）
    let column_width = 20;
    let columns = (area.width as usize / column_width).max(1).min(5);

    // 限制显示的命令数量（最多显示能放下的）
    let max_rows = (area.height.saturating_sub(0)) as usize;
    let max_commands = columns * max_rows;
    let display_commands = &commands[..commands.len().min(max_commands)];

    // 构建多列文本
    let mut lines = Vec::new();
    let rows = (display_commands.len() + columns - 1) / columns;

    for row in 0..rows {
        let mut spans = Vec::new();

        for col in 0..columns {
            let idx = row + col * rows;
            if idx < display_commands.len() {
                let cmd = display_commands[idx];

                // 计算在显示列表中的绝对索引
                let absolute_idx = idx;

                // 命令名（主名称或第一个别名）
                let display_name = if cmd.aliases.is_empty() {
                    cmd.name
                } else {
                    cmd.aliases[0]
                };

                // 检查是否被选中
                let is_selected = selected_index.map(|i| i == absolute_idx).unwrap_or(false);

                // 根据选中状态选择样式
                let style = if is_selected {
                    Style::default()
                        .bg(Color::Rgb(136, 192, 208))
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Rgb(136, 192, 208))
                };

                // 添加命令名
                spans.push(Span::styled(
                    format!("{:<width$}", display_name, width = column_width - 1),
                    style,
                ));
            } else {
                spans.push(Span::raw(" ".repeat(column_width)));
            }
        }

        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines).alignment(Alignment::Left);
    f.render_widget(paragraph, area);
}

/// 渲染命令详细信息（中央弹窗）
fn render_command_detail(f: &mut Frame, area: Rect, cmd: &CommandDef) {
    // 创建居中弹窗
    let popup_width = 60.min(area.width.saturating_sub(4));
    let popup_height = 6;

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    // 清空弹窗区域
    f.render_widget(Clear, popup_area);

    // 构建内容
    let mut lines = Vec::new();

    // 标题行：完整命令名
    lines.push(Line::from(vec![
        Span::styled(
            cmd.name,
            Style::default()
                .fg(Color::Rgb(136, 192, 208))
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    // 空行
    lines.push(Line::from(""));

    // 描述
    lines.push(Line::from(vec![
        Span::styled(cmd.description, Style::default().fg(Color::White)),
    ]));

    // 别名
    if !cmd.aliases.is_empty() {
        lines.push(Line::from(""));
        let aliases_str = format!("Aliases: {}", cmd.aliases.join(", "));
        lines.push(Line::from(vec![
            Span::styled(aliases_str, Style::default().fg(Color::Gray)),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(136, 192, 208)))
                .border_type(ratatui::widgets::BorderType::Rounded)
                .style(Style::default().bg(Color::Rgb(40, 40, 40))),
        )
        .alignment(Alignment::Left);

    f.render_widget(paragraph, popup_area);
}
