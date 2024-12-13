mod render;

use std::path::Path;

pub use raqote;

use raqote::DrawTarget;
use render::render;

#[cfg(feature = "image")]
use image::{EncodableLayout, RgbaImage};

pub fn render_svg(content: impl AsRef<str>, draw_target: &mut DrawTarget) {
    let tree = svg::read(content.as_ref()).unwrap();
    render(tree, draw_target)
}

pub fn render_bytes(content: Vec<u8>, draw_target: &mut DrawTarget) {
    let content = String::from_utf8(content).unwrap();
    let tree = svg::read(&content).unwrap();

    render(tree, draw_target);
}

pub fn render_bytes_to_file(content: Vec<u8>, (width, height): (i32, i32), path: impl AsRef<Path>) {
    let mut draw = DrawTarget::new(width, height);
    let content = String::from_utf8(content).unwrap();
    let tree = svg::read(&content).unwrap();

    render(tree, &mut draw);

    draw.write_png(path).unwrap();
}

#[cfg(feature = "image")]
pub fn render_to_image(content: impl AsRef<str>, (width, height): (u32, u32)) -> RgbaImage {
    let mut draw = DrawTarget::new(width as i32, height as i32);
    let tree = svg::read(content.as_ref()).unwrap();
    render(tree, &mut draw);

    RgbaImage::from_vec(width, height, draw.get_data_u8().to_vec()).unwrap()
}

#[cfg(feature = "image")]
pub fn render_to_image_mut(content: impl AsRef<str>, img: &mut RgbaImage) {
    let rendered = render_to_image(content, (img.width(), img.height()));

    img.copy_from_slice(rendered.as_bytes());
}
