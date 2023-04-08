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
use std::borrow::Cow;
use std::ops::Add;
use ratatui::layout::Alignment;
use ratatui::widgets::{LineGauge, Paragraph, Wrap};
use app_data::AppData;
use tui_menu::Menu;
use crate::consts::DEFAULT_SCREEN_MARGIN;


#[macro_use]
extern crate log;


fn main() -> anyhow::Result<()> {
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
    let mut app = AppData::default();
    let mut log_buffer = app.log_buffer.clone();
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!("[{}] {}", record.level(), message))
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
    match &cli.command {
        crate::cli::Commands::Query(args) => {
            execute_query(&config, args)?;
        }
    };
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
                    KeyCode::Char('q') => { return Ok(()) },
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
            app.on_tick();
            last_tick = Instant::now();
        }
        //endregion
    }
}


// ui

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut AppData) {

    //region Pane Setup
    let size = f.size();
    let panes = Layout::default()
        .direction(Direction::Vertical)
        .margin(DEFAULT_SCREEN_MARGIN)
        .constraints([
            Constraint::Min(1),
            Constraint::Percentage(75),
            Constraint::Percentage(25),
            Constraint::Min(1)
        ].as_ref()
        ).split(size);
    //endregion

    //region Line Chart
    let datasets = vec![
        Dataset::default()
            .name("results")
            .marker(symbols::Marker::Dot)
            .style(Style::default()
                .bg(Color::LightBlue)
                .fg(Color::Cyan))
            .data(&app.data)];
    let chart = Chart::new(datasets)
        .block(create_block("Results"));
    f.render_widget(chart, panes[1]);
    //endregion

    //region Log Pane
    let mut loop_exit: bool = false;
    let mut log_paragraph: String = "".to_string();
    let mut log_buffer = app.log_buffer.lock().unwrap();

    log_paragraph = itertools::join(log_buffer.iter(), "\n");
    // while (log_buffer.len() > 0) || (loop_exit) {
    //
    //     let next = match log_buffer.pop_back() {
    //         Some(s) => s,
    //         None => { break; }
    //     };
    //     log_paragraph = log_paragraph.add(&next);
    // };
    let log = Paragraph::new(log_paragraph)
        .style(Style::default().bg(Color::LightBlue).fg(Color::Gray))
        .block(create_block("Execution Log"))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(log, panes[2]);
    //endregion

    //region Bottom Status Line
    let bottom_line = Paragraph::new("Bottom Line");
    f.render_widget(bottom_line, panes[3]);
    //endregion

    //region Top Menubar
    let menu = tui_menu::Menu::new()
        .default_style(Style::default().bg(Color::White).fg(Color::Red));
    f.render_stateful_widget(menu, panes[0], &mut app.menu);
    //endregion
}

fn create_block(title: &str) -> Block {
    Block::default()
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center)
        .style(
            Style::default()
                .bg(Color::LightBlue)
                .fg(Color::White))
        .title(Span::styled(
            title,
            Style::default()
                .add_modifier(Modifier::BOLD),
        ))
}
fn create_dialog_block(title: &str) -> Block {
    Block::default()
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center)
        .style(
            Style::default()
                .bg(Color::Gray)
                .fg(Color::White))
        .title(Span::styled(
            title,
            Style::default()
                .add_modifier(Modifier::BOLD),
        ))
}