use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
mod tui;

#[derive(Serialize, Deserialize)]
pub(crate) struct Todo {
    task: String,
    completed: bool,
    comment: Option<String>,
}

fn main() {
    let todo_file: String;
    let mut args: Vec<String> = env::args().collect();

    if args.len() > 1
        && (args[1].as_str().to_lowercase() == "global" || args[1].as_str().to_lowercase() == "g")
    {
        args.remove(1);
        let home = dirs::home_dir().expect("Could not find home directory");
        let global_path = home.join(".todos.json");
        todo_file = global_path.to_str().expect("Invalid path").to_string();
    } else {
        if Path::new("todos.json").exists() {
            todo_file = "todos.json".to_string();
        } else {
            let home = dirs::home_dir().expect("Could not find home directory");
            let global_path = home.join(".todos.json");
            todo_file = global_path.to_str().expect("Invalid path").to_string();
        }
    }

    let mut todos: Vec<Todo> = if Path::new(&todo_file).exists() {
        let data = fs::read_to_string(&todo_file).expect("Unable to read todo file");
        serde_json::from_str(&data).expect("Unable to parse JSON")
    } else {
        Vec::new()
    };

    if args.len() < 2 {
        println!("Usage: todo <command>");
        println!("Commands: add, remove, list, done, move, edit, clear, new, delete");
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "add" | "a" => {
            if args.len() < 3 {
                println!("Usage: todo add <task>");
                return;
            }
            let task = args[2..].join(" ");
            todos.push(Todo {
                task: task.clone(),
                completed: false,
                comment: None,
            });
            println!("Added task: {}", task);

            match save_todos(&todo_file, &todos) {
                Ok(()) => (),
                Err(msg) => {
                    println!("{}", msg);
                    return;
                }
            }
        }
        "remove" | "r" => {
            let index = match get_task_index(&args, &todos) {
                Ok(idx) => idx,
                Err(msg) => {
                    println!("{}", msg);
                    return;
                }
            };

            let removed = todos.remove(index);
            println!("Removed task: {}", removed.task);

            match save_todos(&todo_file, &todos) {
                Ok(()) => (),
                Err(msg) => {
                    println!("{}", msg);
                    return;
                }
            }
        }

        "list" | "l" => {
            if todos.is_empty() {
                println!("No tasks yet!");
            } else {
                for (i, todo) in todos.iter().enumerate() {
                    let status = if todo.completed {
                        "✓".bright_green()
                    } else {
                        " ".red()
                    };
                    println!("[{}] {}. {}", status, i + 1, todo.task.bright_cyan());
                }
            }
        }
        "done" | "d" => {
            let index = match get_task_index(&args, &todos) {
                Ok(idx) => idx,
                Err(msg) => {
                    println!("{}", msg);
                    return;
                }
            };

            todos[index].completed = !todos[index].completed;
            if todos[index].completed {
                println!(
                    "{}: {}",
                    "Completed".bright_green(),
                    todos[index].task.bright_cyan()
                );
            } else {
                println!(
                    "{}: {}",
                    "Marked as incomplete".red(),
                    todos[index].task.bright_cyan()
                );
            }

            match save_todos(&todo_file, &todos) {
                Ok(()) => (),
                Err(msg) => {
                    println!("{}", msg);
                    return;
                }
            }
        }

        "move" | "m" => {
            if args.len() < 4 {
                println!("Usage: rustdo move <from> <to>");
                return;
            }
            let index_from = match get_task_index(&args, &todos) {
                Ok(idx) => idx,
                Err(msg) => {
                    println!("{}", msg);
                    return;
                }
            };

            let mut index_to: usize = match args[3].parse() {
                Ok(num) => num,
                Err(_) => {
                    println!("Please provide a valid task number.");
                    return;
                }
            };

            if index_to == 0 || index_to > todos.len() {
                println!("Invalid task number. use 'list' to see available tasks.");
                return;
            }
            index_to -= 1;

            let task = todos.remove(index_from);
            todos.insert(index_to, task);

            if let Err(msg) = save_todos(&todo_file, &todos) {
                println!("{}", msg);
                return;
            }
        }

        "edit" | "e" => {
            let index = match get_task_index(&args, &todos) {
                Ok(idx) => idx,
                Err(msg) => {
                    println!("{}", msg);
                    return;
                }
            };

            if args.len() < 4 {
                println!("Editing task: {}", todos[index].task);
                io::stdout().flush().expect("Failed to flush stdout");
                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to... read input?");
                let input = input.trim();
                if input.is_empty() {
                    println!("Edit cancelled");
                    return;
                }
                todos[index].task = input.to_string();
            } else {
                let task = args[3..].join(" ");
                todos[index].task = task;
                println!("Updated task: {}", todos[index].task);
            }

            match save_todos(&todo_file, &todos) {
                Ok(()) => (),
                Err(msg) => {
                    println!("{}", msg);
                    return;
                }
            }
        }

        "clear" | "c" => {
            let completed: Vec<_> = todos.iter().filter(|todo| todo.completed).collect();

            if completed.is_empty() {
                println!("Nothing to clear");
                return;
            }

            let skip_confirm = args.len() > 2 && args[2] == "-y";

            if !skip_confirm {
                // show the task and get confirmation
                println!("The following tasks will be removed:");

                for (_i, todo) in completed.iter().enumerate() {
                    let status = if todo.completed {
                        "✓".bright_green()
                    } else {
                        " ".red()
                    };
                    println!("[{}] {}", status, todo.task.bright_cyan());
                }
                println!("Remove these {} tasks? (Y/n)", completed.len());
                loop {
                    io::stdout().flush().expect("Failed to flush stdout");
                    let mut input = String::new();
                    io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to... read input?");

                    match input.trim().to_lowercase().as_str() {
                        "y" | "yes" | "" => break,
                        "n" | "no" => {
                            println!("Cancelled");
                            return;
                        }
                        _ => println!("Invalid input. Please enter y or n."),
                    }
                }
            }

            todos.retain(|todo| !todo.completed);

            match save_todos(&todo_file, &todos) {
                Ok(()) => (),
                Err(msg) => {
                    println!("{}", msg);
                    return;
                }
            }
        }

        "new" | "n" => {
            let new_file = "todos.json".to_string();
            let todos: Vec<Todo> = if Path::new(&new_file).exists() {
                println!("local todo list already exists");
                return;
            } else {
                println!("local todo list created");
                Vec::new()
            };
            match save_todos(&new_file, &todos) {
                Ok(()) => (),
                Err(msg) => {
                    println!("{}", msg);
                    return;
                }
            }
        }

        "delete" => {
            println!("Delete todo list (y/N)");
            loop {
                io::stdout().flush().expect("Failed to flush stdout");
                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to... read input?");

                match input.trim().to_lowercase().as_str() {
                    "y" | "yes" => {
                        match fs::remove_file(&todo_file) {
                            Ok(()) => {
                                println!("Todo list deleted");
                                return;
                            }
                            Err(_) => {
                                println!("Failed to delete file");
                            }
                        };
                    }
                    "n" | "no" | "" => {
                        println!("Cancelled");
                        return;
                    }
                    _ => println!("Invalid input. Please enter y or n."),
                }
            }
        }
        "tui" | "t" => {
            if let Err(_) = tui::run_tui(todos, todo_file.clone()) {
                println!("Failed to enter TUI mode");
            }
        }

        _ => {
            println!("Unknown command: {}", command);
            println!("Available commands: add, remove, list, done, move, edit, clear, new, delete");
        }
    }
}

fn get_task_index(args: &[String], todos: &[Todo]) -> Result<usize, String> {
    if args.len() < 3 {
        return Err("Please provide a task number".to_string());
    }

    let task_num: usize = match args[2].parse() {
        Ok(num) => num,
        Err(_) => {
            return Err("Please provide a valid task number.".to_string());
        }
    };
    if task_num == 0 || task_num > todos.len() {
        return Err("Invalid task number. use 'list' to see available tasks.".to_string());
    }

    Ok(task_num - 1)
}

pub(crate) fn save_todos(todo_file: &str, todos: &[Todo]) -> Result<(), String> {
    let json = match serde_json::to_string_pretty(&todos) {
        Ok(t) => t,
        Err(_) => {
            return Err("Failed to serialize todos".to_string());
        }
    };
    match fs::write(todo_file, json) {
        Ok(()) => (),
        Err(_) => {
            return Err("Failed to write todo file".to_string());
        }
    }
    Ok(())
}
