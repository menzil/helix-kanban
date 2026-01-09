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
        .border_style(Style::default().fg(Color::Rgb(136, 192, 208)))  // Nord cyan
        .border_type(ratatui::widgets::BorderType::Rounded)
        .style(Style::default().bg(Color::Rgb(46, 52, 64)));  // Nord background

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
            Span::styled("基础导航", Style::default().fg(Color::Rgb(235, 203, 139)).add_modifier(Modifier::BOLD)),  // Nord yellow
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("↑", Style::default().fg(Color::Rgb(136, 192, 208))),  // Nord cyan
            Span::raw("         上一个任务"),
        ]),
        Line::from(vec![
            Span::styled("↓", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("         下一个任务"),
        ]),
        Line::from(vec![
            Span::styled("←", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("         左边的列"),
        ]),
        Line::from(vec![
            Span::styled("→", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("         右边的列"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Space", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("      命令菜单"),
        ]),
        Line::from(vec![
            Span::styled("?", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          显示帮助"),
        ]),
        Line::from(vec![
            Span::styled("ESC / Enter", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("      取消 / 执行"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("任务操作", Style::default().fg(Color::Rgb(235, 203, 139)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("a", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          创建新任务"),
        ]),
        Line::from(vec![
            Span::styled("A", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          编辑器创建任务"),
        ]),
        Line::from(vec![
            Span::styled("e", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          编辑任务标题"),
        ]),
        Line::from(vec![
            Span::styled("E", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          编辑器编辑任务"),
        ]),
        Line::from(vec![
            Span::styled("v", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          预览任务"),
        ]),
        Line::from(vec![
            Span::styled("V", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          外部工具预览"),
        ]),
        Line::from(vec![
            Span::styled("Y", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          复制到剪贴板"),
        ]),
        Line::from(vec![
            Span::styled("d", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          删除任务"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("设置优先级 (Space t)", Style::default().fg(Color::Rgb(235, 203, 139)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("h", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          高优先级"),
        ]),
        Line::from(vec![
            Span::styled("m", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          中优先级"),
        ]),
        Line::from(vec![
            Span::styled("l", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          低优先级"),
        ]),
        Line::from(vec![
            Span::styled("n", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          无优先级"),
        ]),
    ];

    // 中列：项目管理
    let project_help = vec![
        Line::from(vec![
            Span::styled("任务移动", Style::default().fg(Color::Rgb(235, 203, 139)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("H", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          任务移到左列"),
        ]),
        Line::from(vec![
            Span::styled("L", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          任务移到右列"),
        ]),
        Line::from(vec![
            Span::styled("J", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          任务在列内下移"),
        ]),
        Line::from(vec![
            Span::styled("K", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          任务在列内上移"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("项目管理", Style::default().fg(Color::Rgb(235, 203, 139)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("n", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          新建本地项目"),
        ]),
        Line::from(vec![
            Span::styled("N", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          新建全局项目"),
        ]),
        Line::from(vec![
            Span::styled("D", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          删除项目文件"),
        ]),
        Line::from(vec![
            Span::styled("Space f", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("     快速切换项目"),
        ]),
        Line::from(vec![
            Span::styled("Space r", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("     重新加载当前"),
        ]),
        Line::from(vec![
            Span::styled("Space R", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("     重新加载所有"),
        ]),
        Line::from(vec![
            Span::styled("Space q", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("     退出程序"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("状态管理 (Space s)", Style::default().fg(Color::Rgb(235, 203, 139)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  a", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("         创建新状态"),
        ]),
        Line::from(vec![
            Span::styled("  r", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("         重命名状态"),
        ]),
        Line::from(vec![
            Span::styled("  e", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("         编辑显示名"),
        ]),
        Line::from(vec![
            Span::styled("  h", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("         状态列左移"),
        ]),
        Line::from(vec![
            Span::styled("  l", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("         状态列右移"),
        ]),
        Line::from(vec![
            Span::styled("  d", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("         删除状态"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("布局控制", Style::default().fg(Color::Rgb(235, 203, 139)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("+", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          增加当前列宽"),
        ]),
        Line::from(vec![
            Span::styled("-", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          减少当前列宽"),
        ]),
        Line::from(vec![
            Span::styled("=", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          重置列宽等分"),
        ]),
        Line::from(vec![
            Span::styled("m", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("          最大化/恢复列"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("窗口管理", Style::default().fg(Color::Rgb(235, 203, 139)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Space w w", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("   下一个窗口"),
        ]),
        Line::from(vec![
            Span::styled("Space w v", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("   垂直分屏"),
        ]),
        Line::from(vec![
            Span::styled("Space w s", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("   水平分屏"),
        ]),
        Line::from(vec![
            Span::styled("Space w m", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("   最大化/恢复"),
        ]),
        Line::from(vec![
            Span::styled("Space w q", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("   关闭/清空窗口"),
        ]),
        Line::from(vec![
            Span::styled("Space w hjkl", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw(" 移动焦点"),
        ]),
    ];

    // 右列：对话框操作
    let dialog_help = vec![
        Line::from(vec![
            Span::styled("Space 菜单", Style::default().fg(Color::Rgb(235, 203, 139)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Space", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("       打开主菜单"),
        ]),
        Line::from(vec![
            Span::styled("  p", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("         项目操作子菜单"),
        ]),
        Line::from(vec![
            Span::styled("  w", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("         窗口操作子菜单"),
        ]),
        Line::from(vec![
            Span::styled("  t", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("         任务操作子菜单"),
        ]),
        Line::from(vec![
            Span::styled("    a/e/E/v/V/d/Y", Style::default().fg(Color::Rgb(129, 161, 193))),  // 略浅的蓝色
            Span::raw(" 任务操作"),
        ]),
        Line::from(vec![
            Span::styled("    h/m/l/n", Style::default().fg(Color::Rgb(129, 161, 193))),
            Span::raw("     优先级: 高/中/低/无"),
        ]),
        Line::from(vec![
            Span::styled("  ESC", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("       返回上级/退出"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("输入框 (多行)", Style::default().fg(Color::Rgb(235, 203, 139)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("      确认提交"),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+J", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("     换行（任务输入）"),
        ]),
        Line::from(vec![
            Span::styled("  ←/→", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("        移动光标"),
        ]),
        Line::from(vec![
            Span::styled("  ↑/↓", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("        上下移动（多行）"),
        ]),
        Line::from(vec![
            Span::styled("  Home/End", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("   行首/行尾"),
        ]),
        Line::from(vec![
            Span::styled("  ESC", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("        取消"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("选择对话框", Style::default().fg(Color::Rgb(235, 203, 139)).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ↑/↓", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("        上下选择"),
        ]),
        Line::from(vec![
            Span::styled("  字符", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("        搜索过滤"),
        ]),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(Color::Rgb(136, 192, 208))),
            Span::raw("      确认选择"),
        ]),
        Line::from(""),
        // Line::from(vec![
        //     Span::styled("命令模式", Style::default().fg(Color::Rgb(235, 203, 139)).add_modifier(Modifier::BOLD)),
        // ]),
        // Line::from(""),
        // Line::from(vec![
        //     Span::styled("  :q", Style::default().fg(Color::Rgb(136, 192, 208))),
        //     Span::raw("         退出程序"),
        // ]),
        // Line::from(vec![
        //     Span::styled("  :help", Style::default().fg(Color::Rgb(136, 192, 208))),
        //     Span::raw("      显示帮助"),
        // ]),
        // Line::from(vec![
        //     Span::styled("  :reload", Style::default().fg(Color::Rgb(136, 192, 208))),
        //     Span::raw("    重新加载"),
        // ]),
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
