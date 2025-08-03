use crate::task::TaskStore;

pub fn remove_task(id: u32) -> Result<(), Box<dyn std::error::Error>> {
    let store = TaskStore::new()?;
    
    // Check if task exists
    let task = match store.find_by_id(id)? {
        Some(task) => task,
        None => {
            println!("❌ Task with ID {} not found.", id);
            return Ok(());
        }
    };
    
    // Delete the task
    store.delete(id)?;
    
    println!("🗑️  Task {} removed: '{}'", id, task.title);
    
    Ok(())
}