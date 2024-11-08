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
    Window::<MainApp>::run(config)
}
