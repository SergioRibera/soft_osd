use clap::{Parser, ValueEnum};

#[derive(Debug, Default, Clone, PartialEq, Parser)]
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
    #[clap(
        long,
        short,
        default_value = "#000",
        help = "Background Color of widget, support: '#RRGGBBAA', '#RGBA' and '#RGB'"
    )]
    pub background: String,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, ValueEnum)]
pub enum OsdPosition {
    #[default]
    Top,
    Left,
    Right,
    Bottom,
}
