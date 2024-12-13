use cosmic_text::{Buffer, Color, FontSystem, SwashCache};
use raqote::{DrawOptions, DrawTarget, PathBuilder, SolidSource, Source};

use std::time::Instant;

use crate::config::OsdPosition;

use super::Component;

pub struct Text {
    x: f32,
    y: f32,
    max_width: f32,
    color: SolidSource,
    position: OsdPosition,
    scroll_x: f32,
    scrolling_left: bool,
    is_overflow: bool,
    text_width: f32,
    last_update: Instant,
}

impl<'a> Component<'a> for Text {
    type Args = (f32, f32, f32, SolidSource);
    type DrawArgs = (&'a mut FontSystem, &'a mut SwashCache, &'a Buffer);

    fn new(
        config: &crate::config::Config,
        (x, y): (Option<f32>, Option<f32>),
        (font_size, text_width, max_size, color): Self::Args,
    ) -> Self {
        let window = config.window.clone().unwrap_or_default();
        let position = window.position;
        let radius = window.radius.unwrap_or(100) as f32;
        let max_width = window.width.unwrap_or(600) as f32 - (radius * max_size);

        Text {
            max_width,
            color,
            position,
            scroll_x: 0.0,
            scrolling_left: true,
            is_overflow: text_width >= (max_width + 2.0),
            text_width,
            x: x.unwrap_or((radius * 2.0) - 10.0),
            y: y.map(|y| y - (font_size / 2.0)).unwrap_or(0.0),
            last_update: Instant::now(),
        }
    }

    fn draw(
        &mut self,
        ctx: &mut DrawTarget,
        progress: f32,
        (fonts, cache, buffer): Self::DrawArgs,
    ) {
        let font_size = buffer.metrics().font_size;

        let mut pb = PathBuilder::new();
        let y = if self.position == OsdPosition::Bottom {
            self.y + (self.y * (1.0 - progress))
        } else {
            self.y * progress
        };

        if self.is_overflow {
            let scroll_speed = 30.0;
            let max_x_offset = self.text_width - self.max_width;
            let now = Instant::now();
            let elapsed = now.duration_since(self.last_update).as_secs_f32();

            if self.scrolling_left {
                self.scroll_x += elapsed * scroll_speed;
                if self.scroll_x >= max_x_offset {
                    self.scroll_x = max_x_offset;
                    self.scrolling_left = false;
                }
            } else {
                self.scroll_x -= elapsed * scroll_speed;
                if self.scroll_x <= 0.0 {
                    self.scroll_x = 0.0;
                    self.scrolling_left = true;
                }
            }

            self.last_update = now;
        } else {
            self.scroll_x = 0.0;
        }

        let x_offset = self.x - self.scroll_x;
        let alpha = (self.color.a as f32 * (progress.powf(2.3))).min(255.0);

        // Define the clipping path
        pb.rect(self.x, y - font_size, self.max_width, font_size * 3.0);
        let clip_path = pb.finish();

        ctx.push_clip(&clip_path);

        // Draw the buffer content
        buffer.draw(
            fonts,
            cache,
            Color::rgba(self.color.r, self.color.g, self.color.b, alpha as u8),
            |px, py, w, h, color| {
                let source = Source::Solid(SolidSource::from_unpremultiplied_argb(
                    ((color.a() as f32 / 255.0) * (alpha / 255.0) * 255.0) as u8,
                    color.r(),
                    color.g(),
                    color.b(),
                ));
                ctx.fill_rect(
                    x_offset + px as f32,
                    y + py as f32,
                    w as f32,
                    h as f32,
                    &source,
                    &DrawOptions::default(),
                );
            },
        );

        ctx.pop_clip();
    }
}
