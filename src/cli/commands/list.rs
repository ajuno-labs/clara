use crate::project::ProjectStore;
use crate::repl::command_handler::ReplContext;
use crate::task::{Task, TaskStore};
use std::collections::HashMap;

pub fn list_tasks(context: &ReplContext) -> Result<(), Box<dyn std::error::Error>> {
    let store = TaskStore::new()?;
    let project_store = ProjectStore::new()?;
    
    // Create a map of project_id -> project_name for display
    let projects = project_store.list()?;
    let project_map: HashMap<u32, String> = projects
        .into_iter()
        .map(|p| (p.id, p.name))
        .collect();
    
    let root_tasks = match &context.current_project {
        Some(project) => {
            println!("📋 Tasks in project '{}':", project.name);
            store.find_root_tasks_by_project(project.id)?
        }
        None => {
            println!("📋 All tasks:");
            store.find_root_tasks()?
        }
    };
    
    if root_tasks.is_empty() {
        println!("No tasks found.");
        return Ok(());
    }
    
    for root_task in root_tasks {
        print_task_tree(&store, &root_task, 0, &project_map, context.current_project.is_none())?;
    }
    
    Ok(())
}

fn print_task_tree(store: &TaskStore, task: &Task, indent_level: usize, project_map: &HashMap<u32, String>, show_project: bool) -> Result<(), Box<dyn std::error::Error>> {
    let status_emoji = match task.status {
        crate::task::Status::Todo => "⏳",
        crate::task::Status::InProgress => "🔄",
        crate::task::Status::Done => "✅",
    };
    
    // Create indentation and tree characters
    let indent = "  ".repeat(indent_level);
    let tree_prefix = if indent_level > 0 { "├─ " } else { "" };
    
    // Add project info if we're showing all tasks
    let project_info = if show_project {
        match task.project_id.and_then(|id| project_map.get(&id)) {
            Some(project_name) => format!(" @{}", project_name),
            None => "".to_string(),
        }
    } else {
        "".to_string()
    };
    
    println!("{}{}{}[{}] {}{}", indent, tree_prefix, status_emoji, task.id, task.title, project_info);
    
    // Recursively print children
    let children = store.find_children(task.id)?;
    for (i, child) in children.iter().enumerate() {
        let is_last = i == children.len() - 1;
        print_task_tree_with_prefix(store, child, indent_level + 1, is_last, project_map, show_project)?;
    }
    
    Ok(())
}

fn print_task_tree_with_prefix(store: &TaskStore, task: &Task, indent_level: usize, is_last: bool, project_map: &HashMap<u32, String>, show_project: bool) -> Result<(), Box<dyn std::error::Error>> {
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
    
    // Add project info if we're showing all tasks
    let project_info = if show_project {
        match task.project_id.and_then(|id| project_map.get(&id)) {
            Some(project_name) => format!(" @{}", project_name),
            None => "".to_string(),
        }
    } else {
        "".to_string()
    };
    
    println!("{}{}{}[{}] {}{}", indent, tree_char, status_emoji, task.id, task.title, project_info);
    
    // Recursively print children
    let children = store.find_children(task.id)?;
    for (i, child) in children.iter().enumerate() {
        let child_is_last = i == children.len() - 1;
        print_task_tree_with_prefix(store, child, indent_level + 1, child_is_last, project_map, show_project)?;
    }
    
    Ok(())
}