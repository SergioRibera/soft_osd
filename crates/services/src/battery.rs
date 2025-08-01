use parking_lot::RwLock;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use zbus::Connection;

use crate::Result;

use self::bluez::get_bluez_batteries;
use self::sys::{get_batteries, get_charger};

mod bluez;
mod sys;

/// Battery State Enum
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum BatteryState {
    Discharging,
    Charging,
    NotCharging,
    Full,
    Unknown,
    AtThreshold,
    Invalid,
}

/// Battery Struct
#[derive(Clone, Debug)]
pub struct Battery {
    pub(crate) level: u8,
    pub(crate) name: Arc<str>,
    pub(crate) state: BatteryState,
    pub(crate) path: Option<PathBuf>,
}

impl Battery {
    pub fn level(&self) -> u8 {
        self.level
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn state(&self) -> BatteryState {
        self.state
    }
}

/// Battery Manager Struct
pub struct BatteryManager {
    batteries: RwLock<Vec<Battery>>,
    connection: Connection,
}

impl BatteryManager {
    /// Create a new BatteryManager
    pub async fn new(on_charger: impl Fn(bool) + Send + Sync + 'static) -> Result<Self> {
        let batteries = RwLock::new(Vec::with_capacity(4));
        let on_charger = Arc::new(on_charger);

        let manager = BatteryManager {
            batteries,
            connection: Connection::system().await?,
        };

        manager.refresh().await?;
        Ok(manager)
    }

    pub fn all(&self) -> Vec<Battery> {
        self.batteries.read().clone()
    }

    /// Refresh battery states
    pub async fn refresh(&self) -> Result<()> {
        let mut batteries = get_batteries()?;

        if let Ok(bluez_bats) = get_bluez_batteries(&self.connection).await {
            batteries.extend(bluez_bats);
        }

        batteries.sort_by_key(|b| b.level());
        *self.batteries.write() = batteries;
        Ok(())
    }

    /// Get batteries below a certain level
    pub fn batteries_below(&self, level: u8) -> Vec<Battery> {
        self.batteries
            .read()
            .iter()
            .filter(|b| b.level() < level)
            .cloned()
            .collect()
    }

    /// Get battery by name or path
    pub fn battery_by_name(&self, name: &str) -> Option<Battery> {
        let borrow = self.batteries.read();
        borrow
            .iter()
            .find(|b| {
                b.name() == name || b.path().and_then(|b| b.to_str()).is_some_and(|b| b == name)
            })
            .cloned()
    }
}
