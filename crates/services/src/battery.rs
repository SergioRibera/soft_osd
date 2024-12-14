use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Error, ServiceResult};

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
    state: BatteryState,
    path: PathBuf,
    name: String,
    now_uwh: u64,
    full_uwh: u64,
}

impl Battery {
    pub fn level(&self) -> u8 {
        let level = (self.now_uwh * 100) / self.full_uwh;
        level.min(100) as u8
    }

    pub fn path(&self) -> &Path {
        &self.path
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
}

impl BatteryManager {
    /// Create a new BatteryManager
    pub fn new() -> ServiceResult<Self> {
        let mut manager = BatteryManager {
            batteries: Vec::new(),
        };
        manager.refresh()?;
        Ok(manager)
    }

    pub fn all(&self) -> &[Battery] {
        &self.batteries
    }

    /// Refresh battery states
    pub fn refresh(&mut self) -> ServiceResult<()> {
        self.batteries = get_batteries()?;
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
        self.batteries
            .iter()
            .find(|b| b.name() == name || b.path().to_str().unwrap_or("") == name)
    }
}

/// Read a battery file
fn read_battery_file(dir: &Path, file: &str) -> ServiceResult<String> {
    let mut content = fs::read_to_string(dir.join(file))?;
    if let Some(idx) = content.find('\n') {
        content.truncate(idx);
    }
    Ok(content)
}

/// Parse a battery state name into BatteryState
fn name_to_battery_state(name: &str) -> ServiceResult<BatteryState> {
    serde_plain::from_str(name).map_err(|_| Error::InvalidBatteryState(name.to_owned()))
}

/// Read battery energy or charge file
fn read_battery_file_energy_or_charge(dir: &Path, partial_file: &str) -> ServiceResult<u64> {
    if let Ok(value) = read_battery_file(dir, &format!("energy_{}", partial_file))?.parse() {
        return Ok(value);
    }

    let voltage: u64 = read_battery_file(dir, "voltage_now")?.parse()?;
    let charge: u64 = read_battery_file(dir, &format!("charge_{}", partial_file))?.parse()?;
    Ok((charge * voltage) / 1000)
}

/// Read battery directory and create a Battery
fn read_battery_dir(dir: &Path) -> ServiceResult<Battery> {
    Ok(Battery {
        path: dir.to_path_buf(),
        name: read_battery_file(dir, "model_name").unwrap_or_else(|_| "Unknown".to_owned()),
        state: name_to_battery_state(&read_battery_file(dir, "status")?)?,
        now_uwh: read_battery_file_energy_or_charge(dir, "now")?,
        full_uwh: read_battery_file_energy_or_charge(dir, "full")?,
    })
}

/// Get all batteries
fn get_batteries() -> ServiceResult<Vec<Battery>> {
    let batteries = fs::read_dir("/sys/class/power_supply")?
        .filter_map(|entry| entry.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(OsStr::to_str)
                .unwrap_or("")
                .starts_with("BAT")
        })
        .map(|p| read_battery_dir(&p))
        .collect::<ServiceResult<Vec<_>>>()?;

    Ok(batteries)
}
