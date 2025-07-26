mod task;
mod workspace;
mod workspace_storage;
mod navigation;
mod session;
mod session_store;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "clara", version, about = "CLI productivity assistant")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Folder {
        #[command(subcommand)]
        action: FolderCmd,
    },
    Add { 
        title: String,
        #[arg(long)]
        folder: String,
        #[arg(long)]
        list: String,
    },
    Subtask {
        title: String,
        #[arg(long)]
        parent: String,
    },
    List {
        #[arg(long)]
        folder: Option<String>,
        #[arg(long)]
        list: Option<String>,
        #[arg(long)]
        tree: bool,
    },
    Track {
        #[command(subcommand)]
        action: TrackCmd,
    },
    Done { id: String },
}

#[derive(Subcommand, Debug)]
enum FolderCmd {
    Add { name: String },
    List,
    #[command(subcommand)]
    Lists(ListCmd),
}

#[derive(Subcommand, Debug)]
enum ListCmd {
    Add { 
        #[arg(long)]
        folder: String,
        name: String 
    },
    List { 
        #[arg(long)]
        folder: String 
    },
}

#[derive(Subcommand, Debug)]
enum TrackCmd {
    /// Start a session (defaults to Focus)
    Start {
        #[arg(short, long)]
        kind: Option<String>,            // focus / break / meeting
        #[arg(short, long)]
        task: Option<String>,            // task ID
    },
    /// Stop the current running session
    Stop,
    /// Show current session if any
    Current,
}

fn handle_track_start(kind: Option<String>, task: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    use crate::session::{Session, Kind};
    use chrono::Utc;
    use std::str::FromStr;
    
    // Check if there's already an active session
    if let Some(active) = session_store::load_active()? {
        return Err(format!("Session already active: {} ({}). Stop it first with 'clara track stop'", 
                          active.kind, active.id).into());
    }
    
    // Validate task ID if provided
    if let Some(ref task_id) = task {
        let ws = workspace_storage::load()?;
        if navigation::find_task(&ws, task_id).is_none() {
            return Err(format!("Task '{}' not found", task_id).into());
        }
    }
    
    // Parse kind or default to Focus
    let session_kind = match kind {
        Some(k) => Kind::from_str(&k)?,
        None => Kind::Focus,
    };
    
    // Create new session
    let session = Session {
        id: Session::generate_id(),
        kind: session_kind,
        task_id: task,
        start: Utc::now(),
        end: None,
    };
    
    // Save as active session
    session_store::save_active(&session)?;
    
    let task_info = session.task_id.as_deref().unwrap_or("no task linked");
    println!("🎯 Started {} session {} ({})", session.kind, session.id, task_info);
    
    Ok(())
}

fn handle_track_stop() -> Result<(), Box<dyn std::error::Error>> {
    use chrono::Utc;
    
    // Load active session
    let mut active = match session_store::load_active()? {
        Some(session) => session,
        None => {
            println!("⚠️  No active session to stop");
            return Ok(());
        }
    };
    
    // Complete the session
    active.end = Some(Utc::now());
    let duration = active.duration().unwrap();
    
    // Move to completed sessions
    session_store::add_session(active.clone())?;
    session_store::clear_active()?;
    
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    let task_info = active.task_id.as_deref().unwrap_or("no task linked");
    
    println!("⏹️  Stopped {} session {} after {}h {}m ({})", 
             active.kind, active.id, hours, minutes, task_info);
    
    Ok(())
}

fn handle_track_current() -> Result<(), Box<dyn std::error::Error>> {
    use chrono::Utc;
    
    match session_store::load_active()? {
        Some(session) => {
            let elapsed = Utc::now() - session.start;
            let hours = elapsed.num_hours();
            let minutes = elapsed.num_minutes() % 60;
            let task_info = session.task_id.as_deref().unwrap_or("no task linked");
            
            println!("🔄 Active {} session {} - {}h {}m elapsed ({})", 
                     session.kind, session.id, hours, minutes, task_info);
        }
        None => {
            println!("⏸️  No active session");
        }
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Folder { action } => match action {
            FolderCmd::Add { name } => {
                workspace_storage::add_folder(name)?;
                println!("📁 Folder created!");
            }
            FolderCmd::List => {
                for f in workspace_storage::list_folders()? {
                    println!("{}  {}", f.id, f.name);
                }
            }
            FolderCmd::Lists(list_cmd) => match list_cmd {
                ListCmd::Add { folder, name } => {
                    workspace_storage::add_list(folder, name)?;
                    println!("📋 List created!");
                }
                ListCmd::List { folder } => {
                    for list in workspace_storage::list_lists(folder)? {
                        println!("{}  {}", list.id, list.name);
                    }
                }
            }
        },
        Commands::Add { title, folder, list } => {
            workspace_storage::add_task(folder, list, title)?;
            println!("✅ Task saved!");
        }
        Commands::Subtask { title, parent } => {
            workspace_storage::add_subtask(parent, title)?;
            println!("✅ Subtask saved!");
        }
        Commands::List { folder, list, tree } => {
            workspace_storage::list_tasks(folder, list, tree)?;
        }
        Commands::Track { action } => match action {
            TrackCmd::Start { kind, task } => {
                handle_track_start(kind, task)?;
            }
            TrackCmd::Stop => {
                handle_track_stop()?;
            }
            TrackCmd::Current => {
                handle_track_current()?;
            }
        },
        Commands::Done { id } => {
            if workspace_storage::mark_task_done(id.clone())? {
                println!("🎉 Task #{id} marked done!");
            } else {
                println!("⚠️  No task with id {id}.");
            }
        }
    }

    Ok(())
}
