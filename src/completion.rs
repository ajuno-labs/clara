use crate::path_parser::get_completion_suggestions;
use crate::workspace_storage;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Helper};

pub struct ClaraHelper {
    file_completer: FilenameCompleter,
}

impl ClaraHelper {
    pub fn new() -> Self {
        Self {
            file_completer: FilenameCompleter::new(),
        }
    }
}

impl Helper for ClaraHelper {}

impl Completer for ClaraHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        let line_up_to_cursor = &line[..pos];

        // Try to extract the path from common commands
        if let Some((path, start_pos)) = extract_path_from_command_with_position(line_up_to_cursor)
        {
            if let Ok(workspace) = workspace_storage::load() {
                let suggestions = get_completion_suggestions(&path, &workspace);
                let candidates: Vec<Pair> = suggestions
                    .into_iter()
                    .map(|s| Pair {
                        display: s.clone(),
                        replacement: s,
                    })
                    .collect();

                return Ok((start_pos, candidates));
            }
        }

        // Fall back to default file completion for other cases
        self.file_completer.complete(line, pos, _ctx)
    }
}

impl Hinter for ClaraHelper {
    type Hint = String;
}

impl Highlighter for ClaraHelper {}

impl Validator for ClaraHelper {
    fn validate(&self, _ctx: &mut ValidationContext) -> Result<ValidationResult, ReadlineError> {
        Ok(ValidationResult::Valid(None))
    }
}

