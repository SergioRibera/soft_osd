use std::cell::OnceCell;
use std::sync::{Arc, Mutex};

mod app;
mod buffer;
mod components;
mod utils;
mod window;

use app::MainApp;
use config::{get_config, write_default, Config, OsdType, Parser, ProjectDirs};
use services::ServiceManager;
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
        let is_daemon = command == OsdType::Daemon;
        let global = config.globals.clone();
        let manager = ServiceManager::new(is_daemon, app)
            .await
            .with_battery(
                config.battery.enabled,
                config.battery.refresh_time,
                config
                    .battery
                    .clone()
                    .level
                    .map(|l| l.0.keys().copied().collect::<Vec<_>>())
                    .unwrap_or_default(),
            )
            .await
            .unwrap()
            .with_singletone()
            .await
            .unwrap();

        if is_daemon {
            tokio::spawn(async move {
                manager.run().await;
                std::thread::park();
            });
        } else {
            manager
                .send((global.background, global.foreground_color, command))
                .await
                .unwrap();
            return;
        }
    }

    Window::run(app, config)
}
