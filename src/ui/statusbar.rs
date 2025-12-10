use crate::app::{App, Mode};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// 渲染状态栏（Helix 风格）
pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let mode_text = match app.mode {
        Mode::Normal => ("NORMAL", Color::Green),
        Mode::Command => ("COMMAND", Color::Yellow),
        Mode::TaskSelect => ("SELECT", Color::Cyan),
        Mode::Dialog => ("DIALOG", Color::Magenta),
        Mode::Help => ("HELP", Color::Blue),
        Mode::SpaceMenu => ("MENU", Color::Cyan),
        Mode::Preview => ("PREVIEW", Color::Blue),
    };

    // 显示键序列
    let key_sequence = if !app.key_buffer.is_empty() {
        format!(" [{}]", app.key_buffer.iter().collect::<String>())
    } else {
        String::new()
    };

    // 命令输入
    let command_display = if app.mode == Mode::Command {
        format!(" :{}", app.command_input)
    } else {
        String::new()
    };

    let line = Line::from(vec![
        Span::styled(
            format!(" {} ", mode_text.0),
            Style::default()
                .fg(Color::Black)
                .bg(mode_text.1)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(key_sequence),
        Span::raw(command_display),
        Span::raw(format!(
            " | {} 项目 | 面板 {} ",
            app.projects.len(),
            app.focused_pane
        )),
    ]);

    let paragraph = Paragraph::new(line).style(Style::default().bg(Color::Black));

    f.render_widget(paragraph, area);
}
