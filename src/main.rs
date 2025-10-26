use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct Todo {
    task: String,
    completed: bool,
}

fn main() {
    let todo_file = "todos.json";

    let mut todos: Vec<Todo> = if Path::new(todo_file).exists() {
        let data = fs::read_to_string(todo_file).expect("Unable to read todo file");
        serde_json::from_str(&data).expect("Unable to parse JSON")
    } else {
        Vec::new()
    };

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: todo <command>");
        println!("Commands: add, list, done");
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
            });
            println!("Added task: {}", task);

            match save_todos(todo_file, &todos) {
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

            match save_todos(todo_file, &todos) {
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

            match save_todos(todo_file, &todos) {
                Ok(()) => (),
                Err(msg) => {
                    println!("{}", msg);
                    return;
                }
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

            match save_todos(todo_file, &todos) {
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

            match save_todos(todo_file, &todos) {
                Ok(()) => (),
                Err(msg) => {
                    println!("{}", msg);
                    return;
                }
            }
        }
        _ => {
            println!("Unknown command: {}", command);
            println!("Available commands: add, list");
        }
    }
}

fn get_task_index(args: &[String], todos: &[Todo]) -> Result<usize, String> {
    // do stuff
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

fn save_todos(todo_file: &str, todos: &[Todo]) -> Result<(), String> {
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
