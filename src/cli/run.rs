use crate::cli::args::Commands;
use crate::cli::commands::{add_task, done_task, edit_task, list_tasks, remove_task};
use crate::repl::command_handler::ReplContext;

pub fn execute_command(cmd: Commands, context: &ReplContext) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        Commands::Add { parent } => add_task(parent, context),
        Commands::List => list_tasks(context),
        Commands::Edit { id } => edit_task(id),
        Commands::Remove { id } => remove_task(id),
        Commands::Done { id } => done_task(id),
    }
}
