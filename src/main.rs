//! Binary entrypoint: loads config, sets up logging, builds Axum app, and serves `/send`.
use std::{net::SocketAddr, sync::Arc};
use axum::{routing::post, Router};
use dotenvy::dotenv;
use tracing::{debug, info};
use templar::{email,routes,logger,config::get_defaults as df};
use templar::config::ApiConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fn env_var(name: &str) -> Option<String> {
        std::env::var(name).ok()
    }
    // 1) Load environment (.env is optional)
    dotenv().ok();
    let config:ApiConfig = ApiConfig::from(df());
    let lvl = env_var("LOG_LEVEL").unwrap_or(config.log_level);
    let tf = env_var("LOG_TO_FILE").unwrap_or(config.log_to_file.to_string())== "true";
    let ts = env_var("LOG_TO_STDOUT").unwrap_or(config.log_to_stdout.to_string())== "true";
    let ld = env_var("LOG_DIR").unwrap_or(config.log_dir);
    let file = env_var("LOG_FILE").unwrap_or(config.log_file);
    // 2) Set up logging
    logger::set_logger(lvl,tf,ts,ld,file).unwrap();
    // 3) Build app state (SMTP client, addresses, templates path) from env
    let state = Arc::new(email::EmailState::from_env()?);
    debug!("Templates directory: {}", state.templates_dir.display());
    // 4) Router
    let app = Router::new()
        .route("/send", post(routes::send_email))
        .with_state(state);

    // 5) Bind address
    let ip = env_var("LISTEN_ADDR").unwrap_or(config.listen_addr);
    let port = env_var("LISTEN_PORT").unwrap_or(config.listen_port.to_string());
    let addr: SocketAddr = format!("{ip}:{port}").parse()?;

    info!("Starting server on {addr}");

    // 6) Serve
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
