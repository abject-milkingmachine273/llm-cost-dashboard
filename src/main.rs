use std::path::PathBuf;

use clap::Parser;
use llm_cost_dashboard::ui::{self, App};

#[derive(Parser)]
#[command(name = "llm-dash", about = "Real-time LLM token spend dashboard")]
struct Cli {
    /// Monthly budget limit in USD
    #[arg(long, default_value = "10.0")]
    budget: f64,

    /// JSON log file to tail for live data (newline-delimited JSON)
    #[arg(long)]
    log_file: Option<PathBuf>,

    /// Start with demo data pre-loaded
    #[arg(long)]
    demo: bool,
}

fn main() {
    let cli = Cli::parse();
    let mut app = App::new(cli.budget);

    if cli.demo {
        app.load_demo_data();
    }

    if let Some(path) = &cli.log_file {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let _ = app.ingest_line(line);
            }
        }
    }

    if let Err(e) = ui::run(app) {
        eprintln!("Dashboard error: {e}");
        std::process::exit(1);
    }
}
