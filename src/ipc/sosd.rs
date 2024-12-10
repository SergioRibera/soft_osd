use std::sync::{Arc, Mutex};

use zbus::{fdo::Result, interface, proxy};

use crate::app::{AppMessage, MainApp};
use crate::window::AppTy;

pub struct MainAppIPC<T: AppTy>(pub Arc<Mutex<T>>);

// Define la interfaz D-Bus
#[interface(name = "rs.sergioribera.sosd")]
impl<T: AppTy + 'static> MainAppIPC<T> {
    async fn slider(&self, i: String, v: i32) -> Result<()> {
        let v = v as f32;
        self.0
            .lock()
            .unwrap()
            .update(AppMessage::Slider(i.chars().next().unwrap(), v));
        Ok(())
    }
    async fn notification(&self, i: String, t: String, d: String) -> Result<()> {
        println!("Received Notification");
        self.0
            .lock()
            .unwrap()
            .update(AppMessage::Notification(i.chars().next(), t, Some(d)));
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
    async fn slider(&self, i: String, v: i32) -> Result<()>;
    async fn notification(&self, i: String, t: String, d: String) -> Result<()>;
}
