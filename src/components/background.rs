use raqote::*;

use crate::utils::ToColor;

use config::{Config, OsdPosition};

use super::Component;

pub struct Background {
    radius: f32,
    width: f32,
    height: f32,
    color: Source<'static>,
    position: OsdPosition,
}

impl Background {
    pub fn change_color(&mut self, new_color: SolidSource) {
        self.color = Source::Solid(new_color);
    }
}

impl Component<'_> for Background {
    type Args = ();
    type DrawArgs = ();

    fn new(config: &Config, _: (Option<f32>, Option<f32>), _: Self::Args) -> Self {
        let window = config.window.clone().unwrap_or_default();
        let position = window.position;
        let color = Source::Solid(
            config
                .globals
                .background
                .as_deref()
                .unwrap_or("#000")
                .to_color(),
        );
        let radius = window.radius.unwrap_or(100) as f32;
        let (width, height) = (
            (window.width.unwrap_or(600) as f32 - radius * 4.0),
            window.height.unwrap_or(80) as f32,
        );

        Self {
            color,
            width,
            height,
            radius,
            position,
        }
    }

    fn draw(&mut self, ctx: &mut raqote::DrawTarget, progress: f32, _: Self::DrawArgs) {
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
