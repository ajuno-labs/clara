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
    let trimmed_input = input.trim();
    
    // Special handling for 'add' command with slash metadata
    if trimmed_input.starts_with("add ") && !trimmed_input.contains("--") {
        // Extract everything after "add " as the task text
        let task_text = &trimmed_input[4..]; // Skip "add "
        if task_text.contains('/') {
            // This looks like slash metadata, handle it directly
            use crate::cli::args::Commands;
            let cmd = Commands::Add { 
                parent: None, 
                text: Some(task_text.to_string()) 
            };
            execute_command(cmd, context)?;
            return Ok(true);
        }
    }
    
    // Special handling for 'edit' command with slash metadata
    if trimmed_input.starts_with("edit ") && !trimmed_input.contains("--") {
        // For edit with slash metadata, we need to be more careful about parsing
        // Expected formats:
        // 1. edit 5 /p urgent           (metadata-only)
        // 2. edit 5 "New title" /p urgent  (title + metadata, quoted)
        // 3. edit 5 SingleWord /p urgent   (single word title + metadata)
        
        // Use shell_words to properly handle quoted arguments
        match shell_words::split(trimmed_input) {
            Ok(argv) if argv.len() >= 3 => {
                if let Ok(id) = argv[1].parse::<u32>() {
                    // Check if there's slash metadata anywhere in the remaining args
                    let remaining_args = &argv[2..];
                    if remaining_args.iter().any(|arg| arg.starts_with('/')) {
                        // Found slash metadata, reconstruct the text part
                        let text = remaining_args.join(" ");
                        
                        use crate::cli::args::Commands;
                        let cmd = Commands::Edit {
                            id,
                            text: Some(text)
                        };
                        execute_command(cmd, context)?;
                        return Ok(true);
                    }
                }
            }
            _ => {} // Fall through to normal parsing
        }
    }

    // Fall back to normal shell parsing for other commands
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
            use crate::editor::edit_toml_content;
            
            // Create TOML template and edit using shared editor utility
            let template = ProjectDraft::new();
            let toml_content = template.to_toml()?;
            let edited_content = match edit_toml_content(&toml_content) {
                Ok(content) => content,
                Err(_) => {
                    println!("❌ Editor exited with non-zero status");
                    return Ok(true);
                }
            };
            
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
            print_task_commands();
        }
        Context::Project => {
            print_project_commands();
        }
    }
    println!();
    println!("Use any command followed by --help for detailed usage.");
}

fn print_task_commands() {
    println!("  add                      - Add a new task (opens editor)");
    println!("  add title /p high /tag work - Add with slash-style metadata");
    println!("  add --parent <id>           - Add a subtask under an existing task");
    println!("  list                        - List all tasks in hierarchical tree structure");
    println!("  edit <id>                   - Edit a task (opens editor)");
    println!("  edit <id> /p high /tag work - Edit with slash-style metadata");
    println!("  remove <id>                 - Remove a task");
    println!("  done <id>                   - Mark a task as done");
    println!();
    println!("Slash metadata options:");
    println!("  /p <priority>    - Set priority (low, medium, high, urgent)");
    println!("  /due <date>      - Set due date (YYYY-MM-DD, MM/DD/YYYY, MM-DD)");
    println!("  /tag <tags>      - Set tags (comma-separated)");
    println!("  /parent <id>     - Set parent task");
    println!("  /est <estimate>  - Set time estimate");
}

fn print_project_commands() {
    println!("Project commands:");
    println!("  list                 - List all projects");
    println!("  add                  - Add a new project");
    println!("  edit <id>            - Edit a project");
    println!("  remove <id>          - Remove a project");
}

fn print_project_help() {
    println!("Available project commands:");
    print_project_commands();
    println!();
    println!("Use any command followed by --help for detailed usage.");
}

fn print_task_help() {
    println!("Available task commands:");
    print_task_commands();
    println!();
    println!("Use any command followed by --help for detailed usage.");
}
