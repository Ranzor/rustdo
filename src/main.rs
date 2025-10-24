use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
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
        println!("Commands: add, list");
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "add" => {
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
        "list" => {
            if todos.is_empty() {
                println!("No tasks yet!");
            } else {
                for (i, todo) in todos.iter().enumerate() {
                    let status = if todo.completed { "[✓]" } else { "[ ]" };
                    println!("[{}] {}. {}", status, i + 1, todo.task);
                }
            }
        }
        _ => {
            println!("Unknown command: {}", command);
            println!("Available commands: add, list");
        }
    }
}
