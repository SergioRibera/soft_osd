mod notification;
mod sosd;

use std::sync::{Arc, Mutex};

pub use notification::*;
pub use sosd::*;
use zbus::blocking::connection::Builder;
use zbus::blocking::Connection;

use crate::config::OsdType;
use crate::window::AppTy;

pub const APP_ID: &str = "rs.sergioribera.sosd";
pub const APP_PATH: &str = "/rs/sergioribera/sosd";

pub fn connect<T: AppTy + 'static>(command: &OsdType, app: Arc<Mutex<T>>) {
    let server = MainAppIPC(app.clone());
    let notify_server = NotificationIPC(app);

    let ipc_conn = Builder::session()
        .unwrap()
        .name(APP_ID)
        .unwrap()
        .serve_at(APP_PATH, server)
        .unwrap()
        .serve_at("/org/freedesktop/Notifications", notify_server)
        .unwrap()
        .build();

    if let Err(zbus::Error::NameTaken) = ipc_conn {
        let ipc_conn = zbus::blocking::Connection::session().unwrap();
        let ipc = MainAppIPCSingletoneProxyBlocking::new(&ipc_conn).unwrap();
        println!("Sending slider command");
        match command {
            OsdType::Slider { value, icon } => ipc.slider(icon.to_string(), *value as i32).unwrap(),
            OsdType::Notification {
                icon,
                image: _,
                title,
                description,
                font: _,
            } => ipc
                .notification(
                    icon.unwrap_or_default().to_string(),
                    title.clone(),
                    description.clone().unwrap_or_default(),
                )
                .unwrap(),
            _ => {}
        }

        println!("Mensaje enviado a la instancia existente");
        std::process::exit(0);
    }

    if let Err(ref e) = ipc_conn {
        eprintln!("Error al conectar al bus: {e:?}");
    }

    if let OsdType::Daemon = command {
        println!("Servicio D-Bus registrado, esperando mensajes...");
        loop {}
    }
}
