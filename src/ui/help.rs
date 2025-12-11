use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// 渲染帮助面板
pub fn render(f: &mut Frame, area: Rect) {
    // 渲染半透明背景遮罩
    render_backdrop(f, area);

    // 创建居中的弹窗区域
    let popup_area = centered_rect(80, 85, area);

    // 清空弹窗区域
    f.render_widget(Clear, popup_area);

    // 清空背景
    let block = Block::default()
        .title(" 键盘快捷键帮助 (按 ESC 或 ? 关闭) ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(Color::Black));

    f.render_widget(block.clone(), popup_area);

    let inner = block.inner(popup_area);

    // 分成三列
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(inner);

    // 左列：基础导航
    let navigation_help = vec![
        Line::from(vec![
            Span::styled("基础导航", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("j, ↓", Style::default().fg(Color::Cyan)),
            Span::raw("       下一个任务"),
        ]),
        Line::from(vec![
            Span::styled("k, ↑", Style::default().fg(Color::Cyan)),
            Span::raw("       上一个任务"),
        ]),
        Line::from(vec![
            Span::styled("h, ←", Style::default().fg(Color::Cyan)),
            Span::raw("       左边的列"),
        ]),
        Line::from(vec![
            Span::styled("l, →", Style::default().fg(Color::Cyan)),
            Span::raw("       右边的列"),
        ]),
        Line::from(vec![
            Span::styled("q", Style::default().fg(Color::Cyan)),
            Span::raw("          退出程序"),
        ]),
        Line::from(vec![
            Span::styled("ESC", Style::default().fg(Color::Cyan)),
            Span::raw("        取消/返回"),
        ]),
        Line::from(vec![
            Span::styled(":", Style::default().fg(Color::Cyan)),
            Span::raw("          命令模式"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("任务操作", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("a", Style::default().fg(Color::Cyan)),
            Span::raw("          创建新任务"),
        ]),
        Line::from(vec![
            Span::styled("e", Style::default().fg(Color::Cyan)),
            Span::raw("          编辑任务标题"),
        ]),
        Line::from(vec![
            Span::styled("d", Style::default().fg(Color::Cyan)),
            Span::raw("          删除任务"),
        ]),
        Line::from(vec![
            Span::styled("D", Style::default().fg(Color::Cyan)),
            Span::raw("          删除项目"),
        ]),
        Line::from(vec![
            Span::styled("H", Style::default().fg(Color::Cyan)),
            Span::raw("          任务移到左列"),
        ]),
        Line::from(vec![
            Span::styled("L", Style::default().fg(Color::Cyan)),
            Span::raw("          任务移到右列"),
        ]),
        Line::from(vec![
            Span::styled("J", Style::default().fg(Color::Cyan)),
            Span::raw("          任务在列内下移"),
        ]),
        Line::from(vec![
            Span::styled("K", Style::default().fg(Color::Cyan)),
            Span::raw("          任务在列内上移"),
        ]),
    ];

    // 中列：项目管理
    let project_help = vec![
        Line::from(vec![
            Span::styled("项目管理", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Space p o", Style::default().fg(Color::Cyan)),
            Span::raw("   打开项目"),
        ]),
        Line::from(vec![
            Span::styled("Space p n", Style::default().fg(Color::Cyan)),
            Span::raw("   创建新项目"),
        ]),
        Line::from(vec![
            Span::styled("Space p d", Style::default().fg(Color::Cyan)),
            Span::raw("   删除项目"),
        ]),
        Line::from(vec![
            Span::styled("Space p r", Style::default().fg(Color::Cyan)),
            Span::raw("   重命名项目"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("窗口管理", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Space w w", Style::default().fg(Color::Cyan)),
            Span::raw("   下一个窗口"),
        ]),
        Line::from(vec![
            Span::styled("Space w v", Style::default().fg(Color::Cyan)),
            Span::raw("   垂直分屏"),
        ]),
        Line::from(vec![
            Span::styled("Space w s", Style::default().fg(Color::Cyan)),
            Span::raw("   水平分屏"),
        ]),
        Line::from(vec![
            Span::styled("Space w q", Style::default().fg(Color::Cyan)),
            Span::raw("   关闭窗口"),
        ]),
        Line::from(vec![
            Span::styled("Space w h", Style::default().fg(Color::Cyan)),
            Span::raw("   聚焦左面板"),
        ]),
        Line::from(vec![
            Span::styled("Space w l", Style::default().fg(Color::Cyan)),
            Span::raw("   聚焦右面板"),
        ]),
        Line::from(vec![
            Span::styled("Space w j", Style::default().fg(Color::Cyan)),
            Span::raw("   聚焦下面板"),
        ]),
        Line::from(vec![
            Span::styled("Space w k", Style::default().fg(Color::Cyan)),
            Span::raw("   聚焦上面板"),
        ]),
    ];

    // 右列：对话框操作
    let dialog_help = vec![
        Line::from(vec![
            Span::styled("对话框操作", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("输入对话框:", Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC)),
        ]),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(Color::Cyan)),
            Span::raw("      确认"),
        ]),
        Line::from(vec![
            Span::styled("  ESC", Style::default().fg(Color::Cyan)),
            Span::raw("        取消"),
        ]),
        Line::from(vec![
            Span::styled("  ←/→", Style::default().fg(Color::Cyan)),
            Span::raw("        移动光标"),
        ]),
        Line::from(vec![
            Span::styled("  Home/End", Style::default().fg(Color::Cyan)),
            Span::raw("   跳到开头/结尾"),
        ]),
        Line::from(vec![
            Span::styled("  Backspace", Style::default().fg(Color::Cyan)),
            Span::raw("  删除前一个字符"),
        ]),
        Line::from(vec![
            Span::styled("  Delete", Style::default().fg(Color::Cyan)),
            Span::raw("     删除当前字符"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("选择对话框:", Style::default().fg(Color::Green).add_modifier(Modifier::ITALIC)),
        ]),
        Line::from(vec![
            Span::styled("  j/k, ↑/↓", Style::default().fg(Color::Cyan)),
            Span::raw("   上下选择"),
        ]),
        Line::from(vec![
            Span::styled("  字符", Style::default().fg(Color::Cyan)),
            Span::raw("        搜索过滤"),
        ]),
        Line::from(vec![
            Span::styled("  Backspace", Style::default().fg(Color::Cyan)),
            Span::raw("  删除搜索字符"),
        ]),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(Color::Cyan)),
            Span::raw("      确认选择"),
        ]),
        Line::from(vec![
            Span::styled("  ESC", Style::default().fg(Color::Cyan)),
            Span::raw("        取消"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("提示", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("• 支持完整的汉字输入"),
        Line::from("• 所有操作自动保存"),
        Line::from("• 状态栏显示当前模式"),
        Line::from("• 键序列会显示在状态栏"),
    ];

    let nav_widget = Paragraph::new(navigation_help)
        .block(Block::default().borders(Borders::RIGHT))
        .wrap(Wrap { trim: false });

    let proj_widget = Paragraph::new(project_help)
        .block(Block::default().borders(Borders::RIGHT))
        .wrap(Wrap { trim: false });

    let dialog_widget = Paragraph::new(dialog_help)
        .wrap(Wrap { trim: false });

    f.render_widget(nav_widget, columns[0]);
    f.render_widget(proj_widget, columns[1]);
    f.render_widget(dialog_widget, columns[2]);
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
