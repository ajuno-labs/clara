use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Add {
        #[arg(long, help = "ID of the parent task to create a subtask under")]
        parent: Option<u32>,
        #[arg(help = "Task with slash-style metadata: 'title /p high /due 2025-08-10 /tag work'")]
        text: Option<String>,
    },
    List,
    Edit {
        #[arg(help = "ID of the task to edit")]
        id: u32,
        #[arg(help = "Slash-style metadata updates: '/p high /tag work,urgent'")]
        text: Option<String>,
    },
    Remove {
        #[arg(help = "ID of the task to remove")]
        id: u32,
    },
    Done {
        #[arg(help = "ID of the task to mark as done")]
        id: u32,
    },
}
