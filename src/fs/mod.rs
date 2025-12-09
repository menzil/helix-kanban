pub mod parser;
pub mod project;
pub mod task;

pub use project::{
    create_project, get_data_dir, get_projects_dir, init_data_dir, list_project_dirs,
    load_project, load_project_config, rename_project,
};
pub use task::{delete_task, get_next_task_id, load_task, move_task, save_task};

use crate::models::Project;
use anyhow::Result;

/// 加载所有项目
pub fn load_all_projects() -> Result<Vec<Project>> {
    init_data_dir()?;
    let project_dirs = list_project_dirs()?;
    let mut projects = Vec::new();

    for dir_name in project_dirs {
        match load_project(&dir_name) {
            Ok(project) => projects.push(project),
            Err(e) => eprintln!("警告: 无法加载项目 {}: {}", dir_name.display(), e),
        }
    }

    Ok(projects)
}
