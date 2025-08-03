use crate::cli::{run::execute_command, args::Cli};
use crate::project::{Project, ProjectStore};
use clap::Parser;

#[derive(Debug, Clone)]
pub enum Context {
    Task,
    Project,
}

#[derive(Debug)]
pub struct ReplContext {
    pub mode: Context,
    pub current_project: Option<Project>,
}

impl ReplContext {
    pub fn new() -> Self {
        ReplContext {
            mode: Context::Task,
            current_project: None,
        }
    }

    pub fn get_prompt(&self) -> String {
        match &self.mode {
            Context::Task => {
                if let Some(project) = &self.current_project {
                    format!("task@{} > ", project.name)
                } else {
                    "task > ".to_string()
                }
            }
            Context::Project => "project > ".to_string(),
        }
    }

    pub fn switch_to_project_mode(&mut self) {
        self.mode = Context::Project;
    }

    pub fn switch_to_task_mode(&mut self) {
        self.mode = Context::Task;
    }

    pub fn set_current_project(&mut self, project: Option<Project>) {
        self.current_project = project;
    }
}

pub fn handle_repl_command(input: &str, context: &mut ReplContext) -> std::result::Result<bool, Box<dyn std::error::Error>> {
    let trimmed = input.trim();
    
    if trimmed.starts_with('/') {
        return handle_internal_command(trimmed, context);
    }
    
    if trimmed.is_empty() {
        return Ok(true);
    }
    
    match context.mode {
        Context::Task => handle_task_command(input, context),
        Context::Project => handle_project_command(input, context),
    }
}

fn handle_internal_command(command: &str, context: &mut ReplContext) -> std::result::Result<bool, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    let cmd = parts[0];
    
    match cmd {
        "/help" => {
            print_repl_help(context);
            Ok(true)
        }
        "/quit" => {
            Ok(false)
        }
        "/project" => {
            context.switch_to_project_mode();
            println!("Switched to project mode. Use '/task' to switch back.");
            Ok(true)
        }
        "/task" => {
            context.switch_to_task_mode();
            println!("Switched to task mode. Use '/project' to switch to project mode.");
            Ok(true)
        }
        "/use" => {
            if parts.len() < 2 {
                println!("Usage: /use <project_name>");
                return Ok(true);
            }
            
            let project_name = parts[1];
            let store = ProjectStore::new()?;
            match store.find_by_name(project_name)? {
                Some(project) => {
                    println!("Switched to project: {}", project.name);
                    context.set_current_project(Some(project));
                    context.switch_to_task_mode();
                }
                None => {
                    println!("Project '{}' not found. Use 'project list' to see available projects.", project_name);
                }
            }
            Ok(true)
        }
        "/clear" => {
            context.set_current_project(None);
            context.switch_to_task_mode();
            println!("Cleared project context. Now in global task mode.");
            Ok(true)
        }
        _ => {
            println!("Unknown REPL command: {}", command);
            println!("Available commands: /help, /quit, /project, /task, /use <project>, /clear");
            Ok(true)
        }
    }
}

fn handle_task_command(input: &str, context: &ReplContext) -> std::result::Result<bool, Box<dyn std::error::Error>> {
    let argv = match shell_words::split(input) {
        Ok(v) if v.is_empty() => return Ok(true),
        Ok(v) => v,
        Err(e) => {
            eprintln!("❌ {}", e);
            return Ok(true);
        }
    };

    let args_with_program = std::iter::once("clara").chain(argv.iter().map(|s| s.as_str()));
    
    match Cli::try_parse_from(args_with_program) {
        Ok(cli) => {
            if let Some(cmd) = cli.cmd {
                execute_command(cmd, context)?;
            } else {
                print_task_help();
            }
        }
        Err(err) => {
            err.print()?;
        }
    }
    
    Ok(true)
}

