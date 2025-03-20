use raqote::SolidSource;

pub trait ToColor {
    type Target;

    fn to_color(&self) -> Self::Target;
}

impl ToColor for str {
    type Target = SolidSource;

    fn to_color(&self) -> Self::Target {
        String::from(self).to_color()
    }
}

/// Parse hex color (#RRGGBB or #RRGGBBAA)
impl ToColor for String {
    type Target = SolidSource;

    fn to_color(&self) -> Self::Target {
        assert!(!self.is_empty(), "Cannot parse empty string");
        assert_eq!(
            self.as_bytes()[0],
            b'#',
            "The color string must start with #"
        );
        let mut color = u32::from_str_radix(&self[1..], 16).expect("Cannot parse hex value");

        match self.len() {
            // #RGB o #RGBA
            4 | 5 => {
                let a = if self.len() == 5 {
                    let alpha = (color & 0xf) as u8;
                    color >>= 4;
                    (alpha << 4) | alpha // Expansión de bits 4 a 8
                } else {
                    0xff
                };

                // Expandir cada componente de color de 4 a 8 bits
                let r = ((color >> 8) & 0xf) as u8;
                let r = (r << 4) | r;
                let g = ((color >> 4) & 0xf) as u8;
                let g = (g << 4) | g;
                let b = (color & 0xf) as u8;
                let b = (b << 4) | b;

                SolidSource { a, r, g, b }
            }
            // RRGGBB or RRGGBBAA
            7 | 9 => {
                let a = if self.len() == 9 {
                    let alpha = (color & 0xff) as u8;
                    color >>= 8;
                    alpha
                } else {
                    0xff
                };

                let r = ((color >> 16) & 0xff) as u8;
                let g = ((color >> 8) & 0xff) as u8;
                let b = (color & 0xff) as u8;

                SolidSource { a, r, g, b }
            }
            _ => panic!("Invalid length of string for Parse to Color"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_rgb_colors() {
        // Test basic #RGB colors
        let white = "#fff".to_color();
        assert_eq!((white.r, white.g, white.b, white.a), (255, 255, 255, 255));

        let black = "#000".to_color();
        assert_eq!((black.r, black.g, black.b, black.a), (0, 0, 0, 255));

        let red = "#f00".to_color();
        assert_eq!((red.r, red.g, red.b, red.a), (255, 0, 0, 255));

        let green = "#0f0".to_color();
        assert_eq!((green.r, green.g, green.b, green.a), (0, 255, 0, 255));

        let blue = "#00f".to_color();
        assert_eq!((blue.r, blue.g, blue.b, blue.a), (0, 0, 255, 255));
    }

    #[test]
    fn test_rgba_short_format() {
        // Test #RGBA format
        let semi_white = "#ffff".to_color();
        assert_eq!(
            (semi_white.r, semi_white.g, semi_white.b, semi_white.a),
            (255, 255, 255, 255)
        );

        let semi_transparent_red = "#f00a".to_color();
        assert_eq!(
            (
                semi_transparent_red.r,
                semi_transparent_red.g,
                semi_transparent_red.b,
                semi_transparent_red.a
            ),
            (255, 0, 0, 170)
        );

        let quarter_transparent_blue = "#00f4".to_color();
        assert_eq!(
            (
                quarter_transparent_blue.r,
                quarter_transparent_blue.g,
                quarter_transparent_blue.b,
                quarter_transparent_blue.a
            ),
            (0, 0, 255, 68)
        );
    }

    #[test]
    fn test_rrggbb_colors() {
        // Test #RRGGBB format
        let white = "#ffffff".to_color();
        assert_eq!((white.r, white.g, white.b, white.a), (255, 255, 255, 255));

        let black = "#000000".to_color();
        assert_eq!((black.r, black.g, black.b, black.a), (0, 0, 0, 255));

        let red = "#ff0000".to_color();
        assert_eq!((red.r, red.g, red.b, red.a), (255, 0, 0, 255));

        let lime = "#00ff00".to_color();
        assert_eq!((lime.r, lime.g, lime.b, lime.a), (0, 255, 0, 255));

        let blue = "#0000ff".to_color();
        assert_eq!((blue.r, blue.g, blue.b, blue.a), (0, 0, 255, 255));
    }

    #[test]
    fn test_rrggbbaa_colors() {
        // Test #RRGGBBAA format
        let fully_opaque_white = "#ffffffff".to_color();
        assert_eq!(
            (
                fully_opaque_white.r,
                fully_opaque_white.g,
                fully_opaque_white.b,
                fully_opaque_white.a
            ),
            (255, 255, 255, 255)
        );

        let semi_transparent_red = "#ff0000aa".to_color();
        assert_eq!(
            (
                semi_transparent_red.r,
                semi_transparent_red.g,
                semi_transparent_red.b,
                semi_transparent_red.a
            ),
            (255, 0, 0, 170)
        );

        let fully_transparent_blue = "#0000ff00".to_color();
        assert_eq!(
            (
                fully_transparent_blue.r,
                fully_transparent_blue.g,
                fully_transparent_blue.b,
                fully_transparent_blue.a
            ),
            (0, 0, 255, 0)
        );
    }

    #[test]
    fn test_mixed_values() {
        // Test mixed values in different positions
        let mixed = "#1a2b3c".to_color();
        assert_eq!((mixed.r, mixed.g, mixed.b, mixed.a), (26, 43, 60, 255));

        let mixed_short = "#123".to_color();
        assert_eq!(
            (mixed_short.r, mixed_short.g, mixed_short.b, mixed_short.a),
            (17, 34, 51, 255)
        );

        let mixed_alpha = "#1234".to_color();
        assert_eq!(
            (mixed_alpha.r, mixed_alpha.g, mixed_alpha.b, mixed_alpha.a),
            (17, 34, 51, 68)
        );
    }

    #[test]
    fn test_edge_cases() {
        // Test minimum values
        let min_values = "#010101".to_color();
        assert_eq!(
            (min_values.r, min_values.g, min_values.b, min_values.a),
            (1, 1, 1, 255)
        );

        let min_values_short = "#111".to_color();
        assert_eq!(
            (
                min_values_short.r,
                min_values_short.g,
                min_values_short.b,
                min_values_short.a
            ),
            (17, 17, 17, 255)
        );

        // Test specific alpha values
        let specific_alpha = "#ffffff80".to_color();
        assert_eq!(
            (
                specific_alpha.r,
                specific_alpha.g,
                specific_alpha.b,
                specific_alpha.a
            ),
            (255, 255, 255, 128)
        );
    }

    #[test]
    #[should_panic(expected = "Cannot parse empty string")]
    fn test_empty_string() {
        "".to_color();
    }

    #[test]
    #[should_panic(expected = "The color string must start with #")]
    fn test_invalid_format() {
        "fff".to_color();
    }

    #[test]
    #[should_panic(expected = "Invalid length of string for Parse to Color")]
    fn test_invalid_length() {
        "#ff".to_color();
    }

    #[test]
    #[should_panic(expected = "Cannot parse hex value")]
    fn test_invalid_hex_characters() {
        "#gghhii".to_color();
    }
}
