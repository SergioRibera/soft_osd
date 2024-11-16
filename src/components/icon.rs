use std::sync::Arc;

use font_kit::font::Font;
use raqote::{Point, SolidSource, Source};

use crate::config::OsdPosition;
use crate::utils::load_font_by_glyph;

use super::Component;

pub struct Icon {
    x: f32,
    y: f32,
    size: f32,
    font: Arc<Font>,
    content: String,
    c: SolidSource,
    position: OsdPosition,
}

unsafe impl Send for Icon {}
unsafe impl Sync for Icon {}

impl Icon {
    pub fn change_content(&mut self, new_content: char) {
        self.content = new_content.to_string();
    }
}

impl Component for Icon {
    type Args = (SolidSource, char);

    fn new(
        config: &crate::config::Config,
        (x, y): (Option<f32>, Option<f32>),
        (color, content): Self::Args,
    ) -> Self {
        let position = config.position;
        let size = config.height as f32 * 0.2;

        Self {
            size,
            position,
            c: color,
            x: x.unwrap_or_default(),
            y: y.unwrap_or_default(),
            content: content.to_string(),
            font: Arc::new(load_font_by_glyph(content)),
        }
    }

    fn draw(&mut self, ctx: &mut raqote::DrawTarget, progress: f32) {
        let alpha = (self.c.a as f32 * (progress.powf(2.3))).min(255.0) as u8;
        let y = if self.position == OsdPosition::Bottom {
            self.y + (self.y * (1.0 - progress))
        } else {
            self.y * progress
        };

        ctx.draw_text(
            self.font.as_ref(),
            self.size,
            &self.content,
            Point::new(self.x, y),
            &Source::Solid(raqote::SolidSource::from_unpremultiplied_argb(
                alpha, self.c.r, self.c.g, self.c.b,
            )),
            &Default::default(),
        )
    }
}
