//! Interactive terminal application for tracking meeting costs.
// main.rs
#![warn(clippy::pedantic)]

use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use meeting_cost_tracker::{
    load_attendees, load_categories, save_attendees, save_categories, AttendeeInfo,
    EmployeeCategory, Meeting,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Terminal;

/// Returns the directory where persistent data should be stored.
fn data_dir() -> PathBuf {
    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let mut dir = exe_path
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    dir.push("data");
    dir
}

/// Calculates a centered rectangle taking up the given percentage of the parent area.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

/// UI modes controlling user interaction.
enum Mode {
    /// Normal viewing mode where meeting stats are displayed.
    View,
    /// Mode for creating a new [`EmployeeCategory`].
    AddCategory,
    /// Mode for deleting an existing [`EmployeeCategory`].
    DeleteCategory,
    /// Mode for selecting a category when adding attendees.
    AddAttendeeSelect,
    /// Mode for entering the attendee count after selecting a category.
    AddAttendeeCount,
    /// Mode for removing attendees from the [`Meeting`].
    RemoveAttendee,
    /// Mode for saving attendees to disk.
    SaveAttendees,
    /// Mode for loading attendees from disk.
    LoadAttendees,
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn render_ui(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    meeting: &Meeting,
    categories: &[EmployeeCategory],
    mode: &Mode,
    input_text: &str,
    show_salaries: bool,
    files: &[String],
    selected: usize,
) -> std::io::Result<()> {
    terminal.draw(|f| {
        let size = f.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),  // title
                Constraint::Length(1),  // status line
                Constraint::Length(3),  // cost display
                Constraint::Min(1),     // lists
                Constraint::Length(3),  // input/help
            ])
            .split(size);

        let title = Paragraph::new("Meeting Cost Tracker")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
        f.render_widget(title, chunks[0]);

        let running = meeting.is_running();
        let duration = meeting.duration();
        let cost = meeting.total_cost();
        let cost_display = if cost == 0.0 { 0.0 } else { cost };

        let status = Paragraph::new(Line::from(vec![
            Span::styled(
                format!(
                    "[{}] Duration: {}",
                    if running { "Running" } else { "Stopped" },
                    format_duration(duration)
                ),
                Style::default()
                    .fg(if running { Color::Green } else { Color::Red })
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        f.render_widget(status, chunks[1]);

        let cost_widget = Paragraph::new(Line::from(Span::styled(
            format!("${cost_display:.2}"),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )))
        .alignment(Alignment::Center);
        f.render_widget(cost_widget, chunks[2]);

        match mode {
            Mode::AddCategory => {
                let input_widget = Paragraph::new(input_text)
                    .block(Block::default().title("Enter: Title:Salary").borders(Borders::ALL));
                f.render_widget(input_widget, chunks[4]);
            }
            Mode::View => {
                let help = Paragraph::new(Line::from(vec![
                    Span::styled(
                        "[s] Start/Stop  [c] Reset  [a] Add Category  [d] Delete Category  [e] Add Employee  [r] Remove Employee  [w] Save Attendees  [l] Load Attendees  [p] Toggle Salaries [q] Quit",
                        Style::default().fg(Color::Yellow),
                    ),
                ]))
                .block(Block::default().borders(Borders::ALL).title("Controls"));
                f.render_widget(help, chunks[4]);
            }
            Mode::AddAttendeeSelect => {
                let input_widget = Paragraph::new("")
                    .block(
                        Block::default()
                            .title("Select category to add")
                            .borders(Borders::ALL),
                    );
                f.render_widget(input_widget, chunks[4]);
            }
            Mode::AddAttendeeCount => {
                let input_widget = Paragraph::new(input_text)
                    .block(
                        Block::default()
                            .title("Enter attendee count")
                            .borders(Borders::ALL),
                    );
                f.render_widget(input_widget, chunks[4]);
            }
            Mode::RemoveAttendee => {
                let input_widget = Paragraph::new("")
                    .block(Block::default().title("Select attendee to remove").borders(Borders::ALL));
                f.render_widget(input_widget, chunks[4]);
            }
            Mode::SaveAttendees => {
                let input_widget = Paragraph::new(input_text)
                    .block(Block::default().title("Enter filename to save").borders(Borders::ALL));
                f.render_widget(input_widget, chunks[4]);
            }
            Mode::LoadAttendees => {
                let input_widget = Paragraph::new(input_text)
                    .block(Block::default().title("Enter filename to load").borders(Borders::ALL));
                f.render_widget(input_widget, chunks[4]);
            }
            Mode::DeleteCategory => {
                let input_widget = Paragraph::new("")
                    .block(Block::default().title("Select category to delete").borders(Borders::ALL));
                f.render_widget(input_widget, chunks[4]);
            }
        }

        let lists = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[3]);

        let category_list: Vec<Line> = categories
            .iter()
            .map(|c| {
                let text = if show_salaries {
                    format!("{}: ${}", c.title(), c.salary())
                } else {
                    c.title().to_string()
                };
                Line::from(Span::styled(text, Style::default().fg(Color::Cyan)))
            })
            .collect();
        let list_widget = Paragraph::new(category_list)
            .block(Block::default().borders(Borders::ALL).title("Employee Categories"));
        f.render_widget(list_widget, lists[1]);

        let meeting_list: Vec<Line> = meeting
            .attendees()
            .map(|(title, _salary, count)| {
                Line::from(Span::styled(
                    format!("{title} x {count}"),
                    Style::default().fg(Color::Magenta),
                ))
            })
            .collect();
        let meeting_widget = Paragraph::new(meeting_list)
            .block(Block::default().borders(Borders::ALL).title("Current Meeting"));
        f.render_widget(meeting_widget, lists[0]);

        if matches!(
            mode,
            Mode::LoadAttendees | Mode::DeleteCategory | Mode::RemoveAttendee | Mode::AddAttendeeSelect
        ) {
            let area = centered_rect(50, 50, size);
            let (title, items): (&str, Vec<Line>) = match mode {
                Mode::LoadAttendees => {
                    let items: Vec<Line> = if files.is_empty() {
                        vec![Line::from(Span::raw("No attendee files found"))]
                    } else {
                        files
                            .iter()
                            .enumerate()
                            .map(|(i, name)| {
                                let style = if i == selected {
                                    Style::default().add_modifier(Modifier::REVERSED)
                                } else {
                                    Style::default()
                                };
                                Line::from(Span::styled(name.clone(), style))
                            })
                            .collect()
                    };
                    ("Load attendees", items)
                }
                Mode::DeleteCategory => {
                    let items: Vec<Line> = categories
                        .iter()
                        .enumerate()
                        .map(|(i, cat)| {
                            let style = if i == selected {
                                Style::default().add_modifier(Modifier::REVERSED)
                            } else {
                                Style::default()
                            };
                            Line::from(Span::styled(cat.title().to_string(), style))
                        })
                        .collect();
                    ("Delete category", items)
                }
                Mode::RemoveAttendee => {
                    let items: Vec<Line> = meeting
                        .attendees()
                        .enumerate()
                        .map(|(i, (title, _salary, count))| {
                            let style = if i == selected {
                                Style::default().add_modifier(Modifier::REVERSED)
                            } else {
                                Style::default()
                            };
                            Line::from(Span::styled(format!("{title} x {count}"), style))
                        })
                        .collect();
                    ("Remove attendee", items)
                }
                Mode::AddAttendeeSelect => {
                    let items: Vec<Line> = categories
                        .iter()
                        .enumerate()
                        .map(|(i, cat)| {
                            let style = if i == selected {
                                Style::default().add_modifier(Modifier::REVERSED)
                            } else {
                                Style::default()
                            };
                            Line::from(Span::styled(cat.title().to_string(), style))
                        })
                        .collect();
                    ("Add attendee", items)
                }
                _ => unreachable!(),
            };
            let popup = Paragraph::new(items)
                .block(Block::default().title(title).borders(Borders::ALL));
            f.render_widget(Clear, area);
            f.render_widget(popup, area);
        }
    })?;
    Ok(())
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)]
fn process_key(
    key_event: crossterm::event::KeyEvent,
    mode: &mut Mode,
    input_text: &mut String,
    show_salaries: &mut bool,
    categories: &mut Vec<EmployeeCategory>,
    meeting: &mut Meeting,
    files: &mut Vec<String>,
    selected: &mut usize,
    add_attendee_idx: &mut Option<usize>,
) {
    match *mode {
        Mode::View => match key_event.code {
            KeyCode::Char('q') => *mode = Mode::View, // handled in loop
            KeyCode::Char('s') => {
                if meeting.is_running() {
                    meeting.stop();
                } else {
                    meeting.start();
                }
            }
            KeyCode::Char('c') => meeting.reset(),
            KeyCode::Char('a') => {
                input_text.clear();
                *mode = Mode::AddCategory;
            }
            KeyCode::Char('d') => {
                input_text.clear();
                *mode = Mode::DeleteCategory;
            }
            KeyCode::Char('e') => {
                input_text.clear();
                *selected = 0;
                *mode = Mode::AddAttendeeSelect;
            }
            KeyCode::Char('r') => {
                input_text.clear();
                *mode = Mode::RemoveAttendee;
            }
            KeyCode::Char('w') => {
                input_text.clear();
                *mode = Mode::SaveAttendees;
            }
            KeyCode::Char('l') => {
                *selected = 0;
                files.clear();
                if let Ok(read) = fs::read_dir(data_dir()) {
                    for entry in read.flatten() {
                        if let Ok(ft) = entry.file_type() {
                            if ft.is_file() {
                                if let Some(name) = entry.file_name().to_str() {
                                    if name != "categories.toml" {
                                        files.push(name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
                *mode = Mode::LoadAttendees;
            }
            KeyCode::Char('p') => *show_salaries = !*show_salaries,
            _ => {}
        },
        Mode::AddCategory | Mode::AddAttendeeCount | Mode::SaveAttendees => match key_event.code {
            KeyCode::Enter => {
                match *mode {
                    Mode::AddCategory => {
                        if let Some((title, salary_str)) = input_text.split_once(':') {
                            if let Ok(salary) = salary_str.trim().parse::<u64>() {
                                if let Ok(cat) = EmployeeCategory::new(title.trim(), salary) {
                                    if !categories.iter().any(|c| c.title() == cat.title()) {
                                        categories.push(cat);
                                    }
                                }
                            }
                        }
                    }
                    Mode::AddAttendeeCount => {
                        let count = if input_text.trim().is_empty() {
                            1
                        } else if let Ok(c) = input_text.trim().parse::<u32>() {
                            c
                        } else {
                            return;
                        };
                        if let Some(idx) = add_attendee_idx.take() {
                            if let Some(cat) = categories.get(idx) {
                                meeting.add_attendee(cat, count);
                            }
                        }
                    }
                    Mode::SaveAttendees => {
                        let path = data_dir().join(input_text.trim());
                        let data: Vec<AttendeeInfo> = meeting
                            .attendees()
                            .map(|(t, _s, c)| AttendeeInfo {
                                title: t.to_string(),
                                count: *c,
                            })
                            .collect();
                        if let Err(err) = save_attendees(&path, &data) {
                            let _ = err;
                        }
                    }
                    _ => unreachable!(),
                }
                *mode = Mode::View;
            }
            KeyCode::Esc => *mode = Mode::View,
            KeyCode::Char(c) => input_text.push(c),
            KeyCode::Backspace => {
                input_text.pop();
            }
            _ => {}
        },
        Mode::DeleteCategory => match key_event.code {
            KeyCode::Up => {
                if *selected > 0 {
                    *selected -= 1;
                }
            }
            KeyCode::Down => {
                if *selected + 1 < categories.len() {
                    *selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(cat) = categories.get(*selected) {
                    let title = cat.title().to_string();
                    categories.retain(|c| c.title() != title);
                }
                *mode = Mode::View;
            }
            KeyCode::Esc => *mode = Mode::View,
            _ => {}
        },
        Mode::AddAttendeeSelect => match key_event.code {
            KeyCode::Up => {
                if *selected > 0 {
                    *selected -= 1;
                }
            }
            KeyCode::Down => {
                if *selected + 1 < categories.len() {
                    *selected += 1;
                }
            }
            KeyCode::Enter => {
                *add_attendee_idx = Some(*selected);
                input_text.clear();
                *mode = Mode::AddAttendeeCount;
            }
            KeyCode::Esc => *mode = Mode::View,
            _ => {}
        },
        Mode::RemoveAttendee => match key_event.code {
            KeyCode::Up => {
                if *selected > 0 {
                    *selected -= 1;
                }
            }
            KeyCode::Down => {
                let count = meeting.attendees().count();
                if *selected + 1 < count {
                    *selected += 1;
                }
            }
            KeyCode::Enter => {
                let names: Vec<String> =
                    meeting.attendees().map(|(t, _, _)| t.to_string()).collect();
                if let Some(title) = names.get(*selected) {
                    let remove_count = meeting
                        .attendees()
                        .find(|(t, _, _)| *t == title.as_str())
                        .map_or(0, |(_, _, c)| *c);
                    meeting.remove_attendee(title, remove_count);
                }
                *mode = Mode::View;
            }
            KeyCode::Esc => *mode = Mode::View,
            _ => {}
        },
        Mode::LoadAttendees => match key_event.code {
            KeyCode::Up => {
                if *selected > 0 {
                    *selected -= 1;
                }
            }
            KeyCode::Down => {
                if *selected + 1 < files.len() {
                    *selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Some(name) = files.get(*selected) {
                    let path = data_dir().join(name);
                    if let Ok(entries) = load_attendees(&path) {
                        meeting.clear_attendees();
                        for entry in entries {
                            if let Some(cat) = categories.iter().find(|c| c.title() == entry.title)
                            {
                                meeting.add_attendee(cat, entry.count);
                            }
                        }
                    }
                }
                *mode = Mode::View;
            }
            KeyCode::Esc => *mode = Mode::View,
            _ => {}
        },
    }
}

/// Entry point for the interactive TUI application.
///
/// This function initializes the terminal, loads persisted employee
/// categories, and enters the main event loop. On exit, updated categories
/// are saved back to disk.
///
/// # Errors
///
/// Returns an error if terminal initialization fails or if the category
/// database cannot be loaded or saved.
#[allow(clippy::too_many_lines)]
fn main() -> Result<(), Box<dyn Error>> {
    let dir = data_dir();
    fs::create_dir_all(&dir)?;
    let db_path = dir.join("categories.toml");
    let mut categories = load_categories(&db_path)?;
    let mut meeting = Meeting::new();

    let mut mode = Mode::View;
    let mut input_text = String::new();
    let mut show_salaries = false;
    let mut load_files: Vec<String> = Vec::new();
    let mut selected_idx: usize = 0;
    let mut add_attendee_idx: Option<usize> = None;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(100);
    let mut last_tick = std::time::Instant::now();

    loop {
        render_ui(
            &mut terminal,
            &meeting,
            &categories,
            &mode,
            &input_text,
            show_salaries,
            &load_files,
            selected_idx,
        )?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        if event::poll(timeout)? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Press {
                    if matches!(mode, Mode::View) && matches!(key_event.code, KeyCode::Char('q')) {
                        break;
                    }
                    process_key(
                        key_event,
                        &mut mode,
                        &mut input_text,
                        &mut show_salaries,
                        &mut categories,
                        &mut meeting,
                        &mut load_files,
                        &mut selected_idx,
                        &mut add_attendee_idx,
                    );
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = std::time::Instant::now();
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    save_categories(&db_path, &categories)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_formatting() {
        assert_eq!(format_duration(Duration::from_secs(0)), "00:00:00");
        assert_eq!(format_duration(Duration::from_secs(3661)), "01:01:01");
    }

    #[test]
    fn centered_rect_respects_size() {
        let area = Rect::new(0, 0, 100, 100);
        let popup = centered_rect(50, 50, area);
        assert_eq!(popup.width, 50);
        assert_eq!(popup.height, 50);
    }
}
