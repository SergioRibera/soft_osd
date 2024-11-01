mod hex_rgb;

use font_kit::font::Font;
use font_kit::source::SystemSource;
pub use hex_rgb::ToColor;

pub fn ease_out_cubic(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(3)
}

pub fn load_font(content: char) -> Font {
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
