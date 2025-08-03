use chrono::{DateTime, Utc};
use std::fmt;


#[derive(Debug)]
enum Status {
    Todo,
    InProgress,
    Done,
}

struct Task {
    id: u32,
    title: String,
    status: Status,
    created_at: DateTime<Utc>,
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

fn main() {
    let task = Task {
        id: 1,
        title: String::from("Write clara repl"),
        created_at: Utc::now(),
        status: Status::Todo,
    };

    println!("Task created: {}", task);
}
