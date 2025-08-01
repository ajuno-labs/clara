mod alert;
mod completion;
mod navigation;
mod path_parser;
mod pomodoro_config;
mod session;
mod session_store;
mod task;
mod workspace;
mod workspace_storage;

use path_parser::{parse_hierarchical_path, resolve_path_to_ids};
use rustyline::Editor;
use rustyline::error::ReadlineError;

#[derive(Debug)]
enum Commands {
    // New noun + verb structure
    Folder { action: FolderAction },
    List { action: ListAction },
    Task { action: TaskAction },
    Track { action: TrackCmd },
}

#[derive(Debug)]
enum FolderAction {
    Create { name: String },
    List,
    Delete { path: String },
    Update { path: String },
}

#[derive(Debug)]
enum ListAction {
    Create { name: String, folder_path: String },
    List { folder_path: String },
    Delete { path: String },
    Update { path: String },
}

#[derive(Debug)]
enum TaskAction {
    Create { title: String, list_path: String },
    List { list_path: String, tree: bool },
    Delete { path: String },
    Update { path: String },
    Done { path: String },
}

#[derive(Debug)]
enum TrackCmd {
    Start {
        kind: Option<String>,
        task: Option<String>,
        duration: Option<i64>,
        mode: Option<String>,
    },
    Stop,
    Current,
    Extend {
        minutes: Option<i64>,
    },
}

fn run_interactive_timer(
    session: crate::session::Session,
) -> Result<(), Box<dyn std::error::Error>> {
    use chrono::Utc;
    use std::io::{self, Write, stdin};
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration as StdDuration;

    println!("🎯 Timer Display - Press 'q' + Enter to return to REPL while keeping session active");

    // Create channel for input handling
    let (tx, rx) = mpsc::channel();

    // Spawn thread to handle user input
    thread::spawn(move || {
        let mut input = String::new();
        loop {
            input.clear();
            if stdin().read_line(&mut input).is_ok() {
                if input.trim() == "q" {
                    let _ = tx.send(true);
                    break;
                }
            }
        }
    });

    loop {
        let now = Utc::now();
        let elapsed = now - session.start;

        // Clear line and move cursor to beginning
        print!("\r\x1b[K");

        if let Some(target_end) = session.target_end {
            if now > target_end {
                // Session has ended
                let overtime = now - target_end;
                let overtime_mins = overtime.num_minutes();
                let overtime_secs = overtime.num_seconds() % 60;
                print!(
                    "🔴 SESSION ENDED - OVERTIME: +{}m{}s (Press 'q' + Enter to exit)",
                    overtime_mins, overtime_secs
                );
            } else {
                // Show countdown
                let remaining = target_end - now;
                let remaining_mins = remaining.num_minutes();
                let remaining_secs = remaining.num_seconds() % 60;
                print!(
                    "⏳ {} Session - {}m{}s remaining (Press 'q' + Enter to exit)",
                    session.kind, remaining_mins, remaining_secs
                );
            }
        } else {
            // Show elapsed time for sessions without target end
            let elapsed_mins = elapsed.num_minutes();
            let elapsed_secs = elapsed.num_seconds() % 60;
            print!(
                "⏱️  {} Session - {}m{}s elapsed (Press 'q' + Enter to exit)",
                session.kind, elapsed_mins, elapsed_secs
            );
        }

        io::stdout().flush()?;

        // Check for user input (non-blocking)
        if rx.try_recv().is_ok() {
            println!("\n📝 Returning to REPL - session continues in background");
            break;
        }

        // Sleep for 1 second
        thread::sleep(StdDuration::from_millis(1000));

        // Check if session still exists (might have been stopped externally)
        if session_store::load_active()?.is_none() {
            println!("\n📝 Session ended externally");
            break;
        }
    }

    println!(); // New line after timer display
    Ok(())
}

