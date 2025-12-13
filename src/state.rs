/// 应用状态持久化
use crate::ui::layout::SplitNode;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 应用状态（用于持久化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    /// 窗口分割布局树
    pub split_tree: SplitNode,
    /// 每个面板选中的列
    pub selected_columns: HashMap<usize, usize>,
    /// 每个面板选中的任务索引
    pub selected_task_indices: HashMap<usize, usize>,
    /// 当前聚焦的面板ID
    pub focused_pane: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            split_tree: SplitNode::new_leaf(0),
            selected_columns: HashMap::new(),
            selected_task_indices: HashMap::new(),
            focused_pane: 0,
        }
    }
}

/// 获取状态文件路径
/// All platforms: ~/.kanban/state.json
fn get_state_file_path() -> PathBuf {
    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .expect("Failed to get home directory");
    PathBuf::from(home_dir).join(".kanban").join("state.json")
}

/// 递归重新加载所有面板中的项目
fn reload_all_pane_projects(app: &mut crate::app::App) {
    fn reload_node_projects(node: &mut SplitNode, app: &mut crate::app::App) {
        match node {
            SplitNode::Leaf { project_id, .. } => {
                if let Some(project_name) = project_id {
                    // 检查项目是否存在，并重新加载项目数据
                    if let Some(project) = app.projects.iter().find(|p| &p.name == project_name) {
                        let project_path = project.path.clone();
                        let project_type = project.project_type;

                        // 重新加载项目以获取最新的任务数据
                        if let Ok(reloaded_project) = crate::fs::load_project_with_type(&project_path, project_type) {
                            // 更新项目列表中的数据
                            if let Some(idx) = app.projects.iter().position(|p| &p.name == project_name) {
                                app.projects[idx] = reloaded_project;
                            }
                        }
                    }
                }
            }
            SplitNode::Horizontal { left, right, .. } => {
                reload_node_projects(left, app);
                reload_node_projects(right, app);
            }
            SplitNode::Vertical { top, bottom, .. } => {
                reload_node_projects(top, app);
                reload_node_projects(bottom, app);
            }
        }
    }

    let mut tree = app.split_tree.clone();
    reload_node_projects(&mut tree, app);
    app.split_tree = tree;
}

/// 从应用中提取状态
pub fn extract_state(app: &crate::app::App) -> AppState {
    AppState {
        split_tree: app.split_tree.clone(),
        focused_pane: app.focused_pane,
        selected_columns: app.selected_column.clone(),
        selected_task_indices: app.selected_task_index.clone(),
    }
}

/// 保存状态到文件
pub fn save_state(state: &AppState) -> Result<()> {
    let state_path = get_state_file_path();

    // 确保目录存在
    if let Some(parent) = state_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(state)?;
    std::fs::write(state_path, json)?;

    Ok(())
}

/// 从文件加载状态
pub fn load_state() -> Result<AppState> {
    let state_path = get_state_file_path();

    if !state_path.exists() {
        return Ok(AppState::default());
    }

    let content = std::fs::read_to_string(state_path)?;
    let state: AppState = serde_json::from_str(&content)?;

    Ok(state)
}

/// 应用状态到应用
pub fn apply_state(app: &mut crate::app::App, state: AppState) {
    // 恢复布局树
    app.split_tree = state.split_tree;

    // 重新加载所有面板中的项目数据
    reload_all_pane_projects(app);

    // 恢复选中状态
    app.selected_column = state.selected_columns;
    app.selected_task_index = state.selected_task_indices;

    // 恢复聚焦面板（确保面板存在）
    let all_panes = app.split_tree.collect_pane_ids();
    if all_panes.contains(&state.focused_pane) {
        app.focused_pane = state.focused_pane;
    } else if let Some(&first_pane) = all_panes.first() {
        app.focused_pane = first_pane;
    }

    // 重要：更新 next_pane_id 为当前最大ID+1，避免ID冲突
    if let Some(&max_id) = all_panes.iter().max() {
        app.next_pane_id = max_id + 1;
    }
}
