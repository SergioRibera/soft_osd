use raqote::SolidSource;

pub trait ToColor {
    type Target;
    fn to_color(&self) -> Self::Target;
}

/// Parse hex color (#RRGGBB or #RRGGBBAA)
impl ToColor for String {
    type Target = SolidSource;

    fn to_color(&self) -> Self::Target {
        assert_eq!(self.is_empty(), false, "Cannot parse empty string");
        assert_eq!(
            self.as_bytes()[0],
            b'#',
            "The color string must be start with #"
        );
        let mut color = u32::from_str_radix(&self[1..], 16).expect("Cannot parse hex value");

        match self.len() {
            // RGB or RGBA
            4 | 5 => {
                let a = if self.len() == 5 {
                    let alpha = (color & 0xf) as u8;
                    color >>= 4;
                    alpha
                } else {
                    0xff
                };

                let r = ((color >> 8) & 0xf) as u8;
                let g = ((color >> 4) & 0xf) as u8;
                let b = (color & 0xf) as u8;

                SolidSource::from_unpremultiplied_argb(a, r, g, b)
            }
            // RRGGBB or RRGGBBAA
            7 | 9 => {
                let alpha = if self.len() == 9 {
                    let alpha = (color & 0xff) as u8;
                    color >>= 8;
                    alpha
                } else {
                    0xff
                };

                let r = (color >> 16) as u8;
                let g = (color >> 8) as u8;
                let b = color as u8;

                SolidSource::from_unpremultiplied_argb(alpha, r, g, b)
            }
            _ => panic!("Invalid Legth of string of Parse to Color"),
        }
    }
}

impl ToColor for str {
    type Target = SolidSource;

    fn to_color(&self) -> Self::Target {
        String::from(self).to_color()
    }
}
