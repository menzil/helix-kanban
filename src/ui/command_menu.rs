use crate::app::App;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem};
use ratatui::Frame;

/// 命令定义
struct CommandItem {
    key: &'static str,
    label: &'static str,
}

/// 渲染空格命令菜单（小型弹窗样式）
pub fn render(f: &mut Frame, area: Rect, app: &App) {
    // 渲染半透明背景遮罩
    render_backdrop(f, area);

    // 创建小型居中弹窗（固定宽度40，高度自适应内容）
    let popup_area = centered_rect_fixed(40, 18, area);

    // 清空弹窗区域
    f.render_widget(Clear, popup_area);

    use crate::app::MenuState;

    // 根据菜单状态显示不同的命令列表
    let (commands, title) = match app.menu_state {
        Some(MenuState::Project) => (
            vec![
                CommandItem { key: "o", label: "打开项目" },
                CommandItem { key: "n", label: "新建项目" },
                CommandItem { key: "d", label: "删除项目" },
                CommandItem { key: "r", label: "重命名项目" },
            ],
            " 项目操作 "
        ),
        Some(MenuState::Window) => (
            vec![
                CommandItem { key: "v", label: "垂直分屏" },
                CommandItem { key: "s", label: "水平分屏" },
                CommandItem { key: "c", label: "关闭面板" },
                CommandItem { key: "", label: "" },
                CommandItem { key: "h", label: "聚焦左侧" },
                CommandItem { key: "l", label: "聚焦右侧" },
                CommandItem { key: "k", label: "聚焦上方" },
                CommandItem { key: "j", label: "聚焦下方" },
            ],
            " 窗口操作 "
        ),
        Some(MenuState::Task) => (
            vec![
                CommandItem { key: "n", label: "新建任务" },
                CommandItem { key: "e", label: "编辑任务" },
                CommandItem { key: "d", label: "删除任务" },
            ],
            " 任务操作 "
        ),
        Some(MenuState::Main) | None => (
            vec![
                CommandItem { key: "f", label: "快速切换项目" },
                CommandItem { key: "", label: "" },
                CommandItem { key: "p", label: "项目操作..." },
                CommandItem { key: "w", label: "窗口操作..." },
                CommandItem { key: "t", label: "任务操作..." },
                CommandItem { key: "", label: "" },
                CommandItem { key: "?", label: "显示帮助" },
            ],
            " 命令菜单 "
        ),
    };

    // 构建列表项
    let list_items: Vec<ListItem> = commands
        .iter()
        .map(|cmd| {
            if cmd.key.is_empty() {
                // 空行分隔符
                ListItem::new("")
            } else {
                let line = Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("{:3}", cmd.key),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                    Span::styled(cmd.label, Style::default().fg(Color::White)),
                ]);
                ListItem::new(line)
            }
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .title(title)
            .title_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .border_type(ratatui::widgets::BorderType::Rounded),
    );

    f.render_widget(list, popup_area);

    // 底部提示
    let help_area = Rect {
        x: popup_area.x,
        y: popup_area.y + popup_area.height,
        width: popup_area.width,
        height: 1,
    };

    if help_area.y < area.height {
        let help_text = match app.menu_state {
            Some(MenuState::Main) => "选择分类进入子菜单",
            _ => "输入字母键执行命令",
        };

        let help_line = Line::from(vec![
            Span::styled("  ", Style::default()),
            Span::styled(help_text, Style::default().fg(Color::DarkGray)),
            Span::styled("  ", Style::default()),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::styled(" 返回", Style::default().fg(Color::DarkGray)),
        ]);

        f.render_widget(
            ratatui::widgets::Paragraph::new(help_line)
                .alignment(Alignment::Center)
                .style(Style::default().bg(Color::Rgb(0, 0, 0))),
            help_area,
        );
    }
}

/// 创建固定大小的居中矩形
fn centered_rect_fixed(width: u16, height: u16, r: Rect) -> Rect {
    let x = r.x + (r.width.saturating_sub(width)) / 2;
    let y = r.y + (r.height.saturating_sub(height)) / 2;

    Rect {
        x,
        y,
        width: width.min(r.width),
        height: height.min(r.height),
    }
}

/// 渲染半透明背景遮罩
fn render_backdrop(f: &mut Frame, area: Rect) {
    let block = Block::default().style(Style::default().bg(Color::Rgb(0, 0, 0)));
    f.render_widget(block, area);
}
