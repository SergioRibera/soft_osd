use std::error::Error;
use std::sync::{Arc, Mutex};

use zbus::{fdo, interface, proxy, Connection};

use crate::app::{AppMessage, MainApp};
use crate::window::AppTy;

pub const APP_ID: &str = "rs.sergioribera.sosd";

pub struct MainAppIPC<T: AppTy> {
    pub(crate) app: Arc<Mutex<T>>,
}

// Define la interfaz D-Bus
#[interface(name = "rs.sergioribera.sosd")]
impl<T: AppTy + 'static> MainAppIPC<T> {
    fn slider(&self, i: String, v: i32) -> fdo::Result<()> {
        println!("Received Slider: {v}");
        let v = (v as f32) / 100.0;
        println!("New Value Slider: {v}");
        self.app
            .lock()
            .unwrap()
            .update(AppMessage::Slider(i.chars().next().unwrap(), v));
        Ok(())
    }
    fn notification(&self, i: String, t: String, d: String) -> fdo::Result<()> {
        println!("Received Notification");
        self.app.lock().unwrap().update(AppMessage::Notification(
            i.chars().next().unwrap(),
            t,
            Some(d),
        ));
        Ok(())
    }
}

// Proxy para enviar mensajes a la instancia existente
#[proxy(
    interface = "rs.sergioribera.sosd",
    default_service = "rs.sergioribera.sosd",
    default_path = "/rs/sergioribera/sosd"
)]
pub trait MainAppIPCSingletone {
    fn slider(&self, i: String, v: i32) -> fdo::Result<()>;
    fn notification(&self, i: String, t: String, d: String) -> fdo::Result<()>;
}
