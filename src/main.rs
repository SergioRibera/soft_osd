use clap::Parser;

use self::app::MainApp;
use self::config::Config;
use self::window::Window;

mod app;
mod config;
mod window;

fn main() {
    let config = Config::parse();
    Window::<MainApp>::run(config)
}
