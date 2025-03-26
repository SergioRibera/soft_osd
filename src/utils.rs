mod hex_rgb;

pub use hex_rgb::ToColor;

pub fn ease_out_cubic(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(3)
}

pub fn adjust_brightness((r, g, b): (u8, u8, u8), factor: f32) -> (u8, u8, u8) {
    let new_r = (r as f32 * factor).clamp(0.0, 255.0) as u8;
    let new_g = (g as f32 * factor).clamp(0.0, 255.0) as u8;
    let new_b = (b as f32 * factor).clamp(0.0, 255.0) as u8;
    (new_r, new_g, new_b)
}

pub fn contrast_ratio(c1: (u8, u8, u8), c2: (u8, u8, u8)) -> f32 {
    let lum1 = relative_luminance(c1);
    let lum2 = relative_luminance(c2);
    if lum1 > lum2 {
        (lum1 + 0.05) / (lum2 + 0.05)
    } else {
        (lum2 + 0.05) / (lum1 + 0.05)
    }
}

fn relative_luminance((r, g, b): (u8, u8, u8)) -> f32 {
    let r = linear_component(r);
    let g = linear_component(g);
    let b = linear_component(b);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn linear_component(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c <= 0.03928 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
