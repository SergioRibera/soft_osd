use std::time::Instant;

use raqote::*;
use serde::{Deserialize, Serialize};

use crate::components::{Background, Component, Icon, Slider, Text};
use crate::config::{Config, OsdType};
use crate::utils::{ease_out_cubic, ToColor};

pub trait App: From<Config> + Sized + Sync + Send {
    fn update(&mut self, _: AppMessage) {}
    fn draw(&mut self, ctx: &mut DrawTarget);
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AppMessage {
    Close,
    Slider(char, f32),
    Notification(Option<char>, String, Option<String>),
}

pub struct MainApp {
    // Components
    icon: Option<Icon>,
    background: Background,
    slider: Option<Slider>,
    title: Option<Text>,
    description: Option<Text>,

    // Variable Needed to create Components
    radius: f32,
    half_y: f32,
    config: Config,
    safe_left: f32,
    fg_color: SolidSource,

    // changed_content: bool,
    // content_anim_progress: f32,

    // Flow control
    show: bool,
    time: Instant,
    is_exiting: bool,
    show_duration: f32,
    start_time: Instant,
    animation_progress: f32,
    animation_duration: f32,
}

unsafe impl Send for MainApp {}
unsafe impl Sync for MainApp {}

impl From<Config> for MainApp {
    fn from(config: Config) -> Self {
        let fg_color = config.foreground_color.to_color();
        let animation_duration = config.animation_duration;
        let show_duration = config.show_duration + animation_duration;
        let radius = config.radius as f32;
        let half_y = config.height as f32 / 2.0;
        let safe_left = (radius * 2.0) - 20.0;

        let background = Background::new(&config, (None, None), ());

        Self {
            config,
            half_y,
            radius,
            fg_color,
            safe_left,

            background,
            icon: None,
            title: None,
            slider: None,
            description: None,
            // changed_content: false,
            // content_anim_progress: 0.0,
            show: false,
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
    fn update(&mut self, msg: AppMessage) {
        let mut safe_left = self.safe_left;
        // if self.show {
        //     self.icon = None;
        //     self.title = None;
        //     self.slider = None;
        //     self.description = None;
        //     self.show_duration += self.config.show_duration;
        // }
        match msg {
            AppMessage::Slider(i, value) => {
                println!("Updating Slider");
                if i != '\x00' {
                    self.icon.replace(Icon::new(
                        &self.config,
                        (Some(safe_left), Some(self.half_y)),
                        (self.fg_color.clone(), i),
                    ));
                    safe_left += 40.0;
                }
                self.slider.replace(Slider::new(
                    &self.config,
                    (Some(safe_left), Some(self.half_y)),
                    (value as f32 / 100.0, 4.1),
                ));
                self.time = Instant::now();
                self.start_time = Instant::now();
                self.show = true;
                // self.changed_content = true;
            }
            AppMessage::Notification(i, title_content, description) => {
                println!("Updating Notification");
                let mut has_desc = false;
                let mut max_size_text = 3.7;
                if let Some(i) = i {
                    self.icon.replace(Icon::new(
                        &self.config,
                        (Some(safe_left), Some(self.half_y)),
                        (self.fg_color.clone(), i),
                    ));
                    safe_left += 30.0;
                    max_size_text = 4.0;
                }
                if let Some(description) = description {
                    if !description.is_empty() {
                        has_desc = true;
                        self.description.replace(Text::new(
                            &self.config,
                            (Some(safe_left), Some(50.0)),
                            (
                                0.15,
                                max_size_text,
                                "monospace".to_owned(),
                                self.fg_color,
                                description,
                            ),
                        ));
                    }
                }
                self.title.replace(Text::new(
                    &self.config,
                    (
                        Some(safe_left),
                        Some(if has_desc { 30.0 } else { self.half_y }),
                    ),
                    (
                        0.15,
                        max_size_text,
                        "monospace".to_owned(),
                        self.fg_color,
                        title_content,
                    ),
                ));
                self.time = Instant::now();
                self.start_time = Instant::now();
                self.show = true;
                // self.changed_content = true;
            }
            AppMessage::Close => {
                self.is_exiting = true;
            }
        }
    }
    fn draw(&mut self, ctx: &mut DrawTarget) {
        if !self.show {
            return;
        }

        let delta = self.time.elapsed().as_secs_f32();
        self.time = Instant::now();

        if !self.is_exiting && self.start_time.elapsed().as_secs_f32() >= self.show_duration {
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

        if self.is_exiting && self.animation_progress >= 1.0
        // || self.is_exiting && self.content_anim_progress >= 1.0
        {
            self.is_exiting = false;
            self.animation_progress = 0.0;
            self.show = false;

            self.icon = None;
            self.title = None;
            self.slider = None;
            self.description = None;
        }

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
    }
}
