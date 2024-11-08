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

fn main() {
    let config = Config::parse();
    let app = Arc::new(Mutex::new(MainApp::from(config.clone())));

    {
        let app = app.clone();
        let command = config.command.clone();
        std::thread::spawn(move || ipc::connect(&command, app));
    }
    Window::run(app, config)
}
