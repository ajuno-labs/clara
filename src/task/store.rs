use crate::task::model::{Status, Task};
use directories::ProjectDirs;
use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub struct TaskStore {
    conn: Connection,
}

impl TaskStore {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path();
        let conn = Connection::open(db_path)?;

        let store = TaskStore { conn };
        store.init_tables()?;
        Ok(store)
    }

    fn get_db_path() -> PathBuf {
        ProjectDirs::from("com", "you", "clara")
            .expect("no valid home dir")
            .data_local_dir()
            .join("clara.db")
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id           INTEGER PRIMARY KEY,
                title        TEXT NOT NULL,
                created_at   INTEGER NOT NULL,
                status       TEXT NOT NULL,
                tags         TEXT NOT NULL DEFAULT '[]',
                priority     TEXT NOT NULL DEFAULT 'medium',
                due_date     INTEGER,
                updated_at   INTEGER NOT NULL,
                completed_at INTEGER,
                extras       TEXT
            )",
            [],
        )?;
        Ok(())
    }

    pub fn insert(&self, task: &Task) -> Result<()> {
        let tags_json = serde_json::to_string(&task.tags).unwrap_or_else(|_| "[]".to_string());
        let extras_json = task
            .extras
            .as_ref()
            .map(|e| serde_json::to_string(e).unwrap_or_else(|_| "null".to_string()));

        self.conn.execute(
            "INSERT INTO tasks (title, created_at, status, tags, priority, due_date, updated_at, completed_at, extras)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                task.title,
                task.created_at,
                task.status,
                tags_json,
                task.priority,
                task.due_date,
                task.updated_at,
                task.completed_at,
                extras_json
            ],
        )?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<Task>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, created_at, status, tags, priority, due_date, updated_at, completed_at, extras FROM tasks")?;
        let task_iter = stmt.query_map([], |row| {
            let tags_json: String = row.get("tags").unwrap_or_else(|_| "[]".to_string());
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            let extras_json: Option<String> = row.get("extras")?;
            let extras = extras_json.and_then(|s| serde_json::from_str(&s).ok());

            Ok(Task {
                id: row.get("id")?,
                title: row.get("title")?,
                created_at: row.get("created_at")?,
                status: row.get("status")?,
                tags,
                priority: row.get("priority")?,
                due_date: row.get("due_date")?,
                updated_at: row.get("updated_at")?,
                completed_at: row.get("completed_at")?,
                extras,
            })
        })?;

        let mut tasks = Vec::new();
        for task in task_iter {
            tasks.push(task?);
        }
        Ok(tasks)
    }

    pub fn update(&self, id: u32, status: Status) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET status = ?1 WHERE id = ?2",
            rusqlite::params![status, id],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: u32) -> Result<()> {
        self.conn
            .execute("DELETE FROM tasks WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    pub fn find_by_id(&self, id: u32) -> Result<Option<Task>> {
        let mut stmt = self.conn.prepare("SELECT id, title, created_at, status, tags, priority, due_date, updated_at, completed_at, extras FROM tasks WHERE id = ?1")?;
        let mut task_iter = stmt.query_map([id], |row| {
            let tags_json: String = row.get("tags").unwrap_or_else(|_| "[]".to_string());
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            let extras_json: Option<String> = row.get("extras")?;
            let extras = extras_json.and_then(|s| serde_json::from_str(&s).ok());

            Ok(Task {
                id: row.get("id")?,
                title: row.get("title")?,
                created_at: row.get("created_at")?,
                status: row.get("status")?,
                tags,
                priority: row.get("priority")?,
                due_date: row.get("due_date")?,
                updated_at: row.get("updated_at")?,
                completed_at: row.get("completed_at")?,
                extras,
            })
        })?;

        match task_iter.next() {
            Some(task) => Ok(Some(task?)),
            None => Ok(None),
        }
    }

    pub fn update_task(&self, task: &Task) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET title = ?1, status = ?2 WHERE id = ?3",
            rusqlite::params![task.title, task.status, task.id],
        )?;
        Ok(())
    }
}
