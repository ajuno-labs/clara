use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub done: bool,
    #[serde(default)]
    pub subtasks: Vec<Task>,
}
