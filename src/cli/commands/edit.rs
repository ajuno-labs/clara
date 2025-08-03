use crate::cli::metadata::parse_slash_metadata;
use crate::task::{TaskDraft, TaskStore};
use chrono::Local;
use crate::editor::edit_toml_content;

pub fn edit_task(id: u32, text: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    // Check if we have slash-style metadata or should use TOML editor
    if let Some(text_input) = text {
        return edit_task_with_metadata(id, &text_input);
    } else {
        return edit_task_with_editor(id);
    }
}

fn edit_task_with_metadata(id: u32, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let store = TaskStore::new()?;
    
    // Find the task
    let mut task = match store.find_by_id(id)? {
        Some(task) => task,
        None => {
            println!("❌ Task with ID {} not found.", id);
            return Ok(());
        }
    };
    
    // Parse slash metadata for edit command
    let metadata = if text.starts_with('/') {
        // Metadata-only update: /p high /tag work
        // Prepend dummy title for parsing, then ignore the title
        let dummy_input = format!("DUMMY_TITLE {}", text);
        parse_slash_metadata(&dummy_input)?
    } else {
        // Title + metadata update: New title /p high /tag work
        parse_slash_metadata(text)?
    };
    
    // Update task fields based on metadata
    // For edit, only update title if it's not the dummy and if the input doesn't start with /
    if !text.starts_with('/') && !metadata.title.is_empty() {
        task.title = metadata.title;
    }
    
    if let Some(priority) = metadata.priority {
        let mut draft = task.to_draft();
        draft.priority = priority;
        task = draft.to_task()?;
        task.id = id; // Preserve ID
    }
    
    if let Some(due_date) = metadata.due_date {
        task.due_date = Some(due_date);
    }
    
    if !metadata.tags.is_empty() {
        task.tags = metadata.tags;
    }
    
    if let Some(parent_id) = metadata.parent {
        // Validate parent exists
        match store.find_by_id(parent_id)? {
            Some(_) => task.parent_id = Some(parent_id),
            None => return Err(format!("Parent task with ID {} not found", parent_id).into()),
        }
    }
    
    // Update timestamps
    task.updated_at = Local::now().timestamp();
    
    // Update task in database
    store.update_task(&task)?;
    
    println!("✅ Task {} updated: '{}'", id, task.title);
    
    Ok(())
}

fn edit_task_with_editor(id: u32) -> Result<(), Box<dyn std::error::Error>> {
    let store = TaskStore::new()?;
    
    // Find the task
    let task = match store.find_by_id(id)? {
        Some(task) => task,
        None => {
            println!("❌ Task with ID {} not found.", id);
            return Ok(());
        }
    };
    
    // Convert task to draft and edit using shared editor utility
    let draft = task.to_draft();
    let toml_content = draft.to_toml()?;
    let edited_content = edit_toml_content(&toml_content)?;
    
    // Parse TOML and convert to task
    let task_draft = TaskDraft::from_toml(&edited_content)
        .map_err(|e| format!("Invalid TOML: {}", e))?;
    
    let mut updated_task = task_draft.to_task()
        .map_err(|e| e)?;
    
    // Keep the original ID and created_at
    updated_task.id = task.id;
    updated_task.created_at = task.created_at;
    
    // Update task in database
    store.update_task(&updated_task)?;
    
    println!("✅ Task {} updated: '{}'", id, updated_task.title);
    
    Ok(())
}