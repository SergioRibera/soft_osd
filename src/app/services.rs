use std::time::Duration;

use config::OsdType;
use services::{Battery, Notification, ServiceBroadcast, ServiceReceive, SingletoneListener};

use super::{App, AppMessage, MainApp, ICON_SIZE};

impl<'a> Notification for MainApp<'a> {
    fn notify(
        &mut self,
        id: u32,
        title: String,
        icon: Option<services::Icon>,
        urgency: config::Urgency,
        body: Option<String>,
        value: Option<f32>,
        _actions: Vec<String>,
        timeout: Option<i32>,
    ) -> zbus::fdo::Result<u32> {
        if let Some(value) = value {
            self.update(AppMessage::Slider {
                icon,
                value,
                urgency,
                timeout,
                bg: None,
                fg: None,
            })
        } else {
            self.update(AppMessage::Notification {
                title,
                body,
                icon,
                urgency,
                timeout,
                bg: None,
                fg: None,
            })
        }
        Ok(id)
    }

    fn close_notification(&mut self, _: u32) -> zbus::fdo::Result<()> {
        self.update(AppMessage::Close);
        Ok(())
    }

    fn get_icon_size(&self) -> f32 {
        *ICON_SIZE.read().unwrap()
    }
}

impl<'a> ServiceReceive<'a> for MainApp<'a> {
    fn batteries_below(&mut self, level: u8, _batteries: &[Battery]) {
        if let Some(battery_config) = self.config.battery.clone().level.as_ref() {
            for (alert_level, config) in &battery_config.0 {
                if *alert_level >= level && !self.notified_levels.contains(alert_level) {
                    // Send Notification
                    self.update(AppMessage::Slider {
                        urgency: config::Urgency::Normal,
                        icon: (config.icon.clone(), self.get_icon_size()).try_into().ok(),
                        timeout: config.show_duration.map(|d| d as i32),
                        value: level as f32,
                        bg: config.background.clone(),
                        fg: config.foreground.clone(),
                    });

                    self.notified_levels.insert(*alert_level);

                    std::thread::sleep(Duration::from_secs_f32(
                        config.show_duration.unwrap_or(5.0),
                    ));
                } else if *alert_level == level && self.notified_levels.contains(alert_level) {
                    self.notified_levels.remove(alert_level);
                }
            }
        }
    }

    fn set_broadcast(&mut self, broadcast: ServiceBroadcast<'a>) {
        self.broadcast.replace(broadcast);
    }
}

impl<'a> SingletoneListener<(Option<String>, Option<String>, OsdType)> for MainApp<'a> {
    fn on_message(&mut self, (bg, fg, msg): (Option<String>, Option<String>, OsdType)) {
        let msg = match msg {
            OsdType::Daemon => None,
            OsdType::Init => None,
            OsdType::Close => Some(AppMessage::Close),
            OsdType::Notification {
                title,
                image,
                urgency,
                description: body,
            } => Some(AppMessage::Notification {
                bg,
                fg,
                title,
                body,
                urgency: urgency.unwrap_or_default(),
                icon: image.and_then(|image| (image, self.get_icon_size()).try_into().ok()),
                timeout: None,
            }),
            OsdType::Slider {
                value,
                image,
                urgency,
            } => Some(AppMessage::Slider {
                bg,
                fg,
                value: value as f32,
                urgency: urgency.unwrap_or_default(),
                icon: image.and_then(|image| (image, self.get_icon_size()).try_into().ok()),
                timeout: None,
            }),
        };
        if let Some(msg) = msg {
            self.update(msg);
        }
    }
}
