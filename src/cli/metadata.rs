use chrono::{Datelike, Local, NaiveDate, TimeZone};

#[derive(Debug, Clone)]
pub struct TaskMetadata {
    pub title: String,
    pub priority: Option<String>,
    pub due_date: Option<i64>,
    pub tags: Vec<String>,
    pub estimate: Option<String>,
    pub parent: Option<u32>,
}

impl TaskMetadata {
    pub fn new(title: String) -> Self {
        TaskMetadata {
            title,
            priority: None,
            due_date: None,
            tags: Vec::new(),
            estimate: None,
            parent: None,
        }
    }
}

pub fn parse_slash_metadata(input: &str) -> Result<TaskMetadata, String> {
    let parts: Vec<&str> = input.split('/').collect();
    
    if parts.is_empty() {
        return Err("No input provided".to_string());
    }
    
    // First part is the title (everything before the first /)
    let title = parts[0].trim().to_string();
    
    if title.is_empty() {
        return Err("Task title cannot be empty".to_string());
    }
    
    let mut metadata = TaskMetadata::new(title);
    
    // Parse slash metadata
    let mut i = 1;
    while i < parts.len() {
        let part = parts[i].trim();
        
        if part.is_empty() {
            i += 1;
            continue;
        }
        
        // Split by whitespace to get key and value
        let key_value: Vec<&str> = part.splitn(2, ' ').collect();
        
        if key_value.len() < 2 {
            return Err(format!("Invalid metadata format: /{}", part));
        }
        
        let key = key_value[0];
        let value = key_value[1];
        
        match key {
            "p" | "priority" => {
                metadata.priority = Some(value.to_string());
            }
            "due" => {
                metadata.due_date = Some(parse_due_date(value)?);
            }
            "tag" | "tags" => {
                metadata.tags = value.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            "est" | "estimate" => {
                metadata.estimate = Some(value.to_string());
            }
            "parent" => {
                metadata.parent = Some(value.parse::<u32>()
                    .map_err(|_| format!("Invalid parent ID: {}", value))?);
            }
            _ => {
                return Err(format!("Unknown metadata key: {}", key));
            }
        }
        
        i += 1;
    }
    
    Ok(metadata)
}

fn parse_due_date(date_str: &str) -> Result<i64, String> {
    // Try parsing various date formats
    
    // ISO format: 2025-08-10
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        let datetime = date.and_hms_opt(23, 59, 59)
            .ok_or("Invalid date")?;
        return Ok(Local.from_local_datetime(&datetime).single()
            .ok_or("Invalid date conversion")?.timestamp());
    }
    
    // US format: 08/10/2025
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%m/%d/%Y") {
        let datetime = date.and_hms_opt(23, 59, 59)
            .ok_or("Invalid date")?;
        return Ok(Local.from_local_datetime(&datetime).single()
            .ok_or("Invalid date conversion")?.timestamp());
    }
    
    // Short format: 08-10 (current year)
    let current_year = Local::now().year();
    let with_year = format!("{}-{}", current_year, date_str);
    if let Ok(date) = NaiveDate::parse_from_str(&with_year, "%Y-%m-%d") {
        let datetime = date.and_hms_opt(23, 59, 59)
            .ok_or("Invalid date")?;
        return Ok(Local.from_local_datetime(&datetime).single()
            .ok_or("Invalid date conversion")?.timestamp());
    }
    
    Err(format!("Invalid date format: {}. Use YYYY-MM-DD, MM/DD/YYYY, or MM-DD", date_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_title() {
        let result = parse_slash_metadata("Fix the bug").unwrap();
        assert_eq!(result.title, "Fix the bug");
        assert_eq!(result.priority, None);
        assert_eq!(result.tags, Vec::<String>::new());
    }

    #[test]
    fn test_parse_with_priority() {
        let result = parse_slash_metadata("Fix the bug /p high").unwrap();
        assert_eq!(result.title, "Fix the bug");
        assert_eq!(result.priority, Some("high".to_string()));
    }

    #[test]
    fn test_parse_priority_variations() {
        let result1 = parse_slash_metadata("Task /p low").unwrap();
        assert_eq!(result1.priority, Some("low".to_string()));
        
        let result2 = parse_slash_metadata("Task /p medium").unwrap();
        assert_eq!(result2.priority, Some("medium".to_string()));
        
        let result3 = parse_slash_metadata("Task /p high").unwrap();
        assert_eq!(result3.priority, Some("high".to_string()));
        
        let result4 = parse_slash_metadata("Task /p urgent").unwrap();
        assert_eq!(result4.priority, Some("urgent".to_string()));
        
        let result5 = parse_slash_metadata("Task /priority urgent").unwrap();
        assert_eq!(result5.priority, Some("urgent".to_string()));
    }

    #[test]
    fn test_parse_with_tags() {
        let result = parse_slash_metadata("Fix the bug /tag work,urgent").unwrap();
        assert_eq!(result.title, "Fix the bug");
        assert_eq!(result.tags, vec!["work", "urgent"]);
    }

    #[test]
    fn test_parse_multiple_metadata() {
        let result = parse_slash_metadata("Prepare slides /p 1 /due 2025-08-10 /tag work,presentation /est 2h").unwrap();
        assert_eq!(result.title, "Prepare slides");
        assert_eq!(result.priority, Some("1".to_string()));
        assert_eq!(result.tags, vec!["work", "presentation"]);
        assert_eq!(result.estimate, Some("2h".to_string()));
        assert!(result.due_date.is_some());
    }

    #[test]
    fn test_parse_with_parent() {
        let result = parse_slash_metadata("Subtask /parent 5").unwrap();
        assert_eq!(result.title, "Subtask");
        assert_eq!(result.parent, Some(5));
    }

    #[test]
    fn test_full_task_creation_with_priority() {
        use crate::task::{TaskDraft};
        use crate::task::model::Priority;
        
        // Parse metadata
        let metadata = parse_slash_metadata("Test task /p high").unwrap();
        
        // Create TaskDraft
        let mut draft = TaskDraft::new();
        draft.title = metadata.title;
        if let Some(priority) = metadata.priority {
            draft.priority = priority;
        }
        
        // Convert to Task
        let task = draft.to_task().unwrap();
        
        // Check the priority was set correctly
        assert_eq!(task.title, "Test task");
        
        // The priority should be High, not Medium (the default)
        match task.priority {
            Priority::High => assert!(true),
            other => panic!("Expected Priority::High, got {:?}", other),
        }
    }

    #[test]
    fn test_edit_metadata_only() {
        // Test parsing metadata-only for edit command (starts with /)
        let dummy_input = format!("DUMMY_TITLE {}", "/p urgent /tag critical");
        let metadata = parse_slash_metadata(&dummy_input).unwrap();
        
        assert_eq!(metadata.title, "DUMMY_TITLE");
        assert_eq!(metadata.priority, Some("urgent".to_string()));
        assert_eq!(metadata.tags, vec!["critical"]);
    }

    #[test]
    fn test_edit_title_and_metadata() {
        // Test parsing title + metadata for edit command
        let metadata = parse_slash_metadata("New task title /p high /tag updated").unwrap();
        
        assert_eq!(metadata.title, "New task title");
        assert_eq!(metadata.priority, Some("high".to_string()));
        assert_eq!(metadata.tags, vec!["updated"]);
    }
}