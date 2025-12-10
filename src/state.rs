/// 应用状态持久化
use crate::ui::layout::SplitNode;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 应用状态（用于持久化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    /// 每个面板的项目ID
    pub pane_projects: HashMap<usize, String>,
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
            pane_projects: HashMap::new(),
            selected_columns: HashMap::new(),
            selected_task_indices: HashMap::new(),
            focused_pane: 0,
        }
    }
}

/// 获取状态文件路径
fn get_state_file_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    PathBuf::from(home).join(".kanban").join("state.json")
}

/// 从应用中提取状态
pub fn extract_state(app: &crate::app::App) -> AppState {
    use crate::ui::layout::SplitNode;

    let mut state = AppState {
        focused_pane: app.focused_pane,
        selected_columns: app.selected_column.clone(),
        selected_task_indices: app.selected_task_index.clone(),
        ..Default::default()
    };

    // 收集所有面板的项目ID
    collect_pane_projects(&app.split_tree, &mut state.pane_projects);

    state
}

/// 递归收集面板项目
fn collect_pane_projects(node: &SplitNode, pane_projects: &mut HashMap<usize, String>) {
    match node {
        SplitNode::Leaf { id, project_id } => {
            if let Some(pid) = project_id {
                pane_projects.insert(*id, pid.clone());
            }
        }
        SplitNode::Horizontal { left, right, .. } => {
            collect_pane_projects(left, pane_projects);
            collect_pane_projects(right, pane_projects);
        }
        SplitNode::Vertical { top, bottom, .. } => {
            collect_pane_projects(top, pane_projects);
            collect_pane_projects(bottom, pane_projects);
        }
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
    use crate::ui::layout::SplitNode;

    // 恢复面板项目
    for (pane_id, project_name) in state.pane_projects {
        if let Some(SplitNode::Leaf { project_id, .. }) = app.split_tree.find_pane_mut(pane_id) {
            // 检查项目是否存在，并重新加载项目数据（包括任务）
            if let Some(project) = app.projects.iter().find(|p| p.name == project_name) {
                let project_path = project.path.clone();
                let project_type = project.project_type;

                // 重新加载项目以获取最新的任务数据
                if let Ok(reloaded_project) = crate::fs::load_project_with_type(&project_path, project_type) {
                    // 更新项目列表中的数据
                    if let Some(idx) = app.projects.iter().position(|p| p.name == project_name) {
                        app.projects[idx] = reloaded_project;
                    }
                    *project_id = Some(project_name);
                }
            }
        }
    }

    // 恢复选中状态
    app.selected_column = state.selected_columns;
    app.selected_task_index = state.selected_task_indices;

    // 恢复聚焦面板（确保面板存在）
    let all_panes = app.split_tree.collect_pane_ids();
    if all_panes.contains(&state.focused_pane) {
        app.focused_pane = state.focused_pane;
    }
}
