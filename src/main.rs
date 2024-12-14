use std::cell::OnceCell;
use std::sync::{Arc, Mutex};

mod app;
mod components;
mod ipc;
mod utils;
mod window;

use app::MainApp;
use config::{get_config, write_default, Config, OsdType, Parser, ProjectDirs};
use window::Window;

const PROJECT_PATH: OnceCell<ProjectDirs> = OnceCell::new();

#[tokio::main]
async fn main() {
    let mut args = Config::parse();
    let project = PROJECT_PATH.clone();
    let project = project.get_or_init(|| ProjectDirs::from("rs", "sergioribera", "sosd").unwrap());
    let (path, config) = get_config(&mut args, project).unwrap();

    println!("Args: {:?}", config.command);
    if let OsdType::Init = config.command {
        write_default(&path);
        println!("Configuration init at {path:#?}");
        return;
    }

    let app = Arc::new(Mutex::new(MainApp::from(config.clone())));

    {
        let app = app.clone();
        let command = config.command.clone();
        let global = config.globals.clone();
        tokio::spawn(async move { ipc::connect(&global, &command, app).await });
    }
    Window::run(app, config)
}
