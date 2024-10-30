use std::time::Instant;

use raqote::*;

use crate::config::Config;

pub trait App: From<Config> {
    fn run(&mut self, exit: &mut bool, ctx: &mut DrawTarget, size: (u32, u32));
}

#[derive(Debug, Clone)]
pub struct MainApp {
    time: Instant,
    config: Config,
    is_exiting: bool,
    show_duration: f32,
    start_time: Instant,
    animation_progress: f32,
    animation_duration: f32,
}

impl From<Config> for MainApp {
    fn from(config: Config) -> Self {
        let animation_duration = config.animation_duration;
        let show_duration = config.show_duration + animation_duration;
        Self {
            config,
            show_duration,
            is_exiting: false,
            animation_duration,
            time: Instant::now(),
            animation_progress: 0.0,
            start_time: Instant::now(),
        }
    }
}

fn ease_out_cubic(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(3)
}

impl App for MainApp {
    fn run(&mut self, exit: &mut bool, ctx: &mut DrawTarget, (ow, oh): (u32, u32)) {
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

        let &Config { radius, .. } = &self.config;
        let (width, height) = ((ow - radius * 4) as f32, oh as f32);

        let progress = if self.is_exiting {
            1.0 - ease_out_cubic(self.animation_progress)
        } else {
            ease_out_cubic(self.animation_progress)
        };

        let mut pb = PathBuilder::new();
        let or = radius as f32; // Origin radius
        let rp = or * progress; // Radius progress
        let animated_height = height * progress;

        pb.move_to(rp, 0.0);
        // First part
        pb.cubic_to(rp + or, 0.0, rp, animated_height, or * 2.0, animated_height);

        pb.line_to(width + or * 2.0, animated_height);

        // Last part
        pb.move_to(width + or * 2.0, animated_height);
        pb.cubic_to(
            width + or * 3.0 + or * (1.0 - progress),
            animated_height,
            width + or * 2.0 + or * (1.0 - progress),
            0.0,
            width + or * 3.0 + or * (1.0 - progress),
            0.0,
        );

        // Close
        pb.line_to(rp, 0.0);
        pb.close();

        let path = pb.finish();
        ctx.fill(
            &path,
            &Source::Solid(SolidSource::from_unpremultiplied_argb(255, 255, 255, 255)),
            &DrawOptions {
                alpha: 1.0,
                blend_mode: BlendMode::Add,
                antialias: AntialiasMode::Gray,
            },
        );

        if self.is_exiting && self.animation_progress >= 1.0 {
            println!("Exit: {:?}", self.start_time.elapsed().as_secs_f32());
            *exit = true;
        }
    }
}
