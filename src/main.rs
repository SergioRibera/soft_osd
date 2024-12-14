use std::cell::OnceCell;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use clap::Parser;
use merge2::Merge;

mod app;
mod components;
mod config;
mod ipc;
mod notification;
mod utils;
mod window;

use app::MainApp;
use config::Config;
use directories::ProjectDirs;
use window::Window;

use self::config::OsdType;

const PROJECT_PATH: OnceCell<ProjectDirs> = OnceCell::new();

#[tokio::main]
async fn main() {
    let project = PROJECT_PATH.clone();
    let project = project.get_or_init(|| ProjectDirs::from("rs", "sergioribera", "sosd").unwrap());
    let (path, config) = get_config(project).unwrap();

    if let OsdType::Init = config.command {
        std::fs::write(&path, toml::to_string_pretty(&Config::default()).unwrap()).unwrap();
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

fn get_config(project: &ProjectDirs) -> Option<(PathBuf, Config)> {
    let mut args = Config::parse();

    let config_path = if let Some(path) = args.config.as_ref() {
        // tracing::trace!("Loading custom path");
        println!("Loading custom path");
        path.clone()
    } else {
        let config_path = project.config_dir();

        _ = std::fs::create_dir_all(config_path);

        // tracing::trace!("Loading global config");
        println!("Loading global config");
        config_path.join("config.toml")
    };
    // tracing::info!("Reading configs from path: {config_path:?}");
    println!("Reading configs from path: {config_path:?}");

    if let Ok(cfg_content) = std::fs::read_to_string(&config_path) {
        // tracing::debug!("Merging from config file");
        println!("Merging from config file");
        let mut config: Config = toml::from_str(&cfg_content).ok()?;
        config.merge(&mut args);
        return Some((config_path, config));
    }
    let mut config = Config::default();
    config.merge(&mut args);

    Some((config_path, config))
}
