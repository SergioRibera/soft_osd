use font_kit::font::Font;
use raqote::{Point, SolidSource, Source};

use crate::utils::load_font_by_glyph;

use super::Component;

pub struct Icon {
    y: f32,
    size: f32,
    radius: f32,
    font: Font,
    content: String,
    c: SolidSource,
}

impl Icon {
    pub fn change_content(&mut self, new_content: char) {
        self.content = new_content.to_string();
    }
}

impl Component for Icon {
    type Args = (SolidSource, char);

    fn new(config: &crate::config::Config, (color, content): Self::Args) -> Self {
        let radius = config.radius as f32;
        let size = config.height as f32 * 0.2;
        let y = config.height as f32 / 2.0;

        Self {
            y,
            size,
            radius,
            c: color,
            font: load_font_by_glyph(content),
            content: content.to_string(),
        }
    }

    fn draw(&self, ctx: &mut raqote::DrawTarget, progress: f32) {
        let alpha = (self.c.a as f32 * (progress.powf(2.3))).min(255.0) as u8;
        ctx.draw_text(
            &self.font,
            self.size,
            &self.content,
            Point::new(self.radius + (self.radius - 10.0), self.y * progress),
            &Source::Solid(raqote::SolidSource::from_unpremultiplied_argb(
                alpha, self.c.r, self.c.g, self.c.b,
            )),
            &Default::default(),
        )
    }
}
