#[macro_use]
extern crate tokio;

mod cli;
mod cfg_file;
mod query;
mod prometheus;
mod app_data;
mod consts;
mod ui;

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
use std::borrow::Cow;
use std::ops::Add;
use std::time::SystemTime;
use log::LevelFilter;
use app_data::AppData;
use crate::ui::ui;


#[macro_use]
extern crate log;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    better_panic::install();
    panic::set_hook(Box::new(|panic_info| {
        restore_terminal(Some(format!("Panic occurred")));
        better_panic::Settings::auto().create_panic_handler()(panic_info);
    }));

    let cli = Cli::parse();

    let mut app = AppData::new(cli.config);

    let mut log_level_int = match cli.debug.cmp(&app.config.log_level) {
        Ordering::Greater => cli.debug,
        _ => app.config.log_level
    };
    match std::env::var("RUST_LOG") {
        Ok(log_val) => {
            match log_val.to_lowercase().as_str() {
                "warn" => { log_level_int = 0; }
                "info" => { log_level_int = 1; }
                _ => { log_level_int = 2; }
            };
        }
        Err(_) => {}
    }
    let log_level: LevelFilter = match log_level_int {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        _ => LevelFilter::Debug
    };
    let mut log_buffer = app.log_buffer.clone();
    fern::Dispatch::new()
        .level(log_level.clone())
        .format(move |out, message, record| {
            out.finish(format_args!("[{} {} {}] {}",
                                    humantime::format_rfc3339_seconds(SystemTime::now()),
                                    record.level(),
                                    record.target(),
                                    message))
        })
        .chain(fern::log_file("clifana.log").unwrap())
        .chain(fern::Output::call(move |record| {
            log_buffer.lock().unwrap().push_front(format!("{}", record.args()));
        }))
        .apply().unwrap();
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
/*    match &cli.command {
        crate::cli::Commands::Query(args) => {
            execute_query(&app.config, args)?;
        }
    };
*/    let tick_rate = Duration::from_millis(app.tick_interval_msecs);
    let res = run_app(&mut terminal, app, tick_rate).await;
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
async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: AppData,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        //region Keystroke Processing
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Left => app.menu.left(),
                    KeyCode::Right => app.menu.right(),
                    KeyCode::Up => app.menu.up(),
                    KeyCode::Down => app.menu.down(),
                    KeyCode::Esc => app.menu.reset(),
                    KeyCode::Enter => app.menu.select(),
                    KeyCode::Char('q') => { return Ok(()); }
                    _ => {}
                }
            }
        };
        //endregion

        //region Menu Selection Drain
        for e in app.menu.drain_events() {
            match e {
                tui_menu::MenuEvent::Selected(item) => match item.as_ref() {
                    "exit" => {
                        return Ok(());
                    }
                    _ => {
                        // println!("{} selected", item);
                    }
                },
            }
        }
        //endregion

        //region application tick processing
        if last_tick.elapsed() >= tick_rate {
            app.on_tick().await;
            last_tick = Instant::now();
        }
        //endregion
    }
}


