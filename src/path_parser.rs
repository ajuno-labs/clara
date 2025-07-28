use crate::workspace::Workspace;

#[derive(Debug, Clone, PartialEq)]
pub struct HierarchicalPath {
    pub folder: String,
    pub list: Option<String>,
    pub tasks: Vec<String>, // First is root task, rest are nested subtasks
}

impl HierarchicalPath {
    pub fn new(folder: String) -> Self {
        Self {
            folder,
            list: None,
            tasks: Vec::new(),
        }
    }

    pub fn with_list(mut self, list: String) -> Self {
        self.list = Some(list);
        self
    }

    pub fn with_tasks(mut self, tasks: Vec<String>) -> Self {
        self.tasks = tasks;
        self
    }

    #[allow(dead_code)] // Used in tests and may be useful for future features
    pub fn depth(&self) -> usize {
        let mut depth = 1; // folder
        if self.list.is_some() {
            depth += 1; // list
        }
        depth += self.tasks.len(); // tasks
        depth
    }

    pub fn is_folder_only(&self) -> bool {
        self.list.is_none()
    }

    pub fn is_list_level(&self) -> bool {
        self.list.is_some() && self.tasks.is_empty()
    }

    pub fn is_task_level(&self) -> bool {
        !self.tasks.is_empty()
    }

    #[allow(dead_code)] // Used in tests and may be useful for future features
    pub fn task_depth(&self) -> usize {
        self.tasks.len()
    }

    // Get the parent task path (all tasks except the last one)
    #[allow(dead_code)] // Used in tests and may be useful for future features
    pub fn parent_task_path(&self) -> Vec<String> {
        if self.tasks.len() <= 1 {
            Vec::new()
        } else {
            self.tasks[..self.tasks.len() - 1].to_vec()
        }
    }

    // Get the immediate parent (could be list or parent task)
    #[allow(dead_code)] // May be useful for future features
    pub fn parent_id_path(&self) -> Option<HierarchicalPath> {
        if self.tasks.is_empty() {
            // Current path is at list level, parent is folder
            if self.list.is_some() {
                Some(HierarchicalPath::new(self.folder.clone()))
            } else {
                None // Already at folder level
            }
        } else if self.tasks.len() == 1 {
            // Current path is at root task level, parent is list
            Some(
                HierarchicalPath::new(self.folder.clone())
                    .with_list(self.list.clone().unwrap_or_default()),
            )
        } else {
            // Current path is at subtask level, parent is parent task
            let parent_tasks = self.tasks[..self.tasks.len() - 1].to_vec();
            Some(
                HierarchicalPath::new(self.folder.clone())
                    .with_list(self.list.clone().unwrap_or_default())
                    .with_tasks(parent_tasks),
            )
        }
    }
}

pub fn parse_hierarchical_path(path: &str) -> Result<HierarchicalPath, String> {
    if path.is_empty() {
        return Err("Empty path".to_string());
    }

    let components: Vec<String> = path.split('/').map(|s| s.trim().to_string()).collect();

    if components.is_empty() {
        return Err("Invalid path format".to_string());
    }

    // Remove any empty components from splitting
    let components: Vec<String> = components.into_iter().filter(|s| !s.is_empty()).collect();

    if components.is_empty() {
        return Err("Invalid path format".to_string());
    }

    match components.len() {
        1 => {
            // folder only
            Ok(HierarchicalPath::new(components[0].clone()))
        }
        2 => {
            // folder/list
            Ok(HierarchicalPath::new(components[0].clone()).with_list(components[1].clone()))
        }
        n if n >= 3 => {
            // folder/list/task1/task2/task3/...
            let folder = components[0].clone();
            let list = components[1].clone();
            let tasks = components[2..].to_vec();

            Ok(HierarchicalPath::new(folder)
                .with_list(list)
                .with_tasks(tasks))
        }
        _ => Err("Invalid path format".to_string()),
    }
}

