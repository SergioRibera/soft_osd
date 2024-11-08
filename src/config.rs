use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, ValueEnum)]
pub enum OsdPosition {
    #[default]
    Top,
    Left,
    Right,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Parser)]
#[clap(author, version)]
pub struct Config {
    /// The Position into Screen
    #[clap(long, short, default_value = "top")]
    pub position: OsdPosition,
    /// The width of the widget
    #[clap(long, short, default_value = "600")]
    pub width: u32,
    /// The height of the widget
    #[clap(long, short = 'a', default_value = "80")]
    pub height: u32,
    /// The radius of the widget
    #[clap(long, short, default_value = "100")]
    pub radius: u32,
    /// The animation duration to show the widget (in milliseconds)
    #[clap(long, short = 'd', default_value = "1.0")]
    pub animation_duration: f32,
    /// The animation duration to show the widget (in seconds)
    #[clap(long, short, default_value = "2.0")]
    pub show_duration: f32,
    /// Background Color of widget, support: '#RRGGBBAA', '#RGBA' and '#RGB'
    #[clap(long, short, default_value = "#000")]
    pub background: String,
    /// Foreground Color of widget, support: '#RRGGBBAA', '#RGBA' and '#RGB'
    #[clap(long, short = 'c', default_value = "#FFF")]
    pub foreground_color: String,

    #[clap(subcommand)]
    pub command: OsdType,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
pub enum OsdType {
    Daemon,
    Notification {
        /// Icons for notification
        #[clap(long, short)]
        icon: Option<char>,
        /// Image for notification
        #[clap(long, short = 'm')]
        image: Option<String>,
        /// Title to show
        #[clap(long, short)]
        title: String,
        /// Description to show
        #[clap(long, short)]
        description: Option<String>,
        /// Font for text in notifications mode, support: serif, sans-serif, monospace, cursive, fantasy or explicit name of font
        #[clap(long, short, default_value = "sans-serif")]
        font: String,
    },
    Slider {
        /// Value for slider, from 0.0 to 1.0
        #[clap(long, short)]
        value: f32,
        /// Icons for slider
        #[clap(long, short, default_value = "󰂭")]
        icon: char,
    },
}
