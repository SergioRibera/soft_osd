use raqote::{DrawOptions, Path, PathBuilder, SolidSource, Source};

use config::OsdPosition;

use crate::utils::{adjust_brightness, contrast_ratio, ToColor};

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

fn generate_colors(fg: (u8, u8, u8), bg_x: (u8, u8, u8)) -> ((u8, u8, u8), (u8, u8, u8)) {
    let slider_bg = adjust_brightness(fg, 0.7);
    let slider_fg = adjust_brightness(fg, 1.3);

    let contrast_bg = contrast_ratio(slider_bg, bg_x);
    let slider_bg = if contrast_bg < 1.5 {
        adjust_brightness(slider_bg, 0.6)
    } else {
        slider_bg
    };

    let contrast_fg = contrast_ratio(slider_fg, slider_bg);
    let slider_fg = if contrast_fg < 2.0 {
        adjust_brightness(slider_fg, 1.4)
    } else {
        slider_fg
    };

    (slider_bg, slider_fg)
}

impl Slider {
    pub fn change_value(&mut self, value: f32) {
        self.value = value.clamp(0.036, 1.0);
    }

    pub fn change_size(&mut self, value: f32) {
        self.size = value;
    }

    pub fn change_color(&mut self, bg: SolidSource, new_color: SolidSource) {
        let (b, c) = generate_colors((new_color.r, new_color.g, new_color.b), (bg.r, bg.g, bg.b));
        self.c = SolidSource::from_unpremultiplied_argb(new_color.a, c.0, c.1, c.2);
        self.bg = SolidSource::from_unpremultiplied_argb(bg.a, b.0, b.1, b.2);
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
        config: &config::Config,
        (x, y): (Option<f32>, Option<f32>),
        (value, size_mul): Self::Args,
    ) -> Self {
        let window = config.window.clone().unwrap_or_default();
        let position = window.position;
        let rounded = window.height.unwrap_or(80) as f32 * 0.05; // size of rounded border
        let radius = window.radius.unwrap_or(100) as f32; // padding of widget
        let size = window.width.unwrap_or(600) as f32 - (radius * size_mul); // size of slidebar
        let c = config
            .globals
            .foreground_color
            .as_deref()
            .unwrap_or("#fff")
            .to_color();
        let bg = config
            .globals
            .background
            .as_deref()
            .unwrap_or("#000")
            .to_color();
        let (nc, nb) = generate_colors((c.r, c.g, c.b), (bg.r, bg.g, bg.b));
        let c = SolidSource::from_unpremultiplied_argb(c.a, nc.0, nc.1, nc.2);
        let bg = SolidSource::from_unpremultiplied_argb(bg.a, nb.0, nb.1, nb.2);
        let value = (value / 100.0).clamp(0.036, 1.0);

        Self {
            c,
            bg,
            size,
            value,
            rounded,
            position,
            x: x.unwrap_or(radius * 2.4),
            y: y.map(|y| y - (rounded * 2.0))
                .unwrap_or_else(|| window.height.unwrap_or(80) as f32 / 2.0 - (rounded * 2.0)),
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
