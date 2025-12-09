use crate::models::{Project, Task};
use crate::ui::layout::SplitNode;
use crate::ui::dialogs::DialogType;
use anyhow::Result;
use std::collections::HashMap;

/// 应用模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// 正常模式 - 导航和查看
    Normal,
    /// 命令模式 - 输入命令
    Command,
    /// 任务选择模式
    TaskSelect,
    /// 对话框模式
    Dialog,
    /// 帮助模式 - 显示快捷键
    Help,
    /// 空格菜单模式 - 显示命令菜单
    SpaceMenu,
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
    /// 下一个面板ID
    pub next_pane_id: usize,
    /// 是否应该退出
    pub should_quit: bool,
    /// 当前显示的对话框
    pub dialog: Option<DialogType>,
    /// 空格菜单状态 (None = 关闭)
    pub menu_state: Option<MenuState>,
}

impl App {
    /// 创建新的应用实例
    pub fn new() -> Result<Self> {
        // 加载所有项目
        let projects = crate::fs::load_all_projects()?;

        // 创建初始分屏树，如果有项目则自动加载第一个
        let mut split_tree = SplitNode::new_leaf(0);
        if !projects.is_empty() {
            if let Some(SplitNode::Leaf { project_id, .. }) = split_tree.find_pane_mut(0) {
                *project_id = Some(projects[0].name.clone());
            }
        }

        Ok(Self {
            projects,
            split_tree,
            focused_pane: 0,
            mode: Mode::Normal,
            key_buffer: Vec::new(),
            selected_task_index: HashMap::new(),
            selected_column: HashMap::new(),
            command_input: String::new(),
            next_pane_id: 1,
            should_quit: false,
            dialog: None,
            menu_state: None,
        })
    }

    /// 处理键盘输入
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crate::input::handle_key_input;
        handle_key_input(self, key)
    }

    /// 获取当前聚焦面板显示的项目
    pub fn get_focused_project(&self) -> Option<&Project> {
        if let Some(SplitNode::Leaf { project_id, .. }) = self.split_tree.find_pane(self.focused_pane)
        {
            if let Some(pid) = project_id {
                return self.projects.iter().find(|p| &p.name == pid);
            }
        }
        None
    }

    /// 获取当前聚焦面板显示的项目（可变）
    pub fn get_focused_project_mut(&mut self) -> Option<&mut Project> {
        if let Some(SplitNode::Leaf { project_id, .. }) = self.split_tree.find_pane(self.focused_pane)
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
        if let Some(SplitNode::Leaf { project_id, .. }) = self.split_tree.find_pane_mut(self.focused_pane)
        {
            *project_id = Some(project_name);
            // 重置选中索引到 0
            self.selected_task_index.insert(self.focused_pane, 0);
            self.selected_column.insert(self.focused_pane, 0);
        }
    }
}
