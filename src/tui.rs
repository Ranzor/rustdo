use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use crate::{Todo, save_todos};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::fs;
use std::io;
use std::path::Path;

enum Mode {
    Normal,
    Adding(String),
    Editing(String),
    Commenting(String),
    FilePicker { selected: usize },
}

pub fn run_tui(mut todos: Vec<Todo>, mut todo_file: String) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let active_style = Style::default().fg(Color::Cyan);
    let inactive_style = Style::default().fg(Color::DarkGray);

    let mut mode = Mode::Normal;
    let mut list_state = ListState::default();
    let mut selected: i32 = 0;
    list_state.select(Some(selected as usize));

    // temp file list
    let available_files = vec![
        "/.todos.json",
        "/todos.json",
        "~/projects/website/todos.json",
    ];

    loop {
        terminal.draw(|frame| {
            let is_editing_task = matches!(mode, Mode::Editing(_));
            let is_adding_task = matches!(mode, Mode::Adding(_));
            let is_commenting = matches!(mode, Mode::Commenting(_));

            let tasks_border_style = if is_commenting {
                inactive_style
            } else {
                active_style
            };

            let details_border_style = if is_commenting {
                active_style
            } else {
                inactive_style
            };

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
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Tasks")
                        .border_style(tasks_border_style),
                )
                .highlight_style(Style::default().bg(Color::LightCyan).fg(Color::Black));
            let text = Paragraph::new(match &todos[selected as usize].comment {
                Some(comment) => {
                    let comment_display = if is_commenting {
                        format!("{}_", comment)
                    } else {
                        comment.clone()
                    };
                    format!(
                        "Task: {}\n\nComment: {}",
                        todos[selected as usize].task, comment_display
                    )
                }
                None => {
                    let comment_display = if is_commenting {
                        "_".to_string()
                    } else {
                        "No Comment".to_string()
                    };
                    format!(
                        "Task: {}\n\nComment: {}",
                        todos[selected as usize].task, comment_display
                    )
                }
            })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Details")
                    .border_style(details_border_style),
            )
            .wrap(Wrap { trim: false });
            frame.render_stateful_widget(list, chunks[0], &mut list_state);
            frame.render_widget(text, chunks[1]);

            if matches!(mode, Mode::FilePicker { selected: _ }) {
                let popup_area = centered_rect(60, 50, frame.area());

                frame.render_widget(Clear, popup_area);

                let popup = Paragraph::new("File Picker!\n\nPress Esc to close").block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Switch List")
                        .border_style(Style::default().fg(Color::Yellow)),
                );
                frame.render_widget(popup, popup_area);
            }
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
                    if key.code == KeyCode::Char('n') {
                        let new_file = "todos.json";

                        if !Path::new(new_file).exists() {
                            let empty: Vec<Todo> = Vec::new();
                            let _ = save_todos(new_file, &empty);
                        }

                        todos = load_todos(new_file)?;
                        todo_file = new_file.to_string();
                        selected = 0;
                        list_state.select(Some(selected as usize));
                    }
                    if key.code == KeyCode::Char('f') {
                        mode = Mode::FilePicker { selected: 0 };
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
                Mode::FilePicker { selected: 0 } => {
                    // Future implementation for file picker
                    if key.code == KeyCode::Esc {
                        mode = Mode::Normal;
                    }
                }
                _ => {}
            }
        }
    }
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
fn load_todos(file_path: &str) -> io::Result<Vec<Todo>> {
    if Path::new(file_path).exists() {
        let data = fs::read_to_string(file_path)?;
        Ok(serde_json::from_str(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?)
    } else {
        Ok(Vec::new())
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_width = area.width * percent_x / 100;
    let popup_height = area.height * percent_y / 100;

    let x = (area.width - popup_width) / 2;
    let y = (area.height - popup_height) / 2;

    Rect {
        x: area.x + x,
        y: area.y + y,
        width: popup_width,
        height: popup_height,
    }
}
