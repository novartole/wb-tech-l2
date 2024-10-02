mod error;
mod handler;
mod json;
mod model;
mod repo;
mod state;

use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use handler::{
    create_event, delete_event, get_events_for_day, get_events_for_month, get_events_for_week,
    update_event,
};
use repo::InMemoryStorage;
use state::AppState;
use std::{
    env,
    net::{Ipv4Addr, SocketAddr},
};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    setup_tracing();

    if let Err(why) = run().await {
        println!("failed to serve: {}", why);
    }
}

async fn run() -> Result<()> {
    let app = {
        let db = InMemoryStorage::default();

        Router::new()
            .route("/create_event", post(create_event))
            .route("/update_event", post(update_event))
            .route("/delete_event/", post(delete_event))
            .route("/events_for_day/:date", get(get_events_for_day))
            .route("/events_for_week/:date", get(get_events_for_week))
            .route("/events_for_month/:date", get(get_events_for_month))
            .layer(TraceLayer::new_for_http())
            .with_state(AppState::new(db))
    };

    let ip = Ipv4Addr::LOCALHOST;
    let port = arg_port_or_default(3000);
    let listener = {
        let addr = SocketAddr::from((ip, port));
        TcpListener::bind(addr).await
    }?;

    info!("start listening on {:?}:{}", ip, port);
    Ok(axum::serve(listener, app).await?)
}

fn arg_port_or_default(default: u16) -> u16 {
    std::env::args()
        .nth(1)
        .unwrap_or_default()
        .parse()
        .unwrap_or(default)
}

fn setup_tracing() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "trace");
    }

    // Turn on error backtrace by default.
    // FYI:
    // - if you want panics and errors to both have backtraces, set RUST_BACKTRACE=1,
    // - If you want only errors to have backtraces, set RUST_LIB_BACKTRACE=1,
    // - if you want only panics to have backtraces, set RUST_BACKTRACE=1 and RUST_LIB_BACKTRACE=0.
    if env::var("RUST_LIB_BACKTRACE").is_err() {
        env::set_var("RUST_LIB_BACKTRACE", "1");
    }

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}
