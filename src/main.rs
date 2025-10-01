//! Binary entrypoint: loads config, sets up logging, builds Axum app, and serves `/send`.

use std::{fs::OpenOptions, net::SocketAddr, sync::Arc};

use axum::{routing::post, Router};
use dotenvy::dotenv;
use tracing::{debug, info, Level};
use tracing_subscriber::{filter, fmt, layer::SubscriberExt, Layer, Registry};

mod email;
mod routes;

use crate::email::EmailState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1) Load environment (.env is optional)
    dotenv().ok();

    // 2) Logging: compact console + two JSON files (error.log, debug.log)
    //    - console: human-friendly
    //    - error.log: only ERROR+
    //    - debug.log: DEBUG+
    let err_file = OpenOptions::new().append(true).create(true).open("error.log")?;
    let dbg_file = OpenOptions::new().append(true).create(true).open("debug.log")?;
    let subscriber = Registry::default()
        .with(fmt::layer().compact().with_ansi(true))
        .with(
            fmt::layer()
                .json()
                .with_writer(err_file)
                .with_filter(filter::LevelFilter::from_level(Level::ERROR)),
        )
        .with(
            fmt::layer()
                .json()
                .with_writer(dbg_file)
                .with_filter(filter::LevelFilter::from_level(Level::DEBUG)),
        );
    tracing::subscriber::set_global_default(subscriber)?;

    // 3) Build app state (SMTP client, addresses, templates path) from env
    let state = Arc::new(EmailState::from_env()?);
    debug!("Templates directory: {}", state.templates_dir.display());

    // 4) Router
    let app = Router::new()
        .route("/send", post(routes::send_email))
        .with_state(state);

    // 5) Bind address
    // Required envs: LISTEN_ADDR, LISTEN_PORT
    let listen_addr = std::env::var("LISTEN_ADDR")?;
    let listen_port: u16 = std::env::var("LISTEN_PORT")?.parse()?;
    let addr: SocketAddr = format!("{listen_addr}:{listen_port}").parse()?;

    info!("Starting server on {addr}");

    // 6) Serve
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
