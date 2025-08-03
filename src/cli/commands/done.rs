use crate::task::TaskStore;

pub fn done_task(id: u32) -> Result<(), Box<dyn std::error::Error>> {
    let store = TaskStore::new()?;
    
    // Check if task exists
    let task = match store.find_by_id(id)? {
        Some(task) => task,
        None => {
            println!("❌ Task with ID {} not found.", id);
            return Ok(());
        }
    };
    
    // Update task status to Done
    store.update_status(id, crate::task::Status::Done)?;
    
    println!("✅ Task {} marked as done: '{}'", id, task.title);
    
    Ok(())
}