fn extract_path_from_command_with_position(line: &str) -> Option<(String, usize)> {
    // Parse the command properly handling quoted strings
    let parsed_args = parse_command_args(line);
    
    if parsed_args.is_empty() {
        return None;
    }

    // Handle different command patterns
    match parsed_args[0].as_str() {
        // New noun + verb commands
        "folder" => {
            if parsed_args.len() >= 2 {
                match parsed_args[1].as_str() {
                    "create" | "delete" | "update" => {
                        if parsed_args.len() >= 3 {
                            let folder_name = &parsed_args[2];
                            let start_pos = find_quoted_arg_start_position(line, 2);
                            return Some((folder_name.clone(), start_pos));
                        } else if parsed_args.len() == 2 {
                            // User typed: folder delete or folder update
                            return Some(("".to_string(), line.len()));
                        }
                    }
                    _ => {}
                }
            }
        }
        "list" => {
            if parsed_args.len() >= 2 {
                match parsed_args[1].as_str() {
                    "create" => {
                        if parsed_args.len() >= 4 {
                            let folder_name = &parsed_args[3];
                            let start_pos = find_quoted_arg_start_position(line, 3);
                            return Some((folder_name.clone(), start_pos));
                        } else if parsed_args.len() == 3 {
                            // User typed: list create <name>
                            return Some(("".to_string(), line.len()));
                        }
                    }
                    "list" => {
                        if parsed_args.len() >= 3 {
                            let folder_name = &parsed_args[2];
                            let start_pos = find_quoted_arg_start_position(line, 2);
                            return Some((folder_name.clone(), start_pos));
                        } else if parsed_args.len() == 2 {
                            // User typed: list list
                            return Some(("".to_string(), line.len()));
                        }
                    }
                    "delete" | "update" => {
                        if parsed_args.len() >= 3 {
                            let path = &parsed_args[2];
                            if path.contains('/') {
                                let start_pos = find_quoted_arg_start_position(line, 2);
                                return Some((path.clone(), start_pos));
                            }
                        } else if parsed_args.len() == 2 {
                            // User typed: list delete or list update
                            return Some(("".to_string(), line.len()));
                        }
                    }
                    _ => {}
                }
            }
        }
        "task" => {
            if parsed_args.len() >= 2 {
                match parsed_args[1].as_str() {
                    "create" => {
                        if parsed_args.len() >= 4 {
                            let path = &parsed_args[3];
                            if path.contains('/') {
                                let start_pos = find_quoted_arg_start_position(line, 3);
                                return Some((path.clone(), start_pos));
                            }
                        } else if parsed_args.len() == 3 {
                            // User typed: task create <title>
                            return Some(("".to_string(), line.len()));
                        }
                    }
                    "list" => {
                        if parsed_args.len() >= 3 {
                            let path = &parsed_args[2];
                            if path.contains('/') {
                                let start_pos = find_quoted_arg_start_position(line, 2);
                                return Some((path.clone(), start_pos));
                            }
                        } else if parsed_args.len() == 2 {
                            // User typed: task list
                            return Some(("".to_string(), line.len()));
                        }
                    }
                    "delete" | "update" | "done" => {
                        if parsed_args.len() >= 3 {
                            let path = &parsed_args[2];
                            if path.contains('/') {
                                let start_pos = find_quoted_arg_start_position(line, 2);
                                return Some((path.clone(), start_pos));
                            }
                        } else if parsed_args.len() == 2 {
                            // User typed: task delete/update/done
                            return Some(("".to_string(), line.len()));
                        }
                    }
                    _ => {}
                }
            }
        }
        
        // Legacy commands
        "add" => {
            // add "task title" folder/list/path
            if parsed_args.len() >= 3 {
                let path = &parsed_args[2];
                let start_pos = find_quoted_arg_start_position(line, 2);
                return Some((path.clone(), start_pos));
            } else if parsed_args.len() == 2 {
                // User typed: add "title" 
                // Show all available folders to start the path
                return Some(("".to_string(), line.len()));
            }
        }
        "subtask" => {
            // subtask "title" folder/list/task/path
            if parsed_args.len() >= 3 {
                let path = &parsed_args[2];
                let start_pos = find_quoted_arg_start_position(line, 2);
                return Some((path.clone(), start_pos));
            } else if parsed_args.len() == 2 {
                // User typed: subtask "title"
                // Show all available folders to start the path
                return Some(("".to_string(), line.len()));
            }
        }
        "done" => {
            // done folder/list/task/path
            if parsed_args.len() >= 2 {
                let path = &parsed_args[1];
                let start_pos = find_quoted_arg_start_position(line, 1);
                return Some((path.clone(), start_pos));
            } else if parsed_args.len() == 1 {
                // User typed: done
                // Show all available folders to start the path
                return Some(("".to_string(), line.len()));
            }
        }
        "delete" => {
            // delete folder/list/task/path
            if parsed_args.len() >= 2 {
                let path = &parsed_args[1];
                // Only provide path completion if the argument contains '/'
                if path.contains('/') {
                    let start_pos = find_quoted_arg_start_position(line, 1);
                    return Some((path.clone(), start_pos));
                }
            } else if parsed_args.len() == 1 {
                // User typed: delete
                // Show all available folders to start the path
                return Some(("".to_string(), line.len()));
            }
        }
        "update" => {
            // update folder/list/task/path
            if parsed_args.len() >= 2 {
                let path = &parsed_args[1];
                // Only provide path completion if the argument contains '/'
                if path.contains('/') {
                    let start_pos = find_quoted_arg_start_position(line, 1);
                    return Some((path.clone(), start_pos));
                }
            } else if parsed_args.len() == 1 {
                // User typed: update
                // Show all available folders to start the path
                return Some(("".to_string(), line.len()));
            }
        }
        "track" => {
            // track start --task folder/list/task/path
            if let Some(task_idx) = parsed_args.iter().position(|x| x == "--task") {
                if task_idx + 1 < parsed_args.len() {
                    let path = &parsed_args[task_idx + 1];
                    let start_pos = find_quoted_arg_start_position(line, task_idx + 1);
                    return Some((path.clone(), start_pos));
                } else {
                    // User typed: track start --task
                    // Show all available folders to start the path
                    return Some(("".to_string(), line.len()));
                }
            }
        }
        _ => {}
    }

    None
}

// Simple argument parser that handles quoted strings
fn parse_command_args(line: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    
    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                // Don't include the quotes in the argument
            }
            ' ' | '\t' => {
                if in_quotes {
                    current_arg.push(ch);
                } else if !current_arg.is_empty() {
                    args.push(current_arg.clone());
                    current_arg.clear();
                }
            }
            _ => {
                current_arg.push(ch);
            }
        }
    }
    
    if !current_arg.is_empty() {
        args.push(current_arg);
    }
    
    args
}

