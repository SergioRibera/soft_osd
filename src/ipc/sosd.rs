use std::ops::Not;
use std::sync::{Arc, Mutex};

use zbus::{fdo::Result, interface, proxy};

use crate::app::AppMessage;
use crate::notification::Urgency;
use crate::window::AppTy;

pub struct MainAppIPC<T: AppTy>(pub Arc<Mutex<T>>);

// Define la interfaz D-Bus
#[interface(name = "rs.sergioribera.sosd")]
impl<T: AppTy + 'static> MainAppIPC<T> {
    async fn close(&self) -> Result<()> {
        self.0.lock().unwrap().update(AppMessage::Close);
        Ok(())
    }
    async fn slider(
        &self,
        urgency: u8,
        value: i32,
        icon: String,
        timeout: i32,
        bg: String,
        fg: String,
    ) -> Result<()> {
        let value = value as f32;
        self.0.lock().unwrap().update(AppMessage::Slider {
            value,
            urgency: Urgency::from(urgency),
            icon: icon.try_into().ok(),
            timeout: (timeout > 0).then_some(timeout),
            bg: bg.is_empty().not().then(|| bg),
            fg: fg.is_empty().not().then(|| fg),
        });
        Ok(())
    }
    async fn notification(
        &self,
        title: String,
        urgency: u8,
        body: String,
        icon: String,
        timeout: i32,
        bg: String,
        fg: String,
    ) -> Result<()> {
        self.0.lock().unwrap().update(AppMessage::Notification {
            title,
            body: body.is_empty().not().then(|| body),
            urgency: Urgency::from(urgency),
            icon: icon.try_into().ok(),
            timeout: (timeout > 0).then_some(timeout),
            bg: bg.is_empty().not().then(|| bg),
            fg: fg.is_empty().not().then(|| fg),
        });
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
    async fn close(&self) -> Result<()>;
    async fn slider(
        &self,
        urgency: u8,
        value: i32,
        icon: String,
        timeout: i32,
        bg: String,
        fg: String,
    ) -> Result<()>;
    async fn notification(
        &self,
        title: String,
        urgency: u8,
        body: String,
        icon: String,
        timeout: i32,
        bg: String,
        fg: String,
    ) -> Result<()>;
}
