mod app;
mod auth;
mod error;
mod models;
mod repository;
mod routes;

use app::App;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    App::start().await?;

    Ok(())
}
