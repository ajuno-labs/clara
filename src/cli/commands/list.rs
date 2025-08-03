use crate::task::{Task, TaskStore};

pub fn list_tasks() -> Result<(), Box<dyn std::error::Error>> {
    let store = TaskStore::new()?;
    let all_tasks = store.list()?;
    
    if all_tasks.is_empty() {
        println!("No tasks found.");
        return Ok(());
    }
    
    println!("📋 Tasks:");
    
    // Get root tasks (tasks with no parent)
    let root_tasks = store.find_root_tasks()?;
    
    for root_task in root_tasks {
        print_task_tree(&store, &root_task, 0)?;
    }
    
    Ok(())
}

fn print_task_tree(store: &TaskStore, task: &Task, indent_level: usize) -> Result<(), Box<dyn std::error::Error>> {
    let status_emoji = match task.status {
        crate::task::Status::Todo => "⏳",
        crate::task::Status::InProgress => "🔄",
        crate::task::Status::Done => "✅",
    };
    
    // Create indentation and tree characters
    let indent = "  ".repeat(indent_level);
    let tree_prefix = if indent_level > 0 { "├─ " } else { "" };
    
    println!("{}{}{}[{}] {}", indent, tree_prefix, status_emoji, task.id, task.title);
    
    // Recursively print children
    let children = store.find_children(task.id)?;
    for (i, child) in children.iter().enumerate() {
        let is_last = i == children.len() - 1;
        print_task_tree_with_prefix(store, child, indent_level + 1, is_last)?;
    }
    
    Ok(())
}

fn print_task_tree_with_prefix(store: &TaskStore, task: &Task, indent_level: usize, is_last: bool) -> Result<(), Box<dyn std::error::Error>> {
    let status_emoji = match task.status {
        crate::task::Status::Todo => "⏳",
        crate::task::Status::InProgress => "🔄",
        crate::task::Status::Done => "✅",
    };
    
    // Create proper tree indentation
    let mut indent = String::new();
    for _ in 1..indent_level {
        indent.push_str("│  ");
    }
    
    let tree_char = if is_last { "└─ " } else { "├─ " };
    
    println!("{}{}{}[{}] {}", indent, tree_char, status_emoji, task.id, task.title);
    
    // Recursively print children
    let children = store.find_children(task.id)?;
    for (i, child) in children.iter().enumerate() {
        let child_is_last = i == children.len() - 1;
        print_task_tree_with_prefix(store, child, indent_level + 1, child_is_last)?;
    }
    
    Ok(())
}