mod battery;
pub mod error;
mod notification;

use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub use error::Error;
pub use notification::{Icon, Notification};
use tokio::time::sleep;
pub use zbus;

use battery::*;
use notification::{NotificationIPC, NotificationIPCSignals};

use zbus::connection::Builder;
use zbus::Connection;

pub type Result<T> = std::result::Result<T, Error>;

pub trait ServiceReceive {
    fn new(&mut self, broadcast: ServiceBroadcast) -> Self;
    fn batteries_below(&self, batteries: &[Battery]);
}

// This send to app to call actions who is hear by this crate
pub struct ServiceBroadcast {
    notification: Connection,
}

pub struct ServiceManager<T: Notification + ServiceReceive> {
    battery: Option<BatteryManager>,
    refresh_time: Duration,
    battery_levels: Vec<u8>,
    receiver: Arc<Mutex<T>>,
}

impl ServiceBroadcast {
    pub async fn notify_action<T: Notification + 'static>(&self, id: u32, action: &str) {
        self.notification
            .object_server()
            .interface::<_, NotificationIPC<T>>("/org/freedesktop/Notifications")
            .await
            .unwrap()
            .action_invoked(id, action)
            .await
            .unwrap();
    }
}

impl<T: Notification + ServiceReceive + 'static> ServiceManager<T> {
    pub async fn new(receiver: Arc<Mutex<T>>) -> Self {
        let notification = Builder::session()
            .unwrap()
            .name("org.freedesktop.Notifications")
            .unwrap()
            .serve_at(
                "/org/freedesktop/Notifications",
                NotificationIPC(receiver.clone()),
            )
            .unwrap()
            .build()
            .await
            .unwrap();

        let actionable = ServiceBroadcast { notification };
        receiver.lock().unwrap().new(actionable);
        receiver.clear_poison(); // probably not needed, but its for prevent

        Self {
            battery: None,
            receiver,
            battery_levels: Vec::new(),
            refresh_time: Duration::from_secs_f32(5.0),
        }
    }

    pub async fn with_battery(
        self,
        enable: bool,
        refresh_time: f32,
        levels: Vec<u8>,
    ) -> Result<Self> {
        let battery = if enable {
            Some(BatteryManager::new().await?)
        } else {
            None
        };
        Ok(Self {
            battery,
            battery_levels: levels,
            refresh_time: Duration::from_secs_f32(refresh_time),
            ..self
        })
    }

    pub async fn run(&self) {
        loop {
            if let Some(battery) = self.battery.as_ref() {
                let batteries_bellow = self
                    .battery_levels
                    .iter()
                    .map(|l| battery.batteries_below(*l))
                    .flatten()
                    .collect::<Vec<Battery>>();
                if !batteries_bellow.is_empty() {
                    if let Ok(receiver) = self.receiver.lock() {
                        receiver.batteries_below(&batteries_bellow);
                    }
                }
            }
            sleep(self.refresh_time).await;
            if let Some(battery) = self.battery.as_ref() {
                _ = battery.refresh().await;
            }
        }
    }
}
