mod player;
mod search;
mod ui;
mod app;

use app::App;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Redirect logs to a file to prevent TUI corruption
    let log_file = std::fs::File::create("melody.log")?;
    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .filter_level(log::LevelFilter::Info)
        .init();

    let mut app = App::new();
    app.run().await?;

    Ok(())
}
