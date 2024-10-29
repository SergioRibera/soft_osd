use raqote::*;

use crate::config::Config;

pub trait App: From<Config> {
    fn run(&self, ctx: &mut DrawTarget, size: (u32, u32));
}

#[derive(Debug, Clone)]
pub struct MainApp;

impl From<Config> for MainApp {
    fn from(_: Config) -> Self {
        Self
    }
}

impl App for MainApp {
    fn run(&self, ctx: &mut DrawTarget, (width, height): (u32, u32)) {
        let mut pb = PathBuilder::new();
        pb.rect(0., 0., width as f32, height as f32);
        let path = pb.finish();
        ctx.fill(
            &path,
            &Source::Solid(SolidSource {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            }),
            &DrawOptions::default(),
        );
    }
}
