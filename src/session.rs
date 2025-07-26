use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Kind {
    Focus,
    Break,
    Meeting,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
    pub id: String,              // "S20250726T174501Z"
    pub kind: Kind,
    pub task_id: Option<String>, // dot-path ID, or None for uncoupled
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
}

impl Session {
    /// Generate a new session ID using current timestamp
    pub fn generate_id() -> String {
        format!("S{}", Utc::now().format("%Y%m%dT%H%M%SZ"))
    }
    
    /// Calculate duration if session has ended
    pub fn duration(&self) -> Option<chrono::Duration> {
        self.end.map(|end| end - self.start)
    }
    
    /// Check if session is currently active (no end time)
    pub fn is_active(&self) -> bool {
        self.end.is_none()
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Focus => write!(f, "Focus"),
            Kind::Break => write!(f, "Break"),
            Kind::Meeting => write!(f, "Meeting"),
        }
    }
}

impl std::str::FromStr for Kind {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "focus" => Ok(Kind::Focus),
            "break" => Ok(Kind::Break),
            "meeting" => Ok(Kind::Meeting),
            _ => Err(format!("Invalid session kind: {}", s)),
        }
    }
}