#[allow(dead_code)] // For future validation features
pub fn validate_path_against_workspace(
    path: &HierarchicalPath,
    workspace: &Workspace,
) -> Result<(), String> {
    // Check folder exists
    let folder = workspace
        .folders
        .iter()
        .find(|f| f.name == path.folder)
        .ok_or_else(|| format!("Folder '{}' not found", path.folder))?;

    // If only folder level, we're done
    if path.is_folder_only() {
        return Ok(());
    }

    // Check list exists
    let list_name = path.list.as_ref().unwrap();
    let list = folder
        .lists
        .iter()
        .find(|l| l.name == *list_name)
        .ok_or_else(|| format!("List '{}' not found in folder '{}'", list_name, path.folder))?;

    // If only list level, we're done
    if path.is_list_level() {
        return Ok(());
    }

    // Navigate through task hierarchy
    let mut current_task = list
        .tasks
        .iter()
        .find(|t| t.title == path.tasks[0])
        .ok_or_else(|| {
            format!(
                "Task '{}' not found in list '{}/{}'",
                path.tasks[0], path.folder, list_name
            )
        })?;

    // Navigate through subtasks if any
    for (i, task_name) in path.tasks.iter().skip(1).enumerate() {
        current_task = current_task
            .subtasks
            .iter()
            .find(|st| st.title == *task_name)
            .ok_or_else(|| {
                let path_so_far = path.tasks[0..=i + 1].join("/");
                format!(
                    "Task '{}' not found at path '{}/{}/{}'",
                    task_name, path.folder, list_name, path_so_far
                )
            })?;
    }

    Ok(())
}

