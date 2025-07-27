use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PomodoroCfg {
    #[serde(default = "default_focus_minutes")]
    pub focus_minutes: i64,
    #[serde(default = "default_break_minutes")]
    pub break_minutes: i64,
    #[serde(default = "default_meeting_minutes")]
    pub meeting_minutes: i64,
    #[serde(default = "default_warn_before_secs")]
    pub warn_before_secs: i64,
    #[serde(default = "default_extend_minutes")]
    pub extend_minutes: i64,
    #[serde(default)]
    pub sound_file: Option<String>,
}

fn default_focus_minutes() -> i64 { 25 }
fn default_break_minutes() -> i64 { 5 }
fn default_meeting_minutes() -> i64 { 30 }
fn default_warn_before_secs() -> i64 { 60 }
fn default_extend_minutes() -> i64 { 5 }

impl Default for PomodoroCfg {
    fn default() -> Self {
        Self {
            focus_minutes: default_focus_minutes(),
            break_minutes: default_break_minutes(),
            meeting_minutes: default_meeting_minutes(),
            warn_before_secs: default_warn_before_secs(),
            extend_minutes: default_extend_minutes(),
            sound_file: None,
        }
    }
}

impl PomodoroCfg {
    pub fn focus_duration(&self) -> Duration {
        Duration::minutes(self.focus_minutes)
    }
    
    pub fn break_duration(&self) -> Duration {
        Duration::minutes(self.break_minutes)
    }
    
    pub fn meeting_duration(&self) -> Duration {
        Duration::minutes(self.meeting_minutes)
    }
    
    pub fn warn_before_duration(&self) -> Duration {
        Duration::seconds(self.warn_before_secs)
    }
    
    pub fn extend_duration(&self) -> Duration {
        Duration::minutes(self.extend_minutes)
    }
    
    pub fn duration_for_kind(&self, kind: &crate::session::Kind) -> Duration {
        match kind {
            crate::session::Kind::Focus => self.focus_duration(),
            crate::session::Kind::Break => self.break_duration(),
            crate::session::Kind::Meeting => self.meeting_duration(),
        }
    }
}

fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("clara")
}

fn config_path() -> PathBuf {
    config_dir().join("settings.toml")
}

pub fn load() -> io::Result<PomodoroCfg> {
    let path = config_path();
    
    if !path.exists() {
        // Create default config file
        let default_cfg = PomodoroCfg::default();
        save(&default_cfg)?;
        return Ok(default_cfg);
    }
    
    let content = fs::read_to_string(path)?;
    let cfg: PomodoroCfg = toml::from_str(&content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(cfg)
}

pub fn save(cfg: &PomodoroCfg) -> io::Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let _content = toml::to_string_pretty(cfg)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    
    // Add helpful comments to the config file
    let content_with_comments = format!(
r#"# Clara Pomodoro Configuration
# Duration settings in minutes
focus_minutes = {}
break_minutes = {}
meeting_minutes = {}

# Alert settings
warn_before_secs = {}  # seconds before session end to show warning
extend_minutes = {}    # minutes added when extending a session

# Optional sound file (uncomment and set path to enable)
# sound_file = "/path/to/ding.ogg"
"#,
        cfg.focus_minutes,
        cfg.break_minutes,
        cfg.meeting_minutes,
        cfg.warn_before_secs,
        cfg.extend_minutes
    );
    
    fs::write(path, content_with_comments)?;
    Ok(())
}