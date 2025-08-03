use chrono::{DateTime, Utc};
use std::fmt;

#[derive(Debug)]
pub enum Status {
    Todo,
    InProgress,
    Done,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Todo => write!(f, "todo"),
            Status::InProgress => write!(f, "doing"),
            Status::Done => write!(f, "done"),
        }
    }
}

impl Status {
    pub fn from_string(s: &str) -> Self {
        match s {
            "todo" => Status::Todo,
            "doing" => Status::InProgress,
            "done" => Status::Done,
            _ => Status::Todo,
        }
    }
}

#[derive(Debug)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub status: Status,
    pub created_at: DateTime<Utc>,
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Task ID: {}, Title: {}, Status: {:?}, Created At: {}",
            self.id, self.title, self.status, self.created_at
        )
    }
}