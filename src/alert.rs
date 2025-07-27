use std::io;

/// Display alert message in the REPL
pub fn alert_user(title: &str, message: &str) -> io::Result<()> {
    println!("\n🔔 {}: {}", title, message);
    Ok(())
}

/// Check for session alerts and display them in REPL
pub fn check_active_session() -> io::Result<()> {
    use chrono::Utc;

    if let Some(session) = crate::session_store::load_active()? {
        if let Some(target_end) = session.target_end {
            let now = Utc::now();

            // Load config for warning time
            let config = crate::pomodoro_config::load()?;
            let warn_before = config.warn_before_duration();

            // Check if we should warn and haven't already
            if !session.warned && target_end - now <= warn_before {
                let remaining_mins = (target_end - now).num_minutes().max(0);
                let message = if remaining_mins <= 0 {
                    format!("{} session ending now!", session.kind)
                } else {
                    format!(
                        "{} session ending in {} minute(s)!",
                        session.kind, remaining_mins
                    )
                };

                alert_user("Clara Timer", &message)?;
                
                // Mark as warned to prevent repeated notifications
                crate::session_store::mark_active_warned()?;
            }
        }
    }

    Ok(())
}