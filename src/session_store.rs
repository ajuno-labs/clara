use crate::session::Session;
use std::{fs, io, path::PathBuf};

fn sessions_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("clara")
        .join("sessions.json")
}

fn active_session_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("clara")
        .join("active_session.json")
}

/// Load all completed sessions from storage
pub fn load() -> io::Result<Vec<Session>> {
    let path = sessions_path();
    if !path.exists() {
        return Ok(vec![]);
    }
    let data = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
}

/// Save all completed sessions to storage
pub fn save(sessions: &[Session]) -> io::Result<()> {
    let path = sessions_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(sessions)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, data)?;
    Ok(())
}

/// Load the currently active session if any
pub fn load_active() -> io::Result<Option<Session>> {
    let path = active_session_path();
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(path)?;
    let session = serde_json::from_str(&data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(Some(session))
}

/// Save the currently active session
pub fn save_active(session: &Session) -> io::Result<()> {
    let path = active_session_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(session)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, data)?;
    Ok(())
}

/// Remove the active session file (when session is completed)
pub fn clear_active() -> io::Result<()> {
    let path = active_session_path();
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// Add a completed session to the sessions list
pub fn add_session(session: Session) -> io::Result<()> {
    let mut sessions = load()?;
    sessions.push(session);
    save(&sessions)
}

/// Mark the active session as warned
pub fn mark_active_warned() -> io::Result<()> {
    if let Some(mut session) = load_active()? {
        session.warned = true;
        save_active(&session)?;
    }
    Ok(())
}