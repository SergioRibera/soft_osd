use cosmic_text::{Buffer, Color, FontSystem, SwashCache};
use raqote::{DrawOptions, DrawTarget, Point, SolidSource, Source};

use crate::config::OsdPosition;

use super::Component;

pub struct Icon {
    x: f32,
    y: f32,
    content: String,
    c: SolidSource,
    position: OsdPosition,
}

impl Icon {
    pub fn change_content(&mut self, new_content: char) {}
}

impl<'a> Component<'a> for Icon {
    type Args = (SolidSource, char);
    type DrawArgs = (&'a mut FontSystem, &'a mut SwashCache, &'a Buffer);

    fn new(
        config: &crate::config::Config,
        (x, y): (Option<f32>, Option<f32>),
        (color, content): Self::Args,
    ) -> Self {
        let position = config.position;

        Self {
            position,
            c: color,
            x: x.unwrap_or_default(),
            y: y.unwrap_or_default(),
            content: content.to_string(),
        }
    }

    fn draw(
        &mut self,
        ctx: &mut DrawTarget,
        progress: f32,
        (fonts, cache, buffer): Self::DrawArgs,
    ) {
        let alpha = (self.c.a as f32 * (progress.powf(2.3))).min(255.0);
        let y = if self.position == OsdPosition::Bottom {
            self.y + (self.y * (1.0 - progress))
        } else {
            self.y * progress
        };

        buffer.draw(
            fonts,
            cache,
            Color::rgba(self.c.r, self.c.g, self.c.b, alpha as u8),
            |px, py, w, h, color| {
                let source = Source::Solid(SolidSource::from_unpremultiplied_argb(
                    ((color.a() as f32 / 255.0) * (alpha / 255.0) * 255.0) as u8,
                    color.r(),
                    color.g(),
                    color.b(),
                ));
                ctx.fill_rect(
                    self.x + px as f32,
                    y + py as f32,
                    1.0,
                    1.0,
                    &source,
                    &DrawOptions::default(),
                )
            },
        );
    }
}
