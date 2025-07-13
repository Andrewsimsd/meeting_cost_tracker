//! Interactive terminal application for tracking meeting costs.
// main.rs
#![warn(clippy::pedantic)]

use std::{error::Error, path::PathBuf, time::Duration};

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use meeting_cost_tracker::{load_categories, save_categories, EmployeeCategory, Meeting};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;

/// Formats a list of strings into columnar [`Line`] values based on the given
/// width. The lines attempt to pack as many columns as will fit without
/// exceeding the available area.
fn column_lines(items: &[String], width: u16, style: Style) -> Vec<Line> {
    if items.is_empty() {
        return Vec::new();
    }

    let max_len = items.iter().map(|s| s.len()).max().unwrap_or(0) as u16 + 2;
    let cols = std::cmp::max(1, width / max_len);
    let rows = (items.len() as u16 + cols - 1) / cols;

    let mut lines = Vec::new();
    for row in 0..rows {
        let mut spans = Vec::new();
        for col in 0..cols {
            let idx = row as usize + col as usize * rows as usize;
            if let Some(item) = items.get(idx) {
                spans.push(Span::styled(
                    format!("{item:<width$}", width = max_len as usize),
                    style,
                ));
            }
        }
        lines.push(Line::from(spans));
    }
    lines
}

/// UI modes controlling user interaction.
enum Mode {
    /// Normal viewing mode where meeting stats are displayed.
    View,
    /// Mode for creating a new [`EmployeeCategory`].
    AddCategory,
    /// Mode for deleting an existing [`EmployeeCategory`].
    DeleteCategory,
    /// Mode for adding attendees to the [`Meeting`].
    AddAttendee,
    /// Mode for removing attendees from the [`Meeting`].
    RemoveAttendee,
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
    let db_path = PathBuf::from("categories.toml");
    let mut categories = load_categories(&db_path)?;
    let mut meeting = Meeting::new();

    let mut mode = Mode::View;
    let mut input_text = String::new();
    let mut show_salaries = false;

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(100);
    let mut last_tick = std::time::Instant::now();

    loop {
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
                        "[{}] Duration: {:.1?}",
                        if running { "Running" } else { "Stopped" },
                        duration
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
                    let input_widget = Paragraph::new(input_text.as_str())
                        .block(Block::default().title("Enter: Title:Salary").borders(Borders::ALL));
                    f.render_widget(input_widget, chunks[4]);
                }
                Mode::DeleteCategory => {
                    let input_widget = Paragraph::new(input_text.as_str())
                        .block(Block::default().title("Enter title to delete").borders(Borders::ALL));
                    f.render_widget(input_widget, chunks[4]);
                }
                Mode::View => {
                    let help = Paragraph::new(Line::from(vec![
                        Span::styled(
                            "[s] Start  [t] Stop  [c] Reset  [a] Add Category  [d] Delete Category  [e] Add Employee  [r] Remove Employee  [p] Toggle Salaries  [q] Quit",
                            Style::default().fg(Color::Yellow),
                        ),
                    ]))
                    .block(Block::default().borders(Borders::ALL).title("Controls"));
                    f.render_widget(help, chunks[4]);
                }
                Mode::AddAttendee => {
                    let input_widget = Paragraph::new(input_text.as_str())
                        .block(
                            Block::default()
                                .title("Enter: Title[:Count]")
                                .borders(Borders::ALL),
                        );
                    f.render_widget(input_widget, chunks[4]);
                }
                Mode::RemoveAttendee => {
                    let input_widget = Paragraph::new(input_text.as_str())
                        .block(Block::default().title("Enter: Title:Count to remove").borders(Borders::ALL));
                    f.render_widget(input_widget, chunks[4]);
                }
            }

            let lists = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[3]);

            let category_strings: Vec<String> = categories
                .iter()
                .map(|c| {
                    if show_salaries {
                        format!("{}: ${}", c.title(), c.salary())
                    } else {
                        c.title().to_string()
                    }
                })
                .collect();
            let category_list = column_lines(
                &category_strings,
                lists[1].width.saturating_sub(2),
                Style::default().fg(Color::Cyan),
            );
            let list_widget = Paragraph::new(category_list)
                .block(Block::default().borders(Borders::ALL).title("Employee Categories"));
            f.render_widget(list_widget, lists[1]);

            let meeting_strings: Vec<String> = meeting
                .attendees()
                .map(|(cat, count)| format!("{} x {}", cat.title(), count))
                .collect();
            let meeting_list = column_lines(
                &meeting_strings,
                lists[0].width.saturating_sub(2),
                Style::default().fg(Color::Magenta),
            );
            let meeting_widget = Paragraph::new(meeting_list)
                .block(Block::default().borders(Borders::ALL).title("Current Meeting"));
            f.render_widget(meeting_widget, lists[0]);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key_event) = event::read()? {
                match mode {
                    Mode::View => match key_event.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('s') => meeting.start(),
                        KeyCode::Char('t') => meeting.stop(),
                        KeyCode::Char('c') => meeting.reset(),
                        KeyCode::Char('a') => {
                            input_text.clear();
                            mode = Mode::AddCategory;
                        }
                        KeyCode::Char('d') => {
                            input_text.clear();
                            mode = Mode::DeleteCategory;
                        }
                        KeyCode::Char('e') => {
                            input_text.clear();
                            mode = Mode::AddAttendee;
                        }
                        KeyCode::Char('r') => {
                            input_text.clear();
                            mode = Mode::RemoveAttendee;
                        }
                        KeyCode::Char('p') => show_salaries = !show_salaries,
                        _ => {}
                    },
                    Mode::AddCategory
                    | Mode::DeleteCategory
                    | Mode::AddAttendee
                    | Mode::RemoveAttendee => match key_event.code {
                        KeyCode::Enter => {
                            match mode {
                                Mode::AddCategory => {
                                    if let Some((title, salary_str)) = input_text.split_once(':') {
                                        if let Ok(salary) = salary_str.trim().parse::<u64>() {
                                            if let Ok(cat) =
                                                EmployeeCategory::new(title.trim(), salary)
                                            {
                                                categories.push(cat);
                                            }
                                        }
                                    }
                                }
                                Mode::DeleteCategory => {
                                    let title = input_text.trim();
                                    categories.retain(|c| c.title() != title);
                                }
                                Mode::AddAttendee => {
                                    let (title, count) = match input_text.split_once(':') {
                                        Some((t, c_str)) => {
                                            let count = c_str.trim().parse::<u32>().unwrap_or(1);
                                            (t.trim(), count)
                                        }
                                        None => (input_text.trim(), 1),
                                    };
                                    if let Some(cat) =
                                        categories.iter().find(|c| c.title() == title)
                                    {
                                        meeting.add_attendee(cat, count);
                                    }
                                }
                                Mode::RemoveAttendee => {
                                    if let Some((title, count_str)) = input_text.split_once(':') {
                                        if let Ok(count) = count_str.trim().parse::<u32>() {
                                            meeting.remove_attendee(title.trim(), count);
                                        }
                                    }
                                }
                                Mode::View => {}
                            }
                            mode = Mode::View;
                        }
                        KeyCode::Esc => mode = Mode::View,
                        KeyCode::Char(c) => input_text.push(c),
                        KeyCode::Backspace => {
                            input_text.pop();
                        }
                        _ => {}
                    },
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
