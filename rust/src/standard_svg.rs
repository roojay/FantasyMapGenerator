use crate::MapDrawData;
use serde::Deserialize;
use std::fmt::Write;

#[derive(Clone, Debug, Deserialize)]
struct SvgLayers {
    slope: bool,
    river: bool,
    contour: bool,
    border: bool,
    city: bool,
    town: bool,
    label: bool,
}

impl Default for SvgLayers {
    fn default() -> Self {
        Self {
            slope: true,
            river: true,
            contour: true,
            border: true,
            city: true,
            town: true,
            label: true,
        }
    }
}

#[derive(Clone, Copy)]
struct SvgStyle {
    slope_line_width: f64,
    river_line_width: f64,
    contour_line_width: f64,
    border_line_width: f64,
    city_outer_radius: f64,
    city_inner_radius: f64,
    town_radius: f64,
}

impl SvgStyle {
    fn with_scale(scale: f64) -> Self {
        Self {
            slope_line_width: 1.0 * scale,
            river_line_width: 2.5 * scale,
            contour_line_width: 1.5 * scale,
            border_line_width: 6.0 * scale,
            city_outer_radius: 10.0 * scale,
            city_inner_radius: 5.0 * scale,
            town_radius: 5.0 * scale,
        }
    }
}

pub fn build_map_svg(map_json: &str, layers_json: &str) -> Result<String, String> {
    let map_data: MapDrawData =
        serde_json::from_str(map_json).map_err(|err| format!("Invalid map JSON: {err}"))?;
    let layers = if layers_json.trim().is_empty() {
        SvgLayers::default()
    } else {
        serde_json::from_str(layers_json).map_err(|err| format!("Invalid layers JSON: {err}"))?
    };

    Ok(build_map_svg_from_data(&map_data, &layers))
}

fn build_map_svg_from_data(map_data: &MapDrawData, layers: &SvgLayers) -> String {
    let width = map_data.image_width as f64;
    let height = map_data.image_height as f64;
    let style = SvgStyle::with_scale(map_data.draw_scale);

    let mut svg = String::with_capacity(
        1024 + map_data.slope.len() * 6
            + (map_data.river.len() + map_data.contour.len() + map_data.territory.len()) * 32,
    );

    write!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" preserveAspectRatio=\"xMidYMid meet\">",
        map_data.image_width, map_data.image_height, map_data.image_width, map_data.image_height
    )
    .unwrap();
    write!(
        svg,
        "<rect width=\"{}\" height=\"{}\" fill=\"white\"/>",
        map_data.image_width, map_data.image_height
    )
    .unwrap();

    if layers.slope && !map_data.slope.is_empty() {
        svg.push_str(
            "<path fill=\"none\" stroke=\"rgba(0,0,0,0.75)\" stroke-linecap=\"round\" d=\"",
        );
        append_segments(&mut svg, &map_data.slope, width, height);
        write!(
            svg,
            "\" stroke-width=\"{}\"/>",
            compact(style.slope_line_width)
        )
        .unwrap();
    }

    if layers.border && !map_data.territory.is_empty() {
        write!(
            svg,
            "<g fill=\"none\" stroke=\"white\" stroke-width=\"{}\">",
            compact(style.border_line_width)
        )
        .unwrap();
        append_paths(&mut svg, &map_data.territory, width, height);
        svg.push_str("</g>");
        write!(
            svg,
            "<g fill=\"none\" stroke=\"black\" stroke-width=\"{}\" stroke-dasharray=\"{},{}\" stroke-linecap=\"butt\" stroke-linejoin=\"bevel\">",
            compact(style.border_line_width),
            compact(3.0 * map_data.draw_scale),
            compact(4.0 * map_data.draw_scale)
        )
        .unwrap();
        append_paths(&mut svg, &map_data.territory, width, height);
        svg.push_str("</g>");
    }

    if layers.river && !map_data.river.is_empty() {
        write!(
            svg,
            "<g fill=\"none\" stroke=\"black\" stroke-width=\"{}\">",
            compact(style.river_line_width)
        )
        .unwrap();
        append_paths(&mut svg, &map_data.river, width, height);
        svg.push_str("</g>");
    }

    if layers.contour && !map_data.contour.is_empty() {
        write!(
            svg,
            "<g fill=\"none\" stroke=\"black\" stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\">",
            compact(style.contour_line_width)
        )
        .unwrap();
        append_paths(&mut svg, &map_data.contour, width, height);
        svg.push_str("</g>");
    }

    if layers.city && !map_data.city.is_empty() {
        svg.push_str("<g>");
        for city in map_data.city.chunks_exact(2) {
            let x = city[0] * width;
            let y = height - city[1] * height;
            write!(
                svg,
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"white\" stroke=\"black\" stroke-width=\"{}\"/>",
                compact(x),
                compact(y),
                compact(style.city_outer_radius),
                compact(style.slope_line_width)
            )
            .unwrap();
            write!(
                svg,
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"black\"/>",
                compact(x),
                compact(y),
                compact(style.city_inner_radius)
            )
            .unwrap();
        }
        svg.push_str("</g>");
    }

    if layers.town && !map_data.town.is_empty() {
        write!(
            svg,
            "<g fill=\"white\" stroke=\"black\" stroke-width=\"{}\">",
            compact(style.slope_line_width)
        )
        .unwrap();
        for town in map_data.town.chunks_exact(2) {
            let x = town[0] * width;
            let y = height - town[1] * height;
            write!(
                svg,
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\"/>",
                compact(x),
                compact(y),
                compact(style.town_radius)
            )
            .unwrap();
        }
        svg.push_str("</g>");
    }

    if layers.label && !map_data.label.is_empty() {
        svg.push_str("<g>");
        for label in &map_data.label {
            let x = label.position[0] * width;
            let y = height - label.position[1] * height;
            let font_family = match label.fontface.as_str() {
                "Times New Roman" => "serif",
                _ => "serif",
            };
            let text = escape_text(&label.text);
            write!(
                svg,
                "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\" fill=\"white\" stroke=\"white\" stroke-width=\"3\" stroke-linejoin=\"round\" paint-order=\"stroke fill\" text-anchor=\"start\" dominant-baseline=\"alphabetic\">{}</text>",
                compact(x),
                compact(y),
                font_family,
                label.fontsize,
                text
            )
            .unwrap();
            write!(
                svg,
                "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\" fill=\"black\" text-anchor=\"start\" dominant-baseline=\"alphabetic\">{}</text>",
                compact(x),
                compact(y),
                font_family,
                label.fontsize,
                text
            )
            .unwrap();
        }
        svg.push_str("</g>");
    }

    svg.push_str("</svg>");
    svg
}

