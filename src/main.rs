mod task;

use chrono::Utc;
use task::{Status, Task, TaskStore};

fn main() {
    // Create sample task
    let task = Task {
        id: 1,
        title: String::from("Write clara repl"),
        created_at: Utc::now(),
        status: Status::Todo,
    };

    println!("Task created: {}", task);

    // Initialize database and insert task
    let store = TaskStore::new().expect("Failed to create task store");
    store.insert(&task).expect("Failed to insert task");

    // List all tasks
    let tasks = store.list().expect("Failed to list tasks");
    for task in tasks {
        println!("{:?}", task);
    }
}
