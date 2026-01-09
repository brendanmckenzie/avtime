use chrono::{DateTime, Local};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
    Terminal,
};
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::ntp::NtpSync;

fn get_color_for_offset(offset_ms: i64) -> Color {
    match offset_ms.abs() {
        0..=10 => Color::Green,
        11..=50 => Color::Yellow,
        51..=100 => Color::Magenta,
        _ => Color::Red,
    }
}

fn get_sync_color(seconds_since_sync: u64) -> Color {
    match seconds_since_sync {
        0..=5 => Color::Green,
        6..=15 => Color::Yellow,
        16..=30 => Color::Magenta,
        _ => Color::Red,
    }
}

fn get_sync_indicator(seconds_since_sync: u64) -> &'static str {
    match seconds_since_sync {
        0..=30 => "●",
        _ => "○",
    }
}

async fn show_loading_screen(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    server: &str,
) -> io::Result<()> {
    terminal.draw(|f| {
        let size = f.area();

        // Center content with max width
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Max(80),
                Constraint::Min(0),
            ])
            .split(size);

        // Center content with max height
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Max(20),
                Constraint::Min(0),
            ])
            .split(horizontal_chunks[1]);

        let centered_area = vertical_chunks[1];

        let main_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(
                " AVTIME ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));

        let inner = main_block.inner(centered_area);
        f.render_widget(main_block, centered_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(inner);

        let loading_text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Synchronizing with NTP server...",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                format!("Server: {}", server),
                Style::default().fg(Color::DarkGray),
            )]),
        ];

        let loading_widget = Paragraph::new(loading_text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            );

        f.render_widget(loading_widget, chunks[1]);
    })?;

    Ok(())
}

pub async fn display_time(ntp_sync: Arc<Mutex<NtpSync>>) -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Show loading screen and perform initial sync
    {
        let sync = ntp_sync.lock().await;
        show_loading_screen(&mut terminal, &sync.server).await?;
    }

    {
        let mut sync = ntp_sync.lock().await;
        if let Err(e) = sync.sync().await {
            disable_raw_mode()?;
            execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
            return Err(io::Error::new(io::ErrorKind::Other, e.to_string()));
        }
    }

    loop {
        let (time, offset_ms, seconds_since_sync, server) = {
            let sync = ntp_sync.lock().await;
            let adjusted = sync.adjusted_time();
            let seconds_since = sync.seconds_since_sync();
            (adjusted, sync.offset_ms, seconds_since, sync.server.clone())
        };

        let local_time: DateTime<Local> = time.into();
        let sync_color = get_sync_color(seconds_since_sync);
        let offset_color = get_color_for_offset(offset_ms);
        let sync_indicator = get_sync_indicator(seconds_since_sync);

        terminal.draw(|f| {
            let size = f.area();

            // Center content with max width
            let horizontal_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Max(80),
                    Constraint::Min(0),
                ])
                .split(size);

            // Center content with max height
            let vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Max(20),
                    Constraint::Min(0),
                ])
                .split(horizontal_chunks[1]);

            let centered_area = vertical_chunks[1];

            let main_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(
                    " AVTIME ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ));

            let inner = main_block.inner(centered_area);
            f.render_widget(main_block, centered_area);

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ])
                .split(inner);

            let date_widget = Paragraph::new(local_time.format("%Y-%m-%d").to_string())
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center);
            f.render_widget(date_widget, chunks[0]);

            let time_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray));
            let time_inner = time_block.inner(chunks[1]);
            f.render_widget(time_block, chunks[1]);

            let time_widget = Paragraph::new(local_time.format("%H:%M:%S%.3f").to_string())
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .alignment(Alignment::Center);
            f.render_widget(time_widget, time_inner);

            let server_info = vec![Line::from(vec![
                Span::styled("NTP Server: ", Style::default().fg(Color::DarkGray)),
                Span::styled(server.clone(), Style::default().fg(Color::White)),
            ])];
            let server_widget = Paragraph::new(server_info)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray))
                        .title(" Server "),
                )
                .alignment(Alignment::Left);
            f.render_widget(server_widget, chunks[3]);

            let offset_text = format!("{:+} ms", offset_ms);
            let offset_info = vec![Line::from(vec![
                Span::styled("Clock Offset: ", Style::default().fg(Color::DarkGray)),
                Span::styled(offset_text, Style::default().fg(offset_color)),
            ])];
            let offset_widget = Paragraph::new(offset_info)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray))
                        .title(" Offset "),
                )
                .alignment(Alignment::Left);
            f.render_widget(offset_widget, chunks[4]);

            let sync_age_ratio = (seconds_since_sync.min(30) as f64 / 30.0) * 100.0;
            let sync_label = format!(
                "{} Last sync: {}s ago",
                sync_indicator, seconds_since_sync
            );

            let sync_gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray))
                        .title(" Sync Status "),
                )
                .gauge_style(Style::default().fg(sync_color))
                .label(sync_label)
                .ratio(1.0 - (sync_age_ratio / 100.0));
            f.render_widget(sync_gauge, chunks[5]);

            let help_text = Paragraph::new("Press 'q' or Ctrl+C to quit")
                .style(Style::default().fg(Color::DarkGray))
                .alignment(Alignment::Center);
            f.render_widget(help_text, chunks[7]);
        })?;

        sleep(Duration::from_millis(10)).await;

        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Char('Q') {
                    break;
                }
                if key.code == KeyCode::Char('c')
                    && key.modifiers.contains(event::KeyModifiers::CONTROL)
                {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
