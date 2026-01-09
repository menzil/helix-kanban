use crate::input::CommandRegistry;
use crate::models::Project;
use crate::ui::dialogs::DialogType;
use crate::ui::layout::SplitNode;
use anyhow::Result;
use std::collections::HashMap;
use std::time::Instant;

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
}

/// 通知级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// 通知消息
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub created_at: Instant,
}

impl Notification {
    /// 检查通知是否已过期（3秒后自动消失）
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed().as_secs() >= 3
    }
}

/// 应用模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Mode {
    /// 正常模式 - 导航和查看
    Normal,
    // /// 命令模式 - 输入命令 (已注释)
    // Command,
    /// 任务选择模式
    TaskSelect,
    /// 对话框模式
    Dialog,
    /// 帮助模式 - 显示快捷键
    Help,
    /// 空格菜单模式 - 显示命令菜单
    SpaceMenu,
    /// 预览模式 - 显示任务内容
    Preview,
}

/// 空格菜单状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuState {
    /// 主菜单
    Main,
    /// 项目管理子菜单
    Project,
    /// 窗口管理子菜单
    Window,
    /// 任务管理子菜单
    Task,
    /// 状态管理子菜单
    Status,
}

/// 应用状态
pub struct App {
    /// 所有项目列表
    pub projects: Vec<Project>,
    /// 分屏树结构
    pub split_tree: SplitNode,
    /// 当前聚焦的面板ID
    pub focused_pane: usize,
    /// 当前模式
    pub mode: Mode,
    /// 按键缓冲区（用于键序列匹配）
    pub key_buffer: Vec<char>,
    /// 每个面板选中的任务索引
    pub selected_task_index: HashMap<usize, usize>,
    /// 每个面板选中的列 (0=todo, 1=doing, 2=done)
    pub selected_column: HashMap<usize, usize>,
    /// 命令输入缓冲
    pub command_input: String,
    /// 命令补全选中的索引
    #[allow(dead_code)]
    pub completion_selected_index: Option<usize>,
    /// 下一个面板ID
    pub next_pane_id: usize,
    /// 是否应该退出
    #[allow(dead_code)]
    pub should_quit: bool,
    /// 当前显示的对话框
    pub dialog: Option<DialogType>,
    /// 空格菜单状态 (None = 关闭)
    pub menu_state: Option<MenuState>,
    /// 菜单选中的项目索引 (用于上下键导航)
    pub menu_selected_index: Option<usize>,
    /// 待打开的编辑器文件路径（用于外部编辑器调用）
    pub pending_editor_file: Option<String>,
    /// 标识待编辑文件是否是新任务（true=新任务，false=编辑现有任务）
    pub is_new_task_file: bool,
    /// 待预览的文件路径（用于外部预览工具调用）
    pub pending_preview_file: Option<String>,
    /// 预览模式的内容
    pub preview_content: String,
    /// 预览模式的滚动位置
    pub preview_scroll: u16,
    /// 命令注册表
    pub command_registry: CommandRegistry,
    /// 应用配置
    pub config: crate::config::Config,
    /// 是否显示首次运行欢迎对话框
    pub show_welcome_dialog: bool,
    /// 最大化前的窗口布局（用于恢复）
    pub saved_layout: Option<SplitNode>,
    /// 通知消息
    pub notification: Option<Notification>,
    /// 最后一次列宽调整的时间（用于控制百分比显示）
    pub last_column_resize_time: Option<std::time::Instant>,
}

impl App {
    /// 创建新的应用实例
    pub fn new() -> Result<Self> {
        // 检查首次运行并加载配置
        let (config, is_first_run) = crate::config::check_first_run()?;

        // 加载所有项目
        let projects = crate::fs::load_all_projects()?;

        // 创建初始分屏树，如果有项目则自动加载第一个
        let mut split_tree = SplitNode::new_leaf(0);
        if !projects.is_empty() {
            if let Some(SplitNode::Leaf { project_id, .. }) = split_tree.find_pane_mut(0) {
                *project_id = Some(projects[0].name.clone());
            }
        }

        let mut app = Self {
            projects,
            split_tree,
            focused_pane: 0,
            mode: Mode::Normal,
            key_buffer: Vec::new(),
            selected_task_index: HashMap::new(),
            selected_column: HashMap::new(),
            command_input: String::new(),
            completion_selected_index: None,
            next_pane_id: 1,
            should_quit: false,
            dialog: None,
            menu_state: None,
            menu_selected_index: None,
            pending_editor_file: None,
            is_new_task_file: false,
            pending_preview_file: None,
            preview_content: String::new(),
            preview_scroll: 0,
            command_registry: CommandRegistry::new(),
            config,
            show_welcome_dialog: is_first_run,
            saved_layout: None,
            notification: None,
            last_column_resize_time: None,
        };

        // 调试：记录初始状态
        log_debug(format!(
            "App初始化: focused_pane={}, next_pane_id={}, pane_ids={:?}",
            app.focused_pane,
            app.next_pane_id,
            app.split_tree.collect_pane_ids()
        ));

        // 尝试加载保存的状态
        if let Ok(state) = crate::state::load_state() {
            crate::state::apply_state(&mut app, state);
            log_debug(format!(
                "加载状态后: focused_pane={}, next_pane_id={}, pane_ids={:?}",
                app.focused_pane,
                app.next_pane_id,
                app.split_tree.collect_pane_ids()
            ));
        }

        Ok(app)
    }

