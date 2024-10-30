use std::time::Instant;

use raqote::*;

use crate::config::Config;

pub trait App: From<Config> {
    fn run(&mut self, ctx: &mut DrawTarget, size: (u32, u32));
}

#[derive(Debug, Clone)]
pub struct MainApp {
    time: Instant,
    config: Config,
    animation_progress: f32,
    animation_duration: f32,
}

impl From<Config> for MainApp {
    fn from(config: Config) -> Self {
        let animation_duration = config.animation_duration;
        Self {
            config,
            animation_duration,
            time: Instant::now(),
            animation_progress: 0.0,
        }
    }
}

fn ease_out_cubic(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(3)
}

impl App for MainApp {
    fn run(&mut self, ctx: &mut DrawTarget, (ow, oh): (u32, u32)) {
        let delta = self.time.elapsed().as_secs_f32();
        self.time = Instant::now();
        self.animation_progress =
            (self.animation_progress + delta / self.animation_duration).min(1.0);

        let &Config { radius, .. } = &self.config;
        let (width, height) = ((ow - radius * 2) as f32, oh as f32);
        let ow = ow as f32;
        let radius = radius as f32;
        let progress = ease_out_cubic(self.animation_progress);

        let mut pb = PathBuilder::new();

        // Animar la posición vertical usando el progreso
        let animated_height = height * progress;

        pb.move_to(0.0, 0.0);
        // Primera parte
        pb.cubic_to(radius, 0.0, 0.0, animated_height, radius, animated_height);

        pb.line_to(width + radius, animated_height);

        // Última parte
        pb.move_to(width + radius, animated_height);
        pb.cubic_to(ow, animated_height, width + radius, 0.0, ow, 0.0);

        pb.line_to(0.0, 0.0);
        pb.close();

        let path = pb.finish();
        ctx.fill(
            &path,
            &Source::Solid(SolidSource::from_unpremultiplied_argb(255, 0, 0, 0)),
            &DrawOptions::default(),
        );
    }
}
