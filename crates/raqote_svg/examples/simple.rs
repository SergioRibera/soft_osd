fn main() {
    let content = include_bytes!("./discord.svg").to_vec();
    raqote_svg::render_bytes_to_file(content, (255, 255), "./discord.png");
}