    /// 处理键盘输入
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crate::input::handle_key_input;
        handle_key_input(self, key)
    }

    /// 获取当前聚焦面板显示的项目
    pub fn get_focused_project(&self) -> Option<&Project> {
        if let Some(SplitNode::Leaf { project_id, .. }) =
            self.split_tree.find_pane(self.focused_pane)
        {
            if let Some(pid) = project_id {
                return self.projects.iter().find(|p| &p.name == pid);
            }
        }
        None
    }

    /// 获取当前聚焦面板显示的项目（可变）
    #[allow(dead_code)]
    pub fn get_focused_project_mut(&mut self) -> Option<&mut Project> {
        if let Some(SplitNode::Leaf { project_id, .. }) =
            self.split_tree.find_pane(self.focused_pane)
        {
            if let Some(pid) = project_id.clone() {
                return self.projects.iter_mut().find(|p| p.name == pid);
            }
        }
        None
    }

    /// 设置当前聚焦面板的项目
    pub fn set_focused_project(&mut self, project_name: String) {
        // 重新从文件系统加载项目数据，确保获取最新的任务列表
        let projects_dir = crate::fs::get_projects_dir();
        let project_path = projects_dir.join(&project_name);

        if let Ok(updated_project) = crate::fs::load_project(&project_path) {
            // 更新 projects 列表中的项目数据
            if let Some(project) = self.projects.iter_mut().find(|p| p.name == project_name) {
                *project = updated_project;
            } else {
                // 如果项目不在列表中（新创建的），添加它
                self.projects.push(updated_project);
            }
        }

        // 设置当前面板的项目
        if let Some(SplitNode::Leaf { project_id, .. }) =
            self.split_tree.find_pane_mut(self.focused_pane)
        {
            *project_id = Some(project_name);
            // 重置选中索引到 0
            self.selected_task_index.insert(self.focused_pane, 0);
            self.selected_column.insert(self.focused_pane, 0);

            // 保存状态
            let state = crate::state::extract_state(self);
            let _ = crate::state::save_state(&state);
        }
    }

    /// 重新加载当前聚焦面板的项目（用于外部编辑器保存后刷新）
    pub fn reload_current_project(&mut self) -> Result<()> {
        if let Some(SplitNode::Leaf { project_id, .. }) =
            self.split_tree.find_pane(self.focused_pane)
        {
            if let Some(pid) = project_id {
                // 从项目列表中找到项目路径和类型
                if let Some(project) = self.projects.iter().find(|p| &p.name == pid) {
                    let project_path = project.path.clone();
                    let project_type = project.project_type;

                    // 重新加载项目
                    if let Ok(updated_project) =
                        crate::fs::load_project_with_type(&project_path, project_type)
                    {
                        if let Some(project) = self.projects.iter_mut().find(|p| &p.name == pid) {
                            *project = updated_project;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// 切换当前面板的最大化状态
    pub fn toggle_maximize(&mut self) {
        if let Some(saved) = self.saved_layout.take() {
            // 当前处于最大化状态，恢复原布局
            self.split_tree = saved;

            // 保存状态
            let state = crate::state::extract_state(self);
            let _ = crate::state::save_state(&state);
        } else {
            // 当前不是最大化状态，保存当前布局并最大化
            // 只有在有多个面板时才需要最大化
            if self.split_tree.collect_pane_ids().len() > 1 {
                self.saved_layout = Some(self.split_tree.clone());

                // 获取当前聚焦面板的内容
                if let Some(SplitNode::Leaf { project_id, id }) =
                    self.split_tree.find_pane(self.focused_pane)
                {
                    let project_id = project_id.clone();
                    let pane_id = *id;

                    // 创建只包含当前面板的新布局
                    self.split_tree = SplitNode::Leaf {
                        project_id,
                        id: pane_id,
                    };

                    // 保存状态（最大化后的状态）
                    let state = crate::state::extract_state(self);
                    let _ = crate::state::save_state(&state);
                }
            }
        }
    }

    /// 显示通知消息
    pub fn show_notification(&mut self, message: String, level: NotificationLevel) {
        self.notification = Some(Notification {
            message,
            level,
            created_at: Instant::now(),
        });
    }

    /// 清除已过期的通知
    pub fn clear_expired_notification(&mut self) {
        if let Some(ref notification) = self.notification {
            if notification.is_expired() {
                self.notification = None;
            }
        }
    }

    /// 根据列索引获取状态名称
    pub fn get_status_name_by_column(&self, column: usize) -> Option<String> {
        self.get_focused_project()?
            .statuses
            .get(column)
            .map(|s| s.name.clone())
    }

    /// 获取当前项目的状态列数
    pub fn get_status_count(&self) -> usize {
        self.get_focused_project()
            .map(|p| p.statuses.len())
            .unwrap_or(3)
    }
}