fn handle_track_start(
    kind: Option<String>,
    task: Option<String>,
    duration: Option<i64>,
    mode: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::session::{Kind, Session};
    use chrono::{Duration as ChronoDuration, Utc};
    use std::str::FromStr;

    if let Some(active) = session_store::load_active()? {
        return Err(format!(
            "Session already active: {} ({}). Stop it first with 'track stop'",
            active.kind, active.id
        )
        .into());
    }

    if let Some(ref task_id) = task {
        let ws = workspace_storage::load()?;
        if navigation::find_task(&ws, task_id).is_none() {
            return Err(format!("Task '{}' not found", task_id).into());
        }
    }

    let session_kind = match kind {
        Some(k) => Kind::from_str(&k)?,
        None => Kind::Focus,
    };

    let config = pomodoro_config::load()?;
    let session_duration = match duration {
        Some(minutes) => ChronoDuration::minutes(minutes),
        None => config.duration_for_kind(&session_kind),
    };

    let start_time = Utc::now();
    let target_end_time = start_time + session_duration;

    let session = Session {
        id: Session::generate_id(),
        kind: session_kind,
        task_id: task,
        start: start_time,
        end: None,
        target_end: Some(target_end_time),
        warned: false,
        extend_count: 0,
    };

    session_store::save_active(&session)?;

    let task_info = session.task_id.as_deref().unwrap_or("no task linked");
    let end_time_str = target_end_time.format("%H:%M");
    println!(
        "🎯 Started {} session {} ({}) - ends at {}",
        session.kind, session.id, task_info, end_time_str
    );

    // Handle different modes
    match mode.as_deref() {
        Some("interactive") => {
            println!("⏱️  Interactive mode - showing live timer...");
            run_interactive_timer(session)?;
        }
        Some("detach") => {
            println!("🔌 Detached mode - session running in background");
        }
        None => {
            // Default behavior (detached)
            println!("🔌 Session running in background");
        }
        Some(unknown) => {
            println!(
                "⚠️  Unknown mode '{}', using default detached mode",
                unknown
            );
        }
    }

    Ok(())
}

fn handle_track_stop() -> Result<(), Box<dyn std::error::Error>> {
    use chrono::Utc;

    let mut active = match session_store::load_active()? {
        Some(session) => session,
        None => {
            println!("⚠️  No active session to stop");
            return Ok(());
        }
    };

    active.end = Some(Utc::now());
    let duration = active.duration().unwrap();

    session_store::add_session(active.clone())?;
    session_store::clear_active()?;

    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    let task_info = active.task_id.as_deref().unwrap_or("no task linked");

    println!(
        "⏹️  Stopped {} session {} after {}h {}m ({})",
        active.kind, active.id, hours, minutes, task_info
    );

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

            println!(
                "🔄 Active {} session {} - {}h {}m elapsed ({})",
                session.kind, session.id, hours, minutes, task_info
            );
        }
        None => {
            println!("⏸️  No active session");
        }
    }

    Ok(())
}

fn handle_track_extend(minutes: Option<i64>) -> Result<(), Box<dyn std::error::Error>> {
    use chrono::Duration as ChronoDuration;

    let mut active = match session_store::load_active()? {
        Some(session) => session,
        None => {
            println!("⚠️  No active session to extend");
            return Ok(());
        }
    };

    let config = pomodoro_config::load()?;
    let extend_duration = match minutes {
        Some(m) => ChronoDuration::minutes(m),
        None => config.extend_duration(),
    };

    if let Some(target_end) = active.target_end {
        active.target_end = Some(target_end + extend_duration);
        active.warned = false;
        active.extend_count = active.extend_count.saturating_add(1);

        session_store::save_active(&active)?;

        let new_end_str = active.target_end.unwrap().format("%H:%M");
        let extend_mins = extend_duration.num_minutes();
        println!(
            "⏰ Extended {} session by {} minute(s) - new end time: {}",
            active.kind, extend_mins, new_end_str
        );
    } else {
        println!("⚠️  Cannot extend session without target end time");
    }

    Ok(())
}

