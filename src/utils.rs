mod hex_rgb;

use font_kit::family_name::FamilyName;
use font_kit::font::Font;
use font_kit::source::SystemSource;
pub use hex_rgb::ToColor;

pub fn ease_out_cubic(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(3)
}

pub fn lighten_color(r: u8, g: u8, b: u8, factor: f32) -> (u8, u8, u8) {
    let factor = factor.clamp(0.0, 1.0);

    let lighten = |component: u8| -> u8 {
        ((component as f32) + ((255.0 - component as f32) * factor)).round() as u8
    };

    (lighten(r), lighten(g), lighten(b))
}

pub fn load_font_by_glyph(content: char) -> Font {
    let source = SystemSource::new();
    source
        .all_fonts()
        .unwrap()
        .into_iter()
        .filter_map(|f| f.load().ok()?.glyph_for_char(content).map(|_| f))
        .next()
        .unwrap_or_else(|| panic!("Cannot found Font Family for glyph: {content}"))
        .load()
        .expect("Cannot load font")
}

pub fn load_font(content: &str) -> Font {
    let families = content
        .split(',')
        .map(|family| match family {
            "serif" => FamilyName::Serif,
            "sans-serif" => FamilyName::SansSerif,
            "monospace" => FamilyName::Monospace,
            "cursive" => FamilyName::Cursive,
            "fantasy" => FamilyName::Fantasy,
            _ => FamilyName::Title(family.to_string()),
        })
        .collect::<Vec<_>>();

    SystemSource::new()
        .select_best_match(&families, &Default::default())
        .unwrap_or_else(|_| panic!("Cannot found Font Family: {content}"))
        .load()
        .expect("Cannot load font")
}
