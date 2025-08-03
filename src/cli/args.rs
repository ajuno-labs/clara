use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "clara")]
#[command(about = "A simple task management CLI")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Add,
    List,
    Edit {
        #[arg(help = "ID of the task to edit")]
        id: u32,
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