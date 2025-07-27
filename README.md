# Clara 📝

A powerful interactive productivity assistant for hierarchical task management with integrated time tracking.

## Features

✨ **Interactive REPL Interface**
- Real-time session timer display in prompt
- No CLI arguments needed - launches directly into interactive mode
- Intuitive commands with help system
- Graceful WSL/Linux compatibility

📊 **Hierarchical Organization**
- 📁 **Folders** → 📋 **Lists** → ✅ **Tasks** with unlimited nesting
- Human-readable dot-path IDs: `F1-L2-T1`, `F1-L2-T1.2.1`

🌳 **Smart Display**
- Beautiful tree view with indentation and emojis
- Flat list view for quick scanning
- Filter by folder/list

⏱️ **Pomodoro-Style Time Tracking**
- Real-time timer display in REPL prompt
- Session types: Focus (25min), Break (5min), Meeting (60min) 
- Link sessions to specific tasks for detailed analytics
- Active session management with crash recovery
- Extension support with custom durations
- In-terminal notifications and alerts
- Configurable timing via `~/.config/clara/settings.toml`

## Quick Start

### Installation

```bash
# Clone and build
git clone https://github.com/ajuno-labs/clara.git
cd clara
cargo build --release

# Install binary
cargo install --path .
```

### Basic Usage

```bash
# Launch Clara (interactive mode only)
clara

# Interactive REPL commands:
🎯 Clara Interactive Mode
Type '/help' for commands, '/exit' to quit

clara [⏸️ No active session]> folder add Work
📁 Folder created!

clara [⏸️ No active session]> folder lists add --folder F1 "Today"
📋 List created!

clara [⏸️ No active session]> add "Write proposal" --folder F1 --list F1-L2
✅ Task saved!

clara [⏸️ No active session]> track start --task F1-L2-T1
🎯 Started Focus session S20250727T... (F1-L2-T1) - ends at 14:32

clara [🔵 Focus 24m left]> list --tree
📁 Work > 📋 Today
---
🔲 [F1-L2-T1] Write proposal  (created 2025‑07‑27 13:20)

clara [🔵 Focus 15m left]> track extend --minutes 10
⏰ Extended Focus session by 10 minute(s) - new end time: 14:42

clara [🔵 Focus 25m left]> /exit
👋 Goodbye!
```

### Timer Display States

The REPL prompt shows your current session status in real-time:

```bash
clara [⏸️ No active session]>           # No timer running
clara [🔵 Focus 23m left]>              # Time remaining in session
clara [🔴 Focus OVERTIME +5m]>          # Session has overrun
clara [⏳ Meeting 1h30m]>               # Running time (no target)
```

### In-Terminal Notifications

When your session is ending, Clara displays alerts directly in the REPL:

```bash
clara [🔵 Focus 2m left]> 

🔔 Clara Timer: Focus session ending in 2 minute(s)!

clara [🔵 Focus 1m left]> track extend
⏰ Extended Focus session by 5 minute(s) - new end time: 14:47
```

## Architecture

Clara uses a three-level hierarchy:

```
Workspace
└─ Folder (F1, F2, ...)
   └─ List (F1-L1, F1-L2, ...)
      └─ Task (F1-L2-T1, F1-L2-T2, ...)
         └─ Subtask (F1-L2-T1.1, F1-L2-T1.2, ...)
            └─ Sub-subtask (F1-L2-T1.2.1, ...)
```

- **Dot-path IDs** make tasks easy to reference and understand
- **Unlimited nesting** for complex project breakdowns
- **Fast lookup** with recursive navigation helpers

## Interactive Commands

All commands are used within the Clara REPL. Type `/help` for a quick reference.

### Folder Management
```bash
folder add <name>              # Create new folder
folder list                    # List all folders
```

### List Management  
```bash
folder lists add --folder <id> <name>    # Create list in folder
folder lists list --folder <id>          # List all lists in folder
```

### Task Management
```bash
add <title> --folder <id> --list <id>    # Create root task
subtask <title> --parent <task-id>       # Create subtask
done <task-id>                           # Mark task complete
```

### Display
```bash
list                           # Show all tasks (flat)
list --tree                   # Show all tasks (tree view)
list --folder <id>            # Filter by folder
list --folder <id> --list <id> --tree  # Filter and tree view
```

### Pomodoro Time Tracking
```bash
track start                               # Start 25min focus session
track start --kind <type>                # Session types: focus/break/meeting  
track start --task <task-id>             # Link session to task
track start --duration <minutes>         # Custom duration override
track start --kind focus --task F1-L2-T1 --duration 45  # Combined options
track current                            # Show active session status
track extend                             # Extend by 5min (default)
track extend --minutes <minutes>         # Extend by custom duration
track stop                               # Complete active session
```

### Special Commands
```bash
/help                          # Show command help
/exit or /quit                 # Exit Clara
```

**Session Types & Default Durations:**
- `focus`: 25 minutes (classic Pomodoro)
- `break`: 5 minutes (short break)
- `meeting`: 60 minutes (longer sessions)

## Data Storage

Clara stores data locally in structured files:

**Data Directory:** `~/.local/share/clara/`
- **`workspace.json`** - Task hierarchy with full folder/list/task structure
- **`sessions.json`** - Completed time tracking sessions with Pomodoro metadata
- **`active_session.json`** - Current active session (crash recovery)

**Config Directory:** `~/.config/clara/`
- **`settings.toml`** - Pomodoro timing configuration (auto-generated with defaults)

### Configuration

Clara automatically creates `~/.config/clara/settings.toml` on first use:

```toml
# Pomodoro session durations (minutes)
focus_duration = 25
break_duration = 5  
meeting_duration = 60

# Extension settings
extend_duration = 5       # Default extension time
warn_before_minutes = 2   # Alert before session ends
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

MIT License - see [LICENSE](LICENSE) file for details.

---

**Built with ❤️ using Rust 🦀**
