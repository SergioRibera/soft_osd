use std::collections::{BTreeMap, HashMap};

use serde::{de, ser, Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "reflect", derive(mirror_mirror::Reflect))]
pub struct BatteryConfig {
    pub enabled: bool,
    pub refresh_time: f32,
    pub level: Option<BatteryLevelAlerts>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "reflect", derive(mirror_mirror::Reflect))]
pub struct BatteryLevelAlerts(pub BTreeMap<u8, BatteryLevel>);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "reflect", derive(mirror_mirror::Reflect))]
pub struct BatteryLevel {
    pub icon: String,
    /// The animation duration to show the widget (in seconds)
    pub show_duration: Option<f32>,
    /// Background Color of widget, support: '#RRGGBBAA', '#RGBA' and '#RGB'
    pub background: Option<String>,
    /// Foreground Color of widget, support: '#RRGGBBAA', '#RGBA' and '#RGB'
    pub foreground: Option<String>,
}

impl Serialize for BatteryLevelAlerts {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let map: HashMap<String, &BatteryLevel> =
            self.0.iter().map(|(k, v)| (k.to_string(), v)).collect();
        map.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BatteryLevelAlerts {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let map: HashMap<String, BatteryLevel> = HashMap::deserialize(deserializer)?;
        let map = map
            .into_iter()
            .map(|(k, v)| {
                k.parse::<u8>()
                    .map_err(de::Error::custom)
                    .map(|key| (key, v))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        Ok(BatteryLevelAlerts(map))
    }
}

impl Default for BatteryConfig {
    fn default() -> Self {
        let urgency_default = BatteryLevel {
            icon: "󰁼".to_owned(),
            show_duration: Some(5.0),
            background: Some("#000".to_owned()),
            foreground: Some("#fff".to_owned()),
        };
        Self {
            enabled: false,
            refresh_time: 30.0,
            level: Some(BatteryLevelAlerts(BTreeMap::from_iter([
                (30, urgency_default),
                // Critical
                (
                    15,
                    BatteryLevel {
                        icon: "󰁺".to_owned(),
                        show_duration: Some(10.0),
                        background: Some("#ff6961".to_owned()),
                        foreground: Some("#fff".to_owned()),
                    },
                ),
            ]))),
        }
    }
}
