mod hex_rgb;

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
