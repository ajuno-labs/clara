use crate::cli::args::Commands;
use crate::cli::commands::{add_task, done_task, edit_task, list_tasks, remove_task};

pub fn execute_command(cmd: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        Commands::Add => add_task(),
        Commands::List => list_tasks(),
        Commands::Edit { id } => edit_task(id),
        Commands::Remove { id } => remove_task(id),
        Commands::Done { id } => done_task(id),
    }
}
