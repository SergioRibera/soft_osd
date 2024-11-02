use std::time::Instant;

use raqote::*;

use crate::components::{Background, Component, Icon, Slider};
use crate::config::Config;
use crate::utils::{ease_out_cubic, ToColor};

pub trait App: From<Config> {
    fn run(&mut self, exit: &mut bool, ctx: &mut DrawTarget);
}

pub struct MainApp {
    // Icons
    volume_icons: Vec<char>,

    // Components
    icon: Icon,
    background: Background,
    slider: Slider,

    // Flow control
    time: Instant,
    is_exiting: bool,
    show_duration: f32,
    start_time: Instant,
    animation_progress: f32,
    animation_duration: f32,
}

impl From<Config> for MainApp {
    fn from(config: Config) -> Self {
        let icon_color = config.icon_color.to_color();
        let volume_icons = config.volume_icon.chars().map(|v| v).collect::<Vec<_>>();

        let background = Background::new(&config, ());
        let slider = Slider::new(&config, 0.5);
        let icon = Icon::new(
            &config,
            (
                icon_color,
                *volume_icons.get(0).expect("Cannot get volume icons"),
            ),
        );

        let animation_duration = config.animation_duration;
        let show_duration = config.show_duration + animation_duration;

        Self {
            icon,
            slider,
            background,

            volume_icons,

            show_duration,
            is_exiting: false,
            animation_duration,
            time: Instant::now(),
            animation_progress: 0.0,
            start_time: Instant::now(),
        }
    }
}

impl App for MainApp {
    fn run(&mut self, exit: &mut bool, ctx: &mut DrawTarget) {
        let delta = self.time.elapsed().as_secs_f32();
        self.time = Instant::now();

        if !self.is_exiting && self.start_time.elapsed().as_secs_f32() >= self.show_duration {
            println!("Exiting: {:?}", self.start_time.elapsed().as_secs_f32());
            self.is_exiting = true;
            self.animation_progress = 0.0;
        }
        if !self.is_exiting && self.start_time.elapsed().as_secs_f32() <= self.animation_duration
            || self.is_exiting
        {
            self.animation_progress = self.animation_progress + delta / self.animation_duration;
        }
        let progress = if self.is_exiting {
            1.0 - ease_out_cubic(self.animation_progress)
        } else {
            ease_out_cubic(self.animation_progress)
        };

        self.background.draw(ctx, progress);
        self.slider.draw(ctx, progress);
        self.icon.draw(ctx, progress);

        if self.is_exiting && self.animation_progress >= 1.0 {
            *exit = true;
        }
    }
}
