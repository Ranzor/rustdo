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

            let json =
                serde_json::to_string_pretty(&todos).expect("Unable to serialize todos to JSON");
            fs::write(todo_file, json).expect("Unable to write todo file");
        }
        "remove" | "r" => {
            if args.len() < 3 {
                println!("Please provide the task number to remove.");
                return;
            }

            let task_num: usize = match args[2].parse() {
                Ok(num) => num,
                Err(_) => {
                    println!("Please provide a valid task number.");
                    return;
                }
            };
            if task_num == 0 || task_num > todos.len() {
                println!("Invalid task number. Use 'list' to see available tasks.");
                return;
            }
            let index = task_num - 1;

            let removed = todos.remove(index);
            println!("Removed task: {}", removed.task);

            let json = serde_json::to_string_pretty(&todos).expect("Failed to serialize todos");
            fs::write(todo_file, json).expect("Failed to write todo file");
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
            if args.len() < 3 {
                println!("Usage: todo complete <task_number>");
                return;
            }
            let task_num: usize = match args[2].parse() {
                Ok(num) => num,
                Err(_) => {
                    println!("Please provide a valid task number.");
                    return;
                }
            };

            if task_num == 0 || task_num > todos.len() {
                println!("Invalid task number. Use 'list' to see available tasks.");
                return;
            }
            let index = task_num - 1;

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

            let json = serde_json::to_string_pretty(&todos).expect("Failed to serialize todos");
            fs::write(todo_file, json).expect("Failed to write todo file");
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

                for (i, todo) in completed.iter().enumerate() {
                    let status = if todo.completed {
                        "✓".bright_green()
                    } else {
                        " ".red()
                    };
                    println!("[{}] {}", status, todo.task.bright_cyan());
                }
                println!("Remove these {} tasks? (y/n)", completed.len());
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

            let json = serde_json::to_string_pretty(&todos).expect("Failed to serialize todos");
            fs::write(todo_file, json).expect("Failed to write todo file");
        }
        _ => {
            println!("Unknown command: {}", command);
            println!("Available commands: add, list");
        }
    }
}
