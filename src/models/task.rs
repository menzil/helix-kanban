use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
}
