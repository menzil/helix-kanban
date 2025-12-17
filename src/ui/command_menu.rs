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
    color: Option<Color>,
}

/// 渲染空格命令菜单（小型弹窗样式）
pub fn render(f: &mut Frame, area: Rect, app: &App) {
    // 渲染半透明背景遮罩
    render_backdrop(f, area);

    // 创建小型居中弹窗（固定宽度40，高度自适应内容）
    let popup_area = centered_rect_fixed(40, 20, area);

    // 清空弹窗区域
    f.render_widget(Clear, popup_area);

    use crate::app::MenuState;

    // 根据菜单状态显示不同的命令列表
    let (commands, title) = match app.menu_state {
        Some(MenuState::Project) => {
            // 检查当前项目是否是当前目录的本地项目
            let is_current_local_project = if let Some(project) = app.get_focused_project() {
                if project.project_type == crate::models::ProjectType::Local {
                    let current_local_dir = crate::fs::get_local_kanban_dir();
                    project.path.starts_with(&current_local_dir)
                } else {
                    false
                }
            } else {
                false
            };

            let mut commands = vec![
                CommandItem { key: "o", label: "打开项目", color: None },
                CommandItem { key: "", label: "", color: None },
                CommandItem { key: "n", label: "新建本地项目 [L]", color: None },
                CommandItem { key: "N", label: "新建全局项目 [G]", color: None },
                CommandItem { key: "", label: "", color: None },
            ];

            // 根据项目类型决定显示哪些删除选项
            if is_current_local_project {
                // 当前目录的本地项目：只能硬删除
                commands.push(CommandItem { key: "D", label: "删除项目文件", color: None });
            } else {
                // 全局项目或其他目录的本地项目：可以软删除或硬删除
                commands.push(CommandItem { key: "d", label: "隐藏项目（软删除）", color: None });
                commands.push(CommandItem { key: "D", label: "删除项目文件", color: None });
            }

            commands.push(CommandItem { key: "", label: "", color: None });
            commands.push(CommandItem { key: "r", label: "重命名项目", color: None });
            commands.push(CommandItem { key: "i", label: "复制项目信息", color: None });

            (commands, " 项目操作 ")
        },
        Some(MenuState::Window) => (
            vec![
                CommandItem { key: "w", label: "下一个窗口", color: None },
                CommandItem { key: "", label: "", color: None },
                CommandItem { key: "v", label: "垂直分屏", color: None },
                CommandItem { key: "s", label: "水平分屏", color: None },
                CommandItem { key: "q", label: "关闭当前窗口", color: None },
                CommandItem { key: "m", label: "最大化/恢复", color: None },
                CommandItem { key: "", label: "", color: None },
                CommandItem { key: "h", label: "聚焦左侧", color: None },
                CommandItem { key: "l", label: "聚焦右侧", color: None },
                CommandItem { key: "k", label: "聚焦上方", color: None },
                CommandItem { key: "j", label: "聚焦下方", color: None },
            ],
            " 窗口操作 "
        ),
        Some(MenuState::Task) => (
            vec![
                CommandItem { key: "a", label: "新建任务", color: None },
                CommandItem { key: "e", label: "编辑任务", color: None },
                CommandItem { key: "E", label: "用编辑器编辑", color: None },
                CommandItem { key: "", label: "", color: None },
                CommandItem { key: "v", label: "预览任务", color: None },
                CommandItem { key: "V", label: "外部预览", color: None },
                CommandItem { key: "", label: "", color: None },
                CommandItem { key: "Y", label: "复制到剪贴板", color: None },
                CommandItem { key: "d", label: "删除任务", color: None },
                CommandItem { key: "", label: "", color: None },
                CommandItem { key: "h", label: "优先级：高", color: Some(Color::Red) },
                CommandItem { key: "m", label: "优先级：中", color: Some(Color::Yellow) },
                CommandItem { key: "l", label: "优先级：低", color: Some(Color::Green) },
                CommandItem { key: "n", label: "优先级：无", color: None },
            ],
            " 任务操作 "
        ),
        Some(MenuState::Main) | None => (
            vec![
                CommandItem { key: "f", label: "快速切换项目", color: None },
                CommandItem { key: "", label: "", color: None },
                CommandItem { key: "p", label: "项目操作...", color: None },
                CommandItem { key: "w", label: "窗口操作...", color: None },
                CommandItem { key: "t", label: "任务操作...", color: None },
                CommandItem { key: "", label: "", color: None },
                CommandItem { key: "r", label: "重新加载当前项目", color: None },
                CommandItem { key: "R", label: "重新加载所有项目", color: None },
                CommandItem { key: "", label: "", color: None },
                CommandItem { key: "?", label: "显示帮助", color: None },
                CommandItem { key: "q", label: "退出", color: None },
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
                let mut spans = vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("{:3}", cmd.key),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                ];

                // 如果有颜色，添加彩色圆点
                if let Some(color) = cmd.color {
                    spans.push(Span::styled("● ", Style::default().fg(color)));
                }

                spans.push(Span::styled(cmd.label, Style::default().fg(Color::White)));

                let line = Line::from(spans);
                ListItem::new(line)
            }
        })
        .collect();

    let list = List::new(list_items).block(
        Block::default()
            .title(title)
            .title_style(
                Style::default()
                    .fg(Color::Rgb(235, 203, 139))  // Nord yellow
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(136, 192, 208)))  // Nord cyan
            .border_type(ratatui::widgets::BorderType::Rounded)
            .style(Style::default().bg(Color::Rgb(46, 52, 64))),  // Nord background
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
