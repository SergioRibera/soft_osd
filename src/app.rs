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
        let radius = config.radius as f32;
        let half_y = config.height as f32 / 2.0;
        let mut safe_left = (radius * 2.0) - 20.0;

        let background = Background::new(&config, (None, None), ());
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
                let mut has_desc = false;
                let mut max_size_text = 3.7;
                if let Some(i) = i {
                    icon.replace(Icon::new(
                        &config,
                        (Some(safe_left), Some(half_y)),
                        (fg_color.clone(), *i),
                    ));
                    safe_left += 30.0;
                    max_size_text = 4.0;
                }
                if let Some(d) = d {
                    has_desc = true;
                    description.replace(Text::new(
                        &config,
                        (Some(safe_left), Some(50.0)),
                        (0.15, max_size_text, font.clone(), fg_color, d.clone()),
                    ));
                }
                title.replace(Text::new(
                    &config,
                    (Some(safe_left), Some(if has_desc { 30.0 } else { half_y })),
                    (0.15, max_size_text, font.clone(), fg_color, t.clone()),
                ));
            }
            OsdType::Slider { value, icon: i } => {
                icon.replace(Icon::new(
                    &config,
                    (Some(safe_left), Some(half_y)),
                    (fg_color.clone(), *i),
                ));
                slider.replace(Slider::new(
                    &config,
                    (Some(safe_left + 40.0), Some(half_y)),
                    (*value, 4.1),
                ));
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
