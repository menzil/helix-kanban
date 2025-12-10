use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::status::Status;
use super::task::Task;

/// 项目类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    /// 全局项目 (~/.kanban/projects)
    Global,
    /// 本地项目 (./kanban)
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub created: String,
    pub statuses: StatusesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusesConfig {
    pub order: Vec<String>,
    #[serde(flatten)]
    pub statuses: HashMap<String, StatusConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusConfig {
    pub display: String,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    pub statuses: Vec<Status>,
    pub tasks: Vec<Task>,
    pub project_type: ProjectType,
}

impl Project {
    pub fn new(name: String, path: PathBuf, project_type: ProjectType) -> Self {
        Self {
            name,
            path,
            statuses: Vec::new(),
            tasks: Vec::new(),
            project_type,
        }
    }

    pub fn get_tasks_by_status(&self, status: &str) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| t.status == status)
            .collect()
    }
}
