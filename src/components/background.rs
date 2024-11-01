use raqote::*;

use crate::config::{Config, OsdPosition};
use crate::utils::ToColor;

use super::Component;

pub struct Background {
    radius: f32,
    width: f32,
    height: f32,
    color: Source<'static>,
    position: OsdPosition,
}

impl Component for Background {
    fn new(config: &Config) -> Self {
        let position = config.position;
        let color = Source::Solid(config.background.to_color());
        let radius = config.radius as f32;
        let (width, height) = ((config.width as f32 - radius * 4.0), config.height as f32);

        Self {
            color,
            width,
            height,
            radius,
            position,
        }
    }

    fn draw(&self, ctx: &mut raqote::DrawTarget, progress: f32) {
        let or = self.radius; // Origin radius
        let rp = or * progress; // Radius progress
        let (start_height, animated_height) = if self.position == OsdPosition::Bottom {
            (self.height, self.height * (1.0 - progress))
        } else {
            (0.0, self.height * progress)
        };

        let mut pb = PathBuilder::new();
        pb.move_to(rp, start_height);

        // First segment
        pb.cubic_to(
            rp + or,
            start_height,
            rp,
            animated_height,
            or * 2.0,
            animated_height,
        );

        pb.line_to(self.width + or * 2.0, animated_height);

        // Last segment
        pb.move_to(self.width + or * 2.0, animated_height);
        pb.cubic_to(
            self.width + or * 3.0 + or * (1.0 - progress),
            animated_height,
            self.width + or * 2.0 + or * (1.0 - progress),
            start_height,
            self.width + or * 3.0 + or * (1.0 - progress),
            start_height,
        );

        // Close path
        pb.line_to(rp, start_height);
        pb.close();

        let path = pb.finish();
        ctx.fill(&path, &self.color, &DrawOptions::default());
    }
}
