#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use crate::app::gui::gui_main;
use crate::utils::{OS, OS_VERSION};
use anyhow::Result;
use directories::ProjectDirs;
use once_cell::sync::Lazy;
use std::io;
use std::path::PathBuf;
use tracing::{debug, debug_span, error, info};
use tracing_subscriber::layer::SubscriberExt;
use utils::ARCHITECTURE;

pub mod app;
pub mod minecraft;

mod error;
mod utils;

const LAUNCHER_VERSION: &str = env!("CARGO_PKG_VERSION");
static LAUNCHER_DIRECTORY: Lazy<ProjectDirs> =
    Lazy::new(
        || match ProjectDirs::from("com", "AMT", "AMT Client") {
            Some(proj_dirs) => proj_dirs,
            None => panic!("no application directory"),
        },
    );

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Directory where server instances are stored
static SERVERS_DIR: Lazy<PathBuf> = Lazy::new(|| {
    if let Ok(dir) = std::env::var("AMT_SERVERS_DIR") {
        let path = PathBuf::from(dir);
        let _ = std::fs::create_dir_all(&path);
        return path;
    }
    LAUNCHER_DIRECTORY.data_dir().join("servers")
});

/// HTTP Client with launcher agent - uses oauth2's reqwest to maintain type compatibility
static HTTP_CLIENT: Lazy<oauth2::reqwest::Client> = Lazy::new(|| {
    let client = oauth2::reqwest::ClientBuilder::new()
        .user_agent(APP_USER_AGENT)
        .build()
        .unwrap_or_else(|_| oauth2::reqwest::Client::new());

    client
});

pub fn main() -> Result<()> {
    use tracing_subscriber::{fmt, EnvFilter};

    let logs = LAUNCHER_DIRECTORY.data_dir().join("logs");
    if let Err(e) = utils::clean_directory(&logs, 7) {
        error!("Failed to clear log folder: {:?}", e);
    }

    let file_appender = tracing_appender::rolling::daily(logs, "launcher.log");

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from("amt_client=debug"))
        .with(
            fmt::Layer::new()
                .with_ansi(true)
                .with_writer(io::stdout),
        )
        .with(
            fmt::Layer::new()
                .with_ansi(false)
                .with_writer(file_appender),
        );
    tracing::subscriber::set_global_default(subscriber).expect("Unable to set a global subscriber");

    {
        let span = debug_span!("startup");
        let _guard = span.enter();

        info!(parent: &span, "Starting AMT Client v{}", LAUNCHER_VERSION);
        info!(parent: &span, "OS: {:} {:} {:}", OS, *ARCHITECTURE, OS_VERSION.to_string());

        info!(parent: &span, "Creating application directory");
        debug!(parent: &span, "Application directory: {:?}", LAUNCHER_DIRECTORY.data_dir());
        debug!(parent: &span, "Config directory: {:?}", LAUNCHER_DIRECTORY.config_dir());
        mkdir!(LAUNCHER_DIRECTORY.data_dir());
        mkdir!(LAUNCHER_DIRECTORY.config_dir());

        info!(parent: &span, "Starting GUI using Tauri framework {}", tauri::VERSION);
    }

    gui_main();

    info!("Launcher exited");
    Ok(())
}