// Find the start position of the nth argument (0-indexed) in the original line
fn find_quoted_arg_start_position(line: &str, arg_index: usize) -> usize {
    let mut current_arg_index = 0;
    let mut in_quotes = false;
    let mut current_pos = 0;
    let mut in_whitespace = true;
    
    for ch in line.chars() {
        match ch {
            '"' => {
                if in_whitespace && current_arg_index == arg_index {
                    return current_pos + 1; // Skip the opening quote
                }
                in_quotes = !in_quotes;
                in_whitespace = false;
            }
            ' ' | '\t' => {
                if !in_quotes {
                    if !in_whitespace {
                        current_arg_index += 1;
                        in_whitespace = true;
                    }
                } else {
                    in_whitespace = false;
                }
            }
            _ => {
                if in_whitespace && current_arg_index == arg_index {
                    return current_pos;
                }
                in_whitespace = false;
            }
        }
        current_pos += ch.len_utf8();
    }
    
    line.len()
}

#[allow(dead_code)] // For backward compatibility
fn extract_path_from_command(line: &str) -> Option<String> {
    extract_path_from_command_with_position(line).map(|(path, _)| path)
}

fn find_word_start_position(line: &str, word: &str) -> usize {
    if let Some(pos) = line.rfind(word) {
        pos
    } else {
        line.len()
    }
}

#[allow(dead_code)] // For backward compatibility
fn find_path_start_position(line: &str, path: &str) -> usize {
    find_word_start_position(line, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_path_from_add_command() {
        let result = extract_path_from_command("add \"New task\" Work/Today");
        assert_eq!(result, Some("Work/Today".to_string()));
    }

    #[test]
    fn test_extract_path_from_subtask_command() {
        let result = extract_path_from_command("subtask \"Sub item\" \"Work/Today/Main task\"");
        assert_eq!(result, Some("Work/Today/Main task".to_string()));
    }

    #[test]
    fn test_extract_path_from_track_command() {
        let result = extract_path_from_command("track start --task \"Work/Today/Focus session\"");
        assert_eq!(result, Some("Work/Today/Focus session".to_string()));
    }

    #[test]
    fn test_extract_path_from_done_command() {
        let result = extract_path_from_command("done \"Work/Today/Completed task\"");
        assert_eq!(result, Some("Work/Today/Completed task".to_string()));
    }

    #[test]
    fn test_extract_path_from_delete_command() {
        let result = extract_path_from_command("delete Work/Today/Task");
        assert_eq!(result, Some("Work/Today/Task".to_string()));
        
        // Should not provide completion for ID-based delete
        let result = extract_path_from_command("delete T123");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_path_from_update_command() {
        let result = extract_path_from_command("update Work/Today/Task");
        assert_eq!(result, Some("Work/Today/Task".to_string()));
        
        // Should not provide completion for ID-based update
        let result = extract_path_from_command("update T123");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_path_from_new_folder_commands() {
        let result = extract_path_from_command("folder delete Work");
        assert_eq!(result, Some("Work".to_string()));
        
        let result = extract_path_from_command("folder update Work");
        assert_eq!(result, Some("Work".to_string()));
    }

    #[test]
    fn test_extract_path_from_new_list_commands() {
        let result = extract_path_from_command("list create Today Work");
        assert_eq!(result, Some("Work".to_string()));
        
        let result = extract_path_from_command("list list Work");
        assert_eq!(result, Some("Work".to_string()));
        
        let result = extract_path_from_command("list delete Work/Today");
        assert_eq!(result, Some("Work/Today".to_string()));
    }

    #[test]
    fn test_extract_path_from_new_task_commands() {
        let result = extract_path_from_command("task create \"New task\" Work/Today");
        assert_eq!(result, Some("Work/Today".to_string()));
        
        let result = extract_path_from_command("task list Work/Today");
        assert_eq!(result, Some("Work/Today".to_string()));
        
        let result = extract_path_from_command("task delete Work/Today/Task");
        assert_eq!(result, Some("Work/Today/Task".to_string()));
        
        let result = extract_path_from_command("task done Work/Today/Task");
        assert_eq!(result, Some("Work/Today/Task".to_string()));
    }
}
