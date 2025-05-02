use std::path::PathBuf;

use image::imageops::{resize, FilterType};
use image::{Rgba, RgbaImage};
use zbus::zvariant::{Structure, Value};

use crate::error::IconError;

#[derive(Debug)]
pub enum Icon {
    Char(char),
    Image(RgbaImage),
}

impl TryFrom<(String, f32)> for Icon {
    type Error = IconError;

    fn try_from((s, size): (String, f32)) -> Result<Self, Self::Error> {
        let path = PathBuf::from(&s);
        if !path.exists() {
            if let Some(icon) = s.chars().next() {
                return Ok(Self::Char(icon));
            } else {
                return Err(IconError::CharOrFileNotFound);
            }
        }

        let size = size as u32;

        match path.extension().and_then(|e| e.to_str()) {
            Some("png") | Some("jpeg") | Some("jpg") => {
                if let Ok(img) = image::open(path) {
                    let img = resize(&img, size, size, image::imageops::FilterType::Gaussian);
                    return Ok(Self::Image(img));
                }
                Err(IconError::CannotLoadFormats(&["png", "jpeg", "jpg"]))
            }
            // Some("svg") => {
            //     let img = raqote_svg::render_to_image(std::fs::read_to_string(path)?, (size, size));
            //     Ok(Self::Image(img))
            // }
            e => Err(IconError::CannotLoadFormat(
                e.unwrap_or_default().to_owned(),
            )),
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
    pub fn from_value(value: &Value, size: f32) -> Option<Self> {
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

        let size = size as u32;
        let img = match channels {
            3 => RgbaImage::from_fn(width as u32, height as u32, |x, y| {
                let index = (y as usize * width as usize) + x as usize;
                Rgba([data[index], data[index + 1], data[index + 2], 255])
            }),
            4 => RgbaImage::from_raw(width as u32, height as u32, data)?,
            _ => return None,
        };
        let img = resize(&img, size, size, FilterType::Gaussian);

        Some(Self::Image(img))
    }
}
