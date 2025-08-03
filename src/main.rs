mod cli;
mod project;
mod repl; 
mod task;

use repl::start_repl;

fn main() {
    if let Err(e) = start_repl() {
        eprintln!("Error starting REPL: {}", e);
        std::process::exit(1);
    }
}
