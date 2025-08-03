use crate::task::{TaskDraft, TaskStore};
use std::env;
use std::fs;
use std::process::Command;

pub fn edit_task(id: u32) -> Result<(), Box<dyn std::error::Error>> {
    let store = TaskStore::new()?;
    
    // Find the task
    let task = match store.find_by_id(id)? {
        Some(task) => task,
        None => {
            println!("❌ Task with ID {} not found.", id);
            return Ok(());
        }
    };
    
    // Create a temporary file with current task data
    let temp_dir = env::temp_dir();
    let temp_file = temp_dir.join(format!("clara_task_edit_{}.toml", id));
    
    // Convert task to draft and write TOML
    let draft = task.to_draft();
    let toml_content = draft.to_toml()?;
    fs::write(&temp_file, toml_content)?;
    
    // Get editor from environment variable, default to nano
    let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    
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
    
    let mut updated_task = task_draft.to_task()
        .map_err(|e| e)?;
    
    // Keep the original ID and created_at
    updated_task.id = task.id;
    updated_task.created_at = task.created_at;
    
    // Update task in database
    store.update_task(&updated_task)?;
    
    // Clean up temp file
    let _ = fs::remove_file(&temp_file);
    
    println!("✅ Task {} updated: '{}'", id, updated_task.title);
    
    Ok(())
}