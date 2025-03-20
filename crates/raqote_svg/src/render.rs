use raqote::{DrawOptions, DrawTarget, PathBuilder, SolidSource, Source};
use svg::node::element::path::Command;
use svg::parser::{Event, Parser};

pub fn render(tree: Parser<'_>, draw: &mut DrawTarget) {
    let mut scale_x: f32 = 1.0;
    let mut scale_y: f32 = 1.0;

    let mut last_x: f32 = 0.0;
    let mut last_y: f32 = 0.0;

    for event in tree {
        match event {
            Event::Tag(svg::node::element::tag::SVG, _, attributes) => {
                if let Some(view_box) = attributes.get("viewBox") {
                    let values: Vec<f32> = view_box
                        .split_whitespace()
                        .filter_map(|v| v.parse::<f32>().ok())
                        .collect();

                    if values.len() == 4 {
                        let view_width = values[2];
                        let view_height = values[3];

                        // Scale to fit the DrawTarget dimensions
                        scale_x = draw.width() as f32 / view_width;
                        scale_y = draw.height() as f32 / view_height;
                    }
                }
            }
            Event::Tag(svg::node::element::tag::Path, _, attributes) => {
                let Some(data) = attributes.get("d") else {
                    continue;
                };
                let Ok(data) = svg::node::element::path::Data::parse(data) else {
                    continue;
                };

                let mut path = PathBuilder::new();

                for command in data.iter() {
                    match command {
                        Command::Move(_, params) => {
                            let Some([x, y]) = params.chunks(2).next() else {
                                continue;
                            };
                            last_x = x * scale_x;
                            last_y = y * scale_y;
                            path.move_to(x * scale_x, y * scale_y);
                        }
                        Command::Line(_, params) => {
                            for chunk in params.chunks(2) {
                                let [x, y] = chunk else {
                                    continue;
                                };
                                last_x = x * scale_x;
                                last_y = y * scale_y;
                                path.line_to(x * scale_x, y * scale_y);
                            }
                        }
                        Command::HorizontalLine(_, params) => {
                            for x in params.iter() {
                                last_x = x * scale_x;
                                path.line_to(x * scale_x, last_y);
                            }
                        }
                        Command::VerticalLine(_, params) => {
                            for y in params.iter() {
                                last_y = y * scale_y;
                                path.line_to(last_x, y * scale_y);
                            }
                        }
                        Command::Close => {
                            path.close();
                        }
                        Command::QuadraticCurve(_, params)
                        | Command::SmoothCubicCurve(_, params) => {
                            for chunk in params.chunks(4) {
                                let [x1, y1, x, y] = chunk else {
                                    continue;
                                };
                                path.cubic_to(
                                    last_x,
                                    last_y,
                                    *x1 * scale_x,
                                    *y1 * scale_y,
                                    *x * scale_x,
                                    *y * scale_y,
                                );
                                last_x = x * scale_x;
                                last_y = y * scale_y;
                            }
                        }
                        Command::SmoothQuadraticCurve(_, params) => {
                            for chunk in params.chunks(2) {
                                let [x, y] = chunk else {
                                    continue;
                                };
                                path.quad_to(last_x, last_y, *x * scale_x, *y * scale_y);
                                last_x = x * scale_x;
                                last_y = y * scale_y;
                            }
                        }
                        Command::CubicCurve(_, params) => {
                            for chunk in params.chunks(6) {
                                let [x1, y1, x2, y2, x, y] = chunk else {
                                    continue;
                                };
                                path.cubic_to(
                                    *x1 * scale_x,
                                    *y1 * scale_y,
                                    *x2 * scale_x,
                                    *y2 * scale_y,
                                    *x * scale_x,
                                    *y * scale_y,
                                );
                                last_x = x * scale_x;
                                last_y = y * scale_y;
                            }
                        }
                        Command::EllipticalArc(_, params) => {
                            for chunk in params.chunks(5) {
                                let [rx, ry, x_axis_rotation, large_arc, sweep] = chunk else {
                                    continue;
                                };
                                path.arc(
                                    *rx * scale_x,
                                    *ry * scale_y,
                                    *x_axis_rotation * scale_x,
                                    *large_arc,
                                    *sweep,
                                );
                                last_x = rx * scale_x;
                                last_y = ry * scale_y;
                            }
                        }
                    }
                }

                let path = path.finish();

                // Determine fill color
                let fill_color = if let Some(fill_attr) = attributes.get("fill") {
                    csscolorparser::parse(fill_attr)
                        .ok()
                        .map(|c| {
                            let [r, g, b, a] = c.to_rgba8();
                            SolidSource::from_unpremultiplied_argb(a, r, g, b)
                        })
                        .unwrap_or(SolidSource::from_unpremultiplied_argb(255, 0, 0, 0))
                } else {
                    SolidSource::from_unpremultiplied_argb(255, 0, 0, 0)
                };

                let fill = Source::Solid(fill_color);
                draw.fill(&path, &fill, &DrawOptions::new());
            }
            Event::Tag(svg::node::element::tag::Circle, _, attributes) => {
                let (Some(cx), Some(cy), Some(r)) = (
                    attributes.get("cx"),
                    attributes.get("cy"),
                    attributes.get("r"),
                ) else {
                    continue;
                };

                let cx: f32 = cx.parse().unwrap_or(last_x);
                let cy: f32 = cy.parse().unwrap_or(last_y);
                let r: f32 = r.parse().unwrap_or(0.0);

                let mut path = PathBuilder::new();
                path.arc(
                    cx * scale_x,
                    cy * scale_y,
                    r * scale_x.min(scale_y), // Uniform scaling for circles
                    0.0,
                    2.0 * std::f32::consts::PI,
                );
                path.close();

                let path = path.finish();

                // Determine fill color
                let fill_color = if let Some(fill_attr) = attributes.get("fill") {
                    csscolorparser::parse(fill_attr)
                        .ok()
                        .map(|c| {
                            let [r, g, b, a] = c.to_rgba8();
                            SolidSource::from_unpremultiplied_argb(a, r, g, b)
                        })
                        .unwrap_or(SolidSource::from_unpremultiplied_argb(255, 0, 0, 0))
                } else {
                    SolidSource::from_unpremultiplied_argb(255, 0, 0, 0)
                };

                let fill = Source::Solid(fill_color);
                draw.fill(&path, &fill, &DrawOptions::new());
            }
            // Handle other SVG elements (rectangles, polygons, etc.) here
            _ => {}
        }
    }
}