fn get_status_line() -> String {
    use chrono::Utc;

    match session_store::load_active() {
        Ok(Some(session)) => {
            if let Some(target_end) = session.target_end {
                let now = Utc::now();
                if now > target_end {
                    let overtime = now - target_end;
                    let hours = overtime.num_hours();
                    let minutes = overtime.num_minutes() % 60;
                    format!("🔴 {} OVERTIME +{}h{}m", session.kind, hours, minutes)
                } else {
                    let remaining = target_end - now;
                    let hours = remaining.num_hours();
                    let minutes = remaining.num_minutes() % 60;
                    if hours > 0 {
                        format!("🔵 {} {}h{}m left", session.kind, hours, minutes)
                    } else {
                        format!("🔵 {} {}m left", session.kind, minutes)
                    }
                }
            } else {
                let elapsed = Utc::now() - session.start;
                let hours = elapsed.num_hours();
                let minutes = elapsed.num_minutes() % 60;
                format!("⏳ {} {}h{}m", session.kind, hours, minutes)
            }
        }
        _ => "⏸️  No active session".to_string(),
    }
}

fn parse_command_line(input: &str) -> Result<Commands, Box<dyn std::error::Error>> {
    let args: Vec<&str> = input.trim().split_whitespace().collect();
    if args.is_empty() {
        return Err("Empty command".into());
    }

    if args[0].starts_with('/') {
        match args[0] {
            "/exit" | "/quit" => return Err("exit".into()),
            "/help" => {
                println!("Clara Commands (noun + verb pattern with path completion):");
                println!();
                println!("📁 FOLDER COMMANDS:");
                println!("  folder create <name>");
                println!("  folder list");
                println!("  folder delete <name>");
                println!("  folder update <name>");
                println!();
                println!("📋 LIST COMMANDS:");
                println!("  list create <name> <folder_name>");
                println!("  list list <folder_name>");
                println!("  list delete <folder_name/list_name>");
                println!("  list update <folder_name/list_name>");
                println!();
                println!("✅ TASK COMMANDS:");
                println!("  task create <title> <folder_name/list_name>");
                println!("  task list <folder_name/list_name> [--tree]");
                println!("  task delete <folder_name/list_name/task_path>");
                println!("  task update <folder_name/list_name/task_path>");
                println!("  task done <folder_name/list_name/task_path>");
                println!();
                println!("⏱️  TRACK COMMANDS:");
                println!(
                    "  track start [--kind <type>] [--task <path>] [--duration <mins>] [--d|--it]"
                );
                println!("    --d: detach mode (run in background)");
                println!("    --it: interactive mode (show live timer)");
                println!("  track stop");
                println!("  track current");
                println!("  track extend [--minutes <mins>]");
                println!();
                println!("💡 EXAMPLES (all support tab completion):");
                println!("  folder create Work");
                println!("  list create Today Work");
                println!("  task create \"Write report\" Work/Today");
                println!("  task create \"Research\" Work/Today/Write report  # Creates subtask");
                println!("  task delete Work/Today/Write report");
                println!("  task done Work/Today/Write report/Research");
                println!();
                println!("  /exit, /quit - Exit");
                return Err("help_shown".into());
            }
            _ => return Err(format!("Unknown slash command: {}", args[0]).into()),
        }
    }

    match args[0] {
        "folder" => {
            if args.len() < 2 {
                return Err("Missing folder action (create, list, delete, update)".into());
            }
            match args[1] {
                "create" => {
                    if args.len() < 3 {
                        return Err("Missing folder name".into());
                    }
                    Ok(Commands::Folder {
                        action: FolderAction::Create {
                            name: args[2].to_string(),
                        },
                    })
                }
                "list" => Ok(Commands::Folder {
                    action: FolderAction::List,
                }),
                "delete" => {
                    if args.len() < 3 {
                        return Err("Missing folder name".into());
                    }
                    Ok(Commands::Folder {
                        action: FolderAction::Delete {
                            path: args[2].to_string(),
                        },
                    })
                }
                "update" => {
                    if args.len() < 3 {
                        return Err("Missing folder name".into());
                    }
                    Ok(Commands::Folder {
                        action: FolderAction::Update {
                            path: args[2].to_string(),
                        },
                    })
                }
                _ => Err("Unknown folder action. Use: create, list, delete, update".into()),
            }
        }
        "list" => {
            // Check if this is the new noun + verb pattern by looking at the second argument
            if args.len() >= 2 {
                match args[1] {
                    "create" => {
                        if args.len() < 4 {
                            return Err("Usage: list create <name> <folder_name>".into());
                        }
                        return Ok(Commands::List {
                            action: ListAction::Create {
                                name: args[2].to_string(),
                                folder_path: args[3].to_string(),
                            },
                        });
                    }
                    "list" => {
                        if args.len() < 3 {
                            return Err("Usage: list list <folder_name>".into());
                        }
                        return Ok(Commands::List {
                            action: ListAction::List {
                                folder_path: args[2].to_string(),
                            },
                        });
                    }
                    "delete" => {
                        if args.len() < 3 {
                            return Err("Usage: list delete <folder_name/list_name>".into());
                        }
                        return Ok(Commands::List {
                            action: ListAction::Delete {
                                path: args[2].to_string(),
                            },
                        });
                    }
                    "update" => {
                        if args.len() < 3 {
                            return Err("Usage: list update <folder_name/list_name>".into());
                        }
                        return Ok(Commands::List {
                            action: ListAction::Update {
                                path: args[2].to_string(),
                            },
                        });
                    }
                    _ => Err("Unknown list action. Use: create, list, delete, update".into()),
                }
            } else {
                Err("Missing list action (create, list, delete, update)".into())
            }
        }
        "task" => {
            if args.len() < 2 {
                return Err("Missing task action (create, list, delete, update, done)".into());
            }
            match args[1] {
                "create" => {
                    if args.len() < 4 {
                        return Err("Usage: task create <title> <folder_name/list_name>".into());
                    }
                    Ok(Commands::Task {
                        action: TaskAction::Create {
                            title: args[2].to_string(),
                            list_path: args[3].to_string(),
                        },
                    })
                }
                "list" => {
                    if args.len() < 3 {
                        return Err("Usage: task list <folder_name/list_name> [--tree]".into());
                    }
                    let tree = args.contains(&"--tree");
                    Ok(Commands::Task {
                        action: TaskAction::List {
                            list_path: args[2].to_string(),
                            tree,
                        },
                    })
                }
                "delete" => {
                    if args.len() < 3 {
                        return Err("Usage: task delete <folder_name/list_name/task_path>".into());
                    }
                    Ok(Commands::Task {
                        action: TaskAction::Delete {
                            path: args[2].to_string(),
                        },
                    })
                }
                "update" => {
                    if args.len() < 3 {
                        return Err("Usage: task update <folder_name/list_name/task_path>".into());
                    }
                    Ok(Commands::Task {
                        action: TaskAction::Update {
                            path: args[2].to_string(),
                        },
                    })
                }
                "done" => {
                    if args.len() < 3 {
                        return Err("Usage: task done <folder_name/list_name/task_path>".into());
                    }
                    Ok(Commands::Task {
                        action: TaskAction::Done {
                            path: args[2].to_string(),
                        },
                    })
                }
                _ => Err("Unknown task action. Use: create, list, delete, update, done".into()),
            }
        }
        "track" => {
            if args.len() < 2 {
                return Err("Missing track subcommand".into());
            }
            match args[1] {
                "start" => {
                    let mut kind = None;
                    let mut task = None;
                    let mut duration = None;
                    let mut mode = None;

                    let mut i = 2;
                    while i < args.len() {
                        match args[i] {
                            "--kind" | "-k" => {
                                if i + 1 < args.len() {
                                    kind = Some(args[i + 1].to_string());
                                    i += 2;
                                } else {
                                    return Err("Missing value for --kind".into());
                                }
                            }
                            "--task" | "-t" => {
                                if i + 1 < args.len() {
                                    task = Some(args[i + 1].to_string());
                                    i += 2;
                                } else {
                                    return Err("Missing value for --task".into());
                                }
                            }
                            "--duration" => {
                                if i + 1 < args.len() {
                                    duration = args[i + 1].parse().ok();
                                    i += 2;
                                } else {
                                    return Err("Missing value for --duration".into());
                                }
                            }
                            "--d" => {
                                mode = Some("detach".to_string());
                                i += 1;
                            }
                            "--it" => {
                                mode = Some("interactive".to_string());
                                i += 1;
                            }
                            _ => i += 1,
                        }
                    }

                    Ok(Commands::Track {
                        action: TrackCmd::Start {
                            kind,
                            task,
                            duration,
                            mode,
                        },
                    })
                }
                "stop" => Ok(Commands::Track {
                    action: TrackCmd::Stop,
                }),
                "current" => Ok(Commands::Track {
                    action: TrackCmd::Current,
                }),
                "extend" => {
                    let mut minutes = None;
                    if let Some(pos) = args.iter().position(|&x| x == "--minutes" || x == "-m") {
                        if pos + 1 < args.len() {
                            minutes = args[pos + 1].parse().ok();
                        }
                    }
                    Ok(Commands::Track {
                        action: TrackCmd::Extend { minutes },
                    })
                }
                _ => Err("Unknown track subcommand".into()),
            }
        }
        _ => Err(format!("Unknown command: {}", args[0]).into()),
    }
}

