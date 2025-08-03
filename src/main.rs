use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use rusqlite;
use std::fmt;

fn get_db_path() -> std::path::PathBuf {
    ProjectDirs::from("com", "you", "clara")
        .expect("no valid home dir")
        .data_local_dir()
        .join("clara.db")
}

#[derive(Debug)]
enum Status {
    Todo,
    InProgress,
    Done,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Todo => write!(f, "todo"),
            Status::InProgress => write!(f, "doing"),
            Status::Done => write!(f, "done"),
        }
    }
}

#[derive(Debug)]
struct Task {
    id: u32,
    title: String,
    status: Status,
    created_at: DateTime<Utc>,
}

impl fmt::Display for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Task ID: {}, Title: {}, Status: {:?}, Created At: {}",
            self.id, self.title, self.status, self.created_at
        )
    }
}

fn main() {
    // Create sample task
    let task = Task {
        id: 1,
        title: String::from("Write clara repl"),
        created_at: Utc::now(),
        status: Status::Todo,
    };

    println!("Task created: {}", task);

    // Initialize database and insert task
    let conn = rusqlite::Connection::open(get_db_path()).expect("Failed to open database");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
        id          INTEGER PRIMARY KEY,
        title       TEXT NOT NULL,
        created_at  TEXT NOT NULL,
        status      TEXT NOT NULL
    )",
        [],
    )
    .expect("Failed to create table");
    conn.execute(
        "INSERT INTO tasks (title, created_at, status)
     VALUES (?1, ?2, ?3)",
        rusqlite::params![
            task.title,
            task.created_at.to_rfc3339(),
            task.status.to_string()
        ],
    )
    .expect("Failed to insert task");
    let mut stmt = conn
        .prepare("SELECT id, title, created_at, status FROM tasks")
        .expect("Failed to prepare statement");
    let task_iter = stmt
        .query_map([], |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: DateTime::parse_from_rfc3339(row.get::<_, String>(2)?.as_str())
                    .unwrap()
                    .with_timezone(&Utc),
                status: match row.get::<_, String>(3)?.as_str() {
                    "todo" => Status::Todo,
                    "doing" => Status::InProgress,
                    "done" => Status::Done,
                    _ => Status::Todo,
                },
            })
        })
        .expect("Failed to query tasks");

    for task in task_iter {
        println!("{:?}", task.expect("Failed to read task"));
    }
}
