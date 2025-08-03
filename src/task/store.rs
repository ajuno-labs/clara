use crate::task::model::{Status, Task};
use chrono::{DateTime, Utc};
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
                id          INTEGER PRIMARY KEY,
                title       TEXT NOT NULL,
                created_at  TEXT NOT NULL,
                status      TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn insert(&self, task: &Task) -> Result<()> {
        self.conn.execute(
            "INSERT INTO tasks (title, created_at, status)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![
                task.title,
                task.created_at.to_rfc3339(),
                task.status.to_string()
            ],
        )?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<Task>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, created_at, status FROM tasks")?;
        let task_iter = stmt.query_map([], |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: DateTime::parse_from_rfc3339(row.get::<_, String>(2)?.as_str())
                    .unwrap()
                    .with_timezone(&Utc),
                status: Status::from_string(&row.get::<_, String>(3)?),
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
            rusqlite::params![status.to_string(), id],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: u32) -> Result<()> {
        self.conn
            .execute("DELETE FROM tasks WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    pub fn find_by_id(&self, id: u32) -> Result<Option<Task>> {
        let mut stmt = self.conn.prepare("SELECT id, title, created_at, status FROM tasks WHERE id = ?1")?;
        let mut task_iter = stmt.query_map([id], |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: DateTime::parse_from_rfc3339(row.get::<_, String>(2)?.as_str())
                    .unwrap()
                    .with_timezone(&Utc),
                status: Status::from_string(&row.get::<_, String>(3)?),
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
            rusqlite::params![task.title, task.status.to_string(), task.id],
        )?;
        Ok(())
    }
}