fn execute_command(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        // New noun + verb commands
        Commands::Folder { action } => match action {
            FolderAction::Create { name } => {
                workspace_storage::add_folder(name.clone())?;
                println!("📁 Created folder '{}'!", name);
            }
            FolderAction::List => {
                let folders = workspace_storage::list_folders()?;
                if folders.is_empty() {
                    println!("No folders found.");
                } else {
                    println!("📁 Folders:");
                    for folder in folders {
                        println!("  {} ({})", folder.name, folder.id);
                    }
                }
            }
            FolderAction::Delete { path } => {
                handle_folder_delete(path)?;
            }
            FolderAction::Update { path } => {
                handle_folder_update(path)?;
            }
        },
        Commands::List { action } => match action {
            ListAction::Create { name, folder_path } => {
                handle_list_create(name, folder_path)?;
            }
            ListAction::List { folder_path } => {
                handle_list_list(folder_path)?;
            }
            ListAction::Delete { path } => {
                handle_list_delete(path)?;
            }
            ListAction::Update { path } => {
                handle_list_update(path)?;
            }
        },
        Commands::Task { action } => match action {
            TaskAction::Create { title, list_path } => {
                handle_task_create(title, list_path)?;
            }
            TaskAction::List { list_path, tree } => {
                handle_task_list(list_path, tree)?;
            }
            TaskAction::Delete { path } => {
                handle_task_delete(path)?;
            }
            TaskAction::Update { path } => {
                handle_task_update(path)?;
            }
            TaskAction::Done { path } => {
                handle_task_done(path)?;
            }
        },
        Commands::Track { action } => match action {
            TrackCmd::Start {
                kind,
                task,
                duration,
                mode,
            } => {
                handle_track_start(kind, task, duration, mode)?;
            }
            TrackCmd::Stop => {
                handle_track_stop()?;
            }
            TrackCmd::Current => {
                handle_track_current()?;
            }
            TrackCmd::Extend { minutes } => {
                handle_track_extend(minutes)?;
            }
        },
    }

    Ok(())
}

