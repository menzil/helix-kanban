use crate::app::{App, Mode};
use crate::input::Command;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// 处理键盘输入
/// 返回 false 表示应该退出应用
pub fn handle_key_input(app: &mut App, key: KeyEvent) -> bool {
    // 如果显示欢迎对话框，任意按键都关闭它
    if app.show_welcome_dialog {
        app.show_welcome_dialog = false;
        return true;
    }

    match app.mode {
        Mode::Normal => handle_normal_mode(app, key),
        // Mode::Command => handle_command_mode(app, key), // 已注释
        Mode::TaskSelect => handle_task_select_mode(app, key),
        Mode::Dialog => handle_dialog_mode(app, key),
        Mode::Help => handle_help_mode(app, key),
        Mode::SpaceMenu => handle_space_menu_mode(app, key),
        Mode::Preview => handle_preview_mode(app, key),
    }
}

/// 处理正常模式的按键
fn handle_normal_mode(app: &mut App, key: KeyEvent) -> bool {
    // 特殊处理帮助键
    if let KeyCode::Char('?') = key.code {
        app.mode = Mode::Help;
        app.key_buffer.clear();
        return true;
    }

    // 特殊处理空格键 - 显示命令菜单
    if let KeyCode::Char(' ') = key.code {
        // 空格键总是清空缓冲区并显示菜单
        app.key_buffer.clear();
        app.mode = Mode::SpaceMenu;
        app.menu_state = Some(crate::app::MenuState::Main);
        app.menu_selected_index = Some(0); // 初始化选中第一项
        return true;
    }

    // 尝试匹配命令（使用当前缓冲区和新按键）
    if let Some(cmd) = match_key_sequence(&app.key_buffer, key) {
        app.key_buffer.clear();

        // 特殊处理退出命令
        if cmd == Command::Quit {
            return false;
        }

        execute_command(app, cmd);
        return true;
    }

    // 如果没有匹配到命令，清空缓冲区
    // 因为现在所有多键序列都通过 SpaceMenu 处理，不需要缓冲未匹配的按键
    app.key_buffer.clear();

    true
}

// /// 处理命令模式的按键 - 已注释
// fn handle_command_mode(app: &mut App, key: KeyEvent) -> bool {
//     match key.code {
//         KeyCode::Esc => {
//             app.mode = Mode::Normal;
//             app.command_input.clear();
//             app.completion_selected_index = None;
//         }
//         KeyCode::Tab => {
//             // 下一个补全项
//             let matches = app.command_registry.find_matches(&app.command_input);
//             if !matches.is_empty() {
//                 let current = app.completion_selected_index.unwrap_or(0);
//                 let next = if current + 1 >= matches.len() { 0 } else { current + 1 };
//                 app.completion_selected_index = Some(next);
//             }
//         }
//         KeyCode::BackTab => {
//             // 上一个补全项
//             let matches = app.command_registry.find_matches(&app.command_input);
//             if !matches.is_empty() {
//                 let current = app.completion_selected_index.unwrap_or(0);
//                 let prev = if current == 0 { matches.len() - 1 } else { current - 1 };
//                 app.completion_selected_index = Some(prev);
//             }
//         }
//         KeyCode::Enter => {
//             // 执行命令
//             let should_continue = execute_text_command(app, &app.command_input.clone());
//             app.command_input.clear();
//             app.mode = Mode::Normal;
//             app.completion_selected_index = None;
//             return should_continue;
//         }
//         KeyCode::Backspace => {
//             app.command_input.pop();
//         }
//         KeyCode::Char(c) => {
//             app.command_input.push(c);
//         }
//         _ => {}
//     }
//     true
// }

/// 处理任务选择模式的按键
fn handle_task_select_mode(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            execute_command(app, Command::TaskDown);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            execute_command(app, Command::TaskUp);
        }
        _ => {}
    }
    true
}

/// 处理对话框模式的按键
fn handle_dialog_mode(app: &mut App, key: KeyEvent) -> bool {
    use crate::ui::dialogs::DialogType;
    use crate::ui::text_input::InputAction;

    if let Some(dialog) = &mut app.dialog {
        match dialog {
            DialogType::Input { textarea, .. } => {
                // 使用 HelixTextArea 处理按键
                match textarea.handle_key(key) {
                    InputAction::Submit => {
                        // 提交内容
                        let content = textarea.get_content();
                        let dialog_clone = app.dialog.take().unwrap();
                        handle_dialog_submit(app, dialog_clone, content);
                        app.mode = Mode::Normal;
                        // app.ime_state.exit_dialog();  // 已禁用输入法自动切换
                        return true;
                    }
                    InputAction::Cancel => {
                        // 取消对话框
                        app.dialog = None;
                        app.mode = Mode::Normal;
                        // app.ime_state.exit_dialog();  // 已禁用输入法自动切换
                        return true;
                    }
                    InputAction::Continue => {
                        // 继续编辑
                        return true;
                    }
                }
            }
            DialogType::Select {
                items,
                selected,
                filter,
                ..
            } => {
                match key.code {
                    KeyCode::Esc => {
                        app.dialog = None;
                        app.mode = Mode::Normal;
                        // 退出对话框，保存用户输入法并切换回英文（已禁用）
                        // app.ime_state.exit_dialog();
                    }
                    KeyCode::Enter => {
                        // 选择当前项
                        let filtered_items: Vec<_> = if filter.is_empty() {
                            items.clone()
                        } else {
                            items
                                .iter()
                                .filter(|item| item.to_lowercase().contains(&filter.to_lowercase()))
                                .cloned()
                                .collect()
                        };

                        if *selected < filtered_items.len() {
                            let selected_item = filtered_items[*selected].clone();
                            let dialog_clone = app.dialog.take().unwrap();
                            app.mode = Mode::Normal;
                            // app.ime_state.exit_dialog();  // 已禁用输入法自动切换
                            handle_dialog_submit(app, dialog_clone, selected_item);
                            return true;
                        }
                    }
                    KeyCode::Up => {
                        // 向上移动选择
                        if *selected > 0 {
                            *selected -= 1;
                        }
                    }
                    KeyCode::Down => {
                        // 向下移动选择
                        let filtered_count = if filter.is_empty() {
                            items.len()
                        } else {
                            items
                                .iter()
                                .filter(|item| item.to_lowercase().contains(&filter.to_lowercase()))
                                .count()
                        };

                        if filtered_count > 0 && *selected < filtered_count - 1 {
                            *selected += 1;
                        }
                    }
                    KeyCode::Backspace => {
                        filter.pop();
                        *selected = 0;
                    }
                    KeyCode::Char(c) => {
                        filter.push(c);
                        *selected = 0;
                    }
                    _ => {}
                }
            }
            DialogType::Confirm { yes_selected, .. } => {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('n') => {
                        app.dialog = None;
                        app.mode = Mode::Normal;
                        // 退出对话框，保存用户输入法并切换回英文（已禁用）
                        // app.ime_state.exit_dialog();
                    }
                    KeyCode::Enter => {
                        let confirmed = *yes_selected;
                        let dialog_clone = app.dialog.take().unwrap();
                        app.mode = Mode::Normal;
                        // app.ime_state.exit_dialog();  // 已禁用输入法自动切换
                        if confirmed {
                            handle_dialog_submit(app, dialog_clone, String::new());
                        }
                        return true;
                    }
                    KeyCode::Left | KeyCode::Char('h') => {
                        *yes_selected = false; // 左边是"否"
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        *yes_selected = true; // 右边是"是"
                    }
                    KeyCode::Char('y') => {
                        *yes_selected = true;
                        // 直接确认
                        let dialog_clone = app.dialog.take().unwrap();
                        app.mode = Mode::Normal;
                        // app.ime_state.exit_dialog();  // 已禁用输入法自动切换
                        handle_dialog_submit(app, dialog_clone, String::new());
                        return true;
                    }
                    _ => {}
                }
            }
        }
    }

    true
}

/// 调试日志辅助函数
fn log_debug(msg: String) {
    use std::fs::OpenOptions;
    use std::io::Write;

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/kanban_debug.log")
    {
        let _ = writeln!(
            file,
            "[{}] {}",
            chrono::Local::now().format("%H:%M:%S"),
            msg
        );
    }
    // 移除 eprintln! 避免干扰 TUI 界面
}

