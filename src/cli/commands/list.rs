use crate::project::ProjectStore;
use crate::repl::command_handler::ReplContext;
use crate::task::{Task, TaskStore};
use std::collections::HashMap;

fn get_task_display_info(task: &Task, project_map: &HashMap<u32, String>, show_project: bool) -> (String, String, String) {
    let status_emoji = match task.status {
        crate::task::Status::Todo => "⏳",
        crate::task::Status::InProgress => "🔄",
        crate::task::Status::Done => "✅",
    };
    
    let project_info = if show_project {
        match task.project_id.and_then(|id| project_map.get(&id)) {
            Some(project_name) => format!(" @{}", project_name),
            None => "".to_string(),
        }
    } else {
        "".to_string()
    };
    
    let priority_info = match task.priority {
        crate::task::model::Priority::Low => " !low",
        crate::task::model::Priority::Medium => " !medium",
        crate::task::model::Priority::High => " !high",
        crate::task::model::Priority::Urgent => " !urgent",
    };
    
    (status_emoji.to_string(), project_info, priority_info.to_string())
}

pub fn list_tasks(context: &ReplContext) -> Result<(), Box<dyn std::error::Error>> {
    let store = TaskStore::new()?;
    let project_store = ProjectStore::new()?;

    // Create a map of project_id -> project_name for display
    let projects = project_store.list()?;
    let project_map: HashMap<u32, String> = projects.into_iter().map(|p| (p.id, p.name)).collect();

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
        print_task_tree(
            &store,
            &root_task,
            0,
            &project_map,
            context.current_project.is_none(),
        )?;
    }

    Ok(())
}

fn print_task_tree(
    store: &TaskStore,
    task: &Task,
    indent_level: usize,
    project_map: &HashMap<u32, String>,
    show_project: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    print_task_tree_with_prefix(store, task, indent_level, false, project_map, show_project)
}

fn print_task_tree_with_prefix(
    store: &TaskStore,
    task: &Task,
    indent_level: usize,
    is_last: bool,
    project_map: &HashMap<u32, String>,
    show_project: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let (status_emoji, project_info, priority_info) = get_task_display_info(task, project_map, show_project);
    
    // Create proper tree indentation and characters
    let (indent, tree_char) = build_tree_prefix(indent_level, is_last);

    println!(
        "{}{}{}[{}] {}{}{}",
        indent, tree_char, status_emoji, task.id, task.title, priority_info, project_info
    );

    // Recursively print children
    print_task_children(store, task, indent_level, project_map, show_project)
}

fn build_tree_prefix(indent_level: usize, is_last: bool) -> (String, &'static str) {
    if indent_level == 0 {
        return (String::new(), "");
    }
    
    let mut indent = String::new();
    for _ in 1..indent_level {
        indent.push_str("│  ");
    }
    
    let tree_char = if is_last { "└─ " } else { "├─ " };
    (indent, tree_char)
}

fn print_task_children(
    store: &TaskStore,
    task: &Task,
    indent_level: usize,
    project_map: &HashMap<u32, String>,
    show_project: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let children = store.find_children(task.id)?;
    for (i, child) in children.iter().enumerate() {
        let is_last = i == children.len() - 1;
        print_task_tree_with_prefix(
            store,
            child,
            indent_level + 1,
            is_last,
            project_map,
            show_project,
        )?;
    }
    Ok(())
}