pub fn get_completion_suggestions(partial_path: &str, workspace: &Workspace) -> Vec<String> {
    // Handle the special case where path ends with '/'
    let path_ends_with_slash = partial_path.ends_with('/');
    
    let components: Vec<&str> = partial_path.split('/').collect();
    let components: Vec<&str> = components.into_iter().filter(|s| !s.is_empty()).collect();

    if components.is_empty() {
        // Return all folder names when no input
        return workspace.folders.iter()
            .map(|f| f.name.clone())
            .collect();
    }

    match components.len() {
        1 => {
            if path_ends_with_slash {
                // User typed "Work/" - show all lists in that folder
                let folder_name = components[0];
                if let Some(folder) = workspace.folders.iter().find(|f| f.name == folder_name) {
                    folder.lists.iter()
                        .map(|l| format!("{}/{}", folder_name, l.name))
                        .collect()
                } else {
                    Vec::new()
                }
            } else {
                // Complete folder names
                let prefix = components[0];
                workspace
                    .folders
                    .iter()
                    .filter(|f| f.name.starts_with(prefix))
                    .map(|f| f.name.clone())
                    .collect()
            }
        }
        2 => {
            if path_ends_with_slash {
                // User typed "Work/Today/" - show all root tasks in that list
                let folder_name = components[0];
                let list_name = components[1];
                
                if let Some(folder) = workspace.folders.iter().find(|f| f.name == folder_name) {
                    if let Some(list) = folder.lists.iter().find(|l| l.name == list_name) {
                        list.tasks.iter()
                            .map(|t| format!("{}/{}/{}", folder_name, list_name, t.title))
                            .collect()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            } else {
                // Complete list names
                let folder_name = components[0];
                let list_prefix = components[1];

                if let Some(folder) = workspace.folders.iter().find(|f| f.name == folder_name) {
                    folder
                        .lists
                        .iter()
                        .filter(|l| l.name.starts_with(list_prefix))
                        .map(|l| format!("{}/{}", folder_name, l.name))
                        .collect()
                } else {
                    Vec::new()
                }
            }
        }
        n if n >= 3 => {
            let folder_name = components[0];
            let list_name = components[1];

            if let Some(folder) = workspace.folders.iter().find(|f| f.name == folder_name) {
                if let Some(list) = folder.lists.iter().find(|l| l.name == list_name) {
                    if path_ends_with_slash {
                        // User typed something like "Work/Today/Task1/" - show all subtasks
                        let task_path = &components[2..];
                        get_subtasks_at_path(folder_name, list_name, list, task_path)
                    } else {
                        // Complete task/subtask names at current level
                        let task_prefix = components.last().unwrap_or(&"");
                        let task_path = &components[2..components.len() - 1];

                        if task_path.is_empty() {
                            // Completing root task names
                            list.tasks
                                .iter()
                                .filter(|t| t.title.starts_with(task_prefix))
                                .map(|t| format!("{}/{}/{}", folder_name, list_name, t.title))
                                .collect()
                        } else {
                            // Navigate through task hierarchy to find the right level
                            let mut current_task = if let Some(task) =
                                list.tasks.iter().find(|t| t.title == task_path[0])
                            {
                                task
                            } else {
                                return Vec::new();
                            };

                            // Navigate through subtasks
                            for subtask_name in task_path.iter().skip(1) {
                                if let Some(subtask) = current_task
                                    .subtasks
                                    .iter()
                                    .find(|st| &st.title == subtask_name)
                                {
                                    current_task = subtask;
                                } else {
                                    return Vec::new();
                                }
                            }

                            // Get completions from current level
                            let base_path = components[0..components.len() - 1].join("/");
                            current_task
                                .subtasks
                                .iter()
                                .filter(|st| st.title.starts_with(task_prefix))
                                .map(|st| format!("{}/{}", base_path, st.title))
                                .collect()
                        }
                    }
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        }
        _ => Vec::new(),
    }
}

// Helper function to get all subtasks at a given path
fn get_subtasks_at_path(folder_name: &str, list_name: &str, list: &crate::workspace::List, task_path: &[&str]) -> Vec<String> {
    if task_path.is_empty() {
        return Vec::new();
    }
    
    // Find the target task by navigating through the path
    let mut current_task = if let Some(task) = list.tasks.iter().find(|t| t.title == task_path[0]) {
        task
    } else {
        return Vec::new();
    };
    
    // Navigate through subtasks
    for subtask_name in task_path.iter().skip(1) {
        if let Some(subtask) = current_task.subtasks.iter().find(|st| &st.title == subtask_name) {
            current_task = subtask;
        } else {
            return Vec::new();
        }
    }
    
    // Return all subtasks at this level
    let base_path = format!("{}/{}/{}", folder_name, list_name, task_path.join("/"));
    current_task.subtasks.iter()
        .map(|st| format!("{}/{}", base_path, st.title))
        .collect()
}

pub fn resolve_path_to_ids(
    path: &HierarchicalPath,
    workspace: &Workspace,
) -> Result<(String, String, Option<String>), String> {
    // Find the folder
    let folder = workspace
        .folders
        .iter()
        .find(|f| f.name == path.folder)
        .ok_or_else(|| format!("Folder '{}' not found", path.folder))?;

    // If only folder level
    if path.is_folder_only() {
        return Ok((folder.id.clone(), String::new(), None));
    }

    // Find the list
    let list_name = path.list.as_ref().unwrap();
    let list = folder
        .lists
        .iter()
        .find(|l| l.name == *list_name)
        .ok_or_else(|| format!("List '{}' not found in folder '{}'", list_name, path.folder))?;

    // If only list level
    if path.is_list_level() {
        return Ok((folder.id.clone(), list.id.clone(), None));
    }

    // Navigate through task hierarchy
    let mut current_task = list
        .tasks
        .iter()
        .find(|t| t.title == path.tasks[0])
        .ok_or_else(|| {
            format!(
                "Task '{}' not found in list '{}/{}'",
                path.tasks[0], path.folder, list_name
            )
        })?;

    // Navigate through subtasks if any
    for task_name in path.tasks.iter().skip(1) {
        current_task = current_task
            .subtasks
            .iter()
            .find(|st| st.title == *task_name)
            .ok_or_else(|| format!("Task '{}' not found", task_name))?;
    }

    Ok((
        folder.id.clone(),
        list.id.clone(),
        Some(current_task.id.clone()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_folder_path() {
        let result = parse_hierarchical_path("Work").unwrap();
        assert_eq!(result.folder, "Work");
        assert!(result.is_folder_only());
        assert_eq!(result.depth(), 1);
    }

    #[test]
    fn test_parse_list_path() {
        let result = parse_hierarchical_path("Work/Today").unwrap();
        assert_eq!(result.folder, "Work");
        assert_eq!(result.list, Some("Today".to_string()));
        assert!(result.is_list_level());
        assert_eq!(result.depth(), 2);
    }

    #[test]
    fn test_parse_task_path() {
        let result = parse_hierarchical_path("Work/Today/Write proposal").unwrap();
        assert_eq!(result.folder, "Work");
        assert_eq!(result.list, Some("Today".to_string()));
        assert_eq!(result.tasks, vec!["Write proposal"]);
        assert!(result.is_task_level());
        assert_eq!(result.task_depth(), 1);
        assert_eq!(result.depth(), 3);
    }

    #[test]
    fn test_parse_deeply_nested_path() {
        let result = parse_hierarchical_path(
            "Work/Today/Write proposal/Research/Literature review/Find papers/Academic sources",
        )
        .unwrap();
        assert_eq!(result.folder, "Work");
        assert_eq!(result.list, Some("Today".to_string()));
        assert_eq!(
            result.tasks,
            vec![
                "Write proposal",
                "Research",
                "Literature review",
                "Find papers",
                "Academic sources"
            ]
        );
        assert!(result.is_task_level());
        assert_eq!(result.task_depth(), 5);
        assert_eq!(result.depth(), 7);
    }

    #[test]
    fn test_parent_task_path() {
        let result =
            parse_hierarchical_path("Work/Today/Write proposal/Research/Literature review")
                .unwrap();
        let parent = result.parent_task_path();
        assert_eq!(parent, vec!["Write proposal", "Research"]);
    }

    #[test]
    fn test_infinite_nesting_support() {
        // Test that we can handle arbitrarily deep nesting
        let deep_path = "Work/Today/L1/L2/L3/L4/L5/L6/L7/L8/L9/L10";
        let result = parse_hierarchical_path(deep_path).unwrap();
        assert_eq!(result.task_depth(), 10); // L1 through L10 = 10 tasks
        assert_eq!(result.depth(), 12); // folder + list + 10 tasks

        // Verify all task names are captured
        assert_eq!(
            result.tasks,
            vec!["L1", "L2", "L3", "L4", "L5", "L6", "L7", "L8", "L9", "L10"]
        );
    }

    #[test]
    fn test_completion_with_trailing_slash() {
        use super::*;
        use crate::workspace::{Workspace, Folder, List};
        use crate::task::Task;
        use chrono::Utc;
        
        // Create a test workspace
        let task1 = Task {
            id: "T1".to_string(),
            title: "Task1".to_string(),
            description: None,
            done: false,
            created_at: Utc::now(),
            subtasks: vec![
                Task {
                    id: "T1.1".to_string(),
                    title: "Subtask1".to_string(),
                    description: None,
                    done: false,
                    created_at: Utc::now(),
                    subtasks: vec![],
                }
            ],
        };
        
        let list = List {
            id: "L1".to_string(),
            name: "Today".to_string(),
            tasks: vec![task1],
        };
        
        let folder = Folder {
            id: "F1".to_string(),
            name: "Work".to_string(),
            lists: vec![list],
        };
        
        let workspace = Workspace {
            folders: vec![folder],
        };

        // Test completion with trailing slash shows all options
        let suggestions = get_completion_suggestions("Work/", &workspace);
        assert_eq!(suggestions, vec!["Work/Today"]);
        
        let suggestions = get_completion_suggestions("Work/Today/", &workspace);
        assert_eq!(suggestions, vec!["Work/Today/Task1"]);
        
        let suggestions = get_completion_suggestions("Work/Today/Task1/", &workspace);
        assert_eq!(suggestions, vec!["Work/Today/Task1/Subtask1"]);
        
        // Test completion without trailing slash for partial matches
        let suggestions = get_completion_suggestions("Wo", &workspace);
        assert_eq!(suggestions, vec!["Work"]);
        
        let suggestions = get_completion_suggestions("Work/Tod", &workspace);
        assert_eq!(suggestions, vec!["Work/Today"]);
    }
}
