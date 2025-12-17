pub mod parser;
pub mod project;
pub mod task;

pub use project::{
    create_project, create_local_project, delete_project, get_data_dir, get_projects_dir, get_local_kanban_dir,
    init_data_dir, list_local_project_dirs, list_project_dirs, load_project,
    load_project_config, load_project_with_type, rename_project, ensure_global_ai_config,
};
pub use task::{delete_task, get_next_task_id, load_task, move_task, save_task};

use crate::models::{Project, ProjectType};
use anyhow::Result;

/// 加载所有项目（全局 + 本地），过滤隐藏的项目
pub fn load_all_projects() -> Result<Vec<Project>> {
    // 加载配置以获取隐藏项目列表
    let config = crate::config::load_config().unwrap_or_default();

    init_data_dir()?;
    let mut projects = Vec::new();

    // 加载全局项目 (~/.kanban/projects)，过滤隐藏的
    let global_project_dirs = list_project_dirs()?;
    for dir_name in global_project_dirs {
        match load_project_with_type(&dir_name, ProjectType::Global) {
            Ok(project) => {
                // 检查项目是否被隐藏
                if !crate::config::is_project_hidden(&config, &project.name) {
                    projects.push(project);
                }
            },
            Err(e) => eprintln!("警告: 无法加载全局项目 {}: {}", dir_name.display(), e),
        }
    }

    // 加载本地项目
    // 当前目录的本地项目：永远显示（即使被软删除）
    // 其他目录的本地项目：遵循软删除规则
    let local_project_dirs = list_local_project_dirs()?;
    let current_local_dir = get_local_kanban_dir();

    for dir_name in local_project_dirs {
        match load_project_with_type(&dir_name, ProjectType::Local) {
            Ok(project) => {
                // 判断是否是当前目录的项目
                let is_current_dir = dir_name == current_local_dir;

                // 当前目录的项目永远显示，其他项目检查软删除状态
                if is_current_dir || !crate::config::is_project_hidden(&config, &project.name) {
                    projects.push(project);
                }
            },
            Err(e) => eprintln!("警告: 无法加载本地项目 {}: {}", dir_name.display(), e),
        }
    }

    Ok(projects)
}