// New command handlers
fn handle_folder_delete(name: String) -> Result<(), Box<dyn std::error::Error>> {
    let workspace = workspace_storage::load()?;
    let folder = workspace
        .folders
        .iter()
        .find(|f| f.name == name)
        .ok_or_else(|| format!("Folder '{}' not found", name))?;

    if workspace_storage::delete_item(folder.id.clone())? {
        println!("🗑️  Deleted folder '{}'!", name);
    } else {
        println!("⚠️  Could not delete folder '{}'.", name);
    }
    Ok(())
}

fn handle_folder_update(name: String) -> Result<(), Box<dyn std::error::Error>> {
    let workspace = workspace_storage::load()?;
    let folder = workspace
        .folders
        .iter()
        .find(|f| f.name == name)
        .ok_or_else(|| format!("Folder '{}' not found", name))?;

    match workspace_storage::update_item_with_editor(folder.id.clone()) {
        Ok(true) => println!("✏️  Updated folder '{}'!", name),
        Ok(false) => println!("⚠️  Could not find folder '{}'.", name),
        Err(e) => println!("❌ Error updating folder: {}", e),
    }
    Ok(())
}

fn handle_list_create(name: String, folder_path: String) -> Result<(), Box<dyn std::error::Error>> {
    let workspace = workspace_storage::load()?;
    let folder = workspace
        .folders
        .iter()
        .find(|f| f.name == folder_path)
        .ok_or_else(|| format!("Folder '{}' not found", folder_path))?;

    workspace_storage::add_list(folder.id.clone(), name.clone())?;
    println!("📋 Created list '{}' in folder '{}'!", name, folder_path);
    Ok(())
}

