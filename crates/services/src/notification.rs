use config::Urgency;
use zbus::fdo::Result;

mod bus;
mod icon;

pub use bus::{NotificationIPC, NotificationIPCSignals};
pub use icon::Icon;

pub trait Notification {
    fn get_icon_size(&self) -> f32 {
        18.0
    }
    fn get_capabilities(&self) -> Result<Vec<&'static str>> {
        let capabilities = [
            // "action-icons",
            "actions",
            "body",
            // "body-hyperlinks",
            //"body-markup",
            // "icon-multi",
            "icon-static",
            //"persistence",
            //"sound",
        ];

        Ok(capabilities.to_vec())
    }

    fn get_server_information(&self) -> Result<(String, String, String, String)> {
        Ok((
            env!("CARGO_PKG_NAME").to_string(),
            env!("CARGO_PKG_AUTHORS").to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
            "1.2".to_string(),
        ))
    }

    fn notify(
        &self,
        id: u32,
        summary: String,
        icon: Option<Icon>,
        urgency: Urgency,
        body: Option<String>,
        value: Option<f32>,
        actions: Vec<String>,
        expire_timeout: Option<i32>,
    ) -> Result<u32>;

    fn close_notification(&self, id: u32) -> Result<()>;
}
