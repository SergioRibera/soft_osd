use std::collections::HashMap;
use std::ops::Not;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

use config::Urgency;
use parking_lot::Mutex;
use zbus::fdo::Result;
use zbus::interface;
use zbus::object_server::SignalEmitter;

use crate::Icon;

use super::Notification;

static ID_COUNT: AtomicU32 = AtomicU32::new(1);
fn fetch_id() -> u32 {
    ID_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

pub struct NotificationIPC<T: Notification>(pub(crate) Arc<Mutex<T>>);

unsafe impl<T: Notification> Send for NotificationIPC<T> {}
unsafe impl<T: Notification> Sync for NotificationIPC<T> {}

#[interface(name = "org.freedesktop.Notifications")]
impl<T: Notification + 'static> NotificationIPC<T> {
    /// "action-icons"	Supports using icons instead of text for displaying actions. Using icons for actions must be enabled on a per-notification basis using the "action-icons" hint.
    /// "actions"	The server will provide the specified actions to the user. Even if this cap is missing, actions may still be specified by the client, however the server is free to ignore them.
    /// "body"	Supports body text. Some implementations may only show the summary (for instance, onscreen displays, marquee/scrollers)
    /// "body-hyperlinks"	The server supports hyperlinks in the notifications.
    /// "body-images"	The server supports images in the notifications.
    /// "body-markup"	Supports markup in the body text. If marked up text is sent to a server that does not give this cap, the markup will show through as regular text so must be stripped clientside.
    /// "icon-multi"	The server will render an animation of all the frames in a given image array. The client may still specify multiple frames even if this cap and/or "icon-static" is missing, however the server is free to ignore them and use only the primary frame.
    /// "icon-static"	Supports display of exactly 1 frame of any given image array. This value is mutually exclusive with "icon-multi", it is a protocol error for the server to specify both.
    /// "persistence"	The server supports persistence of notifications. Notifications will be retained until they are acknowledged or removed by the user or recalled by the sender. The presence of this capability allows clients to depend on the server to ensure a notification is seen and eliminate the need for the client to display a reminding function (such as a status icon) of its own.
    /// "sound"	The server supports sounds on notifications. If returned, the server must support the "sound-file" and "suppress-sound" hints.
    fn get_capabilities(&self) -> Result<Vec<&'static str>> {
        self.0.lock().get_capabilities()
    }

    ///
    /// app_name	STRING	The optional name of the application sending the notification. Can be blank.
    ///
    /// replaces_id	UINT32	The optionall notification ID that this notification replaces. The server must atomically (ie with no flicker or other visual cues) replace the given notification with this one. This allows clients to effectively modify the notification while it's active. A value of value of 0 means that this notification won't replace any existing notifications.
    ///
    /// app_icon	STRING	The optional program icon of the calling application. See Icons and Images. Can be an empty string, indicating no icon.
    ///
    /// summary	STRING	The summary text briefly describing the notification.
    ///
    /// body	STRING	The optional detailed body text. Can be empty.
    ///
    /// actions	as	Actions are sent over as a list of pairs. Each even element in the list (starting at index 0) represents the identifier for the action. Each odd element in the list is the localized string that will be displayed to the user.
    ///
    /// hints	a{sv}	Optional hints that can be passed to the server from the client program. Although clients and servers should never assume each other supports any specific hints, they can be used to pass along information, such as the process PID or window ID, that the server may be able to make use of. See Hints. Can be empty.
    /// expire_timeout	INT32
    ///
    /// The timeout time in milliseconds since the display of the notification at which the notification should automatically close.
    /// If -1, the notification's expiration time is dependent on the notification server's settings, and may vary for the type of notification. If 0, never expire.
    fn notify(
        &self,
        _app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, zbus::zvariant::Value>,
        expire_timeout: i32,
    ) -> Result<u32> {
        let mut inner = self.0.lock();
        let icon_size = inner.get_icon_size();
        // The spec says that:
        // If `replaces_id` is 0, we should create a fresh id and notification.
        // If `replaces_id` is not 0, we should create a replace the notification with that id,
        // using the same id.
        // With our implementation, we send a "new" notification anyway, and let management deal
        // with replacing data.
        // When `Config::replacing_enabled` is `false`, we still obey this, those notifications
        // will just have the same `id`, which I think is fine.
        //
        // @NOTE: Some programs don't seem to obey these rules.  Discord will set replaces_id to `id` no
        // matter what.  To workaround this, we just check if a notification with the same ID
        // exists before sending it (see: `main`), rather than relying on `replaces_id` being set
        // correctly.
        // Also note that there is still a bug here, where since Discord sends the `replaces_id` it
        // is effectively assigning its own id, which may interfere with ours.  Not sure how mmuch I can
        // do about this.
        let id = if replaces_id == 0 {
            // Grab an ID atomically.  This is moreso to allow global access to `ID_COUNT`, but I'm
            // also not sure if `notify` is called in a single-threaded way, so it's best to be safe.
            fetch_id()
        } else {
            replaces_id
        };

        let icon: Option<Icon> = (app_icon, icon_size).try_into().ok().or_else(|| {
            if let Some(Ok(path)) = hints
                .get("image-path")
                .or(hints.get("image_path"))
                .map(|p| p.clone().downcast::<String>())
            {
                return (path, icon_size).try_into().ok();
            }
            if let Some(data) = hints
                .get("image-data")
                .or(hints.get("image_data"))
                .or(hints.get("icon_data"))
            {
                return Icon::from_value(data, icon_size);
            }
            None
        });

        let timeout = if expire_timeout <= 0 {
            None
        } else {
            Some(expire_timeout)
        };

        let urgency = hints
            .get("urgency")
            .and_then(|u| u.clone().downcast::<u8>().ok())
            .map(|u| Urgency::from(u))
            .unwrap_or_default();
        let body = body
            .is_empty()
            .not()
            .then(|| body.lines().next().map(|l| l.to_owned()))
            .flatten();

        if let Some(value) = hints
            .get("value")
            .and_then(|v| v.clone().downcast::<i32>().ok())
        {
            let value = value as f32;
            let value = f32::clamp(value * 0.01, 0.0, 1.0);

            return inner.notify(
                id,
                summary,
                icon,
                urgency,
                body,
                Some(value),
                actions,
                timeout,
            );
        }

        inner.notify(id, summary, icon, urgency, body, None, actions, timeout)
    }

    fn close_notification(&self, id: u32) -> Result<()> {
        self.0.lock().close_notification(id)
    }

    #[zbus(out_args("name", "vendor", "version", "spec_version"))]
    fn get_server_information(&self) -> Result<(String, String, String, String)> {
        self.0.lock().get_server_information()
    }

    #[zbus(signal)]
    async fn action_invoked(
        signal_ctxt: &SignalEmitter<'_>,
        id: u32,
        action_key: &str,
    ) -> zbus::Result<()>;

    /// id	UINT32	The ID of the notification that was closed.
    /// reason	UINT32
    ///
    /// The reason the notification was closed.
    ///
    /// 1 - The notification expired.
    ///
    /// 2 - The notification was dismissed by the user.
    ///
    /// 3 - The notification was closed by a call to CloseNotification.
    ///
    /// 4 - Undefined/reserved reasons.
    #[zbus(signal)]
    async fn notification_closed(
        signal_ctxt: &SignalEmitter<'_>,
        id: u32,
        reason: u32,
    ) -> zbus::Result<()>;
}
