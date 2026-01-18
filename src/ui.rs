use chrono::{DateTime, Local, Utc};
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::ntp::NtpSync;

pub async fn display_time(ntp_sync: Arc<Mutex<NtpSync>>) -> io::Result<()> {
    // Perform initial sync
    {
        let mut sync = ntp_sync.lock().await;
        println!("Synchronizing with NTP server...");
        if let Err(e) = sync.sync().await {
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

        let utc_time: DateTime<Utc> = time;
        let local_time: DateTime<Local> = time.into();

        // Calculate time to next sync (10 second interval)
        let time_until_next = 10 - (seconds_since_sync % 10);

        // Clear screen and move cursor to top
        print!("\x1B[2J\x1B[1;1H");

        // Print the 5 lines
        println!("Server: {}", server);
        println!("UTC:    {}", utc_time.format("%Y-%m-%d %H:%M:%S%.3f"));
        println!("Local:  {}", local_time.format("%Y-%m-%d %H:%M:%S%.3f"));
        println!("Sync:   {}s ago (next in {}s)", seconds_since_sync, time_until_next);
        println!("Offset: {:+} ms", offset_ms);

        io::stdout().flush()?;

        sleep(Duration::from_millis(100)).await;
    }
}
