use crate::repl::command_handler::handle_repl_command;
use rustyline::{DefaultEditor, Result};

pub fn start_repl() -> Result<()> {
    let mut rl = DefaultEditor::new()?;

    println!("🎯 Clara Task Manager REPL");
    println!("Type '/help' for available commands or '/quit' to exit.");

    let exit_reason = loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if !line.is_empty() {
                    rl.add_history_entry(line)?;

                    match handle_repl_command(line) {
                        Ok(should_continue) => {
                            if !should_continue {
                                break ExitReason::Command;
                            }
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                break ExitReason::Interrupted;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                break ExitReason::Eof;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break ExitReason::Error;
            }
        }
    };

    match exit_reason {
        ExitReason::Interrupted => {
            println!("^C");
        }
        ExitReason::Eof => {
            println!("^D");
        }
        _ => {}
    }

    say_goodbye();
    Ok(())
}

#[derive(Debug)]
enum ExitReason {
    Command,
    Interrupted,
    Eof,
    Error,
}

fn say_goodbye() {
    println!("Goodbye! 👋");
}
