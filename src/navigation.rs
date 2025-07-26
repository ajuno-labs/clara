use crate::workspace::{Workspace, Folder, List};
use crate::task::Task;

/// Mutable recursive lookup by id string.
/// Returns Some(&mut Task) if found, else None.
pub fn find_task_mut<'a>(
    ws: &'a mut Workspace,
    id: &str,
) -> Option<&'a mut Task> {
    for folder in &mut ws.folders {
        if let Some(t) = find_task_in_folder(folder, id) {
            return Some(t);
        }
    }
    None
}

fn find_task_in_folder<'a>(
    folder: &'a mut Folder,
    id: &str,
) -> Option<&'a mut Task> {
    for list in &mut folder.lists {
        if let Some(t) = find_task_in_list(list, id) {
            return Some(t);
        }
    }
    None
}

fn find_task_in_list<'a>(
    list: &'a mut List,
    id: &str,
) -> Option<&'a mut Task> {
    for task in &mut list.tasks {
        if task.id == id {
            return Some(task);
        }
        if let Some(sub) = find_task_in_task(task, id) {
            return Some(sub);
        }
    }
    None
}

fn find_task_in_task<'a>(
    task: &'a mut Task,
    id: &str,
) -> Option<&'a mut Task> {
    for sub in &mut task.subtasks {
        if sub.id == id {
            return Some(sub);
        }
        if let Some(deeper) = find_task_in_task(sub, id) {
            return Some(deeper);
        }
    }
    None
}

/// Read-only recursive lookup by id string.
/// Returns Some(&Task) if found, else None.
#[allow(dead_code)]
pub fn find_task<'a>(
    ws: &'a Workspace,
    id: &str,
) -> Option<&'a Task> {
    for folder in &ws.folders {
        if let Some(t) = find_task_in_folder_ro(folder, id) {
            return Some(t);
        }
    }
    None
}

#[allow(dead_code)]
fn find_task_in_folder_ro<'a>(
    folder: &'a Folder,
    id: &str,
) -> Option<&'a Task> {
    for list in &folder.lists {
        if let Some(t) = find_task_in_list_ro(list, id) {
            return Some(t);
        }
    }
    None
}

#[allow(dead_code)]
fn find_task_in_list_ro<'a>(
    list: &'a List,
    id: &str,
) -> Option<&'a Task> {
    for task in &list.tasks {
        if task.id == id {
            return Some(task);
        }
        if let Some(sub) = find_task_in_task_ro(task, id) {
            return Some(sub);
        }
    }
    None
}

#[allow(dead_code)]
fn find_task_in_task_ro<'a>(
    task: &'a Task,
    id: &str,
) -> Option<&'a Task> {
    for sub in &task.subtasks {
        if sub.id == id {
            return Some(sub);
        }
        if let Some(deeper) = find_task_in_task_ro(sub, id) {
            return Some(deeper);
        }
    }
    None
}