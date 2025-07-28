use crate::task::Task;
use crate::workspace::{Folder, List, Workspace};
use chrono::Utc;
use serde_json;
use std::{fs, io, path::PathBuf, process::Command};

fn db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("clara")
        .join("workspace.json")
}

/// Load or create empty workspace.
pub fn load() -> io::Result<Workspace> {
    let path = db_path();
    if !path.exists() {
        return Ok(Workspace { folders: vec![] });
    }
    let data = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
}

/// Save whole workspace atomically.
pub fn save(ws: &Workspace) -> io::Result<()> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(ws)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, data)?;
    Ok(())
}

/* ----------  FOLDER HELPERS  ---------- */

/// Return next folder id: F{n+1}
fn next_folder_id(folders: &[Folder]) -> String {
    format!("F{}", folders.len() + 1)
}

pub fn add_folder(name: String) -> io::Result<()> {
    let mut ws = load()?;
    let id = next_folder_id(&ws.folders);
    ws.folders.push(Folder {
        id,
        name,
        lists: vec![List {
            id: "default".into(), // placeholder list
            name: "General".into(),
            tasks: vec![],
        }],
    });
    save(&ws)
}

pub fn list_folders() -> io::Result<Vec<Folder>> {
    Ok(load()?.folders)
}

/* ----------  LIST HELPERS  ---------- */

/// Return next list id: F{folder_num}-L{n+1}
fn next_list_id(folder_id: &str, lists: &[List]) -> String {
    format!("{}-L{}", folder_id, lists.len() + 1)
}

pub fn add_list(folder_id: String, name: String) -> io::Result<()> {
    let mut ws = load()?;

    // Find the folder
    let folder = ws
        .folders
        .iter_mut()
        .find(|f| f.id == folder_id)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Folder {} not found", folder_id),
            )
        })?;

    let id = next_list_id(&folder_id, &folder.lists);
    folder.lists.push(List {
        id,
        name,
        tasks: vec![],
    });

    save(&ws)
}

pub fn list_lists(folder_id: String) -> io::Result<Vec<List>> {
    let ws = load()?;
    let folder = ws
        .folders
        .iter()
        .find(|f| f.id == folder_id)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Folder {} not found", folder_id),
            )
        })?;

    Ok(folder.lists.clone())
}

/* ----------  TASK HELPERS  ---------- */

/// Generate next task ID in dot-path format
fn next_task_id(list_id: &str, list: &List) -> String {
    let next = list.tasks.len() + 1;
    format!("{}-T{}", list_id, next)
}

/// Generate next subtask ID in dot-path format
fn next_subtask_id(parent_id: &str, parent: &Task) -> String {
    let next = parent.subtasks.len() + 1;
    format!("{}.{}", parent_id, next)
}

pub fn add_task(folder_id: String, list_id: String, title: String) -> io::Result<()> {
    let mut ws = load()?;

    // Find the folder
    let folder = ws
        .folders
        .iter_mut()
        .find(|f| f.id == folder_id)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("Folder {} not found", folder_id),
            )
        })?;

    // Find the list
    let list = folder
        .lists
        .iter_mut()
        .find(|l| l.id == list_id)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("List {} not found", list_id),
            )
        })?;

    let id = next_task_id(&list_id, list);

    list.tasks.push(Task {
        id,
        title,
        description: None,
        created_at: Utc::now(),
        done: false,
        subtasks: vec![],
    });

    save(&ws)
}

/// Find and add subtask to parent task by ID
pub fn add_subtask(parent_id: String, title: String) -> io::Result<()> {
    let mut ws = load()?;

    // Find the parent task using navigation helper
    if let Some(parent_task) = crate::navigation::find_task_mut(&mut ws, &parent_id) {
        let subtask_id = next_subtask_id(&parent_id, parent_task);
        parent_task.subtasks.push(Task {
            id: subtask_id,
            title,
            description: None,
            created_at: Utc::now(),
            done: false,
            subtasks: vec![],
        });
        save(&ws)?;
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Parent task {} not found", parent_id),
        ))
    }
}

