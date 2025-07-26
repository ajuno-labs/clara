mod storage;
mod task;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "clara", version, about = "CLI productivity assistant")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add { title: String },
    List,
    Done { id: u32 },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { title } => {
            storage::add(title)?;
            println!("✅ Task saved!");
        }
        Commands::List => {
            let tasks = storage::load()?;
            if tasks.is_empty() {
                println!("No tasks yet.");
            } else {
                for t in tasks {
                    println!(
                        "[{}] {:3}  {}  (created {})",
                        if t.done { "x" } else { " " },
                        t.id,
                        t.title,
                        t.created_at.format("%Y‑%m‑%d %H:%M")
                    );
                }
            }
        }
        Commands::Done { id } => {
            if storage::mark_done(id)? {
                println!("🎉 Task #{id} marked done!");
            } else {
                println!("⚠️  No task with id {id}.");
            }
        }
    }

    Ok(())
}
