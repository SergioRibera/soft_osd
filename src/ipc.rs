mod notification;
mod sosd;

use std::sync::{Arc, Mutex};

pub use notification::*;
pub use sosd::*;
use zbus::connection::Builder;

use crate::config::OsdType;
use crate::window::AppTy;

pub const APP_ID: &str = "rs.sergioribera.sosd";
pub const APP_PATH: &str = "/rs/sergioribera/sosd";

pub async fn connect<T: AppTy + 'static>(command: &OsdType, app: Arc<Mutex<T>>) {
    let server = MainAppIPC(app.clone());

    let ipc_conn = Builder::session()
        .unwrap()
        .name(APP_ID)
        .unwrap()
        .serve_at(APP_PATH, server)
        .unwrap()
        .build()
        .await;

    if let Err(zbus::Error::NameTaken) = ipc_conn {
        let ipc_conn = zbus::Connection::session().await.unwrap();
        let ipc = MainAppIPCSingletoneProxy::new(&ipc_conn).await.unwrap();
        println!("Sending slider command");
        match command {
            OsdType::Slider {
                value,
                image,
                urgency,
                background,
                expire_timeout,
                foreground_color,
            } => ipc
                .slider(
                    urgency.clone().unwrap_or_default().into(),
                    *value as i32,
                    image.clone().unwrap_or_default(),
                    expire_timeout.unwrap_or(-1),
                    background.clone().unwrap_or_default(),
                    foreground_color.clone().unwrap_or_default(),
                )
                .await
                .unwrap(),
            OsdType::Notification {
                image,
                title,
                description,
                urgency,
                expire_timeout,
                background,
                foreground_color,
            } => ipc
                .notification(
                    title.clone(),
                    urgency.clone().unwrap_or_default().into(),
                    description.clone().unwrap_or_default(),
                    image.clone().unwrap_or_default(),
                    expire_timeout.unwrap_or(-1),
                    background.clone().unwrap_or_default(),
                    foreground_color.clone().unwrap_or_default(),
                )
                .await
                .unwrap(),
            OsdType::Close => ipc.close().await.unwrap(),
            OsdType::Daemon => {}
        }

        println!("Mensaje enviado a la instancia existente");
        std::process::exit(0);
    }

    if let Err(ref e) = ipc_conn {
        eprintln!("Error al conectar al bus: {e:?}");
    }

    if let OsdType::Daemon = command {
        let notify_server = NotificationIPC(app);
        _ = Builder::session()
            .unwrap()
            .name("org.freedesktop.Notifications")
            .unwrap()
            .serve_at("/org/freedesktop/Notifications", notify_server)
            .unwrap()
            .build()
            .await
            .unwrap();

        println!("Servicio D-Bus registrado, esperando mensajes...");
        std::thread::park()
    }
}
