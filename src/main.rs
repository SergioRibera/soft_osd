use std::sync::{Arc, Mutex};

use clap::Parser;

mod app;
mod components;
mod config;
mod ipc;
mod utils;
mod window;

use app::MainApp;
use config::Config;
use window::Window;

#[tokio::main]
async fn main() {
    let config = Config::parse();
    let app = Arc::new(Mutex::new(MainApp::from(config.clone())));

    {
        let app = app.clone();
        let command = config.command.clone();
        tokio::spawn(async move { ipc::connect(&command, app).await });
    }
    Window::run(app, config)
}
