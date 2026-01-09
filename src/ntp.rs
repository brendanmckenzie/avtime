use chrono::{DateTime, Utc};
use rsntp::SntpClient;
use std::net::ToSocketAddrs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct NtpSync {
    pub offset_ms: i64,
    pub last_sync: SystemTime,
    pub server: String,
}

impl NtpSync {
    pub fn new(server: String) -> Self {
        Self {
            offset_ms: 0,
            last_sync: UNIX_EPOCH,
            server,
        }
    }

    pub async fn sync(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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

    pub fn adjusted_time(&self) -> DateTime<Utc> {
        let now = Utc::now();
        now + chrono::Duration::milliseconds(self.offset_ms)
    }

    pub fn seconds_since_sync(&self) -> u64 {
        self.last_sync
            .elapsed()
            .unwrap_or(Duration::from_secs(9999))
            .as_secs()
    }
}
