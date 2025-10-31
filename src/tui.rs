use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use crate::{Todo, save_todos};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io;

enum Mode {
    Normal,
    Adding(String),
    Editing(String),
    Commenting(String),
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
            let is_editing_task = matches!(mode, Mode::Editing(_));
            let is_adding_task = matches!(mode, Mode::Adding(_));
            let is_commenting = matches!(mode, Mode::Commenting(_));

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
                .enumerate()
                .map(|(i, todo)| {
                    let status = if todo.completed { "✓" } else { " " };

                    let task_text = if i == selected as usize && (is_editing_task || is_adding_task)
                    {
                        format!("{}_", todo.task)
                    } else {
                        todo.task.clone()
                    };

                    ListItem::new(format!("[{}] {}", status, task_text))
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
            .block(Block::default().borders(Borders::ALL).title("Details"))
            .wrap(Wrap { trim: false });
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
                    if key.code == KeyCode::Char('J') && key.modifiers.contains(KeyModifiers::SHIFT)
                    {
                        if (selected as usize) < todos.len() - 1 {
                            todos.swap(selected as usize, (selected + 1) as usize);
                            selected += 1;
                            list_state.select(Some(selected as usize));
                            let _ = save_todos(&todo_file, &todos);
                        }
                    }
                    if key.code == KeyCode::Char('K') && key.modifiers.contains(KeyModifiers::SHIFT)
                    {
                        // move task up
                        if selected > 0 {
                            todos.swap(selected as usize, (selected - 1) as usize);
                            selected -= 1;
                            list_state.select(Some(selected as usize));
                            let _ = save_todos(&todo_file, &todos);
                        }
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
                    if key.code == KeyCode::Char('e') && !todos.is_empty() {
                        let current_task = todos[selected as usize].task.clone();
                        mode = Mode::Editing(current_task);
                    }
                    if key.code == KeyCode::Char('d') && !todos.is_empty() {
                        todos.remove(selected as usize);
                        selected = (selected - 1).max(0);
                        list_state.select(Some(selected as usize));
                        let _ = save_todos(&todo_file, &todos);
                    }
                    if key.code == KeyCode::Char('c')
                        || key.code == KeyCode::Tab && !todos.is_empty()
                    {
                        let current_comment =
                            todos[selected as usize].comment.clone().unwrap_or_default();
                        mode = Mode::Commenting(current_comment);
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
                Mode::Editing(ref mut input) => match key.code {
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
                Mode::Commenting(ref mut input) => match key.code {
                    KeyCode::Char(c) => {
                        input.push(c);
                        todos[selected as usize].comment = Some(input.clone());
                    }
                    KeyCode::Backspace => {
                        input.pop();
                        if input.is_empty() {
                            todos[selected as usize].comment = None;
                        } else {
                            todos[selected as usize].comment = Some(input.clone());
                        }
                    }
                    KeyCode::Enter => {
                        input.push('\n');
                        todos[selected as usize].comment = Some(input.clone());
                    }
                    KeyCode::Esc => {
                        if input.trim().is_empty() {
                            todos[selected as usize].comment = None;
                        }
                        let _ = save_todos(&todo_file, &todos);
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
