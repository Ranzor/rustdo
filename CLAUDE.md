# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

RustDo is a command-line todo list manager written in Rust. It supports both global and project-specific todo lists with a dual-list system where the application automatically selects the appropriate list based on the working directory.

## Build and Development Commands

```bash
# Build the project
cargo build

# Build release version
cargo build --release

# Run the application (development)
cargo run -- <command> [args]

# Run tests (when available)
cargo test

# Install locally for testing
cargo install --path .

# Check code without building
cargo check

# Format code
cargo fmt

# Run clipper for linting
cargo clippy
```

## Architecture

### Single-File Design
The entire application logic is contained in `src/main.rs`. This is intentional for simplicity and maintainability for a CLI tool of this scope.

### Core Data Structure
```rust
struct Todo {
    task: String,
    completed: bool,
}
```

### File Resolution Logic
The application uses a priority-based file resolution system (see main.rs:14-33):
1. If `global` or `g` command prefix is used → always use `~/.todos.json`
2. If `todos.json` exists in current directory → use local list
3. Otherwise → fall back to `~/.todos.json` (global list)

This resolution happens at startup before any command processing.

### Command Processing Pattern
Commands follow a consistent pattern in the match statement (main.rs:50-316):
1. Parse and validate arguments
2. Use helper functions (`get_task_index`, `save_todos`) for common operations
3. Provide user feedback with colored output
4. Handle errors inline with early returns

### Helper Functions
- `get_task_index()`: Validates and converts 1-indexed user input to 0-indexed Vec position
- `save_todos()`: Serializes and writes the todo list to disk with error handling

## Key Implementation Details

### Index Conversion
User-facing task numbers are 1-indexed for better UX, but internally converted to 0-indexed for Vec operations. This conversion happens in `get_task_index()`.

### State Management
The app loads the entire todo list into memory at startup, performs operations, and saves changes immediately after each modification. There is no concurrent access handling - the last write wins.

### Interactive Commands
Two commands have interactive modes:
- `edit` without a new task prompts for input interactively
- `clear` shows confirmation dialog unless `-y` flag is provided

### Global Command Modifier
The `global`/`g` prefix is stripped from args early in execution (main.rs:18-24) and only affects which file is selected. After removal, command processing is identical.

## Adding New Commands

To add a new command:
1. Add a new match arm in the main command handler (around main.rs:50)
2. Follow the existing pattern: validate args, perform operation, call `save_todos()`
3. Add the command to the help text in two places:
   - Usage message (main.rs:44)
   - Unknown command message (main.rs:314)
4. Consider adding a single-letter alias in the match pattern

## Common Patterns

### Reading User Input
```rust
io::stdout().flush().expect("Failed to flush stdout");
let mut input = String::new();
io::stdin().read_line(&mut input).expect("Failed to read input");
let input = input.trim();
```

### Colored Output
```rust
"✓".bright_green()  // Completed status
task.bright_cyan()  // Task text
"message".red()     // Errors/warnings
```

### Error Handling
The codebase uses early returns with `println!()` for user-facing errors rather than propagating Results through the call stack. This is intentional for CLI simplicity.

## Testing

When adding tests, note that file I/O operations interact with the actual filesystem. Consider:
- Using temporary directories for test files
- Testing the helper functions (`get_task_index`, `save_todos`) independently
- Mocking file operations for command tests
