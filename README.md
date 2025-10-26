# RustDo

A fast, efficient command-line todo list manager written in Rust. Supports both global and project-specific todo lists with a clean, colorful interface.

## Features

- **Simple, intuitive commands** with short aliases for speed
- **Local and global todo lists** - maintain project-specific lists and a universal list
- **Automatic fallback** - uses local list if present, otherwise falls back to global
- **Toggle completion status** - mark tasks done or undone with a single command
- **Reorder tasks** - move tasks to adjust priority
- **Interactive editing** - edit tasks inline or interactively
- **Smart confirmations** - destructive operations require confirmation with sensible defaults
- **Colorful output** - easy-to-scan task lists with visual indicators

## Installation

```bash
# Clone the repository
git clone https://github.com/Ranzor/rustdo.git
cd rustdo

# Build the project
cargo build --release

# Optionally, install to your system
cargo install --path .
```

## Quick Start

```bash
# Add a task
rustdo add "Write documentation"

# List all tasks
rustdo list

# Mark a task as done
rustdo done 1

# Edit a task
rustdo edit 2 "Updated task description"

# Remove a task
rustdo remove 1

# Clear all completed tasks
rustdo clear
```

## Commands

### Core Commands

| Command | Alias | Description | Example |
|---------|-------|-------------|---------|
| `add` | `a` | Add a new task | `rustdo add "Buy groceries"` |
| `list` | `l` | List all tasks | `rustdo list` |
| `done` | `d` | Toggle task completion | `rustdo done 1` |
| `remove` | `r` | Remove a task | `rustdo remove 2` |
| `edit` | `e` | Edit a task | `rustdo edit 3 "New description"` |
| `move` | `m` | Reorder tasks | `rustdo move 5 1` |
| `clear` | `c` | Clear completed tasks | `rustdo clear` |

### File Management

| Command | Alias | Description |
|---------|-------|-------------|
| `new` | `n` | Create a local todo list in current directory |
| `delete` | - | Delete the active todo list (with confirmation) |
| `global <command>` | `g <command>` | Execute command on global list explicitly |

### Interactive Features

**Edit without arguments** - Opens interactive mode:
```bash
rustdo edit 1
# Displays current task and prompts for new text
# Press Enter with no input to cancel
```

**Clear with confirmation** - Shows tasks being removed:
```bash
rustdo clear
# Lists completed tasks and asks for confirmation
# Press Enter to confirm, 'n' to cancel
```

**Clear without confirmation**:
```bash
rustdo clear -y
# Skips confirmation prompt
```

## Local vs Global Lists

RustDo supports both project-specific and global todo lists:

### Default Behavior
- If a `todos.json` exists in the current directory, it's used (local list)
- Otherwise, falls back to `~/.todos.json` (global list)

### Creating a Local List
```bash
cd /path/to/project
rustdo new
# Creates todos.json in current directory
```

### Accessing Global List from a Project Directory
```bash
# When in a directory with a local list, use 'global' to access global
rustdo global add "Personal task"
rustdo global list
```

### Example Workflow
```bash
# Working on a project
cd ~/projects/my-app
rustdo new                    # Create project-specific list
rustdo add "Fix login bug"    # Added to local list
rustdo list                   # Shows project tasks

# Add something to global list without leaving project
rustdo global add "Call dentist"

# In any other directory
cd ~
rustdo list                   # Shows global list
```

## Visual Design

Tasks are displayed with color-coded status indicators:
- ✓ **Green checkmark** - Completed tasks
- **Cyan text** - Task descriptions

Status messages use color for clarity:
- **Green** - Success messages ("Completed: Task name")
- **Red** - Warnings or cancellations ("Marked as incomplete")

## Implementation Details

### File Format
Todo lists are stored as JSON files with a simple structure:
```json
[
  {
    "task": "Example task",
    "completed": false
  }
]
```

### File Locations
- **Local lists**: `todos.json` in the current directory
- **Global list**: `~/.todos.json` in the user's home directory

## Development

Built with Rust, leveraging:
- `serde` for JSON serialization
- `colored` for terminal output
- `dirs` for cross-platform path handling

### Project Structure
- Simple, maintainable codebase with extracted helper functions
- Proper error handling using Rust's `Result` type
- Clean separation between commands and core logic

## Tips & Tricks

**Use aliases for speed**:
```bash
rustdo a "Quick task"    # Add
rustdo l                  # List
rustdo d 1                # Done
```

**Chain operations**:
```bash
rustdo a "Task 1" && rustdo a "Task 2" && rustdo l
```

**Quick reordering**:
```bash
# Move task 5 to the top
rustdo m 5 1
```

**Edit inline for simple changes**:
```bash
rustdo e 2 "Fixed typo in task"
```

## License

GPL-2.0

## Contributing

Contributions welcome! Feel free to open issues or submit pull requests.
