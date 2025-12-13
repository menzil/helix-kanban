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
        Mode::Command => handle_command_mode(app, key),
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

    // 如果没有匹配到命令，将字符添加到缓冲区（用于多键序列）
    if let KeyCode::Char(c) = key.code {
        app.key_buffer.push(c);
    } else {
        // 非字符键清空缓冲区
        app.key_buffer.clear();
    }

    // 如果缓冲区太长，清空它
    if app.key_buffer.len() > 3 {
        app.key_buffer.clear();
    }

    true
}

/// 处理命令模式的按键
fn handle_command_mode(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.mode = Mode::Normal;
            app.command_input.clear();
        }
        KeyCode::Enter => {
            // 执行命令
            let should_continue = execute_text_command(app, &app.command_input.clone());
            app.command_input.clear();
            app.mode = Mode::Normal;
            return should_continue;
        }
        KeyCode::Backspace => {
            app.command_input.pop();
        }
        KeyCode::Char(c) => {
            app.command_input.push(c);
        }
        _ => {}
    }
    true
}

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

    if let Some(dialog) = &mut app.dialog {
        match dialog {
            DialogType::Input {
                value,
                cursor_pos,
                title,
                ..
            } => {
                // 判断是否是任务输入（支持多行）
                let is_task_input = title.contains("任务");

                // 调试：记录按键信息
                log_debug(format!(
                    "对话框按键: code={:?}, modifiers={:?}, is_task_input={}",
                    key.code, key.modifiers, is_task_input
                ));

                match key.code {
                    KeyCode::Esc => {
                        app.dialog = None;
                        app.mode = Mode::Normal;
                        // 退出对话框，保存用户输入法并切换回英文
                        app.ime_state.exit_dialog();
                    }
                    KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) && is_task_input => {
                        // Ctrl+J 换行（用于任务输入）
                        log_debug("检测到 Ctrl+J，插入换行".to_string());
                        let mut chars: Vec<char> = value.chars().collect();
                        chars.insert(*cursor_pos, '\n');
                        *value = chars.into_iter().collect();
                        *cursor_pos += 1;
                    }
                    KeyCode::Enter => {
                        if is_task_input {
                            // 任务输入：Enter 提交，Shift+Enter 或 Ctrl+J 换行
                            log_debug(format!("Enter 键，modifiers={:?}", key.modifiers));
                            if key.modifiers.contains(KeyModifiers::SHIFT) {
                                // Shift+Enter 换行（如果终端支持）
                                log_debug("检测到 Shift+Enter，插入换行".to_string());
                                let mut chars: Vec<char> = value.chars().collect();
                                chars.insert(*cursor_pos, '\n');
                                *value = chars.into_iter().collect();
                                *cursor_pos += 1;
                            } else {
                                // Enter 提交
                                log_debug("普通 Enter，提交任务".to_string());
                                let input_value = value.clone();
                                let dialog_clone = dialog.clone();
                                app.dialog = None;
                                app.mode = Mode::Normal;
                                // 退出对话框，保存用户输入法并切换回英文
                                app.ime_state.exit_dialog();
                                handle_dialog_submit(app, dialog_clone, input_value);
                            }
                        } else {
                            // 普通输入：Enter 直接提交
                            let input_value = value.clone();
                            let dialog_clone = dialog.clone();
                            app.dialog = None;
                            app.mode = Mode::Normal;
                            // 退出对话框，保存用户输入法并切换回英文
                            app.ime_state.exit_dialog();
                            handle_dialog_submit(app, dialog_clone, input_value);
                        }
                    }
                    KeyCode::Backspace => {
                    if *cursor_pos > 0 {
                        // 使用 chars() 正确处理多字节字符
                        let mut chars: Vec<char> = value.chars().collect();
                        if *cursor_pos <= chars.len() {
                            chars.remove(*cursor_pos - 1);
                            *value = chars.into_iter().collect();
                            *cursor_pos -= 1;
                        }
                    }
                }
                KeyCode::Delete => {
                    let char_count = value.chars().count();
                    if *cursor_pos < char_count {
                        // 使用 chars() 正确处理多字节字符
                        let mut chars: Vec<char> = value.chars().collect();
                        chars.remove(*cursor_pos);
                        *value = chars.into_iter().collect();
                    }
                }
                KeyCode::Left => {
                    *cursor_pos = cursor_pos.saturating_sub(1);
                }
                KeyCode::Right => {
                    let char_count = value.chars().count();
                    *cursor_pos = (*cursor_pos + 1).min(char_count);
                }
                KeyCode::Up if is_task_input => {
                    // 上移到上一行
                    *cursor_pos = move_cursor_vertical(value, *cursor_pos, -1);
                }
                KeyCode::Down if is_task_input => {
                    // 下移到下一行
                    *cursor_pos = move_cursor_vertical(value, *cursor_pos, 1);
                }
                KeyCode::Home => {
                    if is_task_input {
                        // 多行输入：移到当前行首
                        *cursor_pos = get_line_start(value, *cursor_pos);
                    } else {
                        // 单行输入：移到开头
                        *cursor_pos = 0;
                    }
                }
                KeyCode::End => {
                    if is_task_input {
                        // 多行输入：移到当前行尾
                        *cursor_pos = get_line_end(value, *cursor_pos);
                    } else {
                        // 单行输入：移到结尾
                        *cursor_pos = value.chars().count();
                    }
                }
                KeyCode::Char(c) => {
                    // 使用 chars() 正确插入字符
                    let mut chars: Vec<char> = value.chars().collect();
                    chars.insert(*cursor_pos, c);
                    *value = chars.into_iter().collect();
                    *cursor_pos += 1;
                }
                _ => {}
                }
            }
            DialogType::Select {
                items,
                selected,
                filter,
                ..
            } => match key.code {
                KeyCode::Esc => {
                    app.dialog = None;
                    app.mode = Mode::Normal;
                    // 退出对话框，保存用户输入法并切换回英文
                    app.ime_state.exit_dialog();
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
                        let dialog_clone = dialog.clone();
                        app.dialog = None;
                        app.mode = Mode::Normal;
                        // 退出对话框，保存用户输入法并切换回英文
                        app.ime_state.exit_dialog();
                        handle_dialog_submit(app, dialog_clone, selected_item);
                    }
                }
                KeyCode::Up => {
                    *selected = selected.saturating_sub(1);
                }
                KeyCode::Down => {
                    let filtered_count = if filter.is_empty() {
                        items.len()
                    } else {
                        items
                            .iter()
                            .filter(|item| item.to_lowercase().contains(&filter.to_lowercase()))
                            .count()
                    };
                    *selected = (*selected + 1).min(filtered_count.saturating_sub(1));
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
            },
            DialogType::Confirm { yes_selected, .. } => match key.code {
                KeyCode::Esc | KeyCode::Char('n') => {
                    app.dialog = None;
                    app.mode = Mode::Normal;
                    // 退出对话框，保存用户输入法并切换回英文
                    app.ime_state.exit_dialog();
                }
                KeyCode::Enter => {
                    let confirmed = *yes_selected;
                    let dialog_clone = dialog.clone();
                    app.dialog = None;
                    app.mode = Mode::Normal;
                    // 退出对话框，保存用户输入法并切换回英文
                    app.ime_state.exit_dialog();
                    if confirmed {
                        handle_dialog_submit(app, dialog_clone, String::new());
                    }
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    *yes_selected = false;  // 左边是"否"
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    *yes_selected = true;   // 右边是"是"
                }
                KeyCode::Char('y') => {
                    let dialog_clone = dialog.clone();
                    app.dialog = None;
                    app.mode = Mode::Normal;
                    // 退出对话框，保存用户输入法并切换回英文
                    app.ime_state.exit_dialog();
                    handle_dialog_submit(app, dialog_clone, String::new());
                }
                _ => {}
            },
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
        let _ = writeln!(file, "[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg);
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
                                    log_debug(format!("调试: 重新加载了 {} 个项目", projects.len()));
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
            } else if (title.contains("创建") || title.contains("新建")) && title.contains("任务") {
                // 创建新任务
                log_debug(format!("调试: 识别为创建任务请求"));
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
            } else if title.contains("重命名项目") {
                // 重命名项目
                if !value.is_empty() {
                    rename_current_project(app, value);
                }
            }
        }
        DialogType::Select { title, .. } => {
            if title.contains("选择项目") || title.contains("打开项目") || title.contains("切换项目") {
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
                        if let Err(e) = crate::config::hide_project(&mut app.config, &project_name) {
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
                        let project_type = project.project_type.clone();
                        let project_path = project.path.clone();

                        log_debug(format!("准备删除项目: 名称='{}', 类型={:?}, 路径={:?}",
                            project_name, project_type, project_path));

                        // 删除项目目录
                        match crate::fs::delete_project(&project_name, &project_type) {
                            Err(e) => {
                                log_debug(format!("删除项目失败: {}", e));
                            }
                            Ok(_) => {
                                log_debug(format!("成功删除项目: {}", project_name));

                                // 从项目列表中移除
                                app.projects.retain(|p| p.name != project_name);
                                log_debug(format!("已从项目列表移除，剩余项目数: {}", app.projects.len()));

                                // 清除当前面板的项目引用
                                if let Some(crate::ui::layout::SplitNode::Leaf { project_id, .. }) =
                                    app.split_tree.find_pane_mut(app.focused_pane)
                                {
                                    *project_id = None;
                                    log_debug("已清除当前面板的项目引用".to_string());
                                }
                            }
                        }
                    } else {
                        log_debug("无法获取当前聚焦的项目".to_string());
                    }
                }
                crate::ui::dialogs::ConfirmAction::DeleteTask => {
                    // 删除任务
                    if let Some(task) = get_selected_task(app) {
                        let task_file = task.file_path.clone();

                        // 删除任务文件
                        if let Err(e) = std::fs::remove_file(&task_file) {
                            log_debug(format!("删除任务文件失败: {}", e));
                        } else {
                            // 重新加载当前项目
                            if let Err(e) = app.reload_current_project() {
                                log_debug(format!("重新加载项目失败: {}", e));
                            }

                            // 调整选中的任务索引
                            let task_idx = app.selected_task_index.entry(app.focused_pane).or_insert(0);
                            if *task_idx > 0 {
                                *task_idx -= 1;
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
        ([], KeyCode::Char(':'), KeyModifiers::NONE) => Some(Command::EnterCommandMode),
        ([], KeyCode::Esc, _) => Some(Command::EnterNormalMode),
        ([], KeyCode::Char('d'), KeyModifiers::NONE) => Some(Command::DeleteTask),  // 删除任务
        ([], KeyCode::Char('D'), KeyModifiers::SHIFT) => Some(Command::DeleteProject),  // 硬删除项目
        ([], KeyCode::Char('a'), KeyModifiers::NONE) => Some(Command::NewTask),
        ([], KeyCode::Char('A'), KeyModifiers::SHIFT) => Some(Command::NewTaskInEditor),  // 外部编辑器创建任务
        ([], KeyCode::Char('n'), KeyModifiers::NONE) => Some(Command::NewLocalProject),
        ([], KeyCode::Char('N'), KeyModifiers::SHIFT) => Some(Command::NewGlobalProject),
        ([], KeyCode::Char('e'), KeyModifiers::NONE) => Some(Command::EditTask),
        ([], KeyCode::Char('E'), KeyModifiers::SHIFT) => Some(Command::EditTaskInEditor),
        ([], KeyCode::Char('v'), KeyModifiers::NONE) => Some(Command::ViewTask),
        ([], KeyCode::Char('V'), KeyModifiers::SHIFT) => Some(Command::ViewTaskExternal),

        ([], KeyCode::Down, _) => Some(Command::TaskDown),
        ([], KeyCode::Up, _) => Some(Command::TaskUp),
        ([], KeyCode::Left, _) => Some(Command::ColumnLeft),
        ([], KeyCode::Right, _) => Some(Command::ColumnRight),

        // ===== Space w 序列（窗口管理）=====
        // 缓冲区包含 [' ', 'w']，当前键是 'v'
        ([' ', 'w'], KeyCode::Char('v'), _) => Some(Command::SplitHorizontal),
        ([' ', 'w'], KeyCode::Char('s'), _) => Some(Command::SplitVertical),
        ([' ', 'w'], KeyCode::Char('c'), _) => Some(Command::ClosePane),
        ([' ', 'w'], KeyCode::Char('h'), _) => Some(Command::FocusLeft),
        ([' ', 'w'], KeyCode::Char('l'), _) => Some(Command::FocusRight),
        ([' ', 'w'], KeyCode::Char('k'), _) => Some(Command::FocusUp),
        ([' ', 'w'], KeyCode::Char('j'), _) => Some(Command::FocusDown),

        // ===== Space p 序列（项目管理）=====
        // 缓冲区包含 [' ', 'p']，当前键是 'o'
        ([' ', 'p'], KeyCode::Char('o'), _) => Some(Command::OpenProject),
        ([' ', 'p'], KeyCode::Char('n'), _) => Some(Command::NewProject),
        ([' ', 'p'], KeyCode::Char('d'), _) => Some(Command::DeleteProject),
        ([' ', 'p'], KeyCode::Char('r'), _) => Some(Command::RenameProject),

        _ => None,
    }
}

/// 执行命令
fn execute_command(app: &mut App, cmd: Command) {
    use crate::ui::dialogs::DialogType;

    match cmd {
        Command::SplitHorizontal => {
            // 水平分割线 = 上下分屏
            if let Some(pane) = app.split_tree.find_pane_mut(app.focused_pane) {
                let new_pane_id = app.next_pane_id;
                pane.split_vertical(new_pane_id);  // split_vertical 创建上下分屏
                app.next_pane_id += 1;
                // 自动对焦新创建的窗口
                app.focused_pane = new_pane_id;
            }
        }
        Command::SplitVertical => {
            // 垂直分割线 = 左右分屏
            if let Some(pane) = app.split_tree.find_pane_mut(app.focused_pane) {
                let new_pane_id = app.next_pane_id;
                pane.split_horizontal(new_pane_id);  // split_horizontal 创建左右分屏
                app.next_pane_id += 1;
                // 自动对焦新创建的窗口
                app.focused_pane = new_pane_id;
            }
        }
        Command::TaskDown => {
            // 获取当前列的任务数量并限制索引
            if let Some(project) = app.get_focused_project() {
                let column = app.selected_column.get(&app.focused_pane).copied().unwrap_or(0);
                let status = match column {
                    0 => "todo",
                    1 => "doing",
                    2 => "done",
                    _ => return,
                };

                let task_count = project.tasks.iter()
                    .filter(|t| t.status == status)
                    .count();

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
            let col = app.selected_column.entry(app.focused_pane).or_insert(0);
            *col = (*col + 1).min(2); // 最多3列
            // 切换列时重置任务索引到 0
            app.selected_task_index.insert(app.focused_pane, 0);
        }
        Command::EnterCommandMode => {
            app.mode = Mode::Command;
        }
        Command::EnterNormalMode | Command::Cancel => {
            app.mode = Mode::Normal;
            app.key_buffer.clear();
            app.command_input.clear();
        }
        Command::NewProject => {
            app.mode = Mode::Dialog;
            app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法
            app.dialog = Some(DialogType::Input {
                title: "创建新项目".to_string(),
                prompt: "请输入项目名称:".to_string(),
                value: String::new(),
                cursor_pos: 0,
            });
        }
        Command::NewLocalProject => {
            app.mode = Mode::Dialog;
            app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法
            app.dialog = Some(DialogType::Input {
                title: "创建新本地项目 [L]".to_string(),
                prompt: "请输入项目名称:".to_string(),
                value: String::new(),
                cursor_pos: 0,
            });
        }
        Command::NewGlobalProject => {
            app.mode = Mode::Dialog;
            app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法
            app.dialog = Some(DialogType::Input {
                title: "创建新全局项目 [G]".to_string(),
                prompt: "请输入项目名称:".to_string(),
                value: String::new(),
                cursor_pos: 0,
            });
        }
        Command::OpenProject => {
            app.mode = Mode::Dialog;
            app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法
            // 生成格式化的项目列表：[G/L] 项目名\n    路径
            let project_items: Vec<String> = app.projects.iter().map(|p| {
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
            }).collect();

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
                let cursor_pos = current_name.chars().count();
                app.mode = Mode::Dialog;
                app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法
                app.dialog = Some(DialogType::Input {
                    title: "重命名项目".to_string(),
                    prompt: "请输入新的项目名称:".to_string(),
                    value: current_name,
                    cursor_pos,
                });
            }
        }
        Command::HideProject => {
            // 隐藏当前项目（软删除）
            if let Some(project) = app.get_focused_project() {
                let project_name = project.name.clone();
                let project_type = project.project_type.clone();
                let project_path = project.path.clone();

                // 检查是否是当前工作目录的本地项目
                if project_type == crate::models::ProjectType::Local {
                    let current_local_dir = crate::fs::get_local_kanban_dir();
                    // 如果项目路径是当前目录的 .kanban，则不支持软删除
                    if project_path.starts_with(&current_local_dir) {
                        log_debug("当前目录的本地项目不支持软删除，请使用 D 键删除项目文件".to_string());
                        return;
                    }
                }

                // 其他项目（全局项目或其他目录的本地项目）：显示确认对话框
                app.mode = Mode::Dialog;
                app.ime_state.enter_dialog();
                app.dialog = Some(DialogType::Confirm {
                    title: "隐藏项目".to_string(),
                    message: format!("确定要隐藏项目 \"{}\" 吗？\n项目文件不会被删除，下次从该目录启动时会重新加载。", project_name),
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
                app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法
                app.dialog = Some(DialogType::Confirm {
                    title: "删除项目文件".to_string(),
                    message: format!("确定要彻底删除项目 \"{}\" 吗？\n这将永久删除项目的所有文件和任务！此操作不可恢复！", project_name),
                    yes_selected: false,  // 默认选择"否"，更安全
                    action: crate::ui::dialogs::ConfirmAction::DeleteProject,
                });
            }
        }
        Command::NewTask => {
            app.mode = Mode::Dialog;
            app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法
            app.dialog = Some(DialogType::Input {
                title: "创建新任务".to_string(),
                prompt: "请输入任务标题:".to_string(),
                value: String::new(),
                cursor_pos: 0,
            });
        }
        Command::NewTaskInEditor => {
            // 用外部编辑器创建新任务
            // 创建临时文件
            use std::io::Write;

            let temp_dir = std::env::temp_dir();
            let temp_file = temp_dir.join(format!("kanban_new_task_{}.md",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()));

            // 写入模板内容
            let template = "# 任务标题\n\n任务描述内容...\n\n## 子任务\n\n- [ ] 子任务 1\n- [ ] 子任务 2\n";
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
                let cursor_pos = title.chars().count();
                app.mode = Mode::Dialog;
                app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法
                app.dialog = Some(DialogType::Input {
                    title: "编辑任务".to_string(),
                    prompt: "修改任务标题:".to_string(),
                    value: title,
                    cursor_pos,
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
            if all_panes.len() > 1 {
                if let Some(current_idx) = all_panes.iter().position(|&id| id == app.focused_pane) {
                    let next_idx = (current_idx + 1) % all_panes.len();
                    app.focused_pane = all_panes[next_idx];
                }
            }
        }
        Command::FocusLeft => {
            if let Some(new_pane_id) = app.split_tree.find_adjacent_pane(
                app.focused_pane,
                crate::ui::layout::Direction::Left,
            ) {
                app.focused_pane = new_pane_id;
            }
        }
        Command::FocusRight => {
            if let Some(new_pane_id) = app.split_tree.find_adjacent_pane(
                app.focused_pane,
                crate::ui::layout::Direction::Right,
            ) {
                app.focused_pane = new_pane_id;
            }
        }
        Command::FocusUp => {
            if let Some(new_pane_id) = app.split_tree.find_adjacent_pane(
                app.focused_pane,
                crate::ui::layout::Direction::Up,
            ) {
                app.focused_pane = new_pane_id;
            }
        }
        Command::FocusDown => {
            if let Some(new_pane_id) = app.split_tree.find_adjacent_pane(
                app.focused_pane,
                crate::ui::layout::Direction::Down,
            ) {
                app.focused_pane = new_pane_id;
            }
        }
        Command::ClosePane => {
            // 关闭当前面板
            let current_pane = app.focused_pane;
            if app.split_tree.close_pane(current_pane) {
                // 关闭成功，需要重新聚焦到一个有效的面板
                let all_panes = app.split_tree.collect_pane_ids();
                if let Some(&first_pane) = all_panes.first() {
                    app.focused_pane = first_pane;
                }
            }
            // 如果关闭失败（比如只有一个面板），不做任何操作
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
                app.ime_state.enter_dialog();  // 进入对话框，恢复用户输入法
                app.dialog = Some(DialogType::Confirm {
                    title: "删除任务".to_string(),
                    message: format!("确定要删除任务 \"{}\" 吗？", task_title),
                    yes_selected: true,
                    action: crate::ui::dialogs::ConfirmAction::DeleteTask,
                });
            }
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
                    log_debug(format!("重新加载所有项目成功，共 {} 个", app.projects.len()));
                }
                Err(e) => {
                    log_debug(format!("重新加载所有项目失败: {}", e));
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
    let column = app.selected_column.get(&app.focused_pane).copied().unwrap_or(0);
    let task_idx = app.selected_task_index.get(&app.focused_pane).copied().unwrap_or(0);

    let status = match column {
        0 => "todo",
        1 => "doing",
        2 => "done",
        _ => return None,
    };

    let tasks: Vec<_> = project.tasks.iter().filter(|t| t.status == status).collect();
    tasks.get(task_idx).copied()
}

/// 获取当前选中的任务（可变）- 返回任务 ID
fn get_selected_task_id(app: &App) -> Option<u32> {
    let column = app.selected_column.get(&app.focused_pane).copied().unwrap_or(0);
    let task_idx = app.selected_task_index.get(&app.focused_pane).copied().unwrap_or(0);

    let status = match column {
        0 => "todo",
        1 => "doing",
        2 => "done",
        _ => return None,
    };

    let project = app.get_focused_project()?;
    let tasks: Vec<_> = project.tasks.iter().filter(|t| t.status == status).collect();
    tasks.get(task_idx).map(|t| t.id)
}

/// 移动任务到相邻状态
fn move_task_to_status(app: &mut App, direction: i32) {
    let column = app.selected_column.get(&app.focused_pane).copied().unwrap_or(0);
    let new_column = (column as i32 + direction).clamp(0, 2) as usize;

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
        app.split_tree.find_pane(app.focused_pane) {
        if let Some(name) = project_id {
            name.clone()
        } else {
            return;
        }
    } else {
        return;
    };

    // 找到任务并修改
    if let Some(project) = app.projects.iter_mut().find(|p| p.name == project_name) {
        if let Some(task) = project.tasks.iter_mut().find(|t| t.id == task_id) {
            let new_status = match new_column {
                0 => "todo",
                1 => "doing",
                2 => "done",
                _ => return,
            };

            let old_status = task.status.clone();
            task.status = new_status.to_string();

            // 移动文件到新的状态目录（使用项目的实际路径）
            let project_path = project.path.clone();

            match crate::fs::move_task(&project_path, task, new_status) {
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
}

/// 在列内上下移动任务
fn move_task_in_column(app: &mut App, direction: i32) {
    let column = app.selected_column.get(&app.focused_pane).copied().unwrap_or(0);
    let task_idx = app.selected_task_index.get(&app.focused_pane).copied().unwrap_or(0);

    let status = match column {
        0 => "todo",
        1 => "doing",
        2 => "done",
        _ => return,
    };

    // 获取当前列的所有任务
    if let Some(project) = app.get_focused_project() {
        let tasks: Vec<_> = project.tasks.iter().filter(|t| t.status == status).collect();
        let task_count = tasks.len();

        if task_count < 2 {
            return; // 不足以移动
        }

        let new_idx = (task_idx as i32 + direction).clamp(0, task_count as i32 - 1) as usize;

        if new_idx == task_idx {
            return; // 已经在边界
        }

        // 获取要交换的两个任务的 id
        let task1_id = tasks.get(task_idx).map(|t| t.id);
        let task2_id = tasks.get(new_idx).map(|t| t.id);

        if let (Some(id1), Some(id2)) = (task1_id, task2_id) {
            // 在项目中找到并交换
            if let Some(project) = app.get_focused_project_mut() {
                let pos1 = project.tasks.iter().position(|t| t.id == id1);
                let pos2 = project.tasks.iter().position(|t| t.id == id2);

                if let (Some(p1), Some(p2)) = (pos1, pos2) {
                    project.tasks.swap(p1, p2);
                    app.selected_task_index.insert(app.focused_pane, new_idx);

                    // TODO: 保存到文件系统（可以添加序号或排序字段）
                }
            }
        }
    }
}

/// 创建新任务
fn create_new_task(app: &mut App, title: String) {
    use crate::models::Task;

    log_debug(format!("调试: 准备创建任务 '{}'", title));

    // 获取当前项目
    let project_name = if let Some(crate::ui::layout::SplitNode::Leaf { project_id, .. }) =
        app.split_tree.find_pane(app.focused_pane) {
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
    let project_path = if let Some(project) = app.projects.iter().find(|p| &p.name == &project_name) {
        project.path.clone()
    } else {
        log_debug("调试: 在项目列表中找不到项目".to_string());
        return;
    };

    // 获取下一个任务 ID
    if let Ok(next_id) = crate::fs::get_next_task_id(&project_path) {
            log_debug(format!("调试: 下一个任务ID {}", next_id));
            // 获取当前选中的列作为初始状态
            let column = app.selected_column.get(&app.focused_pane).copied().unwrap_or(0);
            let status = match column {
                0 => "todo",
                1 => "doing",
                2 => "done",
                _ => "todo",
            };
            log_debug(format!("调试: 状态 '{}'", status));

            // 创建任务
            let task = Task::new(next_id, title, status.to_string());

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
                log_debug(format!("调试: 重新加载项目，共 {} 个任务", updated_project.tasks.len()));
                if let Some(project) = app.projects.iter_mut().find(|p| p.name == project_name) {
                    *project = updated_project;

                    // 找到新任务在当前列的索引（应该是最后一个）
                    let new_task_idx = project.tasks.iter()
                        .filter(|t| t.status == status)
                        .count()
                        .saturating_sub(1);

                    // 自动选中新创建的任务
                    app.selected_task_index.insert(app.focused_pane, new_task_idx);
                    log_debug(format!("调试: 选中任务索引 {}", new_task_idx));
                } else {
                    log_debug(format!("调试: 在 app.projects 中找不到项目 '{}'", project_name));
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
        app.split_tree.find_pane(app.focused_pane) {
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
        app.split_tree.find_pane(app.focused_pane) {
        if let Some(name) = project_id {
            name.clone()
        } else {
            return;
        }
    } else {
        return;
    };

    // 找到任务并更新
    if let Some(project) = app.projects.iter_mut().find(|p| p.name == project_name) {
        if let Some(task) = project.tasks.iter_mut().find(|t| t.id == task_id) {
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
                    app.mode = Mode::Normal;
                    app.key_buffer.clear();
                }
                Some(_) => {
                    app.menu_state = Some(MenuState::Main);
                }
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
                        'd' => Some(Command::HideProject),     // 小写d = 软删除（隐藏）
                        'D' => Some(Command::DeleteProject),   // 大写D = 硬删除
                        'r' => Some(Command::RenameProject),
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
                        'm' => Some(Command::MaximizePane),  // 添加最大化快捷键
                        _ => None,
                    };
                    if let Some(cmd) = cmd {
                        app.mode = Mode::Normal;
                        app.menu_state = None;
                        app.key_buffer.clear();
                        execute_command(app, cmd);
                    }
                }
                Some(MenuState::Task) => {
                    // 任务子菜单：立即执行命令并退出菜单
                    let cmd = match c {
                        'a' => Some(Command::NewTask),  // 改为 a 键新建任务
                        'e' => Some(Command::EditTask),
                        'E' => Some(Command::EditTaskInEditor),
                        'v' => Some(Command::ViewTask),
                        'V' => Some(Command::ViewTaskExternal),
                        'd' => Some(Command::DeleteTask),
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
