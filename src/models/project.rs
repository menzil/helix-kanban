use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::status::Status;
use super::task::Task;

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
}

impl Project {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            statuses: Vec::new(),
            tasks: Vec::new(),
        }
    }

    pub fn get_tasks_by_status(&self, status: &str) -> Vec<&Task> {
        self.tasks
            .iter()
            .filter(|t| t.status == status)
            .collect()
    }
}
