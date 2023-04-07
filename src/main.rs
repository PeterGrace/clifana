mod cli;
mod cfg_file;
mod query;
mod prometheus;
mod app_data;
mod consts;

use std::panic::catch_unwind;
use std::cmp::Ordering;
use clap::Parser;
use cli::Cli;
use cfg_file::ConfigFile;
use query::execute_query;

use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame, Terminal,
};
use std::{error::Error, io, panic, time::{Duration, Instant}};
use std::ops::Add;
use ratatui::layout::Alignment;
use ratatui::widgets::{Paragraph, Wrap};
use app_data::AppData;
use crate::consts::DEFAULT_SCREEN_MARGIN;


#[macro_use]
extern crate log;


fn main() -> anyhow::Result<()> {
    // setup a hook when panic occurs
    better_panic::install();
    panic::set_hook(Box::new(|panic_info| {
        restore_terminal(Some(format!("Panic occurred")));
        better_panic::Settings::auto().create_panic_handler()(panic_info);
    }));

    let cli = Cli::parse();
    let config = ConfigFile::new(cli.config)?;

    let log_level_int = match cli.debug.cmp(&config.log_level) {
        Ordering::Greater => cli.debug,
        _ => config.log_level
    };
    let log_level: String = match log_level_int {
        0 => "WARN".to_string(),
        1 => "INFO".to_string(),
        _ => "DEBUG".to_string()
    };
    std::env::set_var("RUST_LOG", log_level);
    fern::Dispatch::new()
        .chain(fern::log_file("clifana.log").unwrap())
        .format(move |out, message, record| {
            out.finish(format_args!("[{}] {}", record.level(), message))
        }).apply().unwrap();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    match &cli.command {
        crate::cli::Commands::Query(args) => {
            execute_query(&config, args)?;
        }
    };
    let app = AppData::default();
    let tick_rate = Duration::from_millis(app.tick_interval_msecs);
    let res = run_app(&mut terminal, app, tick_rate);
    if res.is_err() {
        restore_terminal(Some(format!("{:#?}", res)));
    }
    restore_terminal(None);
    Ok(())
}

fn restore_terminal(err: Option<String>) {
    // clean up
    let mut stdout = io::stdout();
    execute!(
        stdout,
        LeaveAlternateScreen,
        DisableMouseCapture,
        cursor::Show
    );
    disable_raw_mode();


    if let Some(err) = err {
        warn!("{:?}", err)
    }
}

// run_app logic
fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: AppData,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
            }
        };
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}


// ui

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut AppData) {
    let size = f.size();
    let panes = Layout::default()
        .direction(Direction::Vertical)
        .margin(DEFAULT_SCREEN_MARGIN)
        .constraints([
            Constraint::Percentage(75),
            Constraint::Percentage(25)
        ].as_ref()
        ).split(size);

    let datasets = vec![
        Dataset::default()
            .name("results")
            .marker(symbols::Marker::Dot)
            .style(Style::default().fg(Color::Cyan))
            .data(&app.data)];

    let chart = Chart::new(datasets)
        .block(create_block("Results"));
    let mut loop_exit: bool = false;
    let mut log_paragraph: String = "".to_string();
    while (app.log_buffer.len() > 0) || (loop_exit) {
        let next = match app.log_buffer.pop_back() {
            Some(s) => s,
            None => { break;}
        };
        log_paragraph = log_paragraph.add(&next);
    };

    let log = Paragraph::new(log_paragraph)
        .style(Style::default().bg(Color::LightBlue).fg(Color::Gray))
        .block(create_block("Execution Log"))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(chart, panes[0]);
    f.render_widget(log, panes[1]);
}

fn create_block(title: &str) -> Block{
Block::default()
.borders(Borders::ALL)
.style(Style::default().bg(Color::White).fg(Color::Black))
.title(Span::styled(
title,
Style::default().add_modifier(Modifier::BOLD),
))
}