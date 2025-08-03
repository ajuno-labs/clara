use crate::cli::args::Commands;
use crate::task::{TaskDraft, TaskStore};
use std::env;
use std::fs;
use std::process::Command;

pub fn execute_command(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Add => {
            add_task()?;
        }
    }
    Ok(())
}

fn add_task() -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary file with TOML template
    let temp_dir = env::temp_dir();
    let temp_file = temp_dir.join("clara_task.toml");
    
    // Create and write TOML template
    let template = TaskDraft::new();
    let toml_content = template.to_toml()?;
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
    
    let task = task_draft.to_task()
        .map_err(|e| e)?;
    
    // Save task to database
    let store = TaskStore::new()?;
    store.insert(&task)?;
    
    // Clean up temp file
    let _ = fs::remove_file(&temp_file);
    
    println!("✅ Task added: '{}'", task.title);
    
    Ok(())
}