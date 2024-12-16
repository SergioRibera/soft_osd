mod battery;
mod notification;
mod singletone;

pub mod error;

use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use zbus::connection::Builder;
use zbus::Connection;

pub use battery::*;
pub use error::Error;
pub use notification::{Icon, Notification};
pub use singletone::SingletoneListener;
pub use zbus;

use notification::{NotificationIPC, NotificationIPCSignals};
use singletone::{SingletoneClientProxy, SingletoneServer};

use self::singletone::GenericMessage;

pub type Result<T> = std::result::Result<T, Error>;

pub trait ServiceReceive<'a> {
    fn set_broadcast(&mut self, broadcast: ServiceBroadcast<'a>);
    fn batteries_below(&mut self, level: u8, batteries: &[Battery]);
}

// This send to app to call actions who is hear by this crate
#[derive(Clone)]
pub struct ServiceBroadcast<'a> {
    notification: Option<Connection>,
    singletone: Option<SingletoneClientProxy<'a>>,
}

pub struct ServiceManager<'a, T, Message>
where
    T: Notification + ServiceReceive<'a>,
{
    is_daemon: bool,
    broadcast: ServiceBroadcast<'a>,
    battery: Option<BatteryManager>,
    refresh_time: Duration,
    battery_levels: Vec<u8>,
    receiver: Arc<Mutex<T>>,
    _msg: PhantomData<Message>,
}

impl<'a> ServiceBroadcast<'a> {
    pub async fn notify_action<T: Notification + 'static>(&self, id: u32, action: &str) {
        let Some(notification) = self.notification.as_ref() else {
            return;
        };

        notification
            .object_server()
            .interface::<_, NotificationIPC<T>>("/org/freedesktop/Notifications")
            .await
            .unwrap()
            .action_invoked(id, action)
            .await
            .unwrap();
    }
}

impl<'a, T, Message> ServiceManager<'a, T, Message>
where
    T: Notification + ServiceReceive<'a> + SingletoneListener<Message> + 'static,
    Message: Serialize + Deserialize<'static> + Send + Sync + 'static,
{
    pub async fn new(is_daemon: bool, receiver: Arc<Mutex<T>>) -> Self {
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
            .await;

        let broadcast = ServiceBroadcast {
            notification: if is_daemon {
                Some(notification.unwrap())
            } else {
                notification.ok()
            },
            singletone: None,
        };
        receiver.lock().unwrap().set_broadcast(broadcast.clone());
        receiver.clear_poison(); // probably not needed, but its for prevent

        Self {
            receiver,
            is_daemon,
            broadcast,
            battery: None,
            battery_levels: Vec::new(),
            refresh_time: Duration::from_secs_f32(5.0),
            _msg: PhantomData::default(),
        }
    }

    pub async fn with_battery(
        self,
        enable: bool,
        refresh_time: f32,
        levels: Vec<u8>,
    ) -> Result<Self> {
        let battery = if enable && self.is_daemon {
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
                let Ok(mut receiver) = self.receiver.lock() else {
                    continue;
                };
                self.battery_levels.iter().for_each(|l| {
                    let batteries_below = battery.batteries_below(*l);
                    if !batteries_below.is_empty() {
                        receiver.batteries_below(*l, &batteries_below);
                    }
                });
            }
            sleep(self.refresh_time).await;
            if let Some(battery) = self.battery.as_ref() {
                _ = battery.refresh().await;
            }
        }
    }

    pub async fn send(&self, msg: Message) -> Result<()> {
        if let Some(singletone) = self.broadcast.singletone.as_ref() {
            let msg = bincode::serialize(&GenericMessage(msg))?;
            return singletone.process_message(msg).await;
        }

        Err(Error::SingletoneNotCreated)
    }

    pub async fn with_singletone(self) -> Result<Self> {
        let server = SingletoneServer(self.receiver.clone(), PhantomData::default());
        let server_conn = Builder::session()?
            .name("rs.sergioribera.sosd")?
            .serve_at("/rs/sergioribera/sosd", server)?
            .build()
            .await;

        if let Err(zbus::Error::NameTaken) = server_conn {
            let conn = Connection::session().await.unwrap();
            let ipc = SingletoneClientProxy::new(&conn).await?;

            return Ok(Self {
                broadcast: ServiceBroadcast {
                    singletone: Some(ipc),
                    ..self.broadcast
                },
                ..self
            });
        }

        let conn = server_conn?;
        let ipc = SingletoneClientProxy::new(&conn).await?;

        Ok(Self {
            broadcast: ServiceBroadcast {
                singletone: Some(ipc),
                ..self.broadcast
            },
            ..self
        })
    }
}
