// main.rs

use std::{error::Error, path::PathBuf, time::Duration};

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use meeting_cost_tracker::{load_categories, save_categories, EmployeeCategory, Meeting};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;

enum Mode {
    View,
    AddCategory,
    DeleteCategory,
    AddAttendee,
    RemoveAttendee,
}

fn main() -> Result<(), Box<dyn Error>> {
    let db_path = PathBuf::from("categories.toml");
    let mut categories = load_categories(&db_path)?;
    let mut meeting = Meeting::new();

    let mut mode = Mode::View;
    let mut input_text = String::new();

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
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(1),
                ])
                .split(size);

            let title = Paragraph::new("Meeting Cost Tracker")
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
            f.render_widget(title, chunks[0]);

            let running = meeting.is_running();
            let duration = meeting.duration();
            let cost = meeting.total_cost();

            let info = Paragraph::new(Line::from(vec![
                Span::raw(format!(
                    "[{}] Duration: {:.1?}  Cost: ${:.2}",
                    if running { "Running" } else { "Stopped" },
                    duration,
                    cost
                )),
            ]));
            f.render_widget(info, chunks[1]);

            match mode {
                Mode::AddCategory => {
                    let input_widget = Paragraph::new(input_text.as_str())
                        .block(Block::default().title("Enter: Title:Salary").borders(Borders::ALL));
                    f.render_widget(input_widget, chunks[2]);
                }
                Mode::DeleteCategory => {
                    let input_widget = Paragraph::new(input_text.as_str())
                        .block(Block::default().title("Enter title to delete").borders(Borders::ALL));
                    f.render_widget(input_widget, chunks[2]);
                }
                Mode::View => {
                    let help = Paragraph::new(Line::from(vec![
                        Span::raw("[s] Start  [t] Stop  [a] Add Category  [d] Delete Category  [e] Add Employee  [r] Remove Employee  [q] Quit"),
                    ]))
                    .block(Block::default().borders(Borders::ALL).title("Controls"));
                    f.render_widget(help, chunks[2]);
                }
                Mode::AddAttendee => {
                    let input_widget = Paragraph::new(input_text.as_str())
                        .block(Block::default().title("Enter: Title:Count").borders(Borders::ALL));
                    f.render_widget(input_widget, chunks[2]);
                }
                Mode::RemoveAttendee => {
                    let input_widget = Paragraph::new(input_text.as_str())
                        .block(Block::default().title("Enter: Title:Count to remove").borders(Borders::ALL));
                    f.render_widget(input_widget, chunks[2]);
                }
            }

            let lists = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[3]);

            let category_list: Vec<Line> = categories
                .iter()
                .map(|c| Line::from(Span::raw(format!("{}: ${}", c.title(), c.salary()))))
                .collect();
            let list_widget = Paragraph::new(category_list)
                .block(Block::default().borders(Borders::ALL).title("Employee Categories"));
            f.render_widget(list_widget, lists[0]);

            let meeting_list: Vec<Line> = meeting
                .attendees()
                .map(|(title, (_, count))| Line::from(Span::raw(format!("{} x {}", title, count))))
                .collect();
            let meeting_widget = Paragraph::new(meeting_list)
                .block(Block::default().borders(Borders::ALL).title("Current Meeting"));
            f.render_widget(meeting_widget, lists[1]);
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
                                        if let Ok(salary) = salary_str.trim().parse::<f64>() {
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
                                    if let Some((title, count_str)) = input_text.split_once(':') {
                                        if let Ok(count) = count_str.trim().parse::<u32>() {
                                            if let Some(cat) = categories
                                                .iter()
                                                .find(|c| c.title() == title.trim())
                                            {
                                                meeting.add_attendee(cat.clone(), count);
                                            }
                                        }
                                    }
                                }
                                Mode::RemoveAttendee => {
                                    if let Some((title, count_str)) = input_text.split_once(':') {
                                        if let Ok(count) = count_str.trim().parse::<u32>() {
                                            meeting.remove_attendee(title.trim(), count);
                                        }
                                    }
                                }
                                _ => {}
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
