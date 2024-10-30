use std::time::Instant;

use raqote::*;

use crate::config::Config;

pub trait App: From<Config> {
    fn run(&mut self, ctx: &mut DrawTarget, size: (u32, u32));
}

#[derive(Debug, Clone)]
pub struct MainApp {
    speed: f32,
    time: Instant,
    config: Config,
}

impl From<Config> for MainApp {
    fn from(config: Config) -> Self {
        let speed = 5.0;
        Self {
            speed,
            config,
            time: Instant::now(),
        }
    }
}

impl App for MainApp {
    fn run(&mut self, ctx: &mut DrawTarget, (ow, _): (u32, u32)) {
        let delta = self.time.elapsed().as_secs_f32() * self.speed;
        let &Config { radius, .. } = &self.config;
        let (width, height) = ((ow - radius * 2) as f32, oh as f32);
        let ow = ow as f32;
        let radius = radius as f32;

        let mut pb = PathBuilder::new();

        pb.move_to(0.0, 0.0);
        // first part
        pb.cubic_to(radius, 0.0, 0.0, height, radius, height);

        // pb.move_to(radius, height);
        pb.line_to(width + radius, height);

        // // last part
        pb.move_to(width + radius, height);
        pb.cubic_to(ow, height, width + radius, 0.0, ow, 0.0);

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