fn append_paths(output: &mut String, paths: &[Vec<f64>], width: f64, height: f64) {
    for path in paths {
        if path.len() < 2 {
            continue;
        }

        output.push_str("<path d=\"");
        for (index, point) in path.chunks_exact(2).enumerate() {
            let x = point[0] * width;
            let y = height - point[1] * height;
            if index == 0 {
                output.push('M');
            } else {
                output.push('L');
            }
            output.push_str(&compact(x));
            output.push(' ');
            output.push_str(&compact(y));
        }
        output.push_str("\"/>");
    }
}

fn append_segments(output: &mut String, segments: &[f64], width: f64, height: f64) {
    for segment in segments.chunks_exact(4) {
        let x1 = segment[0] * width;
        let y1 = height - segment[1] * height;
        let x2 = segment[2] * width;
        let y2 = height - segment[3] * height;
        output.push('M');
        output.push_str(&compact(x1));
        output.push(' ');
        output.push_str(&compact(y1));
        output.push('L');
        output.push_str(&compact(x2));
        output.push(' ');
        output.push_str(&compact(y2));
    }
}

fn compact(value: f64) -> String {
    let mut text = format!("{value:.2}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    text
}

fn escape_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::build_map_svg;

    #[test]
    fn standard_svg_contains_core_layers() {
        let map_json = r#"{
            "image_width": 100,
            "image_height": 50,
            "draw_scale": 1.0,
            "contour": [[0.0, 0.0, 1.0, 1.0]],
            "river": [[0.1, 0.1, 0.9, 0.9]],
            "slope": [0.2, 0.2, 0.3, 0.3],
            "city": [0.5, 0.5],
            "town": [0.6, 0.6],
            "territory": [[0.2, 0.2, 0.8, 0.8]],
            "label": [{
                "text": "Test",
                "fontface": "Times New Roman",
                "fontsize": 12,
                "position": [0.4, 0.4],
                "extents": [0.0, 0.0, 0.0, 0.0],
                "charextents": [],
                "score": 1.0
            }]
        }"#;
        let layers_json = r#"{
            "slope": true,
            "river": true,
            "contour": true,
            "border": true,
            "city": true,
            "town": true,
            "label": true
        }"#;

        let svg = build_map_svg(map_json, layers_json).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("<path"));
        assert!(svg.contains("<circle"));
        assert!(svg.contains("<text"));
    }
}
