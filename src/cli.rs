use clap::Parser;

#[derive(Parser)]
#[command(name = "avtime")]
#[command(about = "NTP-synchronized time display", long_about = None)]
pub struct Cli {
    #[arg(help = "NTP server address (e.g., time.google.com or pool.ntp.org)")]
    pub server: String,
}
