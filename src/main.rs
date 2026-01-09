mod cli;
mod ntp;
mod ui;

use clap::Parser;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

use cli::Cli;
use ntp::NtpSync;
use ui::display_time;

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

    let ntp_sync_clone = Arc::clone(&ntp_sync);
    tokio::spawn(async move {
        sync_loop(ntp_sync_clone).await;
    });

    let result = display_time(ntp_sync).await;

    result?;

    Ok(())
}
