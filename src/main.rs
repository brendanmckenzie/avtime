use chrono::{DateTime, Local, Utc};
use clap::Parser;
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
use rsntp::SntpClient;
use std::io::{self, Write};
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::time::sleep;

#[derive(Parser)]
#[command(name = "avtime")]
#[command(about = "NTP-synchronized time display", long_about = None)]
struct Cli {
    #[arg(help = "NTP server address (e.g., time.google.com or pool.ntp.org)")]
    server: String,
}

struct NtpSync {
    offset_ms: i64,
    last_sync: SystemTime,
    server: String,
}

impl NtpSync {
    fn new(server: String) -> Self {
        Self {
            offset_ms: 0,
            last_sync: UNIX_EPOCH,
            server,
        }
    }

    async fn sync(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:123", self.server)
            .to_socket_addrs()?
            .next()
            .ok_or("Unable to resolve NTP server")?;

        let client = SntpClient::new();
        let result = client.synchronize(addr)?;

        let offset = result.clock_offset();
        self.offset_ms = (offset.as_secs_f64() * 1000.0) as i64;
        self.last_sync = SystemTime::now();

        Ok(())
    }

    fn adjusted_time(&self) -> DateTime<Utc> {
        let now = Utc::now();
        now + chrono::Duration::milliseconds(self.offset_ms)
    }
}

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

async fn display_time(ntp_sync: Arc<Mutex<NtpSync>>) -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        let (time, offset_ms, seconds_since_sync, server) = {
            let sync = ntp_sync.lock().await;
            let adjusted = sync.adjusted_time();
            let elapsed = sync
                .last_sync
                .elapsed()
                .unwrap_or(Duration::from_secs(9999));
            (
                adjusted,
                sync.offset_ms,
                elapsed.as_secs(),
                sync.server.clone(),
            )
        };

        let local_time: DateTime<Local> = time.into();
        let sync_color = get_sync_color(seconds_since_sync);
        let offset_color = get_color_for_offset(offset_ms);
        let sync_indicator = get_sync_indicator(seconds_since_sync);

        terminal.draw(|f| {
            let size = f.area();

            let main_block = Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .title(Span::styled(
                    " AVTIME ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ));

            let inner = main_block.inner(size);
            f.render_widget(main_block, size);

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

async fn sync_loop(ntp_sync: Arc<Mutex<NtpSync>>) {
    loop {
        sleep(Duration::from_secs(10)).await;
        {
            let mut sync = ntp_sync.lock().await;
            let _ = sync.sync().await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let ntp_sync = Arc::new(Mutex::new(NtpSync::new(cli.server)));

    {
        let mut sync = ntp_sync.lock().await;
        print!("Synchronizing with NTP server...");
        io::stdout().flush()?;
        sync.sync().await?;
        println!(" Done!");
    }

    let ntp_sync_clone = Arc::clone(&ntp_sync);
    tokio::spawn(async move {
        sync_loop(ntp_sync_clone).await;
    });

    let result = display_time(ntp_sync).await;

    result?;

    Ok(())
}
