use cosmic_text::{Attrs, Buffer, Color, FontSystem, Shaping, SwashCache};
use image::imageops::resize;
use image::{Rgba, RgbaImage};
use raqote::{DrawOptions, DrawTarget, SolidSource, Source};

use std::path::PathBuf;
use zbus::zvariant::{Structure, Value};

use crate::app::ICON_SIZE;
use crate::config::OsdPosition;

use super::Component;

#[derive(Debug)]
pub enum Icon {
    Char(char),
    Image(RgbaImage),
}

impl TryFrom<String> for Icon {
    type Error = &'static str;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let path = PathBuf::from(&s);
        if !path.exists() {
            if let Some(icon) = s.chars().next() {
                return Ok(Self::Char(icon));
            }
        }

        let size = *ICON_SIZE.read().unwrap() as u32;

        match path.extension().map(|e| e.to_str()).flatten() {
            Some("png") | Some("jpeg") | Some("jpg") => {
                if let Ok(img) = image::open(path) {
                    let img = resize(&img, size, size, image::imageops::FilterType::Gaussian);
                    return Ok(Self::Image(img));
                }
                Err("No se pudo cargar el icono")
            }
            Some("svg") => {
                let img = raqote_svg::render_to_image(
                    std::fs::read_to_string(path).unwrap(),
                    (size, size),
                );
                return Ok(Self::Image(img));
            }
            _ => Err("No se pudo cargar el icono"),
        }
    }
}

macro_rules! get_field {
    ($fields:ident, $name:expr) => {
        $fields
            .next()
            .ok_or(concat!(stringify!($name), "not found").to_owned())
            .ok()?
            .downcast_ref()
            .ok()
    };
    ($t:ty, $fields:ident, $name:expr) => {
        $fields
            .next()
            .ok_or(concat!(stringify!($name), "not found").to_owned())
            .ok()?
            .downcast_ref::<$t>()
            .ok()
    };
}

impl Icon {
    pub fn from_value(value: &Value) -> Option<Self> {
        let downcast_ref = value.downcast_ref::<Structure>();
        let structure = downcast_ref.ok()?;
        let mut fields = structure.fields().iter();

        let width: i32 = get_field!(fields, "width")?;
        let height: i32 = get_field!(fields, "height")?;
        let rowstride: i32 = get_field!(fields, "rowstride")?;
        let _one_point_two_bit_alpha: bool = get_field!(fields, "one_point_two_bit_alpha")?;
        let bits_per_sample: i32 = get_field!(fields, "bits_per_sample")?;
        let channels: i32 = get_field!(fields, "channels")?;
        let data: Vec<u8> = get_field!(zbus::zvariant::Array, fields, "bytes")?
            .iter()
            .map(|b| b.downcast_ref::<u8>().ok())
            .collect::<Option<_>>()?;

        // Sometimes dbus (or the application) can give us junk image data, usually when lots of
        // stuff is sent at the same time the same time, so we should sanity check the image.
        // https://github.com/dunst-project/dunst/blob/3f3082efb3724dcd369de78dc94d41190d089acf/src/icon.c#L316
        let pixelstride = (channels * bits_per_sample + 7) / 8;
        let len_expected = (height - 1) * rowstride + width * pixelstride;
        let len_actual = data.len() as i32;
        if len_actual != len_expected {
            // "Expected image data to be of length: {len_expected}, but got a length of {len_actual}."
            return None;
        }

        let size = *ICON_SIZE.read().unwrap() as u32;
        let img = match channels {
            3 => RgbaImage::from_fn(width as u32, height as u32, |x, y| {
                let index = (y as usize * width as usize) + x as usize;
                Rgba([data[index], data[index + 1], data[index + 2], 255])
            }),
            4 => RgbaImage::from_raw(width as u32, height as u32, data)?,
            _ => return None,
        };
        let img = image::imageops::resize(&img, size, size, image::imageops::FilterType::Gaussian);

        Some(Self::Image(img))
    }
}

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
        config: &crate::config::Config,
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
