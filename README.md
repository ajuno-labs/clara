# Clara 📝

A powerful CLI productivity assistant for hierarchical task management.

## Features

✨ **Hierarchical Organization**
- 📁 **Folders** → 📋 **Lists** → ✅ **Tasks** with unlimited nesting
- Human-readable dot-path IDs: `F1-L2-T1`, `F1-L2-T1.2.1`

🌳 **Smart Display**
- Beautiful tree view with indentation and emojis
- Flat list view for quick scanning
- Filter by folder/list

🎯 **Powerful Commands**
- Create folders, lists, tasks, and subtasks
- Mark any task/subtask as done
- Intuitive CLI with help text

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
# Create workspace structure
clara folder add Work
clara folder lists add --folder F1 "Today"

# Add tasks and subtasks
clara add "Write proposal" --folder F1 --list F1-L2
clara subtask "Research competitors" --parent F1-L2-T1
clara subtask "Create outline" --parent F1-L2-T1
clara subtask "Draft introduction" --parent F1-L2-T1.2

# View tasks
clara list                    # All tasks
clara list --tree            # Tree view with nesting
clara list --folder F1       # Filter by folder

# Mark tasks complete
clara done F1-L2-T1.2.1     # Complete nested subtask
```

### Example Output

```
📁 Work > 📋 Today
---
🔲 [F1-L2-T1] Write proposal  (created 2025‑07‑26 13:20)
  🔲 [F1-L2-T1.1] Research competitors  (created 2025‑07‑26 13:20)
  🔲 [F1-L2-T1.2] Create outline  (created 2025‑07‑26 13:20)
    ✅ [F1-L2-T1.2.1] Draft introduction  (created 2025‑07‑26 13:20)
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

## Commands

### Folder Management
```bash
clara folder add <name>              # Create new folder
clara folder list                    # List all folders
```

### List Management  
```bash
clara folder lists add --folder <id> <name>    # Create list in folder
clara folder lists list --folder <id>          # List all lists in folder
```

### Task Management
```bash
clara add <title> --folder <id> --list <id>    # Create root task
clara subtask <title> --parent <task-id>       # Create subtask
clara done <task-id>                           # Mark task complete
```

### Display
```bash
clara list                           # Show all tasks (flat)
clara list --tree                   # Show all tasks (tree view)
clara list --folder <id>            # Filter by folder
clara list --folder <id> --list <id> --tree  # Filter and tree view
```

## Data Storage

Clara stores data in `~/.local/share/clara/workspace.json` as structured JSON with full hierarchy preserved.

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