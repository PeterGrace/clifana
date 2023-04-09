use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame, Terminal,
};
use ratatui::layout::Alignment;
use ratatui::widgets::{Paragraph, Wrap};
use crate::consts::*;
use crate::AppData;
// ui

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut AppData) {

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
    let time_start:f64 = {
        let s = SystemTime::now();
        (s.duration_since(UNIX_EPOCH).unwrap() - Duration::from_secs(3600)).as_secs_f64()
    };
    let time_end:f64 = {
        let s = SystemTime::now();
        s.duration_since(UNIX_EPOCH).unwrap().as_secs_f64()
    };
    let datasets = vec![
        Dataset::default()
            .name("results")
            .graph_type(GraphType::Line)
            .marker(symbols::Marker::Dot)
            .style(Style::default()
                .bg(Color::LightBlue)
                .fg(Color::Cyan))
            .data(&app.data)];
    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(Span::styled(
                    "Chart 1",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title("X Axis")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0,1.0]),
        )
        .y_axis(
            Axis::default()
                .title("Y Axis")
                .style(Style::default().fg(Color::Gray))
                .labels(vec![
                    Span::styled("0.0", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw("0"),
                    Span::styled("1.0", Style::default().add_modifier(Modifier::BOLD)),
                ])
                .bounds([time_start, time_end]),
        );
    debug!("Chart: {:#?}", chart);
    f.render_widget(chart, panes[1]);
    //endregion

    //region Log Pane
    let mut loop_exit: bool = false;
    let mut log_paragraph: String = "".to_string();
    let mut log_buffer = app.log_buffer.lock().unwrap();

    log_paragraph = itertools::join(log_buffer.iter(), "\n");

    let log = Paragraph::new(log_paragraph)
        .style(Style::default().bg(Color::LightBlue).fg(Color::Gray))
        .block(create_block("Execution Log"))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(log, panes[2]);
    log_buffer.truncate(MAX_RETAINED_LOG_LINES);
    //endregion

    //region Bottom Status Line
    let bottom_line = Paragraph::new(format!("{}",humantime::format_rfc3339_seconds(SystemTime::now())));
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