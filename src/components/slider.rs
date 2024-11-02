use raqote::{DrawOptions, Path, PathBuilder, SolidSource, Source};

use crate::utils::{lighten_color, ToColor};

use super::Component;

pub struct Slider {
    y: f32,
    size: f32,
    value: f32,
    radius: f32,
    rounded: f32,
    c: SolidSource,
    bg: SolidSource,
}

impl Slider {
    pub fn change_value(&mut self, value: f32) {
        self.value = value;
    }

    pub fn draw_slide(&self, x: f32, y: f32, slider_width: f32) -> Path {
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

impl Component for Slider {
    type Args = f32;

    fn new(config: &crate::config::Config, value: Self::Args) -> Self {
        let rounded = config.height as f32 * 0.05; // size of rounded border
        let radius = config.radius as f32; // padding of widget
        let size = config.width as f32 - (radius * 4.4); // size of slidebar
        let y = config.height as f32 / 2.0 - (rounded * 2.0); // position of slidebar
        let c = config.foreground_color.to_color();
        let bg = config.background.to_color();
        let (r, g, b) = lighten_color(bg.r, bg.g, bg.b, 0.3);
        let bg = SolidSource::from_unpremultiplied_argb(bg.a, r, g, b);

        Self {
            y,
            c,
            bg,
            size,
            value,
            radius,
            rounded,
        }
    }

    fn draw(&self, ctx: &mut raqote::DrawTarget, progress: f32) {
        let x = self.radius * 2.4;
        let y = self.y * progress;
        let slider_width = self.size * self.value * progress;

        // Fondo del slider
        let bg = self.draw_slide(x, y, self.size);
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
        let fg = self.draw_slide(x, y, slider_width);
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
