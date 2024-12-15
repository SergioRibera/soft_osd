use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use zbus::Connection;

use crate::Result;

use self::bluez::get_bluez_batteries;
use self::sys::get_batteries;

mod bluez;
mod sys;

/// Battery State Enum
#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum BatteryState {
    Discharging,
    Charging,
    #[serde(rename = "Not charging")]
    NotCharging,
    Full,
    Unknown,
    #[serde(rename = "At threshold")]
    AtThreshold,
    Invalid,
}

/// Battery Struct
#[derive(Debug)]
pub struct Battery {
    pub(crate) level: u8,
    pub(crate) name: String,
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
    batteries: Vec<Battery>,
    connection: Connection,
}

impl BatteryManager {
    /// Create a new BatteryManager
    pub async fn new() -> Result<Self> {
        let mut manager = BatteryManager {
            batteries: Vec::new(),
            connection: Connection::system().await?,
        };
        manager.refresh().await?;
        Ok(manager)
    }

    pub fn all(&self) -> &[Battery] {
        &self.batteries
    }

    /// Refresh battery states
    pub async fn refresh(&mut self) -> Result<()> {
        let mut batteries = get_batteries()?;

        if let Ok(bluez_bats) = get_bluez_batteries(&self.connection).await {
            batteries.extend(bluez_bats);
        }

        self.batteries = batteries;
        Ok(())
    }

    /// Get batteries below a certain level
    pub fn batteries_below(&self, level: u8) -> Vec<&Battery> {
        self.batteries
            .iter()
            .filter(|b| b.level() < level)
            .collect()
    }

    /// Get battery by name or path
    pub fn battery_by_name(&self, name: &str) -> Option<&Battery> {
        self.batteries.iter().find(|b| {
            b.name() == name || b.path().and_then(|b| b.to_str()).is_some_and(|b| b == name)
        })
    }
}
