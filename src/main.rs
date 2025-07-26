mod task;
mod workspace;
mod workspace_storage;
mod navigation;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "clara", version, about = "CLI productivity assistant")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Folder {
        #[command(subcommand)]
        action: FolderCmd,
    },
    Add { 
        title: String,
        #[arg(long)]
        folder: String,
        #[arg(long)]
        list: String,
    },
    Subtask {
        title: String,
        #[arg(long)]
        parent: String,
    },
    List {
        #[arg(long)]
        folder: Option<String>,
        #[arg(long)]
        list: Option<String>,
        #[arg(long)]
        tree: bool,
    },
    Done { id: String },
}

#[derive(Subcommand, Debug)]
enum FolderCmd {
    Add { name: String },
    List,
    #[command(subcommand)]
    Lists(ListCmd),
}

#[derive(Subcommand, Debug)]
enum ListCmd {
    Add { 
        #[arg(long)]
        folder: String,
        name: String 
    },
    List { 
        #[arg(long)]
        folder: String 
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Folder { action } => match action {
            FolderCmd::Add { name } => {
                workspace_storage::add_folder(name)?;
                println!("📁 Folder created!");
            }
            FolderCmd::List => {
                for f in workspace_storage::list_folders()? {
                    println!("{}  {}", f.id, f.name);
                }
            }
            FolderCmd::Lists(list_cmd) => match list_cmd {
                ListCmd::Add { folder, name } => {
                    workspace_storage::add_list(folder, name)?;
                    println!("📋 List created!");
                }
                ListCmd::List { folder } => {
                    for list in workspace_storage::list_lists(folder)? {
                        println!("{}  {}", list.id, list.name);
                    }
                }
            }
        },
        Commands::Add { title, folder, list } => {
            workspace_storage::add_task(folder, list, title)?;
            println!("✅ Task saved!");
        }
        Commands::Subtask { title, parent } => {
            workspace_storage::add_subtask(parent, title)?;
            println!("✅ Subtask saved!");
        }
        Commands::List { folder, list, tree } => {
            workspace_storage::list_tasks(folder, list, tree)?;
        }
        Commands::Done { id } => {
            if workspace_storage::mark_task_done(id.clone())? {
                println!("🎉 Task #{id} marked done!");
            } else {
                println!("⚠️  No task with id {id}.");
            }
        }
    }

    Ok(())
}
