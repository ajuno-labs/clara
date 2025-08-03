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
}