use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use merge2::Merge;
use serde::{Deserialize, Serialize};

use crate::{swap_option, Urgency};

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, ValueEnum, Serialize, Deserialize)]
pub enum OsdPosition {
    #[default]
    Top,
    Left,
    Right,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Parser, Serialize, Deserialize, Merge)]
#[clap(author, version)]
pub struct Config {
    /// Path to load config
    #[clap(long, short)]
    #[serde(skip)]
    pub config: Option<PathBuf>,

    #[clap(flatten)]
    #[serde(flatten)]
    pub globals: Global,

    #[clap(flatten)]
    #[merge(strategy = merge2::option::recursive)]
    pub window: Option<Window>,

    #[clap(skip)]
    pub urgency_low: UrgencyConfig,
    #[clap(skip)]
    pub urgency_normal: UrgencyConfig,
    #[clap(skip)]
    pub urgency_critical: UrgencyConfig,

    #[clap(subcommand)]
    #[serde(skip)]
    #[merge(strategy = merge2::any::overwrite)]
    pub command: OsdType,
}

#[derive(Debug, Clone, PartialEq, Parser, Serialize, Deserialize, Merge)]
pub struct Global {
    /// The animation duration to show the widget (in milliseconds)
    #[clap(long, short = 'd', default_value = "1.0")]
    #[merge(strategy = swap_option)]
    pub animation_duration: Option<f32>,
    /// The animation duration to show the widget (in seconds)
    #[clap(long, short, default_value = "2.0")]
    #[merge(strategy = swap_option)]
    pub show_duration: Option<f32>,
    /// Background Color of widget, support: '#RRGGBBAA', '#RGBA' and '#RGB'
    #[clap(long, short)]
    #[merge(strategy = swap_option)]
    pub background: Option<String>,
    /// Foreground Color of widget, support: '#RRGGBBAA', '#RGBA' and '#RGB'
    #[clap(long, short)]
    #[merge(strategy = swap_option)]
    pub foreground_color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Parser, Serialize, Deserialize, Merge)]
pub struct Window {
    /// The Position into Screen
    #[clap(long, short, default_value = "top")]
    #[merge(strategy = merge2::any::overwrite)]
    pub position: OsdPosition,
    /// The radius of the widget [default: 100]
    #[clap(long, short)]
    #[merge(strategy = merge2::any::overwrite)]
    pub radius: Option<u32>,
    /// The width of the widget [default: 600]
    #[clap(long, short)]
    #[merge(strategy = merge2::any::overwrite)]
    pub width: Option<u32>,
    /// The height of the widget [default: 80]
    #[clap(long, short = 'a')]
    #[merge(strategy = merge2::any::overwrite)]
    pub height: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Merge)]
pub struct UrgencyConfig {
    /// The animation duration to show the widget (in seconds)
    #[merge(strategy = swap_option)]
    pub show_duration: Option<f32>,
    /// Background Color of widget, support: '#RRGGBBAA', '#RGBA' and '#RGB'
    #[merge(strategy = swap_option)]
    pub background: Option<String>,
    /// Foreground Color of widget, support: '#RRGGBBAA', '#RGBA' and '#RGB'
    #[merge(strategy = swap_option)]
    pub foreground_color: Option<String>,
}

#[derive(Subcommand, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OsdType {
    #[default]
    Daemon,
    Close,
    Init,
    Notification {
        /// Title to show
        #[clap(long, short)]
        title: String,
        /// Image for notification, path or char
        #[clap(long, short = 'm')]
        image: Option<String>,
        /// Urgency of notification
        #[clap(long, short)]
        urgency: Option<Urgency>,
        /// Description to show
        #[clap(long, short)]
        description: Option<String>,
    },
    Slider {
        /// Value for slider, from 0 to 100
        #[clap(long, short)]
        value: i32,
        /// Image for notification, path or char
        #[clap(long, short = 'm')]
        image: Option<String>,
        /// Urgency of notification
        #[clap(long, short)]
        urgency: Option<Urgency>,
    },
}

impl Default for Global {
    fn default() -> Self {
        Self {
            animation_duration: Some(1.0),
            show_duration: Some(5.0),
            background: Some("#000".to_owned()),
            foreground_color: Some("#fff".to_owned()),
        }
    }
}

impl Default for Window {
    fn default() -> Self {
        Self {
            position: Default::default(),
            radius: Some(100),
            width: Some(600),
            height: Some(80),
        }
    }
}

impl Default for UrgencyConfig {
    fn default() -> Self {
        Self {
            show_duration: Some(5.0),
            background: None,
            foreground_color: None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let globals = Global::default();
        let urgency_default = UrgencyConfig {
            show_duration: Some(5.0),
            background: globals.background.clone(),
            foreground_color: globals.foreground_color.clone(),
        };

        Self {
            globals,
            config: None,
            window: Some(Default::default()),
            urgency_low: urgency_default.clone(),
            urgency_normal: urgency_default,
            urgency_critical: UrgencyConfig {
                show_duration: Some(10.0),
                background: Some("#ff6961".to_owned()),
                foreground_color: Some("#fff".to_owned()),
            },
            command: OsdType::Daemon,
        }
    }
}
