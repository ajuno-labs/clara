use crate::cli::{run::execute_command, args::Cli};
use clap::Parser;

pub fn handle_repl_command(input: &str) -> std::result::Result<bool, Box<dyn std::error::Error>> {
    let trimmed = input.trim();
    
    if trimmed.starts_with('/') {
        return handle_internal_command(trimmed);
    }
    
    if trimmed.is_empty() {
        return Ok(true);
    }
    
    handle_task_command(input)
}

fn handle_internal_command(command: &str) -> std::result::Result<bool, Box<dyn std::error::Error>> {
    match command {
        "/help" => {
            print_repl_help();
            Ok(true)
        }
        "/quit" => {
            Ok(false)
        }
        _ => {
            println!("Unknown REPL command: {}", command);
            println!("Available commands: /help, /quit");
            Ok(true)
        }
    }
}

fn handle_task_command(input: &str) -> std::result::Result<bool, Box<dyn std::error::Error>> {
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
                execute_command(cmd)?;
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

fn print_repl_help() {
    println!("Available REPL commands:");
    println!("  /help  - Show this help message");
    println!("  /quit  - Exit the REPL");
    println!();
    println!("Task commands:");
    println!("  add           - Add a new task");
    println!("  list          - List all tasks");
    println!("  edit <id>     - Edit a task");
    println!("  remove <id>   - Remove a task");
    println!("  done <id>     - Mark a task as done");
    println!();
    println!("Use any command followed by --help for detailed usage.");
}

fn print_task_help() {
    println!("Available task commands:");
    println!("  add           - Add a new task");
    println!("  list          - List all tasks");
    println!("  edit <id>     - Edit a task");
    println!("  remove <id>   - Remove a task");
    println!("  done <id>     - Mark a task as done");
    println!();
    println!("Use any command followed by --help for detailed usage.");
}
