use std::sync::{Arc, Mutex};

use services::{zbus, Battery, Notification, ServiceBroadcast, ServiceManager, ServiceReceive};

#[derive(Default)]
struct App {
    broadcast: Option<ServiceBroadcast>,
    notifications: Vec<u32>,
}

impl Notification for App {
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

impl ServiceReceive for App {
    fn batteries_below(&self, level: u8, batteries: &[Battery]) {
        println!("Oh no, batteries are below of {level}: {batteries:?}");
    }

    fn set_broadcast(&mut self, broadcast: ServiceBroadcast) {
        self.broadcast.replace(broadcast);
    }
}

#[tokio::main]
async fn main() {
    let receiver = Arc::new(Mutex::new(App::default()));
    let manager = ServiceManager::new(receiver)
        .await
        .with_battery(true, 5.0, vec![80, 50, 30, 15])
        .await
        .unwrap();

    println!("Starting listen services: Battery & Notification");

    manager.run().await;
}
