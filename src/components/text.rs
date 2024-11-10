use font_kit::font::Font;
use raqote::{PathBuilder, Point, SolidSource, Source};
use std::time::Instant;

use super::Component;
use crate::utils::load_font;

pub struct Text {
    x: f32,
    y: f32,
    size: f32,
    font: Font,
    max_width: f32,
    c: SolidSource,
    content: String,
    is_overflow: bool,
    text_width: f32,
    scroll_x: f32,
    scrolling_left: bool,
    last_update: Instant,
}

fn calcule_content(font: &Font, max_width: f32, point_size: f32, content: &str) -> (bool, f32) {
    let calcule_glyph = |id: u32| font.advance(id).unwrap().x() * point_size / 24.0 / 96.0;
    let mut is_overflow = false;
    let mut size = 0.0;

    for c in content.chars() {
        let id = font.glyph_for_char(c).unwrap();
        size += calcule_glyph(id);
    }

    if size >= (max_width + 2.0) {
        is_overflow = true;
    }

    (is_overflow, size)
}

impl Text {
    pub fn change_value(&mut self, content: String) {
        let (is_overflow, text_width) =
            calcule_content(&self.font, self.max_width, self.size, &content);
        self.scrolling_left = true;
        self.is_overflow = is_overflow;
        self.text_width = text_width;
        self.content = content;
    }
}

impl Component for Text {
    type Args = (f32, f32, String, SolidSource, String);

    fn new(
        config: &crate::config::Config,
        (x, y): (Option<f32>, Option<f32>),
        (size_mul, max_size, font, color, content): Self::Args,
    ) -> Self {
        let radius = config.radius as f32;
        let size = config.height as f32 * size_mul;
        let font = load_font(&font);
        let max_width = config.width as f32 - (radius * max_size);
        let (is_overflow, text_width) = calcule_content(&font, max_width, size, &content);

        Self {
            font,
            size,
            content,
            c: color,
            max_width,
            is_overflow,
            text_width,
            scroll_x: 0.0,
            x: x.unwrap_or((radius * 2.0) - 10.0),
            y: y.map(|y| y - (size / 2.0)).unwrap_or(0.0),
            scrolling_left: true,
            last_update: Instant::now(),
        }
    }

    fn draw(&mut self, ctx: &mut raqote::DrawTarget, progress: f32) {
        let mut pb = PathBuilder::new();

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
        }

        let alpha = (self.c.a as f32 * (progress.powf(2.3))).min(255.0) as u8;
        pb.rect(
            self.x,
            (self.y - self.size) * progress,
            self.max_width,
            self.size + 10.0,
        );
        let clip_path = pb.finish();

        ctx.push_clip(&clip_path);

        ctx.draw_text(
            &self.font,
            self.size,
            &self.content,
            Point::new(self.x - self.scroll_x, self.y * progress),
            &Source::Solid(SolidSource::from_unpremultiplied_argb(
                alpha, self.c.r, self.c.g, self.c.b,
            )),
            &Default::default(),
        );

        ctx.pop_clip();
    }
}