fn handle_list_list(folder_path: String) -> Result<(), Box<dyn std::error::Error>> {
    let workspace = workspace_storage::load()?;
    let folder = workspace
        .folders
        .iter()
        .find(|f| f.name == folder_path)
        .ok_or_else(|| format!("Folder '{}' not found", folder_path))?;

    let lists = workspace_storage::list_lists(folder.id.clone())?;
    if lists.is_empty() {
        println!("No lists found in folder '{}'.", folder_path);
    } else {
        println!("📋 Lists in folder '{}':", folder_path);
        for list in lists {
            println!("  {} ({})", list.name, list.id);
        }
    }
    Ok(())
}

fn handle_list_delete(path: String) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_path = parse_hierarchical_path(&path)?;
    if !parsed_path.is_list_level() {
        return Err("Path must be folder_name/list_name format".into());
    }

    let workspace = workspace_storage::load()?;
    let (_, list_id, _) = resolve_path_to_ids(&parsed_path, &workspace)?;

    if workspace_storage::delete_item(list_id.clone())? {
        println!("🗑️  Deleted list '{}'!", path);
    } else {
        println!("⚠️  Could not delete list '{}'.", path);
    }
    Ok(())
}

fn handle_list_update(path: String) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_path = parse_hierarchical_path(&path)?;
    if !parsed_path.is_list_level() {
        return Err("Path must be folder_name/list_name format".into());
    }

    let workspace = workspace_storage::load()?;
    let (_, list_id, _) = resolve_path_to_ids(&parsed_path, &workspace)?;

    match workspace_storage::update_item_with_editor(list_id.clone()) {
        Ok(true) => println!("✏️  Updated list '{}'!", path),
        Ok(false) => println!("⚠️  Could not find list '{}'.", path),
        Err(e) => println!("❌ Error updating list: {}", e),
    }
    Ok(())
}

fn handle_task_create(title: String, list_path: String) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_path = parse_hierarchical_path(&list_path)?;

    if parsed_path.is_list_level() {
        // Adding to a list
        let workspace = workspace_storage::load()?;
        let (folder_id, list_id, _) = resolve_path_to_ids(&parsed_path, &workspace)?;
        workspace_storage::add_task(folder_id, list_id, title.clone())?;
        println!("✅ Created task '{}' in '{}'!", title, list_path);
    } else if parsed_path.is_task_level() {
        // Adding subtask to existing task
        let workspace = workspace_storage::load()?;
        let (_, _, parent_task_id) = resolve_path_to_ids(&parsed_path, &workspace)?;
        if let Some(parent_id) = parent_task_id {
            workspace_storage::add_subtask(parent_id, title.clone())?;
            println!("✅ Created subtask '{}' in '{}'!", title, list_path);
        } else {
            return Err("Could not resolve parent task ID".into());
        }
    } else {
        return Err(
            "Path must be folder_name/list_name or folder_name/list_name/task_path format".into(),
        );
    }
    Ok(())
}

