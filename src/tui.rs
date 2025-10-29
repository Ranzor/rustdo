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
                .highlight_style(Style::default().bg(Color::Blue));
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
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                    if key.code == KeyCode::Char('j') && !todos.is_empty() {
                        selected += 1;
                        if selected as usize > todos.len() - 1 {
                            selected = 0;
                        }
                        list_state.select(Some(selected as usize));
                    }

                    if key.code == KeyCode::Char('k') && !todos.is_empty() {
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
                        mode = Mode::Adding(String::new());
                        println!("In Add Mode");
                    }
                }
                Mode::Adding(_) => {
                    if key.code == KeyCode::Esc {
                        mode = Mode::Normal;
                        println!("In Normal Mode");
                    }
                }
            }
        }
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
