// use anyhow::Result;
// use dotenv::dotenv;
// use runners_toolkit::gh_api::GitHubClient;
// use std::env;
// use tokio;

// #[tokio::main]
// async fn main() -> Result<()> {
//     dotenv().ok();

//     let token = env::var("GH_PAT")?;
//     let owner = env::var("OWNER")?;

//     let ghc = GitHubClient::new(token, owner.clone());

//     let runners = ghc.runners().await.unwrap();

//     println!(
//         "👀 GitHub Self-hosted Runners for org '{}'\n{}",
//         owner,
//         "-".repeat(50)
//     );

//     for r in runners.runners {
//         let status = if r.busy { "⛔ busy" } else { "✅ idle" };
//         println!("{:<25} → {}", r.name, status);
//     }

//     Ok(())
// }

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
    Terminal,
};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dotenv::dotenv;
use runners_toolkit::gh_api::{GitHubClient, Runner};
use std::env;
use std::{error::Error, io};
use tokio::{
    sync::watch,
    task,
    time::{sleep, Duration},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Channel for UI update
    let (tx, mut rx) = watch::channel::<Vec<Runner>>(vec![]);

    // Background task to poll runners
    tokio::spawn(async move {
        dotenv().ok();
        let token = env::var("GH_PAT").unwrap();
        let owner = env::var("OWNER").unwrap();
        let ghc = GitHubClient::new(token, owner.clone());

        loop {
            let runners = ghc.runners().await.unwrap();
            // let runners = get_runners().await;
            let _ = tx.send(runners.runners);
            sleep(Duration::from_secs(5)).await;
        }
    });

    let mut table_state = TableState::default();
    table_state.select(Some(0));

    loop {
        let runners = rx.borrow().clone();

        // Clamp selected index to runners length
        let selected = table_state.selected().unwrap_or(0);
        let selected = if runners.is_empty() {
            None
        } else {
            Some(selected.min(runners.len() - 1))
        };
        table_state.select(selected);

        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Min(1)].as_ref())
                .split(size);

            let header = ["ID", "Name", "Status"];
            let rows = runners.iter().map(|r| {
                let status = if r.busy { "⛔ BUSY" } else { "✅ IDLE" };
                Row::new(vec![r.id.to_string(), r.name.clone(), status.into()])
            });

            let table = Table::new(
                rows,
                [
                    Constraint::Length(6),
                    Constraint::Length(20),
                    Constraint::Length(10),
                ],
            )
            .header(
                Row::new(header).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            )
            .block(
                Block::default()
                    .title("GitHub Runners")
                    .borders(Borders::ALL),
            )
            .row_highlight_style(Style::default().fg(Color::Black).bg(Color::White))
            .highlight_symbol(">> ");

            f.render_stateful_widget(table, chunks[0], &mut table_state);
        })?;

        // Handle input with timeout 100ms
        tokio::select! {
            _ = rx.changed() => {}, // On runners update, redraw immediately
            key_event = task::spawn_blocking(|| {
                if event::poll(Duration::from_millis(100))? {
                    if let Event::Key(key) = event::read()? {
                        return Ok::<Option<event::KeyEvent>, io::Error>(Some(key));
                    }
                }
                Ok::<Option<event::KeyEvent>, io::Error>(None)
            }) => {
                if let Some(key) = key_event?? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Down => {
                            let i = table_state.selected().unwrap_or(0);
                            let next = if runners.is_empty() {
                                0
                            } else if i >= runners.len() -1 {
                                0
                            } else {
                                i + 1
                            };
                            table_state.select(Some(next));
                        }
                        KeyCode::Up => {
                            let i = table_state.selected().unwrap_or(0);
                            let prev = if runners.is_empty() {
                                0
                            } else if i == 0 {
                                runners.len() - 1
                            } else {
                                i - 1
                            };
                            table_state.select(Some(prev));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
