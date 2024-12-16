use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use services::{
    zbus, Battery, Notification, ServiceBroadcast, ServiceManager, ServiceReceive,
    SingletoneListener,
};

#[derive(Default)]
struct App<'a> {
    broadcast: Option<ServiceBroadcast<'a>>,
    notifications: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
enum AppMessage {
    Ping,
    Pong,
}

impl<'a> Notification for App<'a> {
    fn notify(
        &mut self,
        id: u32,
        _summary: String,
        _icon: Option<services::Icon>,
        _urgency: config::Urgency,
        _body: Option<String>,
        _value: Option<f32>,
        _actions: Vec<String>,
        _expire_timeout: Option<i32>,
    ) -> zbus::fdo::Result<u32> {
        self.notifications.push(id);
        Ok(id)
    }

    fn close_notification(&mut self, id: u32) -> zbus::fdo::Result<()> {
        if let Some(id) = self.notifications.iter().position(|i| *i == id) {
            self.notifications.remove(id);
        }
        Ok(())
    }
}

impl<'a> ServiceReceive<'a> for App<'a> {
    fn batteries_below(&mut self, level: u8, batteries: &[Battery]) {
        println!("Oh no, batteries are below of {level}: {batteries:?}");
    }

    fn set_broadcast(&mut self, broadcast: ServiceBroadcast<'a>) {
        self.broadcast.replace(broadcast);
    }
}

impl<'a> SingletoneListener<AppMessage> for App<'a> {
    fn on_message(&mut self, msg: AppMessage) {
        println!("Message: {msg:?}");
    }
}

#[tokio::main]
async fn main() {
    let receiver = Arc::new(Mutex::new(App::default()));
    let manager = Arc::new(
        ServiceManager::new(true, receiver)
            .await
            .with_battery(true, 5.0, vec![80, 50, 30, 15])
            .await
            .unwrap()
            .with_singletone()
            .await
            .unwrap(),
    );

    println!("Starting listen services: Battery & Notification");

    {
        let manager = manager.clone();
        tokio::spawn(async move { manager.run().await });
    }

    manager.send(AppMessage::Ping).await.unwrap();

    std::thread::park();
}
