use clap::{Parser, ValueEnum};

#[derive(Debug, Default, Clone, Eq, PartialEq, Parser)]
#[clap(author, version)]
pub struct Config {
    /// The Position into Screen
    #[clap(long, short, default_value = "top")]
    pub position: OsdPosition,
    /// The width of the widget
    #[clap(long, short, default_value = "400")]
    pub width: u32,
    /// The height of the widget
    #[clap(long, short = 'a', default_value = "80")]
    pub height: u32,
    /// The radius of the widget
    #[clap(long, short, default_value = "80")]
    pub radius: u32,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, ValueEnum)]
pub enum OsdPosition {
    #[default]
    Top,
    Left,
    Right,
    Bottom,
}
