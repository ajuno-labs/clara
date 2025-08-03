use crate::cli::metadata::parse_slash_metadata;
use crate::repl::command_handler::ReplContext;
use crate::task::{TaskDraft, TaskStore};
use std::env;
use std::fs;
use std::process::Command;

pub fn add_task(parent_id: Option<u32>, text: Option<String>, context: &ReplContext) -> Result<(), Box<dyn std::error::Error>> {
    // Check if we have slash-style metadata or should use TOML editor
    if let Some(text_input) = text {
        return add_task_with_metadata(parent_id, &text_input, context);
    } else {
        return add_task_with_editor(parent_id, context);
    }
}

fn add_task_with_metadata(parent_id: Option<u32>, text: &str, context: &ReplContext) -> Result<(), Box<dyn std::error::Error>> {
    // Parse slash metadata
    let metadata = parse_slash_metadata(text)?;
    
    // Use parent from metadata if provided, otherwise use parent_id parameter
    let final_parent_id = metadata.parent.or(parent_id);
    
    // Validate parent exists if provided
    if let Some(parent_id) = final_parent_id {
        let store = TaskStore::new()?;
        match store.find_by_id(parent_id)? {
            Some(_) => {},
            None => return Err(format!("Parent task with ID {} not found", parent_id).into()),
        }
    }
    
    // Create TaskDraft from metadata
    let mut task_draft = TaskDraft::new();
    task_draft.title = metadata.title;
    task_draft.parent_id = final_parent_id;
    task_draft.project_id = context.current_project.as_ref().map(|p| p.id);
    
    if let Some(priority) = metadata.priority {
        task_draft.priority = priority;
    }
    
    if let Some(due_date) = metadata.due_date {
        task_draft.due_date = Some(due_date);
    }
    
    task_draft.tags = metadata.tags;
    
    // Convert to task and save
    let task = task_draft.to_task()?;
    let store = TaskStore::new()?;
    store.insert(&task)?;
    
    let project_info = context.current_project.as_ref()
        .map(|p| format!(" in project '{}'", p.name))
        .unwrap_or_default();
    
    match final_parent_id {
        Some(parent_id) => println!("✅ Subtask added to parent #{}{}: '{}'", parent_id, project_info, task.title),
        None => println!("✅ Task added{}: '{}'", project_info, task.title),
    }
    
    Ok(())
}

fn add_task_with_editor(parent_id: Option<u32>, context: &ReplContext) -> Result<(), Box<dyn std::error::Error>> {
    // Validate parent exists if provided
    if let Some(parent_id) = parent_id {
        let store = TaskStore::new()?;
        match store.find_by_id(parent_id)? {
            Some(_) => {},
            None => return Err(format!("Parent task with ID {} not found", parent_id).into()),
        }
    }

    // Create a temporary file with TOML template
    let temp_dir = env::temp_dir();
    let temp_file = temp_dir.join("clara_task.toml");
    
    // Create and write TOML template
    let mut template = TaskDraft::new();
    template.parent_id = parent_id;
    template.project_id = context.current_project.as_ref().map(|p| p.id);
    let toml_content = template.to_toml()?;
    fs::write(&temp_file, toml_content)?;
    
    // Get editor from environment variable, default to nano
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    
    // Open editor
    let status = Command::new(&editor)
        .arg(&temp_file)
        .status()?;
    
    if !status.success() {
        return Err("Editor exited with non-zero status".into());
    }
    
    // Read the edited content
    let edited_content = fs::read_to_string(&temp_file)?;
    
    // Parse TOML and convert to task
    let task_draft = TaskDraft::from_toml(&edited_content)
        .map_err(|e| format!("Invalid TOML: {}", e))?;
    
    let task = task_draft.to_task()
        .map_err(|e| e)?;
    
    // Save task to database
    let store = TaskStore::new()?;
    store.insert(&task)?;
    
    // Clean up temp file
    let _ = fs::remove_file(&temp_file);
    
    let project_info = context.current_project.as_ref()
        .map(|p| format!(" in project '{}'", p.name))
        .unwrap_or_default();
    
    match parent_id {
        Some(parent_id) => println!("✅ Subtask added to parent #{}{}: '{}'", parent_id, project_info, task.title),
        None => println!("✅ Task added{}: '{}'", project_info, task.title),
    }
    
    Ok(())
}
