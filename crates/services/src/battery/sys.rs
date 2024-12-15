use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use crate::{Battery, BatteryState, Error, ServiceResult};

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
    let now_uwh = read_battery_file_energy_or_charge(dir, "now")?;
    let full_uwh = read_battery_file_energy_or_charge(dir, "full")?;

    let level = ((now_uwh * 100) / full_uwh).min(100) as u8;
    Ok(Battery {
        level,
        path: Some(dir.to_path_buf()),
        name: read_battery_file(dir, "model_name").unwrap_or_else(|_| "Unknown".to_owned()),
        state: name_to_battery_state(&read_battery_file(dir, "status")?)?,
    })
}

/// Get all batteries
pub fn get_batteries() -> ServiceResult<Vec<Battery>> {
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