/// 处理对话框提交
fn handle_dialog_submit(app: &mut App, dialog: crate::ui::dialogs::DialogType, value: String) {
    use crate::ui::dialogs::DialogType;

    match dialog {
        DialogType::Input { title, .. } => {
            log_debug(format!("对话框提交: title='{}', value='{}'", title, value));

            if (title.contains("创建") || title.contains("新建")) && title.contains("项目") {
                // 创建新项目
                if !value.is_empty() {
                    log_debug(format!("调试: 准备创建项目 '{}'", value));

                    // 根据标题判断是本地项目还是全局项目
                    let is_local = title.contains("[L]");
                    let is_global = title.contains("[G]");

                    let result = if is_local {
                        // 创建本地项目
                        log_debug("调试: 创建本地项目".to_string());
                        crate::fs::create_local_project(&value)
                    } else if is_global {
                        // 创建全局项目
                        log_debug("调试: 创建全局项目".to_string());
                        crate::fs::create_project(&value)
                    } else {
                        // 默认创建全局项目（向后兼容）
                        log_debug("调试: 创建默认项目（全局）".to_string());
                        crate::fs::create_project(&value)
                    };

                    match result {
                        Ok(path) => {
                            log_debug(format!("调试: 项目创建成功于 {:?}", path));
                            // 重新加载项目列表
                            match crate::fs::load_all_projects() {
                                Ok(projects) => {
                                    log_debug(format!(
                                        "调试: 重新加载了 {} 个项目",
                                        projects.len()
                                    ));
                                    app.projects = projects;
                                }
                                Err(e) => {
                                    log_debug(format!("调试: 重新加载项目失败: {}", e));
                                }
                            }
                            // 在当前面板打开新项目
                            app.set_focused_project(value);
                        }
                        Err(e) => {
                            log_debug(format!("创建项目失败: {}", e));
                        }
                    }
                } else {
                    log_debug("调试: 项目名称为空".to_string());
                }
            } else if (title.contains("创建") || title.contains("新建")) && title.contains("任务")
            {
                // 创建新任务
                log_debug("调试: 识别为创建任务请求".to_string());
                if !value.is_empty() {
                    create_new_task(app, value);
                } else {
                    log_debug("调试: 任务标题为空".to_string());
                }
            } else if title.contains("编辑任务") {
                // 编辑任务
                if !value.is_empty() {
                    update_task_title(app, value);
                }
            } else if title.contains("编辑标签") {
                // 编辑标签
                update_task_tags(app, value);
            } else if title.contains("重命名项目") {
                // 重命名项目
                if !value.is_empty() {
                    rename_current_project(app, value);
                }
            } else if title.contains("创建新状态") {
                // 创建新状态
                if !value.is_empty()
                    && let Some(project) = app.get_focused_project() {
                        let project_path = project.path.clone();

                        // 使用输入值作为显示名
                        match crate::fs::status::create_status(&project_path, &value, &value) {
                            Ok(_) => {
                                // 重新加载项目
                                if let Err(e) = app.reload_current_project() {
                                    log_debug(format!("重新加载项目失败: {}", e));
                                }
                                app.show_notification(
                                    format!("已创建状态「{}」", value),
                                    crate::app::NotificationLevel::Success,
                                );
                            }
                            Err(e) => {
                                app.show_notification(
                                    format!("创建失败: {}", e),
                                    crate::app::NotificationLevel::Error,
                                );
                            }
                        }
                    }
            } else if title.contains("重命名状态") {
                // 重命名状态
                if !value.is_empty()
                    && let Some(project) = app.get_focused_project() {
                        let column = app
                            .selected_column
                            .get(&app.focused_pane)
                            .copied()
                            .unwrap_or(0);
                        if let Some(status) = project.statuses.get(column) {
                            let old_name = status.name.clone();
                            let old_display = status.display.clone();
                            let project_path = project.path.clone();

                            match crate::fs::status::rename_status(
                                &project_path,
                                &old_name,
                                &value,
                                &value,
                            ) {
                                Ok(_) => {
                                    // 重新加载项目
                                    if let Err(e) = app.reload_current_project() {
                                        log_debug(format!("重新加载项目失败: {}", e));
                                    }
                                    app.show_notification(
                                        format!("已将「{}」重命名为「{}」", old_display, value),
                                        crate::app::NotificationLevel::Success,
                                    );
                                }
                                Err(e) => {
                                    app.show_notification(
                                        format!("重命名失败: {}", e),
                                        crate::app::NotificationLevel::Error,
                                    );
                                }
                            }
                        }
                    }
            } else if title.contains("编辑显示名") {
                // 编辑状态显示名
                if !value.is_empty()
                    && let Some(project) = app.get_focused_project() {
                        let column = app
                            .selected_column
                            .get(&app.focused_pane)
                            .copied()
                            .unwrap_or(0);
                        if let Some(status) = project.statuses.get(column) {
                            let status_name = status.name.clone();
                            let project_path = project.path.clone();

                            match crate::fs::status::update_status_display(
                                &project_path,
                                &status_name,
                                &value,
                            ) {
                                Ok(_) => {
                                    // 重新加载项目
                                    if let Err(e) = app.reload_current_project() {
                                        log_debug(format!("重新加载项目失败: {}", e));
                                    }
                                    app.show_notification(
                                        format!("已更新显示名为「{}」", value),
                                        crate::app::NotificationLevel::Success,
                                    );
                                }
                                Err(e) => {
                                    app.show_notification(
                                        format!("更新失败: {}", e),
                                        crate::app::NotificationLevel::Error,
                                    );
                                }
                            }
                        }
                    }
            }
        }
        DialogType::Select { title, .. } => {
            if title.contains("选择项目")
                || title.contains("打开项目")
                || title.contains("切换项目")
            {
                // 从格式化的字符串中提取项目名
                // 格式: "[G/L] 项目名\n    路径"
                let project_name = value
                    .lines()
                    .next()
                    .unwrap_or(&value)
                    .trim_start_matches("[G] ")
                    .trim_start_matches("[L] ")
                    .trim();

                // 打开选中的项目
                app.set_focused_project(project_name.to_string());
            }
        }
        DialogType::Confirm { action, .. } => {
            match action {
                crate::ui::dialogs::ConfirmAction::HideProject => {
                    // 隐藏项目（软删除）
                    if let Some(project) = app.get_focused_project() {
                        let project_name = project.name.clone();

                        // 添加到隐藏列表
                        if let Err(e) = crate::config::hide_project(&mut app.config, &project_name)
                        {
                            log_debug(format!("隐藏项目失败: {}", e));
                        } else {
                            log_debug(format!("成功隐藏项目: {}", project_name));

                            // 从项目列表中移除
                            app.projects.retain(|p| p.name != project_name);

                            // 清除当前面板的项目引用
                            if let Some(crate::ui::layout::SplitNode::Leaf { project_id, .. }) =
                                app.split_tree.find_pane_mut(app.focused_pane)
                            {
                                *project_id = None;
                            }
                        }
                    }
                }
                crate::ui::dialogs::ConfirmAction::DeleteProject => {
                    // 删除项目文件（硬删除）
                    log_debug("收到 DeleteProject 确认".to_string());
                    if let Some(project) = app.get_focused_project() {
                        let project_name = project.name.clone();
                        let project_path = project.path.clone();

                        log_debug(format!(
                            "准备删除项目: 名称='{}', 路径={:?}",
                            project_name, project_path
                        ));

                        // 使用项目路径直接删除
                        match crate::fs::delete_project_by_path(&project_path) {
                            Err(e) => {
                                log_debug(format!("删除项目失败: {}", e));
                                app.show_notification(
                                    format!("删除项目失败: {}", e),
                                    crate::app::NotificationLevel::Error,
                                );
                            }
                            Ok(_) => {
                                log_debug(format!("成功删除项目: {}", project_name));

                                // 从项目列表中移除
                                app.projects.retain(|p| p.name != project_name);
                                log_debug(format!(
                                    "已从项目列表移除，剩余项目数: {}",
                                    app.projects.len()
                                ));

                                // 清除所有面板中对该项目的引用
                                app.split_tree.clear_project_from_all_panes(&project_name);
                                log_debug("已清除所有面板中的项目引用".to_string());

                                // 显示删除成功通知
                                app.show_notification(
                                    format!("已删除项目: {}", project_name),
                                    crate::app::NotificationLevel::Success,
                                );
                            }
                        }
                    } else {
                        log_debug("无法获取当前聚焦的项目".to_string());
                    }
                }
                crate::ui::dialogs::ConfirmAction::DeleteTask => {
                    // 删除任务
                    if let Some(task) = get_selected_task(app) {
                        // 获取项目路径
                        if let Some(project) = app.get_focused_project() {
                            let project_path = project.path.clone();

                            // 删除任务（包括文件和 tasks.toml 中的元数据）
                            if let Err(e) = crate::fs::delete_task(&project_path, task) {
                                log_debug(format!("删除任务失败: {}", e));
                            } else {
                                // 重新加载当前项目
                                if let Err(e) = app.reload_current_project() {
                                    log_debug(format!("重新加载项目失败: {}", e));
                                }

                                // 调整选中的任务索引
                                let task_idx =
                                    app.selected_task_index.entry(app.focused_pane).or_insert(0);
                                if *task_idx > 0 {
                                    *task_idx -= 1;
                                }
                            }
                        }
                    }
                }
                crate::ui::dialogs::ConfirmAction::DeleteStatus => {
                    // 删除状态
                    if let Some(project) = app.get_focused_project() {
                        let column = app
                            .selected_column
                            .get(&app.focused_pane)
                            .copied()
                            .unwrap_or(0);
                        if let Some(status) = project.statuses.get(column) {
                            let status_name = status.name.clone();
                            let status_display = status.display.clone();
                            let project_path = project.path.clone();

                            match crate::fs::status::delete_status(
                                &project_path,
                                &status_name,
                                None,
                            ) {
                                Ok(_) => {
                                    log_debug(format!("成功删除状态: {}", status_name));

                                    // 重新加载项目
                                    if let Err(e) = app.reload_current_project() {
                                        log_debug(format!("重新加载项目失败: {}", e));
                                    }

                                    // 调整选中列到第一列
                                    app.selected_column.insert(app.focused_pane, 0);

                                    app.show_notification(
                                        format!("已删除状态「{}」", status_display),
                                        crate::app::NotificationLevel::Success,
                                    );
                                }
                                Err(e) => {
                                    log_debug(format!("删除状态失败: {}", e));
                                    app.show_notification(
                                        format!("删除失败: {}", e),
                                        crate::app::NotificationLevel::Error,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 匹配键序列到命令
/// buffer: 之前按过的键（不包含当前键）
/// key: 当前正在按的键
pub fn match_key_sequence(buffer: &[char], key: KeyEvent) -> Option<Command> {
    match (buffer, key.code, key.modifiers) {
        // ===== 单键命令（空缓冲区）=====
        // 注意：移除了 'q' 键退出，改用 ':q' 命令或 Space q
        ([], KeyCode::Char('j'), KeyModifiers::NONE) => Some(Command::TaskDown),
        ([], KeyCode::Char('k'), KeyModifiers::NONE) => Some(Command::TaskUp),
        ([], KeyCode::Char('h'), KeyModifiers::NONE) => Some(Command::ColumnLeft),
        ([], KeyCode::Char('l'), KeyModifiers::NONE) => Some(Command::ColumnRight),
        ([], KeyCode::Char('H'), KeyModifiers::SHIFT) => Some(Command::MoveTaskLeft),
        ([], KeyCode::Char('L'), KeyModifiers::SHIFT) => Some(Command::MoveTaskRight),
        ([], KeyCode::Char('J'), KeyModifiers::SHIFT) => Some(Command::MoveTaskDown),
        ([], KeyCode::Char('K'), KeyModifiers::SHIFT) => Some(Command::MoveTaskUp),
        // ([], KeyCode::Char(':'), KeyModifiers::NONE) => Some(Command::EnterCommandMode), // 已注释
        ([], KeyCode::Esc, _) => Some(Command::EnterNormalMode),
        ([], KeyCode::Char('d'), KeyModifiers::NONE) => Some(Command::DeleteTask), // 删除任务
        ([], KeyCode::Char('D'), KeyModifiers::SHIFT) => Some(Command::DeleteProject), // 硬删除项目
        ([], KeyCode::Char('a'), KeyModifiers::NONE) => Some(Command::NewTask),
        ([], KeyCode::Char('A'), KeyModifiers::SHIFT) => Some(Command::NewTaskInEditor), // 外部编辑器创建任务
        ([], KeyCode::Char('n'), KeyModifiers::NONE) => Some(Command::NewLocalProject),
        ([], KeyCode::Char('N'), KeyModifiers::SHIFT) => Some(Command::NewGlobalProject),
        ([], KeyCode::Char('e'), KeyModifiers::NONE) => Some(Command::EditTask),
        ([], KeyCode::Char('E'), KeyModifiers::SHIFT) => Some(Command::EditTaskInEditor),
        ([], KeyCode::Char('v'), KeyModifiers::NONE) => Some(Command::ViewTask),
        ([], KeyCode::Char('V'), KeyModifiers::SHIFT) => Some(Command::ViewTaskExternal),
        ([], KeyCode::Char('Y'), KeyModifiers::SHIFT) => Some(Command::CopyTask), // 复制任务到剪贴板
        ([], KeyCode::Char('t'), KeyModifiers::NONE) => Some(Command::EditTags),  // 编辑标签

        // 列宽调整
        ([], KeyCode::Char('+'), KeyModifiers::NONE) => Some(Command::IncreaseColumnWidth),
        ([], KeyCode::Char('-'), KeyModifiers::NONE) => Some(Command::DecreaseColumnWidth),
        ([], KeyCode::Char('='), KeyModifiers::NONE) => Some(Command::ResetColumnWidths),
        ([], KeyCode::Char('m'), KeyModifiers::NONE) => Some(Command::ToggleMaximizeColumn),

        ([], KeyCode::Down, _) => Some(Command::TaskDown),
        ([], KeyCode::Up, _) => Some(Command::TaskUp),
        ([], KeyCode::Left, _) => Some(Command::ColumnLeft),
        ([], KeyCode::Right, _) => Some(Command::ColumnRight),

        _ => None,
    }
}

/// 执行命令
fn execute_command(app: &mut App, cmd: Command) {
    use crate::ui::dialogs::DialogType;

    match cmd {
        Command::SplitHorizontal => {
            // 水平分割线 = 上下分屏
            log_debug(format!(
                "执行 SplitHorizontal, 当前焦点: {}",
                app.focused_pane
            ));
            if let Some(pane) = app.split_tree.find_pane_mut(app.focused_pane) {
                let new_pane_id = app.next_pane_id;
                pane.split_vertical(new_pane_id); // split_vertical 创建上下分屏
                app.next_pane_id += 1;
                // 自动对焦新创建的窗口
                app.focused_pane = new_pane_id;
                log_debug(format!(
                    "创建新面板 {}, 新焦点: {}",
                    new_pane_id, app.focused_pane
                ));

                // 保存状态
                let state = crate::state::extract_state(app);
                let _ = crate::state::save_state(&state);
            } else {
                log_debug("找不到当前面板".to_string());
            }
        }
        Command::SplitVertical => {
            // 垂直分割线 = 左右分屏
            log_debug(format!(
                "执行 SplitVertical, 当前焦点: {}",
                app.focused_pane
            ));
            if let Some(pane) = app.split_tree.find_pane_mut(app.focused_pane) {
                let new_pane_id = app.next_pane_id;
                pane.split_horizontal(new_pane_id); // split_horizontal 创建左右分屏
                app.next_pane_id += 1;
                // 自动对焦新创建的窗口
                app.focused_pane = new_pane_id;
                log_debug(format!(
                    "创建新面板 {}, 新焦点: {}",
                    new_pane_id, app.focused_pane
                ));

                // 保存状态
                let state = crate::state::extract_state(app);
                let _ = crate::state::save_state(&state);
            } else {
                log_debug("找不到当前面板".to_string());
            }
        }
        Command::TaskDown => {
            // 获取当前列的任务数量并限制索引
            if let Some(project) = app.get_focused_project() {
                let column = app
                    .selected_column
                    .get(&app.focused_pane)
                    .copied()
                    .unwrap_or(0);

                // 动态获取状态名称
                let status = if let Some(status_name) = app.get_status_name_by_column(column) {
                    status_name
                } else {
                    return;
                };

                let task_count = project.tasks.iter().filter(|t| t.status == status).count();

                if task_count > 0 {
                    let idx = app.selected_task_index.entry(app.focused_pane).or_insert(0);
                    *idx = (*idx + 1).min(task_count - 1);
                }
            }
        }
        Command::TaskUp => {
            let idx = app.selected_task_index.entry(app.focused_pane).or_insert(0);
            *idx = idx.saturating_sub(1);
        }
        Command::ColumnLeft => {
            let col = app.selected_column.entry(app.focused_pane).or_insert(0);
            *col = col.saturating_sub(1);
            // 切换列时重置任务索引到 0
            app.selected_task_index.insert(app.focused_pane, 0);
        }
        Command::ColumnRight => {
            let max_col = app.get_status_count().saturating_sub(1);
            let focused_pane = app.focused_pane;
            let col = app.selected_column.entry(focused_pane).or_insert(0);
            *col = (*col + 1).min(max_col);
            // 切换列时重置任务索引到 0
            app.selected_task_index.insert(focused_pane, 0);
        }
        // Command::EnterCommandMode => {
        //     app.mode = Mode::Command; // 已注释
        // }
        Command::EnterNormalMode | Command::Cancel => {
            app.mode = Mode::Normal;
            app.key_buffer.clear();
            app.command_input.clear(); // 已注释，但仍保留清理逻辑
        }
        Command::NewProject => {
            app.mode = Mode::Dialog;
            // app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法（已禁用）
            app.dialog = Some(DialogType::Input {
                title: "创建新项目".to_string(),
                prompt: "请输入项目名称:".to_string(),
                textarea: crate::ui::text_input::HelixTextArea::new(String::new(), true, false), // 默认 Insert 模式
            });
        }
        Command::NewLocalProject => {
            app.mode = Mode::Dialog;
            // app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法（已禁用）
            app.dialog = Some(DialogType::Input {
                title: "创建新本地项目 [L]".to_string(),
                prompt: "请输入项目名称:".to_string(),
                textarea: crate::ui::text_input::HelixTextArea::new(String::new(), true, false), // 默认 Insert 模式
            });
        }
        Command::NewGlobalProject => {
            app.mode = Mode::Dialog;
            // app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法（已禁用）
            app.dialog = Some(DialogType::Input {
                title: "创建新全局项目 [G]".to_string(),
                prompt: "请输入项目名称:".to_string(),
                textarea: crate::ui::text_input::HelixTextArea::new(String::new(), true, false), // 默认 Insert 模式
            });
        }
        Command::OpenProject => {
            app.mode = Mode::Dialog;
            // app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法（已禁用）
            // 生成格式化的项目列表：[G/L] 项目名\n    路径
            let project_items: Vec<String> = app
                .projects
                .iter()
                .map(|p| {
                    let type_marker = match p.project_type {
                        crate::models::ProjectType::Global => "[G]",
                        crate::models::ProjectType::Local => "[L]",
                    };
                    let path = match &p.project_type {
                        crate::models::ProjectType::Global => {
                            format!("~/.kanban/projects/{}", p.name)
                        }
                        crate::models::ProjectType::Local => {
                            format!(".kanban/{}", p.name)
                        }
                    };
                    format!("{} {}\n    {}", type_marker, p.name, path)
                })
                .collect();

            app.dialog = Some(DialogType::Select {
                title: "快速切换项目...".to_string(),
                items: project_items,
                selected: 0,
                filter: String::new(),
            });
        }
        Command::RenameProject => {
            // 获取当前项目名
            if let Some(project) = app.get_focused_project() {
                let current_name = project.name.clone();
                app.mode = Mode::Dialog;
                // app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法（已禁用）
                app.dialog = Some(DialogType::Input {
                    title: "重命名项目".to_string(),
                    prompt: "请输入新的项目名称:".to_string(),
                    textarea: crate::ui::text_input::HelixTextArea::new(current_name, true, false), // 默认 Insert 模式
                });
            }
        }
        Command::HideProject => {
            // 隐藏当前项目（软删除）
            if let Some(project) = app.get_focused_project() {
                let project_name = project.name.clone();
                let project_type = project.project_type;
                let project_path = project.path.clone();

                // 检查是否是当前工作目录的本地项目
                if project_type == crate::models::ProjectType::Local {
                    let current_local_dir = crate::fs::get_local_kanban_dir();
                    // 如果项目路径是当前目录的 .kanban，则不支持软删除
                    if project_path.starts_with(&current_local_dir) {
                        log_debug(
                            "当前目录的本地项目不支持软删除，请使用 D 键删除项目文件".to_string(),
                        );
                        return;
                    }
                }

                // 其他项目（全局项目或其他目录的本地项目）：显示确认对话框
                app.mode = Mode::Dialog;
                // app.ime_state.enter_dialog();  // 已禁用输入法自动切换
                app.dialog = Some(DialogType::Confirm {
                    title: "隐藏项目".to_string(),
                    message: format!(
                        "确定要隐藏项目 \"{}\" 吗？\n项目文件不会被删除，下次从该目录启动时会重新加载。",
                        project_name
                    ),
                    yes_selected: true,
                    action: crate::ui::dialogs::ConfirmAction::HideProject,
                });
            }
        }
        Command::DeleteProject => {
            // 删除当前项目（硬删除）
            if let Some(project) = app.get_focused_project() {
                let project_name = project.name.clone();

                // 显示确认对话框
                app.mode = Mode::Dialog;
                // app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法（已禁用）
                app.dialog = Some(DialogType::Confirm {
                    title: "删除项目文件".to_string(),
                    message: format!(
                        "确定要彻底删除项目 \"{}\" 吗？\n这将永久删除项目的所有文件和任务！此操作不可恢复！",
                        project_name
                    ),
                    yes_selected: false, // 默认选择"否"，更安全
                    action: crate::ui::dialogs::ConfirmAction::DeleteProject,
                });
            }
        }
        Command::NewTask => {
            app.mode = Mode::Dialog;
            // app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法（已禁用）
            app.dialog = Some(DialogType::Input {
                title: "创建新任务".to_string(),
                prompt: "任务标题和内容:".to_string(),
                textarea: crate::ui::text_input::HelixTextArea::new(String::new(), true, true), // 默认 Normal 模式
            });
        }
        Command::NewTaskInEditor => {
            // 用外部编辑器创建新任务
            // 创建临时文件
            use std::io::Write;

            let temp_dir = std::env::temp_dir();
            let temp_file = temp_dir.join(format!(
                "kanban_new_task_{}.md",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ));

            // 写入模板内容
            let template =
                "# 任务标题\n\n任务描述内容...\n\n## 子任务\n\n- [ ] 子任务 1\n- [ ] 子任务 2\n";
            if let Ok(mut file) = std::fs::File::create(&temp_file) {
                let _ = file.write_all(template.as_bytes());
            }

            // 设置待打开的文件路径和新任务标志（main.rs 会处理）
            app.pending_editor_file = Some(temp_file.to_string_lossy().to_string());
            app.is_new_task_file = true;
        }
        Command::EditTask => {
            // 获取当前选中的任务
            if let Some(task) = get_selected_task(app) {
                let title = task.title.clone();
                app.mode = Mode::Dialog;
                // app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法（已禁用）
                app.dialog = Some(DialogType::Input {
                    title: "编辑任务".to_string(),
                    prompt: "任务标题和内容:".to_string(),
                    textarea: crate::ui::text_input::HelixTextArea::new(title, true, true), // 默认 Normal 模式
                });
            }
        }
        Command::MoveTaskLeft => {
            move_task_to_status(app, -1);
        }
        Command::MoveTaskRight => {
            move_task_to_status(app, 1);
        }
        Command::MoveTaskUp => {
            move_task_in_column(app, -1);
        }
        Command::MoveTaskDown => {
            move_task_in_column(app, 1);
        }
        Command::FocusNextPane => {
            // 获取所有窗格 ID 并找到下一个
            let all_panes = app.split_tree.collect_pane_ids();
            log_debug(format!(
                "FocusNextPane, 当前焦点: {}, 所有面板: {:?}",
                app.focused_pane, all_panes
            ));
            if all_panes.len() > 1 {
                if let Some(current_idx) = all_panes.iter().position(|&id| id == app.focused_pane) {
                    let next_idx = (current_idx + 1) % all_panes.len();
                    app.focused_pane = all_panes[next_idx];
                    log_debug(format!("切换到面板: {}", app.focused_pane));
                }
            } else {
                log_debug("只有一个面板，无需切换".to_string());
            }
        }
        Command::FocusLeft => {
            log_debug(format!("FocusLeft, 当前焦点: {}", app.focused_pane));
            if let Some(new_pane_id) = app
                .split_tree
                .find_adjacent_pane(app.focused_pane, crate::ui::layout::Direction::Left)
            {
                app.focused_pane = new_pane_id;
                log_debug(format!("移动到左侧面板: {}", new_pane_id));
            } else {
                log_debug("左侧没有面板".to_string());
            }
        }
        Command::FocusRight => {
            log_debug(format!("FocusRight, 当前焦点: {}", app.focused_pane));
            if let Some(new_pane_id) = app
                .split_tree
                .find_adjacent_pane(app.focused_pane, crate::ui::layout::Direction::Right)
            {
                app.focused_pane = new_pane_id;
                log_debug(format!("移动到右侧面板: {}", new_pane_id));
            } else {
                log_debug("右侧没有面板".to_string());
            }
        }
        Command::FocusUp => {
            log_debug(format!("FocusUp, 当前焦点: {}", app.focused_pane));
            if let Some(new_pane_id) = app
                .split_tree
                .find_adjacent_pane(app.focused_pane, crate::ui::layout::Direction::Up)
            {
                app.focused_pane = new_pane_id;
                log_debug(format!("移动到上方面板: {}", new_pane_id));
            } else {
                log_debug("上方没有面板".to_string());
            }
        }
        Command::FocusDown => {
            log_debug(format!("FocusDown, 当前焦点: {}", app.focused_pane));
            if let Some(new_pane_id) = app
                .split_tree
                .find_adjacent_pane(app.focused_pane, crate::ui::layout::Direction::Down)
            {
                app.focused_pane = new_pane_id;
                log_debug(format!("移动到下方面板: {}", new_pane_id));
            } else {
                log_debug("下方没有面板".to_string());
            }
        }
        Command::ClosePane => {
            // 如果当前处于最大化状态，关闭当前窗口并恢复布局
            if app.saved_layout.is_some() {
                log_debug("最大化状态下按 q，关闭窗口并恢复布局".to_string());

                // 先恢复布局
                if let Some(saved) = app.saved_layout.take() {
                    app.split_tree = saved;
                    log_debug("已恢复布局".to_string());
                }

                // 然后关闭当前聚焦的面板
                let current_pane = app.focused_pane;
                log_debug(format!("关闭面板: {}", current_pane));
                if app.split_tree.close_pane(current_pane) {
                    // 关闭成功，重新聚焦到一个有效的面板
                    let all_panes = app.split_tree.collect_pane_ids();
                    if let Some(&first_pane) = all_panes.first() {
                        app.focused_pane = first_pane;
                        log_debug(format!("关闭后聚焦到: {}", first_pane));
                    }

                    // 保存状态
                    let state = crate::state::extract_state(app);
                    let _ = crate::state::save_state(&state);
                } else {
                    log_debug("无法关闭面板".to_string());
                }
            } else {
                // 非最大化状态：尝试关闭当前面板
                log_debug(format!("关闭面板: {}", app.focused_pane));
                let current_pane = app.focused_pane;
                if app.split_tree.close_pane(current_pane) {
                    // 关闭成功，需要重新聚焦到一个有效的面板
                    let all_panes = app.split_tree.collect_pane_ids();
                    if let Some(&first_pane) = all_panes.first() {
                        app.focused_pane = first_pane;
                        log_debug(format!("关闭后聚焦到: {}", first_pane));
                    }

                    // 保存状态
                    let state = crate::state::extract_state(app);
                    let _ = crate::state::save_state(&state);
                } else {
                    // 只有一个面板时，清空该面板的项目
                    log_debug("只有一个面板，清空当前项目".to_string());
                    if let Some(crate::ui::layout::SplitNode::Leaf { project_id, .. }) =
                        app.split_tree.find_pane_mut(app.focused_pane)
                    {
                        *project_id = None;

                        // 保存状态
                        let state = crate::state::extract_state(app);
                        let _ = crate::state::save_state(&state);

                        log_debug("已清空项目".to_string());
                    }
                }
            }
        }
        Command::MaximizePane => {
            // 最大化/恢复当前面板
            app.toggle_maximize();
        }
        Command::EditTaskInEditor => {
            // 用外部编辑器编辑当前选中的任务
            if let Some(task) = get_selected_task(app) {
                // 设置待打开的文件路径
                app.pending_editor_file = Some(task.file_path.to_string_lossy().to_string());
            }
        }
        Command::ViewTaskExternal => {
            // 用外部工具预览当前选中的任务
            if let Some(task) = get_selected_task(app) {
                // 设置待预览的文件路径
                app.pending_preview_file = Some(task.file_path.to_string_lossy().to_string());
            }
        }
        Command::ViewTask => {
            // TUI 内预览当前选中的任务
            if let Some(task) = get_selected_task(app) {
                // 读取任务文件内容
                if let Ok(content) = std::fs::read_to_string(&task.file_path) {
                    app.preview_content = content;
                    app.preview_scroll = 0;
                    app.mode = Mode::Preview;
                } else {
                    log_debug("读取任务文件失败".to_string());
                }
            }
        }
        Command::DeleteTask => {
            // 删除当前选中的任务
            if let Some(task) = get_selected_task(app) {
                let task_title = task.title.clone();

                // 显示确认对话框
                app.mode = Mode::Dialog;
                // app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法（已禁用）
                app.dialog = Some(DialogType::Confirm {
                    title: "删除任务".to_string(),
                    message: format!("确定要删除任务 \"{}\" 吗？", task_title),
                    yes_selected: true,
                    action: crate::ui::dialogs::ConfirmAction::DeleteTask,
                });
            }
        }
        Command::CopyTask => {
            // 复制任务到剪贴板
            #[cfg(feature = "clipboard")]
            {
                if let Some(task) = get_selected_task(app) {
                    // 读取任务文件内容
                    if let Ok(content) = std::fs::read_to_string(&task.file_path) {
                        // 复制到剪贴板
                        match arboard::Clipboard::new() {
                            Ok(mut clipboard) => {
                                if let Err(e) = clipboard.set_text(content) {
                                    app.show_notification(
                                        format!("复制失败: {}", e),
                                        crate::app::NotificationLevel::Error,
                                    );
                                } else {
                                    app.show_notification(
                                        format!("已复制任务「{}」到剪贴板", task.title),
                                        crate::app::NotificationLevel::Success,
                                    );
                                }
                            }
                            Err(e) => {
                                app.show_notification(
                                    format!("无法访问剪贴板: {}", e),
                                    crate::app::NotificationLevel::Error,
                                );
                            }
                        }
                    } else {
                        app.show_notification(
                            "读取任务文件失败".to_string(),
                            crate::app::NotificationLevel::Error,
                        );
                    }
                }
            }
            #[cfg(not(feature = "clipboard"))]
            {
                app.show_notification(
                    "剪贴板功能未启用".to_string(),
                    crate::app::NotificationLevel::Warning,
                );
            }
        }
        Command::SetTaskPriority(priority) => {
            // 设置任务优先级
            if let Some(task) = get_selected_task(app) {
                // 读取任务文件
                if let Ok(content) = std::fs::read_to_string(&task.file_path) {
                    let lines: Vec<&str> = content.lines().collect();
                    let mut new_lines = Vec::new();
                    let mut found_priority = false;

                    for line in &lines {
                        if line.starts_with("priority:") {
                            // 替换现有优先级
                            if priority != "none" {
                                new_lines.push(format!("priority: {}", priority));
                            }
                            // 如果是 "none" 就跳过这行（删除优先级）
                            found_priority = true;
                        } else if line.starts_with("created:") && !found_priority {
                            // 在 created 行后面插入 priority
                            new_lines.push(line.to_string());
                            if priority != "none" {
                                new_lines.push(format!("priority: {}", priority));
                                found_priority = true;
                            }
                        } else {
                            new_lines.push(line.to_string());
                        }
                    }

                    // 如果还没找到 priority 位置，在标题后添加
                    if !found_priority && priority != "none" {
                        let mut final_lines = Vec::new();
                        let mut after_title = false;
                        for line in new_lines {
                            final_lines.push(line.clone());
                            if line.starts_with("# ") && !after_title {
                                final_lines.push(String::new());
                                final_lines.push(format!("priority: {}", priority));
                                after_title = true;
                            }
                        }
                        new_lines = final_lines;
                    }

                    let new_content = new_lines.join("\n");
                    if let Err(e) = std::fs::write(&task.file_path, new_content) {
                        log_debug(format!("保存任务失败: {}", e));
                    } else {
                        // 同步更新 tasks.toml 中的 priority 字段
                        if let Some(project) = app.get_focused_project() {
                            let project_path = project.path.clone();
                            let tasks_toml = project_path.join("tasks.toml");
                            if tasks_toml.exists()
                                && let Ok(mut metadata_map) =
                                    crate::fs::task::load_tasks_metadata(&project_path)
                                    && let Some(metadata) =
                                        metadata_map.get_mut(&task.id.to_string())
                                    {
                                        metadata.priority = if priority == "none" {
                                            None
                                        } else {
                                            Some(priority.clone())
                                        };
                                        let _ = crate::fs::task::save_tasks_metadata(
                                            &project_path,
                                            &metadata_map,
                                        );
                                    }
                        }
                        log_debug(format!("已设置优先级为: {}", priority));
                        // 重新加载项目
                        let _ = app.reload_current_project();
                    }
                } else {
                    log_debug("读取任务文件失败".to_string());
                }
            }
        }
        Command::EditTags => {
            // 编辑任务标签
            if let Some(task) = get_selected_task(app) {
                let current_tags = task.tags.join(", ");
                app.mode = Mode::Dialog;
                // app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法（已禁用）
                app.dialog = Some(DialogType::Input {
                    title: "编辑标签".to_string(),
                    prompt: "标签（逗号分隔）:".to_string(),
                    textarea: crate::ui::text_input::HelixTextArea::new(current_tags, true, false),
                });
            }
        }
        Command::IncreaseColumnWidth => {
            adjust_column_width(app, 5);
        }
        Command::DecreaseColumnWidth => {
            adjust_column_width(app, -5);
        }
        Command::ResetColumnWidths => {
            reset_column_widths(app);
        }
        Command::ToggleMaximizeColumn => {
            toggle_maximize_column(app);
        }
        Command::ReloadCurrentProject => {
            // 重新加载当前项目
            if let Err(e) = app.reload_current_project() {
                log_debug(format!("重新加载当前项目失败: {}", e));
            } else {
                log_debug("重新加载当前项目成功".to_string());
            }
        }
        Command::ReloadAllProjects => {
            // 重新加载所有项目（本地+全局）
            match crate::fs::load_all_projects() {
                Ok(projects) => {
                    app.projects = projects;
                    log_debug(format!(
                        "重新加载所有项目成功，共 {} 个",
                        app.projects.len()
                    ));
                }
                Err(e) => {
                    log_debug(format!("重新加载所有项目失败: {}", e));
                }
            }
        }
        Command::CopyProjectInfo => {
            // 复制项目信息到剪贴板
            #[cfg(feature = "clipboard")]
            {
                if let Some(project) = app.get_focused_project() {
                    let project_type_label = match project.project_type {
                        crate::models::ProjectType::Global => "[G]",
                        crate::models::ProjectType::Local => "[L]",
                    };

                    let kanban_path = project.path.to_string_lossy();
                    // CLAUDE.md 路径（全局数据目录）
                    let claude_md_path = crate::fs::get_data_dir().join("CLAUDE.md");
                    let claude_md_str = claude_md_path.to_string_lossy().to_string();

                    // 格式化项目信息
                    let project_info = format!(
                        "{} {}\n看板路径: {}\n文档: {}\n\n# 可用命令\n\n# 查看\nhxk list                                    # 列出所有项目\nhxk config show                           # 查看配置\n\n# 项目操作\nhxk create {}                         # 创建新项目\n\n# 任务操作\nhxk add \"任务标题\"                         # 快速添加任务\n\n# 结构化命令（推荐，但需要重新安装）\nhxk project list                          # 项目管理\nhxk task list {} --status todo           # 列出任务\nhxk task create {} --status todo --title \"标题\"  # 创建任务\nhxk status list {}                      # 状态管理\n\n详细文档: {}",
                        project_type_label,
                        project.name,
                        kanban_path,
                        claude_md_str,
                        project.name,
                        project.name,
                        project.name,
                        project.name,
                        claude_md_str
                    );

                    match arboard::Clipboard::new() {
                        Ok(mut clipboard) => {
                            if let Err(e) = clipboard.set_text(project_info) {
                                app.show_notification(
                                    format!("复制失败: {}", e),
                                    crate::app::NotificationLevel::Error,
                                );
                            } else {
                                app.show_notification(
                                    format!("已复制项目「{}」信息到剪贴板", project.name),
                                    crate::app::NotificationLevel::Success,
                                );
                            }
                        }
                        Err(e) => {
                            app.show_notification(
                                format!("无法访问剪贴板: {}", e),
                                crate::app::NotificationLevel::Error,
                            );
                        }
                    }
                }
            }
            #[cfg(not(feature = "clipboard"))]
            {
                app.show_notification(
                    "剪贴板功能未启用".to_string(),
                    crate::app::NotificationLevel::Warning,
                );
            }
        }
        Command::CreateStatus => {
            // 创建新状态
            app.mode = Mode::Dialog;
            // app.ime_state.enter_dialog();  // 已禁用输入法自动切换
            app.dialog = Some(DialogType::Input {
                title: "创建新状态".to_string(),
                prompt: "请输入状态内部名称（英文、数字、下划线）:".to_string(),
                textarea: crate::ui::text_input::HelixTextArea::new(String::new(), true, false), // 默认 Insert 模式
            });
        }
        Command::RenameStatus => {
            // 重命名状态 - 收集信息后再修改 app
            let status_info = {
                let project = app.get_focused_project();
                if let Some(project) = project {
                    let column = app
                        .selected_column
                        .get(&app.focused_pane)
                        .copied()
                        .unwrap_or(0);
                    project
                        .statuses
                        .get(column)
                        .map(|s| (s.name.clone(), s.display.clone()))
                } else {
                    None
                }
            };

            if let Some((current_name, current_display)) = status_info {
                app.mode = Mode::Dialog;
                // app.ime_state.enter_dialog();  // 已禁用输入法自动切换
                app.dialog = Some(DialogType::Input {
                    title: format!("重命名状态: {}", current_display),
                    prompt: "请输入新的状态名称（英文、数字、下划线）:".to_string(),
                    textarea: crate::ui::text_input::HelixTextArea::new(current_name, true, false), // 默认 Insert 模式
                });
            }
        }
        Command::EditStatusDisplay => {
            // 编辑状态显示名 - 收集信息后再修改 app
            let status_info = {
                let project = app.get_focused_project();
                if let Some(project) = project {
                    let column = app
                        .selected_column
                        .get(&app.focused_pane)
                        .copied()
                        .unwrap_or(0);
                    project
                        .statuses
                        .get(column)
                        .map(|s| (s.name.clone(), s.display.clone()))
                } else {
                    None
                }
            };

            if let Some((status_name, current_display)) = status_info {
                app.mode = Mode::Dialog;
                // app.ime_state.enter_dialog();  // 已禁用输入法自动切换
                app.dialog = Some(DialogType::Input {
                    title: format!("编辑显示名: {}", status_name),
                    prompt: "请输入新的显示名称:".to_string(),
                    textarea: crate::ui::text_input::HelixTextArea::new(
                        current_display,
                        true,
                        false,
                    ),
                });
            }
        }
        Command::MoveStatusLeft => {
            // 左移状态列 - 收集信息后再修改 app
            let focused_pane = app.focused_pane;
            let status_info = {
                let project = app.get_focused_project();
                if let Some(project) = project {
                    let column = app.selected_column.get(&focused_pane).copied().unwrap_or(0);
                    project.statuses.get(column).map(|s| {
                        (
                            s.name.clone(),
                            s.display.clone(),
                            project.path.clone(),
                            column,
                        )
                    })
                } else {
                    None
                }
            };

            if let Some((status_name, status_display, project_path, column)) = status_info {
                match crate::fs::status::move_status_order(&project_path, &status_name, -1) {
                    Ok(_) => {
                        // 重新加载项目
                        if let Err(e) = app.reload_current_project() {
                            log_debug(format!("重新加载项目失败: {}", e));
                        }
                        // 更新选中列
                        if column > 0 {
                            app.selected_column.insert(focused_pane, column - 1);
                        }
                        app.show_notification(
                            format!("状态「{}」已左移", status_display),
                            crate::app::NotificationLevel::Success,
                        );
                    }
                    Err(e) => {
                        app.show_notification(
                            format!("移动失败: {}", e),
                            crate::app::NotificationLevel::Error,
                        );
                    }
                }
            }
        }
        Command::MoveStatusRight => {
            // 右移状态列 - 收集信息后再修改 app
            let focused_pane = app.focused_pane;
            let status_info = {
                let project = app.get_focused_project();
                if let Some(project) = project {
                    let column = app.selected_column.get(&focused_pane).copied().unwrap_or(0);
                    let statuses_len = project.statuses.len();
                    project.statuses.get(column).map(|s| {
                        (
                            s.name.clone(),
                            s.display.clone(),
                            project.path.clone(),
                            column,
                            statuses_len,
                        )
                    })
                } else {
                    None
                }
            };

            if let Some((status_name, status_display, project_path, column, statuses_len)) =
                status_info
            {
                match crate::fs::status::move_status_order(&project_path, &status_name, 1) {
                    Ok(_) => {
                        // 重新加载项目
                        if let Err(e) = app.reload_current_project() {
                            log_debug(format!("重新加载项目失败: {}", e));
                        }
                        // 更新选中列
                        if column < statuses_len - 1 {
                            app.selected_column.insert(focused_pane, column + 1);
                        }
                        app.show_notification(
                            format!("状态「{}」已右移", status_display),
                            crate::app::NotificationLevel::Success,
                        );
                    }
                    Err(e) => {
                        app.show_notification(
                            format!("移动失败: {}", e),
                            crate::app::NotificationLevel::Error,
                        );
                    }
                }
            }
        }
        Command::MoveStatusToFirst => {
            // 移动状态列到最左侧
            let focused_pane = app.focused_pane;
            let status_info = {
                let project = app.get_focused_project();
                if let Some(project) = project {
                    let column = app.selected_column.get(&focused_pane).copied().unwrap_or(0);
                    project.statuses.get(column).map(|s| {
                        (
                            s.name.clone(),
                            s.display.clone(),
                            project.path.clone(),
                            column,
                        )
                    })
                } else {
                    None
                }
            };

            if let Some((status_name, status_display, project_path, column)) = status_info {
                if column == 0 {
                    app.show_notification(
                        "状态已在最左侧".to_string(),
                        crate::app::NotificationLevel::Info,
                    );
                    return;
                }

                // 移动到最左侧（索引 0）
                let move_count = -(column as i32);
                match crate::fs::status::move_status_order(&project_path, &status_name, move_count)
                {
                    Ok(_) => {
                        // 重新加载项目
                        if let Err(e) = app.reload_current_project() {
                            log_debug(format!("重新加载项目失败: {}", e));
                        }
                        // 更新选中列到第一列
                        app.selected_column.insert(focused_pane, 0);
                        app.show_notification(
                            format!("状态「{}」已移至最左侧", status_display),
                            crate::app::NotificationLevel::Success,
                        );
                    }
                    Err(e) => {
                        app.show_notification(
                            format!("移动失败: {}", e),
                            crate::app::NotificationLevel::Error,
                        );
                    }
                }
            }
        }
        Command::MoveStatusToLast => {
            // 移动状态列到最右侧
            let focused_pane = app.focused_pane;
            let status_info = {
                let project = app.get_focused_project();
                if let Some(project) = project {
                    let column = app.selected_column.get(&focused_pane).copied().unwrap_or(0);
                    let statuses_len = project.statuses.len();
                    project.statuses.get(column).map(|s| {
                        (
                            s.name.clone(),
                            s.display.clone(),
                            project.path.clone(),
                            column,
                            statuses_len,
                        )
                    })
                } else {
                    None
                }
            };

            if let Some((status_name, status_display, project_path, column, statuses_len)) =
                status_info
            {
                if column == statuses_len - 1 {
                    app.show_notification(
                        "状态已在最右侧".to_string(),
                        crate::app::NotificationLevel::Info,
                    );
                    return;
                }

                // 移动到最右侧
                let move_count = (statuses_len - 1 - column) as i32;
                match crate::fs::status::move_status_order(&project_path, &status_name, move_count)
                {
                    Ok(_) => {
                        // 重新加载项目
                        if let Err(e) = app.reload_current_project() {
                            log_debug(format!("重新加载项目失败: {}", e));
                        }
                        // 更新选中列到最后一列
                        app.selected_column.insert(focused_pane, statuses_len - 1);
                        app.show_notification(
                            format!("状态「{}」已移至最右侧", status_display),
                            crate::app::NotificationLevel::Success,
                        );
                    }
                    Err(e) => {
                        app.show_notification(
                            format!("移动失败: {}", e),
                            crate::app::NotificationLevel::Error,
                        );
                    }
                }
            }
        }
        Command::DeleteStatus => {
            // 删除状态
            if let Some(project) = app.get_focused_project() {
                let column = app
                    .selected_column
                    .get(&app.focused_pane)
                    .copied()
                    .unwrap_or(0);
                if let Some(status) = project.statuses.get(column) {
                    let status_name = status.name.clone();
                    let status_display = status.display.clone();

                    // 检查任务数量
                    let task_count = project
                        .tasks
                        .iter()
                        .filter(|t| t.status == status_name)
                        .count();

                    let message = if task_count > 0 {
                        format!(
                            "确定要删除状态「{}」吗？\n该状态下有 {} 个任务，删除后任务将无法访问。",
                            status_display, task_count
                        )
                    } else {
                        format!("确定要删除状态「{}」吗？", status_display)
                    };

                    app.mode = Mode::Dialog;
                    // app.ime_state.enter_dialog();  // 已禁用输入法自动切换
                    app.dialog = Some(DialogType::Confirm {
                        title: "删除状态".to_string(),
                        message,
                        yes_selected: false, // 默认选择"否"，更安全
                        action: crate::ui::dialogs::ConfirmAction::DeleteStatus,
                    });
                }
            }
        }
        // 未实现的命令：静默忽略（不报错，不执行）
        _ => {
            // 不做任何处理，避免报错
        }
    }
}

/// 执行文本命令（从命令模式输入）
/// 返回 false 表示应该退出应用
#[allow(dead_code)]
fn execute_text_command(app: &mut App, cmd_str: &str) -> bool {
    let cmd_str = cmd_str.trim();

    // 查找命令定义
    let cmd_def = app.command_registry.find_exact(cmd_str);

    if let Some(cmd_def) = cmd_def {
        // 根据命令名执行对应操作
        match cmd_def.name {
            "quit" => {
                return false; // 退出应用
            }
            "project-open" => execute_command(app, Command::OpenProject),
            "project-new" => execute_command(app, Command::NewGlobalProject),
            "project-new-local" => execute_command(app, Command::NewLocalProject),
            "project-delete" => execute_command(app, Command::DeleteProject),
            "project-rename" => execute_command(app, Command::RenameProject),
            "task-new" => execute_command(app, Command::NewTask),
            "task-edit" => execute_command(app, Command::EditTask),
            "task-delete" => execute_command(app, Command::DeleteTask),
            "task-view" => execute_command(app, Command::ViewTask),
            "task-view-external" => execute_command(app, Command::ViewTaskExternal),
            "task-edit-external" => execute_command(app, Command::EditTaskInEditor),
            "priority-high" => execute_command(app, Command::SetTaskPriority("high".to_string())),
            "priority-medium" => {
                execute_command(app, Command::SetTaskPriority("medium".to_string()))
            }
            "priority-low" => execute_command(app, Command::SetTaskPriority("low".to_string())),
            "priority-none" => execute_command(app, Command::SetTaskPriority("none".to_string())),
            "split-horizontal" => execute_command(app, Command::SplitHorizontal),
            "split-vertical" => execute_command(app, Command::SplitVertical),
            "close-pane" => execute_command(app, Command::ClosePane),
            "focus-next" => execute_command(app, Command::FocusNextPane),
            "focus-left" => execute_command(app, Command::FocusLeft),
            "focus-right" => execute_command(app, Command::FocusRight),
            "focus-up" => execute_command(app, Command::FocusUp),
            "focus-down" => execute_command(app, Command::FocusDown),
            "reload" => execute_command(app, Command::ReloadCurrentProject),
            "reload-all" => execute_command(app, Command::ReloadAllProjects),
            "help" => {
                app.mode = Mode::Help;
            }
            _ => {
                // 未实现的命令：静默忽略
            }
        }
    } else {
        // 未知命令：静默忽略
    }

    true // 继续运行
}

/// 获取当前选中的任务
fn get_selected_task(app: &App) -> Option<&crate::models::Task> {
    let project = app.get_focused_project()?;
    let column = app
        .selected_column
        .get(&app.focused_pane)
        .copied()
        .unwrap_or(0);
    let task_idx = app
        .selected_task_index
        .get(&app.focused_pane)
        .copied()
        .unwrap_or(0);

    let status = app.get_status_name_by_column(column)?;

    let tasks: Vec<_> = project
        .tasks
        .iter()
        .filter(|t| t.status == status)
        .collect();
    tasks.get(task_idx).copied()
}

/// 获取当前选中的任务（可变）- 返回任务 ID
fn get_selected_task_id(app: &App) -> Option<u32> {
    let column = app
        .selected_column
        .get(&app.focused_pane)
        .copied()
        .unwrap_or(0);
    let task_idx = app
        .selected_task_index
        .get(&app.focused_pane)
        .copied()
        .unwrap_or(0);

    let status = app.get_status_name_by_column(column)?;

    let project = app.get_focused_project()?;
    let tasks: Vec<_> = project
        .tasks
        .iter()
        .filter(|t| t.status == status)
        .collect();
    tasks.get(task_idx).map(|t| t.id)
}

/// 移动任务到相邻状态
fn move_task_to_status(app: &mut App, direction: i32) {
    let column = app
        .selected_column
        .get(&app.focused_pane)
        .copied()
        .unwrap_or(0);
    let status_count = app.get_status_count();
    let new_column = (column as i32 + direction).clamp(0, status_count as i32 - 1) as usize;

    if new_column == column {
        return; // 已经在边界
    }

    // 获取任务 ID
    let task_id = if let Some(id) = get_selected_task_id(app) {
        id
    } else {
        return;
    };

    // 获取项目名称和路径
    let project_name = if let Some(crate::ui::layout::SplitNode::Leaf { project_id, .. }) =
        app.split_tree.find_pane(app.focused_pane)
    {
        if let Some(name) = project_id {
            name.clone()
        } else {
            return;
        }
    } else {
        return;
    };

    // 获取新状态名称
    let new_status = if let Some(status) = app.get_status_name_by_column(new_column) {
        status
    } else {
        return;
    };

    // 找到任务并修改
    if let Some(project) = app.projects.iter_mut().find(|p| p.name == project_name)
        && let Some(task) = project.tasks.iter_mut().find(|t| t.id == task_id) {
            let old_status = task.status.clone();
            task.status = new_status.clone();

            // 移动文件到新的状态目录（使用项目的实际路径）
            let project_path = project.path.clone();

            match crate::fs::move_task(&project_path, task, &new_status) {
                Ok(new_path) => {
                    // 更新任务的文件路径
                    task.file_path = new_path;
                    // 更新界面：移动到新列
                    app.selected_column.insert(app.focused_pane, new_column);
                    app.selected_task_index.insert(app.focused_pane, 0);
                }
                Err(e) => {
                    log_debug(format!("移动任务文件失败: {}", e));
                    task.status = old_status; // 回滚
                }
            }
        }
}

/// 在列内上下移动任务
fn move_task_in_column(app: &mut App, direction: i32) {
    let column = app
        .selected_column
        .get(&app.focused_pane)
        .copied()
        .unwrap_or(0);
    let task_idx = app
        .selected_task_index
        .get(&app.focused_pane)
        .copied()
        .unwrap_or(0);

    let status = if let Some(s) = app.get_status_name_by_column(column) {
        s
    } else {
        return;
    };

    // 获取项目名称和路径
    let (project_name, project_path) =
        if let Some(crate::ui::layout::SplitNode::Leaf { project_id, .. }) =
            app.split_tree.find_pane(app.focused_pane)
        {
            if let Some(name) = project_id {
                if let Some(project) = app.projects.iter().find(|p| &p.name == name) {
                    (name.clone(), project.path.clone())
                } else {
                    return;
                }
            } else {
                return;
            }
        } else {
            return;
        };

    // 获取当前列的所有任务（已按order排序）
    if let Some(project) = app.projects.iter_mut().find(|p| p.name == project_name) {
        let mut tasks: Vec<&mut crate::models::Task> = project
            .tasks
            .iter_mut()
            .filter(|t| t.status == status)
            .collect();

        if tasks.len() < 2 {
            return; // 不足以移动
        }

        // 按order排序
        tasks.sort_by_key(|t| t.order);

        let new_idx = (task_idx as i32 + direction).clamp(0, tasks.len() as i32 - 1) as usize;

        if new_idx == task_idx {
            return; // 已经在边界
        }

        // 获取当前任务
        let current_task_id = tasks[task_idx].id;

        // 计算新的order值
        let new_order = if new_idx == 0 {
            // 移到最顶部
            tasks[0].order - 1000
        } else if new_idx == tasks.len() - 1 {
            // 移到最底部
            tasks[tasks.len() - 1].order + 1000
        } else {
            // 移到中间：计算上下任务的中间值
            let (order_above, order_below) = if direction > 0 {
                // 向下移动
                (
                    tasks[new_idx].order,
                    tasks
                        .get(new_idx + 1)
                        .map(|t| t.order)
                        .unwrap_or(tasks[new_idx].order + 1000),
                )
            } else {
                // 向上移动
                (
                    tasks
                        .get(new_idx.saturating_sub(1))
                        .map(|t| t.order)
                        .unwrap_or(tasks[new_idx].order - 1000),
                    tasks[new_idx].order,
                )
            };

            // 检查间隙是否足够
            if (order_below - order_above).abs() < 2 {
                // 间隙不够，需要重平衡
                log_debug("Order值间隙不足，执行重平衡".to_string());
                rebalance_order_in_column(&mut tasks);

                // 重新计算new_order
                if direction > 0 {
                    let below_order = tasks
                        .get(new_idx + 1)
                        .map(|t| t.order)
                        .unwrap_or(tasks[new_idx].order + 1000);
                    (tasks[new_idx].order + below_order) / 2
                } else {
                    let above_order = tasks
                        .get(new_idx.saturating_sub(1))
                        .map(|t| t.order)
                        .unwrap_or(tasks[new_idx].order - 1000);
                    (above_order + tasks[new_idx].order) / 2
                }
            } else {
                (order_above + order_below) / 2
            }
        };

        // 更新当前任务的order
        let need_rebalance = tasks
            .iter()
            .any(|t| t.order % 1000 == 0 && t.id != current_task_id);

        if let Some(task) = project.tasks.iter_mut().find(|t| t.id == current_task_id) {
            task.order = new_order;

            // 持久化到文件
            if let Err(e) = crate::fs::save_task(&project_path, task) {
                log_debug(format!("保存任务失败: {}", e));
                return;
            }

            log_debug(format!("任务 {} 的order更新为 {}", task.id, task.order));
        }

        // 如果进行了重平衡，保存所有任务
        if need_rebalance {
            let tasks_to_save: Vec<crate::models::Task> = project
                .tasks
                .iter()
                .filter(|t| t.status == status)
                .cloned()
                .collect();

            for task in tasks_to_save {
                let _ = crate::fs::save_task(&project_path, &task);
            }
        }

        // 更新UI选中索引
        app.selected_task_index.insert(app.focused_pane, new_idx);

        // 重新加载项目以刷新排序
        let _ = app.reload_current_project();
    }
}

/// 创建新任务
fn create_new_task(app: &mut App, input: String) {
    use crate::models::Task;

    log_debug(format!("调试: 准备创建任务，输入内容: '{}'", input));

    // 解析输入：第一行是标题，其余是内容
    let lines: Vec<&str> = input.lines().collect();
    let title = if lines.is_empty() {
        log_debug("调试: 输入为空".to_string());
        return;
    } else {
        lines[0].trim().to_string()
    };

    let content = if lines.len() > 1 {
        lines[1..].join("\n")
    } else {
        String::new()
    };

    log_debug(format!("调试: 标题='{}', 内容长度={}", title, content.len()));

    // 获取当前项目
    let project_name = if let Some(crate::ui::layout::SplitNode::Leaf { project_id, .. }) =
        app.split_tree.find_pane(app.focused_pane)
    {
        if let Some(name) = project_id {
            log_debug(format!("调试: 当前项目 '{}'", name));
            name.clone()
        } else {
            log_debug("调试: 当前面板没有项目".to_string());
            return;
        }
    } else {
        log_debug("调试: 找不到当前面板".to_string());
        return;
    };

    // 获取项目路径（支持本地和全局项目）
    let project_path = if let Some(project) = app.projects.iter().find(|p| p.name == project_name)
    {
        project.path.clone()
    } else {
        log_debug("调试: 在项目列表中找不到项目".to_string());
        return;
    };

    // 获取下一个任务 ID
    if let Ok(next_id) = crate::fs::get_next_task_id(&project_path) {
        log_debug(format!("调试: 下一个任务ID {}", next_id));
        // 获取当前选中的列作为初始状态
        let column = app
            .selected_column
            .get(&app.focused_pane)
            .copied()
            .unwrap_or(0);
        let status = app
            .get_status_name_by_column(column)
            .unwrap_or_else(|| "todo".to_string());
        log_debug(format!("调试: 状态 '{}'", status));

        // 获取当前列的最大order值
        let max_order = crate::fs::get_max_order_in_status(&project_path, &status).unwrap_or(-1000);
        let new_order = max_order + 1000;
        log_debug(format!("调试: 新任务order值 {}", new_order));

        // 创建任务并设置order和content
        let mut task = Task::new(next_id, title.clone(), status.clone());
        task.order = new_order;
        task.content = content;

        // 保存到文件
        match crate::fs::save_task(&project_path, &task) {
            Ok(_) => {
                log_debug("调试: 任务保存成功".to_string());
            }
            Err(e) => {
                log_debug(format!("保存任务失败: {}", e));
                return;
            }
        }

        // 重新加载项目以确保任务列表是最新的
        match crate::fs::load_project(&project_path) {
            Ok(updated_project) => {
                log_debug(format!(
                    "调试: 重新加载项目，共 {} 个任务",
                    updated_project.tasks.len()
                ));
                if let Some(project) = app.projects.iter_mut().find(|p| p.name == project_name) {
                    *project = updated_project;

                    // 找到新任务在当前列的索引（应该是最后一个）
                    let new_task_idx = project
                        .tasks
                        .iter()
                        .filter(|t| t.status == status)
                        .count()
                        .saturating_sub(1);

                    // 自动选中新创建的任务
                    app.selected_task_index
                        .insert(app.focused_pane, new_task_idx);
                    log_debug(format!("调试: 选中任务索引 {}", new_task_idx));
                } else {
                    log_debug(format!(
                        "调试: 在 app.projects 中找不到项目 '{}'",
                        project_name
                    ));
                }
            }
            Err(e) => {
                log_debug(format!("调试: 重新加载项目失败: {}", e));
            }
        }
    } else {
        log_debug("调试: 获取下一个任务ID失败".to_string());
    }
}

/// 重命名当前项目
fn rename_current_project(app: &mut App, new_name: String) {
    // 获取当前项目名
    let old_name = if let Some(crate::ui::layout::SplitNode::Leaf { project_id, .. }) =
        app.split_tree.find_pane(app.focused_pane)
    {
        if let Some(name) = project_id {
            name.clone()
        } else {
            return;
        }
    } else {
        return;
    };

    if old_name == new_name {
        return; // 名称没有变化
    }

    // 找到项目
    if let Some(project) = app.projects.iter().find(|p| p.name == old_name).cloned() {
        let old_path = project.path.clone();
        let new_path = old_path.parent().unwrap().join(&new_name);

        // 重命名目录
        if let Err(e) = std::fs::rename(&old_path, &new_path) {
            log_debug(format!("重命名项目目录失败: {}", e));
            return;
        }

        // 更新配置文件中的项目名
        let config_path = new_path.join(".kanban.toml");
        if let Ok(mut content) = std::fs::read_to_string(&config_path) {
            // 简单替换项目名（第一行）
            let lines: Vec<&str> = content.lines().collect();
            if !lines.is_empty() {
                content = format!("name = \"{}\"\n{}", new_name, lines[1..].join("\n"));
                let _ = std::fs::write(&config_path, content);
            }
        }

        // 重新加载所有项目
        match crate::fs::load_all_projects() {
            Ok(projects) => {
                app.projects = projects;
                // 更新当前面板的项目ID
                app.set_focused_project(new_name);
            }
            Err(e) => {
                log_debug(format!("重新加载项目失败: {}", e));
            }
        }
    }
}

/// 更新任务标题
fn update_task_title(app: &mut App, new_title: String) {
    // 获取任务 ID
    let task_id = if let Some(id) = get_selected_task_id(app) {
        id
    } else {
        return;
    };

    // 获取项目名称
    let project_name = if let Some(crate::ui::layout::SplitNode::Leaf { project_id, .. }) =
        app.split_tree.find_pane(app.focused_pane)
    {
        if let Some(name) = project_id {
            name.clone()
        } else {
            return;
        }
    } else {
        return;
    };

    // 找到任务并更新
    if let Some(project) = app.projects.iter_mut().find(|p| p.name == project_name)
        && let Some(task) = project.tasks.iter_mut().find(|t| t.id == task_id) {
            let old_title = task.title.clone();
            task.title = new_title;

            // 保存到文件（使用项目的实际路径）
            let project_path = project.path.clone();
            if let Err(e) = crate::fs::save_task(&project_path, task) {
                log_debug(format!("保存任务失败: {}", e));
                task.title = old_title; // 回滚
            }
        }
}

/// 更新任务标签
fn update_task_tags(app: &mut App, tags_string: String) {
    // 获取任务 ID
    let task_id = if let Some(id) = get_selected_task_id(app) {
        id
    } else {
        return;
    };

    // 获取项目名称
    let project_name = if let Some(crate::ui::layout::SplitNode::Leaf { project_id, .. }) =
        app.split_tree.find_pane(app.focused_pane)
    {
        if let Some(name) = project_id {
            name.clone()
        } else {
            return;
        }
    } else {
        return;
    };

    // 解析标签（从逗号分隔的字符串）
    let new_tags: Vec<String> = tags_string
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // 找到任务并更新
    let mut result = Ok(());
    if let Some(project) = app.projects.iter_mut().find(|p| p.name == project_name)
        && let Some(task) = project.tasks.iter_mut().find(|t| t.id == task_id) {
            let old_tags = task.tags.clone();
            task.tags = new_tags;

            // 保存到文件（使用项目的实际路径）
            let project_path = project.path.clone();
            if let Err(e) = crate::fs::save_task(&project_path, task) {
                result = Err(e);
                task.tags = old_tags; // 回滚
            }
        }

    // 显示通知
    match result {
        Ok(_) => {
            app.show_notification(
                "标签已更新".to_string(),
                crate::app::NotificationLevel::Success,
            );
        }
        Err(e) => {
            app.show_notification(
                format!("保存标签失败: {}", e),
                crate::app::NotificationLevel::Error,
            );
        }
    }
}

/// 处理帮助模式的按键
fn handle_help_mode(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('?') => {
            app.mode = Mode::Normal;
        }
        _ => {}
    }
    true
}

/// 处理空格菜单模式的按键
fn handle_space_menu_mode(app: &mut App, key: KeyEvent) -> bool {
    use crate::app::MenuState;

    match key.code {
        KeyCode::Esc => {
            // ESC：子菜单返回主菜单，主菜单退出
            match app.menu_state {
                Some(MenuState::Main) | None => {
                    app.menu_state = None;
                    app.menu_selected_index = None;
                    app.mode = Mode::Normal;
                    app.key_buffer.clear();
                }
                Some(_) => {
                    app.menu_state = Some(MenuState::Main);
                    app.menu_selected_index = Some(0); // 返回主菜单时重置选中索引
                }
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            // 向上导航菜单
            navigate_menu_up(app);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            // 向下导航菜单
            navigate_menu_down(app);
        }
        KeyCode::Enter => {
            // 执行选中的命令
            if let Some(idx) = app.menu_selected_index {
                execute_selected_menu_command(app, idx);
                return true;
            }
        }
        KeyCode::Char(c) => {
            match app.menu_state {
                Some(MenuState::Main) => {
                    // 主菜单：切换到子菜单或执行命令
                    match c {
                        'p' => app.menu_state = Some(MenuState::Project),
                        'w' => app.menu_state = Some(MenuState::Window),
                        't' => app.menu_state = Some(MenuState::Task),
                        's' => app.menu_state = Some(MenuState::Status),
                        'f' => {
                            // 快速切换项目
                            app.mode = Mode::Normal;
                            app.menu_state = None;
                            app.key_buffer.clear();
                            execute_command(app, Command::OpenProject);
                        }
                        'r' => {
                            // 重新加载当前项目
                            app.mode = Mode::Normal;
                            app.menu_state = None;
                            app.key_buffer.clear();
                            execute_command(app, Command::ReloadCurrentProject);
                        }
                        'R' => {
                            // 重新加载所有项目
                            app.mode = Mode::Normal;
                            app.menu_state = None;
                            app.key_buffer.clear();
                            execute_command(app, Command::ReloadAllProjects);
                        }
                        'q' => {
                            // 退出应用
                            app.mode = Mode::Normal;
                            app.menu_state = None;
                            app.key_buffer.clear();
                            execute_command(app, Command::Quit);
                            return false;
                        }
                        '?' => {
                            app.mode = Mode::Help;
                            app.menu_state = None;
                            app.key_buffer.clear();
                        }
                        _ => {}
                    }
                }
                Some(MenuState::Project) => {
                    // 项目子菜单：立即执行命令并退出菜单
                    let cmd = match c {
                        'o' => Some(Command::OpenProject),
                        'n' => Some(Command::NewLocalProject),
                        'N' => Some(Command::NewGlobalProject),
                        'd' => Some(Command::HideProject), // 小写d = 软删除（隐藏）
                        'D' => Some(Command::DeleteProject), // 大写D = 硬删除
                        'r' => Some(Command::RenameProject),
                        'i' => Some(Command::CopyProjectInfo), // 复制项目信息
                        _ => None,
                    };
                    if let Some(cmd) = cmd {
                        app.mode = Mode::Normal;
                        app.menu_state = None;
                        app.key_buffer.clear();
                        execute_command(app, cmd);
                    }
                }
                Some(MenuState::Window) => {
                    // 窗口子菜单：立即执行命令并退出菜单
                    let cmd = match c {
                        'w' => Some(Command::FocusNextPane),
                        'v' => Some(Command::SplitVertical),
                        's' => Some(Command::SplitHorizontal),
                        'q' => Some(Command::ClosePane),
                        'h' => Some(Command::FocusLeft),
                        'l' => Some(Command::FocusRight),
                        'k' => Some(Command::FocusUp),
                        'j' => Some(Command::FocusDown),
                        'm' => Some(Command::MaximizePane),
                        _ => None,
                    };
                    if let Some(cmd) = cmd {
                        log_debug(format!("执行窗口命令: {:?}", cmd));
                        app.mode = Mode::Normal;
                        app.menu_state = None;
                        app.key_buffer.clear();
                        execute_command(app, cmd);
                    }
                }
                Some(MenuState::Task) => {
                    // 任务子菜单：立即执行命令并退出菜单
                    let cmd = match c {
                        'a' => Some(Command::NewTask), // 改为 a 键新建任务
                        'e' => Some(Command::EditTask),
                        'E' => Some(Command::EditTaskInEditor),
                        'v' => Some(Command::ViewTask),
                        'V' => Some(Command::ViewTaskExternal),
                        't' => Some(Command::EditTags), // 编辑标签
                        'd' => Some(Command::DeleteTask),
                        'Y' => Some(Command::CopyTask), // 大写 Y 复制任务
                        'h' => Some(Command::SetTaskPriority("high".to_string())),
                        'm' => Some(Command::SetTaskPriority("medium".to_string())),
                        'l' => Some(Command::SetTaskPriority("low".to_string())),
                        'n' => Some(Command::SetTaskPriority("none".to_string())),
                        _ => None,
                    };
                    if let Some(cmd) = cmd {
                        app.mode = Mode::Normal;
                        app.menu_state = None;
                        app.key_buffer.clear();
                        execute_command(app, cmd);
                    }
                }
                Some(MenuState::Status) => {
                    // 状态子菜单：立即执行命令并退出菜单
                    let cmd = match c {
                        'a' => Some(Command::CreateStatus),
                        'r' => Some(Command::RenameStatus),
                        'e' => Some(Command::EditStatusDisplay),
                        'h' => Some(Command::MoveStatusLeft),
                        'l' => Some(Command::MoveStatusRight),
                        'H' => Some(Command::MoveStatusToFirst), // 大写 H 移到最左侧
                        'L' => Some(Command::MoveStatusToLast),  // 大写 L 移到最右侧
                        'd' => Some(Command::DeleteStatus),
                        _ => None,
                    };
                    if let Some(cmd) = cmd {
                        app.mode = Mode::Normal;
                        app.menu_state = None;
                        app.key_buffer.clear();
                        execute_command(app, cmd);
                    }
                }
                None => {}
            }
        }
        _ => {}
    }
    true
}

/// 处理预览模式的按键
fn handle_preview_mode(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.preview_content.clear();
            app.preview_scroll = 0;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            // 向下滚动
            app.preview_scroll = app.preview_scroll.saturating_add(1);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            // 向上滚动
            app.preview_scroll = app.preview_scroll.saturating_sub(1);
        }
        _ => {}
    }
    true
}

/// 获取当前光标所在行的起始位置
#[allow(dead_code)]
fn get_line_start(text: &str, cursor_pos: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();

    // 从光标位置向前查找换行符
    let mut pos = cursor_pos;
    while pos > 0 {
        if chars[pos - 1] == '\n' {
            return pos;
        }
        pos -= 1;
    }
    0 // 第一行的起始位置
}

/// 获取当前光标所在行的结束位置
#[allow(dead_code)]
fn get_line_end(text: &str, cursor_pos: usize) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    // 从光标位置向后查找换行符
    let mut pos = cursor_pos;
    while pos < len {
        if chars[pos] == '\n' {
            return pos;
        }
        pos += 1;
    }
    len // 最后一行的结束位置
}

/// 垂直移动光标（上下移动行）
/// direction: -1 表示上移，1 表示下移
#[allow(dead_code)]
fn move_cursor_vertical(text: &str, cursor_pos: usize, direction: i32) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    if len == 0 {
        return 0;
    }

    // 获取当前行的起始和结束位置
    let current_line_start = get_line_start(text, cursor_pos);
    let current_line_end = get_line_end(text, cursor_pos);

    // 计算当前列位置
    let column = cursor_pos - current_line_start;

    if direction < 0 {
        // 向上移动
        if current_line_start == 0 {
            // 已经在第一行，移动到行首
            return 0;
        }

        // 找到上一行的起始位置（current_line_start - 1 是上一行的换行符）
        let prev_line_end = current_line_start - 1;
        let prev_line_start = get_line_start(text, prev_line_end.saturating_sub(1));
        let prev_line_len = prev_line_end - prev_line_start;

        // 移动到上一行的相同列位置，或行尾（如果上一行更短）
        prev_line_start + column.min(prev_line_len)
    } else {
        // 向下移动
        if current_line_end >= len {
            // 已经在最后一行，移动到行尾
            return len;
        }

        // 找到下一行的起始位置（current_line_end 是当前行的换行符）
        let next_line_start = current_line_end + 1;
        let next_line_end = get_line_end(text, next_line_start);
        let next_line_len = next_line_end - next_line_start;

        // 移动到下一行的相同列位置，或行尾（如果下一行更短）
        next_line_start + column.min(next_line_len)
    }
}

/// 重平衡列内任务的order值，使其均匀分布
fn rebalance_order_in_column(tasks: &mut [&mut crate::models::Task]) {
    // tasks已按order排序
    for (idx, task) in tasks.iter_mut().enumerate() {
        task.order = (idx as i32) * 1000;
    }
}

/// 调整当前列的宽度
fn adjust_column_width(app: &mut App, delta: i16) {
    let project_name = match get_focused_project_name(app) {
        Some(name) => name,
        None => return,
    };

    let project = match app.get_focused_project() {
        Some(p) => p,
        None => return,
    };

    let column = app
        .selected_column
        .get(&app.focused_pane)
        .copied()
        .unwrap_or(0);
    let num_columns = project.statuses.len();

    // 获取或初始化列宽配置
    let widths = app
        .config
        .column_widths
        .entry(project_name.clone())
        .or_insert_with(|| vec![100 / num_columns as u16; num_columns]);

    if column >= widths.len() {
        return;
    }

    // 计算新宽度（限制在 10%-80% 之间）
    let new_width = (widths[column] as i16 + delta).clamp(10, 80) as u16;
    let old_width = widths[column];
    let diff = new_width as i16 - old_width as i16;

    if diff == 0 {
        return;
    }

    // 调整当前列宽度
    widths[column] = new_width;

    // 从其他列平均分配/回收空间
    let other_count = num_columns - 1;
    if other_count > 0 {
        let per_column_adjust = -(diff as f32 / other_count as f32);
        for (i, width) in widths.iter_mut().enumerate() {
            if i != column {
                let adjusted = (*width as f32 + per_column_adjust).clamp(5.0, 95.0) as u16;
                *width = adjusted;
            }
        }
    }

    // 确保总和为 100%
    normalize_widths(widths);

    // 保存配置
    if let Err(e) = crate::config::save_config(&app.config) {
        log_debug(format!("保存配置失败: {}", e));
    }

    // 记录调整时间
    app.last_column_resize_time = Some(std::time::Instant::now());
}

/// 重置为等宽
fn reset_column_widths(app: &mut App) {
    let project_name = match get_focused_project_name(app) {
        Some(name) => name,
        None => return,
    };

    // 移除配置和最大化状态
    app.config.column_widths.remove(&project_name);
    app.config.maximized_column.remove(&project_name);

    // 保存配置
    if let Err(e) = crate::config::save_config(&app.config) {
        log_debug(format!("保存配置失败: {}", e));
    }
}

/// 切换最大化当前列
fn toggle_maximize_column(app: &mut App) {
    log_debug("调用 toggle_maximize_column".to_string());

    let project_name = match get_focused_project_name(app) {
        Some(name) => name,
        None => {
            log_debug("无法获取项目名称".to_string());
            return;
        }
    };

    let column = app
        .selected_column
        .get(&app.focused_pane)
        .copied()
        .unwrap_or(0);
    log_debug(format!("当前列: {}, 项目: {}", column, project_name));

    // 获取当前最大化状态
    let current_max = app
        .config
        .maximized_column
        .get(&project_name)
        .and_then(|&opt| opt);

    log_debug(format!("当前最大化状态: {:?}", current_max));

    // 切换状态
    if current_max == Some(column) {
        // 已最大化当前列 -> 取消最大化
        log_debug("取消最大化".to_string());
        app.config
            .maximized_column
            .insert(project_name.clone(), None);
    } else {
        // 最大化当前列
        log_debug(format!("最大化列 {}", column));
        app.config
            .maximized_column
            .insert(project_name.clone(), Some(column));
    }

    // 保存配置
    if let Err(e) = crate::config::save_config(&app.config) {
        log_debug(format!("保存配置失败: {}", e));
    }

    // 记录调整时间
    app.last_column_resize_time = Some(std::time::Instant::now());
}

/// 归一化列宽，确保总和为 100%
fn normalize_widths(widths: &mut Vec<u16>) {
    let total: u16 = widths.iter().sum();
    if total == 100 {
        return;
    }

    // 按比例调整
    let scale = 100.0 / total as f32;
    for width in widths.iter_mut() {
        *width = (*width as f32 * scale).round() as u16;
    }

    // 处理舍入误差
    let new_total: u16 = widths.iter().sum();
    if new_total != 100 && !widths.is_empty() {
        let diff = 100i16 - new_total as i16;
        widths[0] = (widths[0] as i16 + diff) as u16;
    }
}

/// 获取当前聚焦项目的名称
fn get_focused_project_name(app: &App) -> Option<String> {
    if let Some(crate::ui::layout::SplitNode::Leaf { project_id, .. }) =
        app.split_tree.find_pane(app.focused_pane)
    {
        project_id.clone()
    } else {
        None
    }
}

/// 获取当前菜单的命令列表（不含空行）
fn get_menu_commands(menu_state: Option<crate::app::MenuState>) -> Vec<char> {
    use crate::app::MenuState;

    match menu_state {
        Some(MenuState::Main) | None => {
            vec!['f', 'p', 'w', 't', 's', 'r', 'R', '?', 'q']
        }
        Some(MenuState::Project) => {
            vec!['o', 'n', 'N', 'd', 'D', 'r', 'i']
        }
        Some(MenuState::Window) => {
            vec!['w', 'v', 's', 'q', 'm', 'h', 'l', 'k', 'j']
        }
        Some(MenuState::Task) => {
            vec!['a', 'e', 'E', 'v', 'V', 't', 'Y', 'd', 'h', 'm', 'l', 'n']
        }
        Some(MenuState::Status) => {
            vec!['a', 'r', 'e', 'h', 'l', 'd']
        }
    }
}

/// 向上导航菜单（跳过空行）
fn navigate_menu_up(app: &mut App) {
    let commands = get_menu_commands(app.menu_state);
    if commands.is_empty() {
        return;
    }

    let current = app.menu_selected_index.unwrap_or(0);
    if current > 0 {
        app.menu_selected_index = Some(current - 1);
    } else {
        // 循环到最后一项
        app.menu_selected_index = Some(commands.len() - 1);
    }
}

/// 向下导航菜单（跳过空行）
fn navigate_menu_down(app: &mut App) {
    let commands = get_menu_commands(app.menu_state);
    if commands.is_empty() {
        return;
    }

    let current = app.menu_selected_index.unwrap_or(0);
    if current < commands.len() - 1 {
        app.menu_selected_index = Some(current + 1);
    } else {
        // 循环到第一项
        app.menu_selected_index = Some(0);
    }
}

/// 执行选中的菜单命令
fn execute_selected_menu_command(app: &mut App, index: usize) {
    use crate::app::MenuState;

    let commands = get_menu_commands(app.menu_state);
    if index >= commands.len() {
        return;
    }

    let c = commands[index];

    // 执行命令（复用现有的字符处理逻辑）
    match app.menu_state {
        Some(MenuState::Main) => match c {
            'p' => {
                app.menu_state = Some(MenuState::Project);
                app.menu_selected_index = Some(0);
            }
            'w' => {
                app.menu_state = Some(MenuState::Window);
                app.menu_selected_index = Some(0);
            }
            't' => {
                app.menu_state = Some(MenuState::Task);
                app.menu_selected_index = Some(0);
            }
            's' => {
                app.menu_state = Some(MenuState::Status);
                app.menu_selected_index = Some(0);
            }
            'f' => {
                app.mode = Mode::Normal;
                app.menu_state = None;
                app.menu_selected_index = None;
                app.key_buffer.clear();
                execute_command(app, Command::OpenProject);
            }
            'r' => {
                app.mode = Mode::Normal;
                app.menu_state = None;
                app.menu_selected_index = None;
                app.key_buffer.clear();
                execute_command(app, Command::ReloadCurrentProject);
            }
            'R' => {
                app.mode = Mode::Normal;
                app.menu_state = None;
                app.menu_selected_index = None;
                app.key_buffer.clear();
                execute_command(app, Command::ReloadAllProjects);
            }
            'q' => {
                app.mode = Mode::Normal;
                app.menu_state = None;
                app.menu_selected_index = None;
                app.key_buffer.clear();
                execute_command(app, Command::Quit);
            }
            '?' => {
                app.mode = Mode::Help;
                app.menu_state = None;
                app.menu_selected_index = None;
                app.key_buffer.clear();
            }
            _ => {}
        },
        Some(MenuState::Project) => {
            let cmd = match c {
                'o' => Some(Command::OpenProject),
                'n' => Some(Command::NewLocalProject),
                'N' => Some(Command::NewGlobalProject),
                'd' => Some(Command::HideProject),
                'D' => Some(Command::DeleteProject),
                'r' => Some(Command::RenameProject),
                'i' => Some(Command::CopyProjectInfo),
                _ => None,
            };
            if let Some(cmd) = cmd {
                app.mode = Mode::Normal;
                app.menu_state = None;
                app.menu_selected_index = None;
                app.key_buffer.clear();
                execute_command(app, cmd);
            }
        }
        Some(MenuState::Window) => {
            let cmd = match c {
                'w' => Some(Command::FocusNextPane),
                'v' => Some(Command::SplitVertical),
                's' => Some(Command::SplitHorizontal),
                'q' => Some(Command::ClosePane),
                'h' => Some(Command::FocusLeft),
                'l' => Some(Command::FocusRight),
                'm' => Some(Command::MaximizePane),
                _ => None,
            };
            if let Some(cmd) = cmd {
                app.mode = Mode::Normal;
                app.menu_state = None;
                app.menu_selected_index = None;
                app.key_buffer.clear();
                execute_command(app, cmd);
            }
        }
        Some(MenuState::Task) => {
            let cmd = match c {
                'a' => Some(Command::NewTask),
                'e' => Some(Command::EditTask),
                'E' => Some(Command::EditTaskInEditor),
                'v' => Some(Command::ViewTask),
                'V' => Some(Command::ViewTaskExternal),
                't' => Some(Command::EditTags),
                'd' => Some(Command::DeleteTask),
                'Y' => Some(Command::CopyTask),
                'h' => Some(Command::SetTaskPriority("high".to_string())),
                'm' => Some(Command::SetTaskPriority("medium".to_string())),
                'l' => Some(Command::SetTaskPriority("low".to_string())),
                'n' => Some(Command::SetTaskPriority("none".to_string())),
                _ => None,
            };
            if let Some(cmd) = cmd {
                app.mode = Mode::Normal;
                app.menu_state = None;
                app.menu_selected_index = None;
                app.key_buffer.clear();
                execute_command(app, cmd);
            }
        }
        Some(MenuState::Status) => {
            let cmd = match c {
                'a' => Some(Command::CreateStatus),
                'r' => Some(Command::RenameStatus),
                'e' => Some(Command::EditStatusDisplay),
                'h' => Some(Command::MoveStatusLeft),
                'l' => Some(Command::MoveStatusRight),
                'd' => Some(Command::DeleteStatus),
                _ => None,
            };
            if let Some(cmd) = cmd {
                app.mode = Mode::Normal;
                app.menu_state = None;
                app.menu_selected_index = None;
                app.key_buffer.clear();
                execute_command(app, cmd);
            }
        }
        None => {}
    }
}
