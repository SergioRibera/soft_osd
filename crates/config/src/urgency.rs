use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::{Config, UrgencyItemConfig};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, ValueEnum)]
pub enum Urgency {
    Low,
    #[default]
    Normal,
    Critical,
}

impl From<u8> for Urgency {
    fn from(value: u8) -> Self {
        match value {
            0 => Urgency::Low,
            2 => Urgency::Critical,
            _ => Urgency::Normal,
        }
    }
}

impl From<Urgency> for u8 {
    fn from(value: Urgency) -> Self {
        match value {
            Urgency::Low => 0,
            Urgency::Normal => 1,
            Urgency::Critical => 2,
        }
    }
}

impl From<(&'_ Config, Urgency)> for UrgencyItemConfig {
    fn from((config, urg): (&'_ Config, Urgency)) -> Self {
        match urg {
            Urgency::Low => config.urgency.low.clone(),
            Urgency::Normal => config.urgency.normal.clone(),
            Urgency::Critical => config.urgency.critical.clone(),
        }
    }
}
