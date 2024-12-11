use raqote::{DrawOptions, Path, PathBuilder, SolidSource, Source};

use crate::config::OsdPosition;
use crate::utils::{lighten_color, ToColor};

use super::Component;

pub struct Slider {
    x: f32,
    y: f32,
    size: f32,
    value: f32,
    rounded: f32,
    c: SolidSource,
    bg: SolidSource,
    position: OsdPosition,
}

impl Slider {
    pub fn change_value(&mut self, value: f32) {
        self.value = value.min(1.0).max(0.036);
    }

    pub fn draw_slide(&self, y: f32, slider_width: f32) -> Path {
        let x = self.x;
        let mut pb_fg = PathBuilder::new();

        pb_fg.move_to(x + self.rounded, y);
        pb_fg.line_to(x + slider_width - self.rounded, y);
        pb_fg.cubic_to(
            x + slider_width - self.rounded / 2.0,
            y,
            x + slider_width,
            y + self.rounded / 2.0,
            x + slider_width,
            y + self.rounded,
        );

        pb_fg.line_to(x + slider_width, y + self.rounded * 2.0 - self.rounded);
        pb_fg.cubic_to(
            x + slider_width,
            y + self.rounded * 2.0 - self.rounded / 2.0,
            x + slider_width - self.rounded / 2.0,
            y + self.rounded * 2.0,
            x + slider_width - self.rounded,
            y + self.rounded * 2.0,
        );

        pb_fg.line_to(x + self.rounded, y + self.rounded * 2.0);
        pb_fg.cubic_to(
            x + self.rounded / 2.0,
            y + self.rounded * 2.0,
            x,
            y + self.rounded * 2.0 - self.rounded / 2.0,
            x,
            y + self.rounded * 2.0 - self.rounded,
        );

        pb_fg.line_to(x, y + self.rounded);
        pb_fg.cubic_to(
            x,
            y + self.rounded / 2.0,
            x + self.rounded / 2.0,
            y,
            x + self.rounded,
            y,
        );

        pb_fg.close();
        pb_fg.finish()
    }
}

impl Component<'_> for Slider {
    type Args = (f32, f32);
    type DrawArgs = ();

    fn new(
        config: &crate::config::Config,
        (x, y): (Option<f32>, Option<f32>),
        (value, size_mul): Self::Args,
    ) -> Self {
        let position = config.position;
        let rounded = config.height as f32 * 0.05; // size of rounded border
        let radius = config.radius as f32; // padding of widget
        let size = config.width as f32 - (radius * size_mul); // size of slidebar
        let c = config.foreground_color.to_color();
        let bg = config.background.to_color();
        let (r, g, b) = lighten_color(bg.r, bg.g, bg.b, 0.3);
        let bg = SolidSource::from_unpremultiplied_argb(bg.a, r, g, b);
        let value = (value / 100.0).min(1.0).max(0.036);

        Self {
            c,
            bg,
            size,
            value,
            rounded,
            position,
            x: x.unwrap_or_else(|| radius * 2.4),
            y: y.map(|y| y - (rounded * 2.0))
                .unwrap_or_else(|| config.height as f32 / 2.0 - (rounded * 2.0)),
        }
    }

    fn draw(&mut self, ctx: &mut raqote::DrawTarget, progress: f32, _: Self::DrawArgs) {
        let y = if self.position == OsdPosition::Bottom {
            self.y + (self.y * (1.0 - progress))
        } else {
            self.y * progress
        };
        let slider_width = self.size * self.value;

        // Fondo del slider
        let bg = self.draw_slide(y, self.size);
        ctx.fill(
            &bg,
            &Source::Solid(raqote::SolidSource::from_unpremultiplied_argb(
                (self.bg.a as f32 * (progress.powf(2.3))).min(255.0) as u8,
                self.bg.r,
                self.bg.g,
                self.bg.b,
            )),
            &DrawOptions::default(),
        );

        // Barra de progreso del slider
        let fg = self.draw_slide(y, slider_width);
        ctx.fill(
            &fg,
            &Source::Solid(raqote::SolidSource::from_unpremultiplied_argb(
                (self.c.a as f32 * (progress.powf(2.3))).min(255.0) as u8,
                self.c.r,
                self.c.g,
                self.c.b,
            )),
            &DrawOptions::default(),
        );
    }
}