/// Mark task as done by ID (searches recursively through workspace)
pub fn mark_task_done(task_id: String) -> io::Result<bool> {
    let mut ws = load()?;

    if let Some(task) = crate::navigation::find_task_mut(&mut ws, &task_id) {
        task.done = true;
        save(&ws)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// List tasks with optional filtering and tree view
pub fn list_tasks(
    folder_filter: Option<String>,
    list_filter: Option<String>,
    tree_view: bool,
) -> io::Result<()> {
    let ws = load()?;
    let mut found_any = false;

    for folder in &ws.folders {
        // Filter by folder if specified
        if let Some(ref filter) = folder_filter {
            if folder.id != *filter {
                continue;
            }
        }

        for list in &folder.lists {
            // Filter by list if specified
            if let Some(ref filter) = list_filter {
                if list.id != *filter {
                    continue;
                }
            }

            if !list.tasks.is_empty() {
                if !found_any {
                    found_any = true;
                }

                // Show folder/list header
                println!("\n📁 {} > 📋 {}", folder.name, list.name);
                println!("---");

                for task in &list.tasks {
                    if tree_view {
                        print_task_tree(task, 0);
                    } else {
                        print_task_flat(task);
                    }
                }
            }
        }
    }

    if !found_any {
        println!("No tasks found.");
    }

    Ok(())
}

/// Print task in flat format
fn print_task_flat(task: &Task) {
    println!(
        "[{}] {}  {}  (created {})",
        if task.done { "x" } else { " " },
        task.id,
        task.title,
        task.created_at.format("%Y‑%m‑%d %H:%M")
    );
}

/// Print task in tree format with indentation
fn print_task_tree(task: &Task, depth: usize) {
    let indent = "  ".repeat(depth);
    let icon = if task.done { "✅" } else { "🔲" };

    println!(
        "{}{} [{}] {}  (created {})",
        indent,
        icon,
        task.id,
        task.title,
        task.created_at.format("%Y‑%m‑%d %H:%M")
    );

    // Print subtasks recursively
    for subtask in &task.subtasks {
        print_task_tree(subtask, depth + 1);
    }
}

/// Delete an item (folder, list, or task) by ID
pub fn delete_item(item_id: String) -> io::Result<bool> {
    let mut ws = load()?;
    let mut found = false;

    // Try to delete as folder first
    if let Some(pos) = ws.folders.iter().position(|f| f.id == item_id) {
        ws.folders.remove(pos);
        found = true;
    } else {
        // Try to delete as list
        for folder in &mut ws.folders {
            if let Some(pos) = folder.lists.iter().position(|l| l.id == item_id) {
                folder.lists.remove(pos);
                found = true;
                break;
            }
        }

        // Try to delete as task if not found as folder or list
        if !found {
            if delete_task_recursive(&mut ws, &item_id) {
                found = true;
            }
        }
    }

    if found {
        save(&ws)?;
    }

    Ok(found)
}

/// Recursively search and delete a task by ID
fn delete_task_recursive(ws: &mut Workspace, task_id: &str) -> bool {
    for folder in &mut ws.folders {
        for list in &mut folder.lists {
            if let Some(pos) = list.tasks.iter().position(|t| t.id == task_id) {
                list.tasks.remove(pos);
                return true;
            }
            
            // Check subtasks recursively
            for task in &mut list.tasks {
                if delete_subtask_recursive(task, task_id) {
                    return true;
                }
            }
        }
    }
    false
}

/// Recursively search and delete a subtask by ID
fn delete_subtask_recursive(task: &mut Task, task_id: &str) -> bool {
    if let Some(pos) = task.subtasks.iter().position(|t| t.id == task_id) {
        task.subtasks.remove(pos);
        return true;
    }
    
    for subtask in &mut task.subtasks {
        if delete_subtask_recursive(subtask, task_id) {
            return true;
        }
    }
    
    false
}

/// Update an item using an external editor (vim by default)
pub fn update_item_with_editor(item_id: String) -> io::Result<bool> {
    let mut ws = load()?;
    
    // Find the item and create its JSON representation
    let item_json = if let Some(folder) = ws.folders.iter().find(|f| f.id == item_id) {
        serde_json::to_string_pretty(folder).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
    } else if let Some((folder_idx, list_idx)) = find_list_by_id(&ws, &item_id) {
        let list = &ws.folders[folder_idx].lists[list_idx];
        serde_json::to_string_pretty(list).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
    } else if let Some(task) = crate::navigation::find_task(&ws, &item_id) {
        serde_json::to_string_pretty(task).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
    } else {
        return Ok(false);
    };

    // Create temporary file
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("clara_edit_{}.json", item_id));
    fs::write(&temp_file, &item_json)?;

    // Get editor from environment or use vim as default
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    
    // Launch editor
    let status = Command::new(&editor)
        .arg(&temp_file)
        .status()?;

    if !status.success() {
        fs::remove_file(&temp_file).ok(); // Clean up temp file
        return Err(io::Error::new(io::ErrorKind::Other, "Editor exited with non-zero status"));
    }

    // Read the modified content
    let edited_content = fs::read_to_string(&temp_file)?;
    
    // Clean up temp file
    fs::remove_file(&temp_file).ok();

    // Parse the edited JSON and update the workspace
    if let Some(folder) = ws.folders.iter_mut().find(|f| f.id == item_id) {
        let updated_folder: Folder = serde_json::from_str(&edited_content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid JSON: {}", e)))?;
        *folder = updated_folder;
    } else if let Some((folder_idx, list_idx)) = find_list_by_id(&ws, &item_id) {
        let updated_list: List = serde_json::from_str(&edited_content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid JSON: {}", e)))?;
        ws.folders[folder_idx].lists[list_idx] = updated_list;
    } else if let Some(task) = crate::navigation::find_task_mut(&mut ws, &item_id) {
        let updated_task: Task = serde_json::from_str(&edited_content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Invalid JSON: {}", e)))?;
        *task = updated_task;
    } else {
        return Ok(false);
    }

    save(&ws)?;
    Ok(true)
}

/// Helper function to find a list by ID and return its indices
fn find_list_by_id(ws: &Workspace, list_id: &str) -> Option<(usize, usize)> {
    for (folder_idx, folder) in ws.folders.iter().enumerate() {
        for (list_idx, list) in folder.lists.iter().enumerate() {
            if list.id == list_id {
                return Some((folder_idx, list_idx));
            }
        }
    }
    None
}
