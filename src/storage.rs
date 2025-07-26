use crate::task::Task;
use chrono::Utc;
use std::{fs, io};
use std::path::PathBuf;

fn db_path() -> PathBuf {
    let base = dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    base.join("clara").join("tasks.json")
}

pub fn load() -> io::Result<Vec<Task>> {
    let path = db_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(path)?;
    let tasks = serde_json::from_str(&data)?;
    Ok(tasks)
}

/// Save tasks, creating parent folder if needed.
pub fn save(mut tasks: Vec<Task>) -> io::Result<()> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    // give every new task a sequential id
    for (idx, t) in tasks.iter_mut().enumerate() {
        if t.id == 0 {
            t.id = (idx as u32) + 1;
        }
    }
    let data = serde_json::to_string_pretty(&tasks)?;
    fs::write(path, data)?;
    Ok(())
}

pub fn add(title: String) -> io::Result<()> {
    let mut tasks = load()?;
    tasks.push(Task {
        id: 0,
        title,
        created_at: Utc::now(),
        done: false,
    });
    save(tasks)
}

pub fn mark_done(id: u32) -> std::io::Result<bool> {
    let mut tasks = load()?;
    let mut found = false;
    for t in &mut tasks {
        if t.id == id {
            t.done = true;
            found = true;
            break;
        }
    }
    if found {
        save(tasks)?;
    }
    Ok(found)   // returns true if we actually changed something
}
