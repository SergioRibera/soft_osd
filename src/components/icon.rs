use config::OsdPosition;
use cosmic_text::{Attrs, Buffer, Color, FontSystem, Shaping, SwashCache};
use raqote::{DrawOptions, DrawTarget, SolidSource, Source};
use services::Icon;

use super::Component;

pub struct IconComponent {
    x: f32,
    y: f32,
    c: SolidSource,
    position: OsdPosition,
    icon: Icon,
}

impl<'a> Component<'a> for IconComponent {
    type Args = (SolidSource, Icon);
    type DrawArgs = (&'a mut FontSystem, &'a mut SwashCache, &'a mut Buffer);

    fn new(
        config: &config::Config,
        (x, y): (Option<f32>, Option<f32>),
        (c, icon): Self::Args,
    ) -> Self {
        let position = config.window.clone().unwrap_or_default().position;

        Self {
            c,
            icon,
            position,
            x: x.unwrap_or_default(),
            y: y.unwrap_or_default(),
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

        match &self.icon {
            Icon::Char(i) => {
                buffer.set_text(fonts, &i.to_string(), Attrs::new(), Shaping::Advanced);
                buffer.draw(
                    fonts,
                    cache,
                    Color::rgba(self.c.r, self.c.g, self.c.b, alpha as u8),
                    |px, py, _w, _h, color| {
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
            Icon::Image(img) => {
                for (px, py, pixel) in img.enumerate_pixels() {
                    let bytes = pixel.0;
                    let alpha = ((bytes[3] as f32 / 255.0) * (alpha / 255.0) * 255.0) as u8;
                    let source = Source::Solid(SolidSource::from_unpremultiplied_argb(
                        alpha, bytes[0], bytes[1], bytes[2],
                    ));
                    ctx.fill_rect(
                        self.x + px as f32,
                        y + py as f32,
                        1.0,
                        1.0,
                        &source,
                        &DrawOptions::default(),
                    )
                }
            }
        }
    }
}
