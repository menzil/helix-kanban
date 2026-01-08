use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 任务元数据（存储在 tasks.toml 中）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetadata {
    pub id: u32,
    pub order: i32,
    pub title: String,
    pub status: String,
    pub created: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl From<&Task> for TaskMetadata {
    fn from(task: &Task) -> Self {
        Self {
            id: task.id,
            order: task.order,
            title: task.title.clone(),
            status: task.status.clone(),
            created: task.created.clone(),
            priority: task.priority.clone(),
            tags: task.tags.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u32,
    pub order: i32,
    pub title: String,
    pub content: String,
    pub created: String,
    pub priority: Option<String>,
    pub status: String,
    pub tags: Vec<String>,
    #[serde(skip)]
    pub file_path: PathBuf,
}

impl Task {
    pub fn new(id: u32, title: String, status: String) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id,
            order: 0,
            title,
            content: String::new(),
            created: timestamp.to_string(),
            priority: None,
            status,
            tags: Vec::new(),
            file_path: PathBuf::new(),
        }
    }

    /// 从元数据创建Task（内容需单独加载）
    pub fn from_metadata(metadata: TaskMetadata, content: String, file_path: PathBuf) -> Self {
        Self {
            id: metadata.id,
            order: metadata.order,
            title: metadata.title,
            content,
            created: metadata.created,
            priority: metadata.priority,
            status: metadata.status,
            tags: metadata.tags,
            file_path,
        }
    }
}