fn handle_task_list(list_path: String, tree: bool) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_path = parse_hierarchical_path(&list_path)?;
    if !parsed_path.is_list_level() {
        return Err("Path must be folder_name/list_name format".into());
    }

    let folder_name = parsed_path.folder.clone();
    let list_name = parsed_path.list.clone();

    workspace_storage::list_tasks(Some(folder_name), list_name, tree)?;
    Ok(())
}

fn handle_task_delete(path: String) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_path = parse_hierarchical_path(&path)?;
    if !parsed_path.is_task_level() {
        return Err("Path must be folder_name/list_name/task_path format".into());
    }

    let workspace = workspace_storage::load()?;
    let (_, _, task_id) = resolve_path_to_ids(&parsed_path, &workspace)?;

    if let Some(id) = task_id {
        if workspace_storage::delete_item(id.clone())? {
            println!("🗑️  Deleted task '{}'!", path);
        } else {
            println!("⚠️  Could not delete task '{}'.", path);
        }
    } else {
        return Err("Could not resolve task ID".into());
    }
    Ok(())
}

fn handle_task_update(path: String) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_path = parse_hierarchical_path(&path)?;
    if !parsed_path.is_task_level() {
        return Err("Path must be folder_name/list_name/task_path format".into());
    }

    let workspace = workspace_storage::load()?;
    let (_, _, task_id) = resolve_path_to_ids(&parsed_path, &workspace)?;

    if let Some(id) = task_id {
        match workspace_storage::update_item_with_editor(id.clone()) {
            Ok(true) => println!("✏️  Updated task '{}'!", path),
            Ok(false) => println!("⚠️  Could not find task '{}'.", path),
            Err(e) => println!("❌ Error updating task: {}", e),
        }
    } else {
        return Err("Could not resolve task ID".into());
    }
    Ok(())
}

fn handle_task_done(path: String) -> Result<(), Box<dyn std::error::Error>> {
    let parsed_path = parse_hierarchical_path(&path)?;
    if !parsed_path.is_task_level() {
        return Err("Path must be folder_name/list_name/task_path format".into());
    }

    let workspace = workspace_storage::load()?;
    let (_, _, task_id) = resolve_path_to_ids(&parsed_path, &workspace)?;

    if let Some(id) = task_id {
        if workspace_storage::mark_task_done(id.clone())? {
            println!("🎉 Marked task '{}' as done!", path);
        } else {
            println!("⚠️  Could not find task '{}'.", path);
        }
    } else {
        return Err("Could not resolve task ID".into());
    }
    Ok(())
}

fn run_interactive_mode() -> Result<(), Box<dyn std::error::Error>> {
    let mut rl: Editor<completion::ClaraHelper, _> = Editor::new()?;
    let helper = completion::ClaraHelper::new();
    rl.set_helper(Some(helper));

    println!("🎯 Clara Interactive Mode");
    println!("Type '/help' for commands, '/exit' to quit");
    println!();

    loop {
        // Check for alerts before showing prompt
        if let Err(e) = alert::check_active_session() {
            eprintln!("Alert check error: {}", e);
        }

        // Create prompt with status
        let status = get_status_line();
        let prompt = format!("clara [{}]> ", status);

        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }

                rl.add_history_entry(line.as_str())?;

                match parse_command_line(&line) {
                    Ok(command) => {
                        if let Err(e) = execute_command(command) {
                            eprintln!("Error: {}", e);
                        }
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        if err_msg == "exit" {
                            println!("👋 Goodbye!");
                            break;
                        } else if err_msg == "help_shown" {
                            continue;
                        } else {
                            eprintln!("Error: {}", e);
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("👋 Goodbye!");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("👋 Goodbye!");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_interactive_mode()
}
