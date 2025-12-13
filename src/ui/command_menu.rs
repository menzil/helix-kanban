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
                CommandItem { key: "o", label: "打开项目" },
                CommandItem { key: "", label: "" },
                CommandItem { key: "n", label: "新建本地项目 [L]" },
                CommandItem { key: "N", label: "新建全局项目 [G]" },
                CommandItem { key: "", label: "" },
            ];

            // 根据项目类型决定显示哪些删除选项
            if is_current_local_project {
                // 当前目录的本地项目：只能硬删除
                commands.push(CommandItem { key: "D", label: "删除项目文件" });
            } else {
                // 全局项目或其他目录的本地项目：可以软删除或硬删除
                commands.push(CommandItem { key: "d", label: "隐藏项目（软删除）" });
                commands.push(CommandItem { key: "D", label: "删除项目文件" });
            }

            commands.push(CommandItem { key: "", label: "" });
            commands.push(CommandItem { key: "r", label: "重命名项目" });

            (commands, " 项目操作 ")
        },
        Some(MenuState::Window) => (
            vec![
                CommandItem { key: "w", label: "下一个窗口" },
                CommandItem { key: "", label: "" },
                CommandItem { key: "v", label: "垂直分屏" },
                CommandItem { key: "s", label: "水平分屏" },
                CommandItem { key: "q", label: "关闭当前窗口" },
                CommandItem { key: "m", label: "最大化/恢复" },
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
                CommandItem { key: "a", label: "新建任务" },
                CommandItem { key: "e", label: "编辑任务" },
                CommandItem { key: "E", label: "用编辑器编辑" },
                CommandItem { key: "", label: "" },
                CommandItem { key: "v", label: "预览任务" },
                CommandItem { key: "V", label: "外部预览" },
                CommandItem { key: "", label: "" },
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
                CommandItem { key: "r", label: "重新加载当前项目" },
                CommandItem { key: "R", label: "重新加载所有项目" },
                CommandItem { key: "", label: "" },
                CommandItem { key: "?", label: "显示帮助" },
                CommandItem { key: "q", label: "退出" },
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
