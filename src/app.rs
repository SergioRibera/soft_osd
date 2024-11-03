use std::time::Instant;

use raqote::*;

use crate::components::{Background, Component, Icon, Slider, Text};
use crate::config::{Config, OsdType};
use crate::utils::{ease_out_cubic, ToColor};

pub trait App: From<Config> {
    fn run(&mut self, exit: &mut bool, ctx: &mut DrawTarget);
}

pub struct MainApp {
    // Components
    icon: Option<Icon>,
    background: Background,
    slider: Option<Slider>,
    title: Option<Text>,
    description: Option<Text>,

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
        let fg_color = config.foreground_color.to_color();
        let animation_duration = config.animation_duration;
        let show_duration = config.show_duration + animation_duration;

        let background = Background::new(&config, ());
        let mut slider = None;
        let mut icon = None;
        let mut title = None;
        let mut description = None;

        match &config.command {
            OsdType::Notification {
                icon: i,
                image: _,
                title: t,
                description: d,
                font,
            } => {
                if let Some(i) = i {
                    icon.replace(Icon::new(&config, (fg_color.clone(), *i)));
                }
                title.replace(Text::new(
                    &config,
                    (30.0, 0.2, font.clone(), fg_color, t.clone()),
                ));
                if let Some(d) = d {
                    description.replace(Text::new(
                        &config,
                        (50.0, 0.15, font.clone(), fg_color, d.clone()),
                    ));
                }
            }
            OsdType::Slider { value, icon: i } => {
                icon.replace(Icon::new(&config, (fg_color.clone(), *i)));
                slider.replace(Slider::new(&config, *value));
            }
        }

        Self {
            icon,
            title,
            slider,
            background,
            description,

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
        if let Some(slider) = self.slider.as_mut() {
            slider.draw(ctx, progress);
        }
        if let Some(icon) = self.icon.as_mut() {
            icon.draw(ctx, progress);
        }
        if let Some(title) = self.title.as_mut() {
            title.draw(ctx, progress);
        }
        if let Some(description) = self.description.as_mut() {
            description.draw(ctx, progress);
        }

        if self.is_exiting && self.animation_progress >= 1.0 {
            *exit = true;
        }
    }
}
