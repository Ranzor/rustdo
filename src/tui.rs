use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use crate::{Todo, save_todos};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::io;

enum Mode {
    Normal,
    Adding(String),
}

pub fn run_tui(mut todos: Vec<Todo>, todo_file: String) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut mode = Mode::Normal;
    let mut list_state = ListState::default();
    let mut selected: i32 = 0;
    list_state.select(Some(selected as usize));

    loop {
        terminal.draw(|frame| {
            if todos.is_empty() {
                let text = Paragraph::new("Press 'a' to start adding tasks")
                    .block(Block::default().borders(Borders::ALL));
                frame.render_widget(text, frame.area());
                return;
            }

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(frame.area());

            let items: Vec<ListItem> = todos
                .iter()
                .map(|todo| {
                    let status = if todo.completed { "✓" } else { " " };
                    ListItem::new(format!("[{}] {}", status, todo.task))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Tasks"))
                .highlight_style(Style::default().bg(Color::LightCyan).fg(Color::Black));
            let text = Paragraph::new(match &todos[selected as usize].comment {
                Some(comment) => {
                    format!(
                        "Task: {}\n\nComment: {}",
                        todos[selected as usize].task, comment
                    )
                }
                None => {
                    format!("Task: {}\n\nNo Comment", todos[selected as usize].task)
                }
            })
            .block(Block::default().borders(Borders::ALL).title("Details"));
            frame.render_stateful_widget(list, chunks[0], &mut list_state);
            frame.render_widget(text, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match mode {
                Mode::Normal => {
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                        break;
                    }
                    if key.code == KeyCode::Char('j')
                        || key.code == KeyCode::Down && !todos.is_empty()
                    {
                        selected += 1;
                        if selected as usize > todos.len() - 1 {
                            selected = 0;
                        }
                        list_state.select(Some(selected as usize));
                    }

                    if key.code == KeyCode::Char('k')
                        || key.code == KeyCode::Up && !todos.is_empty()
                    {
                        selected -= 1;
                        if selected < 0 {
                            selected = todos.len() as i32 - 1;
                        }
                        list_state.select(Some(selected as usize));
                    }
                    if key.code == KeyCode::Char(' ') && !todos.is_empty() {
                        // do something
                        todos[selected as usize].completed = !todos[selected as usize].completed;

                        match save_todos(&todo_file, &todos) {
                            Ok(()) => (),
                            Err(msg) => {
                                println!("{}", msg);
                            }
                        }
                    }
                    if key.code == KeyCode::Char('a') {
                        todos.push(Todo {
                            task: String::new(),
                            completed: false,
                            comment: None,
                        });

                        selected = (todos.len() - 1) as i32;
                        list_state.select(Some(selected as usize));

                        mode = Mode::Adding(String::new());
                    }
                    if key.code == KeyCode::Char('d') && !todos.is_empty() {
                        todos.remove(selected as usize);
                        selected = (selected - 1).max(0);
                        list_state.select(Some(selected as usize));
                        let _ = save_todos(&todo_file, &todos);
                    }
                }
                Mode::Adding(ref mut input) => match key.code {
                    KeyCode::Char(c) => {
                        input.push(c);
                        todos[selected as usize].task = input.clone();
                    }
                    KeyCode::Backspace => {
                        input.pop();
                        todos[selected as usize].task = input.clone();
                    }
                    KeyCode::Enter | KeyCode::Esc => {
                        if input.trim().is_empty() {
                            todos.remove(selected as usize);
                            selected = (todos.len() as i32 - 1).max(0);
                            list_state.select(Some(selected as usize));
                        } else {
                            let _ = save_todos(&todo_file, &todos);
                        }
                        mode = Mode::Normal;
                    }
                    _ => {}
                },
            }
        }
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
