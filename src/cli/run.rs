use crate::cli::args::Commands;
use crate::cli::commands::{add_task, done_task, list_tasks, remove_task, update_task};
use crate::repl::command_handler::ReplContext;

pub fn execute_command(
    cmd: Commands,
    context: &ReplContext,
) -> Result<(), Box<dyn std::error::Error>> {
    match cmd {
        Commands::Add { parent, text } => add_task(parent, text, context),
        Commands::List => list_tasks(context),
        Commands::Update { id, text } => update_task(id, text),
        Commands::Remove { id } => remove_task(id),
        Commands::Done { id } => done_task(id),
    }
}
