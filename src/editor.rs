use std::env;
use std::fs;
use std::process::Command;
use tempfile::NamedTempFile;

pub fn edit_toml_content(initial_content: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Create a temporary file
    let temp_file = NamedTempFile::new()?;
    
    // Write initial content to temp file
    fs::write(temp_file.path(), initial_content)?;
    
    // Get editor from environment variable, default to vim
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    
    // Open editor
    let status = Command::new(&editor)
        .arg(temp_file.path())
        .status()?;
    
    if !status.success() {
        return Err("Editor exited with non-zero status".into());
    }
    
    // Read the edited content
    let edited_content = fs::read_to_string(temp_file.path())?;
    
    Ok(edited_content)
}