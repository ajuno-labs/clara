use crate::project::model::Project;
use directories::ProjectDirs;
use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub struct ProjectStore {
    conn: Connection,
}

impl ProjectStore {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path();
        let conn = Connection::open(db_path)?;

        let store = ProjectStore { conn };
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
            "CREATE TABLE IF NOT EXISTS projects (
                id           INTEGER PRIMARY KEY,
                name         TEXT NOT NULL UNIQUE,
                description  TEXT,
                created_at   INTEGER NOT NULL,
                updated_at   INTEGER NOT NULL,
                extras       TEXT
            )",
            [],
        )?;
        Ok(())
    }

    pub fn insert(&self, project: &Project) -> Result<()> {
        let extras_json = project
            .extras
            .as_ref()
            .map(|e| serde_json::to_string(e).unwrap_or_else(|_| "null".to_string()));

        self.conn.execute(
            "INSERT INTO projects (name, description, created_at, updated_at, extras)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                project.name,
                project.description,
                project.created_at,
                project.updated_at,
                extras_json
            ],
        )?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<Project>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, description, created_at, updated_at, extras FROM projects ORDER BY name")?;
        let project_iter = stmt.query_map([], |row| {
            let extras_json: Option<String> = row.get("extras")?;
            let extras = extras_json.and_then(|s| serde_json::from_str(&s).ok());

            Ok(Project {
                id: row.get("id")?,
                name: row.get("name")?,
                description: row.get("description")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
                extras,
            })
        })?;

        let mut projects = Vec::new();
        for project in project_iter {
            projects.push(project?);
        }
        Ok(projects)
    }

    pub fn find_by_id(&self, id: u32) -> Result<Option<Project>> {
        let mut stmt = self.conn.prepare("SELECT id, name, description, created_at, updated_at, extras FROM projects WHERE id = ?1")?;
        let mut project_iter = stmt.query_map([id], |row| {
            let extras_json: Option<String> = row.get("extras")?;
            let extras = extras_json.and_then(|s| serde_json::from_str(&s).ok());

            Ok(Project {
                id: row.get("id")?,
                name: row.get("name")?,
                description: row.get("description")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
                extras,
            })
        })?;

        match project_iter.next() {
            Some(project) => Ok(Some(project?)),
            None => Ok(None),
        }
    }

    pub fn find_by_name(&self, name: &str) -> Result<Option<Project>> {
        let mut stmt = self.conn.prepare("SELECT id, name, description, created_at, updated_at, extras FROM projects WHERE name = ?1")?;
        let mut project_iter = stmt.query_map([name], |row| {
            let extras_json: Option<String> = row.get("extras")?;
            let extras = extras_json.and_then(|s| serde_json::from_str(&s).ok());

            Ok(Project {
                id: row.get("id")?,
                name: row.get("name")?,
                description: row.get("description")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
                extras,
            })
        })?;

        match project_iter.next() {
            Some(project) => Ok(Some(project?)),
            None => Ok(None),
        }
    }

    pub fn update(&self, project: &Project) -> Result<()> {
        let extras_json = project
            .extras
            .as_ref()
            .map(|e| serde_json::to_string(e).unwrap_or_else(|_| "null".to_string()));

        self.conn.execute(
            "UPDATE projects SET name = ?1, description = ?2, updated_at = ?3, extras = ?4 WHERE id = ?5",
            rusqlite::params![
                project.name,
                project.description,
                project.updated_at,
                extras_json,
                project.id
            ],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: u32) -> Result<()> {
        self.conn
            .execute("DELETE FROM projects WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }
}