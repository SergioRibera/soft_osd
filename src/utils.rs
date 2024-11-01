mod hex_rgb;

pub use hex_rgb::ToColor;

pub fn ease_out_cubic(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(3)
}
