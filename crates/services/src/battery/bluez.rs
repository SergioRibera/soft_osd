use std::collections::HashMap;
use zbus::zvariant::{ObjectPath, Value};
use zbus::Connection;

use crate::{Battery, BatteryState, Error};

type ManagedObjects<'a> = HashMap<ObjectPath<'a>, HashMap<String, HashMap<String, Value<'a>>>>;

/// Probably you need enable https://wiki.archlinux.org/title/Bluetooth#Enabling_experimental_features
/// https://wiki.archlinux.org/title/Bluetooth_headset#Battery_level_reporting
/// https://nixos.wiki/wiki/Bluetooth#Showing_battery_charge_of_bluetooth_devices
pub async fn get_bluez_batteries(conn: &Connection) -> Result<Vec<Battery>, Error> {
    let ret = conn
        .call_method(
            Some("org.bluez"),
            "/",
            Some("org.freedesktop.DBus.ObjectManager"),
            "GetManagedObjects",
            &(),
        )
        .await?;
    let body = ret.body();
    let (devices,): (ManagedObjects<'_>,) = body.deserialize()?;

    Ok(devices
        .iter()
        .filter_map(|(_, ifs)| {
            let bat = ifs.get("org.bluez.Battery1")?;
            let level = bat
                .get("Percentage")
                .and_then(|p| p.clone().downcast::<u8>().ok())?;
            let dev = ifs.get("org.bluez.Device1")?;
            let name = dev
                .get("Name")
                .and_then(|n| n.clone().downcast::<String>().ok())?;
            let icon = dev
                .get("Icon")
                .and_then(|n| n.clone().downcast::<String>().ok())?;
            println!("Icon: {icon}");
            Some(Battery {
                level,
                name,
                // idk how get status, but I can assume is discharging
                state: BatteryState::Discharging,
                // not have Path, right?
                path: None,
            })
        })
        .collect::<Vec<_>>())
}
