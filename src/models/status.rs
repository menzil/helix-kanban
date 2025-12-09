use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    pub name: String,
    pub display: String,
}

impl Status {
    pub fn new(name: String, display: String) -> Self {
        Self { name, display }
    }
}
