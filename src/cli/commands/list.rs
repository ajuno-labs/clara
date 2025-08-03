use crate::task::TaskStore;

pub fn list_tasks() -> Result<(), Box<dyn std::error::Error>> {
    let store = TaskStore::new()?;
    let tasks = store.list()?;
    
    if tasks.is_empty() {
        println!("No tasks found.");
        return Ok(());
    }
    
    println!("📋 Tasks:");
    for task in tasks {
        let status_emoji = match task.status {
            crate::task::Status::Todo => "⏳",
            crate::task::Status::InProgress => "🔄",
            crate::task::Status::Done => "✅",
        };
        println!("  {} [{}] {}", status_emoji, task.id, task.title);
    }
    
    Ok(())
}