fn handle_project_command(input: &str, _context: &ReplContext) -> std::result::Result<bool, Box<dyn std::error::Error>> {
    let argv = match shell_words::split(input) {
        Ok(v) if v.is_empty() => return Ok(true),
        Ok(v) => v,
        Err(e) => {
            eprintln!("❌ {}", e);
            return Ok(true);
        }
    };

    match argv[0].as_str() {
        "list" => {
            let store = ProjectStore::new()?;
            let projects = store.list()?;
            
            if projects.is_empty() {
                println!("No projects found.");
                return Ok(true);
            }
            
            println!("📁 Projects:");
            for project in projects {
                println!("  [{}] {}", project.id, project.name);
                if let Some(description) = &project.description {
                    println!("      {}", description);
                }
            }
        }
        "add" => {
            use crate::project::{ProjectDraft, ProjectStore};
            use std::env;
            use std::fs;
            use std::process::Command;
            
            // Create a temporary file with TOML template
            let temp_dir = env::temp_dir();
            let temp_file = temp_dir.join("clara_project.toml");
            
            // Create and write TOML template
            let template = ProjectDraft::new();
            let toml_content = template.to_toml()?;
            fs::write(&temp_file, toml_content)?;
            
            // Get editor from environment variable, default to vim
            let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
            
            // Open editor
            let status = Command::new(&editor)
                .arg(&temp_file)
                .status()?;
            
            if !status.success() {
                println!("❌ Editor exited with non-zero status");
                return Ok(true);
            }
            
            // Read the edited content
            let edited_content = fs::read_to_string(&temp_file)?;
            
            // Parse TOML and convert to project
            let project_draft = ProjectDraft::from_toml(&edited_content)
                .map_err(|e| format!("Invalid TOML: {}", e))?;
            
            let project = project_draft.to_project()
                .map_err(|e| e)?;
            
            // Save project to database
            let store = ProjectStore::new()?;
            match store.insert(&project) {
                Ok(()) => {
                    println!("✅ Project added: '{}'", project.name);
                }
                Err(e) => {
                    if e.to_string().contains("UNIQUE constraint failed") {
                        println!("❌ Project with name '{}' already exists", project.name);
                    } else {
                        return Err(e.into());
                    }
                }
            }
            
            // Clean up temp file
            let _ = fs::remove_file(&temp_file);
        }
        "help" | "--help" => {
            print_project_help();
        }
        _ => {
            println!("Unknown project command: {}", argv[0]);
            print_project_help();
        }
    }
    
    Ok(true)
}

fn print_repl_help(context: &ReplContext) {
    println!("Available REPL commands:");
    println!("  /help              - Show this help message");
    println!("  /quit              - Exit the REPL");
    println!("  /project           - Switch to project mode");
    println!("  /task              - Switch to task mode");
    println!("  /use <project>     - Switch to a specific project context");
    println!("  /clear             - Clear project context (global task mode)");
    println!();
    
    match context.mode {
        Context::Task => {
            if let Some(project) = &context.current_project {
                println!("Task commands (in project: {}):", project.name);
            } else {
                println!("Task commands (global mode):");
            }
            println!("  add                  - Add a new task");
            println!("  add --parent <id>    - Add a subtask under an existing task");
            println!("  list                 - List all tasks in hierarchical tree structure");
            println!("  edit <id>            - Edit a task");
            println!("  remove <id>          - Remove a task");
            println!("  done <id>            - Mark a task as done");
        }
        Context::Project => {
            println!("Project commands:");
            println!("  list                 - List all projects");
            println!("  add                  - Add a new project");
            println!("  edit <id>            - Edit a project");
            println!("  remove <id>          - Remove a project");
        }
    }
    println!();
    println!("Use any command followed by --help for detailed usage.");
}

fn print_project_help() {
    println!("Available project commands:");
    println!("  list                 - List all projects");
    println!("  add                  - Add a new project");
    println!("  edit <id>            - Edit a project");
    println!("  remove <id>          - Remove a project");
    println!();
    println!("Use any command followed by --help for detailed usage.");
}

fn print_task_help() {
    println!("Available task commands:");
    println!("  add                  - Add a new task");
    println!("  add --parent <id>    - Add a subtask under an existing task");
    println!("  list                 - List all tasks in hierarchical tree structure");
    println!("  edit <id>            - Edit a task");
    println!("  remove <id>          - Remove a task");
    println!("  done <id>            - Mark a task as done");
    println!();
    println!("Use any command followed by --help for detailed usage.");
}
