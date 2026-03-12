use crate::presentation::{
    default_layer_config_section, default_layer_metadata, PresentationOutputKind,
    PresentationPluginCapabilities, PresentationPluginMetadata, RenderDataPlugin,
};
use crate::MapDrawData;
use serde::Deserialize;
use std::fmt::Write;

#[derive(Clone, Debug, Deserialize)]
pub struct StandardSvgLayers {
    pub slope: bool,
    pub river: bool,
    pub contour: bool,
    pub border: bool,
    pub city: bool,
    pub town: bool,
    pub label: bool,
}

impl Default for StandardSvgLayers {
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

#[derive(Clone, Copy, Debug)]
pub struct StandardSvgStyle {
    pub slope_line_width: f64,
    pub river_line_width: f64,
    pub contour_line_width: f64,
    pub border_line_width: f64,
    pub city_outer_radius: f64,
    pub city_inner_radius: f64,
    pub town_radius: f64,
}

impl StandardSvgStyle {
    pub fn with_scale(scale: f64) -> Self {
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

#[derive(Clone, Debug)]
pub struct SvgPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug)]
pub struct SvgSegment {
    pub start: SvgPoint,
    pub end: SvgPoint,
}

#[derive(Clone, Debug)]
pub struct SvgPath {
    pub points: Vec<SvgPoint>,
}

#[derive(Clone, Debug)]
pub struct SvgCircleMarker {
    pub center: SvgPoint,
    pub radius: f64,
}

#[derive(Clone, Debug)]
pub struct SvgLabel {
    pub text: String,
    pub font_family: String,
    pub font_size: i32,
    pub anchor: SvgPoint,
}

#[derive(Clone, Debug)]
pub struct StandardSvgScene {
    pub width: u32,
    pub height: u32,
    pub draw_scale: f64,
    pub style: StandardSvgStyle,
    pub slope_segments: Vec<SvgSegment>,
    pub river_paths: Vec<SvgPath>,
    pub contour_paths: Vec<SvgPath>,
    pub border_paths: Vec<SvgPath>,
    pub city_markers: Vec<(SvgCircleMarker, SvgCircleMarker)>,
    pub town_markers: Vec<SvgCircleMarker>,
    pub labels: Vec<SvgLabel>,
}

#[derive(Default)]
pub struct StandardSvgPlugin;

impl RenderDataPlugin for StandardSvgPlugin {
    type Config = StandardSvgLayers;
    type Output = StandardSvgScene;
    type Error = String;

    fn build(map_data: &MapDrawData, config: &Self::Config) -> Result<Self::Output, Self::Error> {
        Ok(build_standard_svg_scene(map_data, config))
    }

    fn metadata() -> PresentationPluginMetadata {
        PresentationPluginMetadata {
            id: "standard_svg",
            display_name: "Standard SVG",
            description: "Classic paper-style vector SVG scene with lightweight layer toggles.",
            output_kind: PresentationOutputKind::SvgScene,
            capabilities: PresentationPluginCapabilities {
                supports_layer_config: true,
                supports_direct_svg_export: true,
                requires_raster_data: false,
                requires_heightmap: false,
                requires_land_mask: false,
                embeds_raster_images: false,
            },
            supported_layers: default_layer_metadata(),
            config_sections: vec![default_layer_config_section()],
        }
    }
}

pub fn build_standard_svg_scene(
    map_data: &MapDrawData,
    layers: &StandardSvgLayers,
) -> StandardSvgScene {
    let width = map_data.image_width as f64;
    let height = map_data.image_height as f64;
    let style = StandardSvgStyle::with_scale(map_data.draw_scale);

    let slope_segments = if layers.slope {
        map_data
            .slope
            .chunks_exact(4)
            .map(|segment| SvgSegment {
                start: normalize_to_svg_point(segment[0], segment[1], width, height),
                end: normalize_to_svg_point(segment[2], segment[3], width, height),
            })
            .collect()
    } else {
        Vec::new()
    };

    let river_paths = if layers.river {
        build_svg_paths(&map_data.river, width, height)
    } else {
        Vec::new()
    };

    let contour_paths = if layers.contour {
        build_svg_paths(&map_data.contour, width, height)
    } else {
        Vec::new()
    };

    let border_paths = if layers.border {
        build_svg_paths(&map_data.territory, width, height)
    } else {
        Vec::new()
    };

    let city_markers = if layers.city {
        map_data
            .city
            .chunks_exact(2)
            .map(|city| {
                let center = normalize_to_svg_point(city[0], city[1], width, height);
                (
                    SvgCircleMarker {
                        center: center.clone(),
                        radius: style.city_outer_radius,
                    },
                    SvgCircleMarker {
                        center,
                        radius: style.city_inner_radius,
                    },
                )
            })
            .collect()
    } else {
        Vec::new()
    };

    let town_markers = if layers.town {
        map_data
            .town
            .chunks_exact(2)
            .map(|town| SvgCircleMarker {
                center: normalize_to_svg_point(town[0], town[1], width, height),
                radius: style.town_radius,
            })
            .collect()
    } else {
        Vec::new()
    };

    let labels = if layers.label {
        map_data
            .label
            .iter()
            .map(|label| SvgLabel {
                text: label.text.clone(),
                font_family: resolve_svg_font_family(&label.fontface).to_string(),
                font_size: label.fontsize,
                anchor: normalize_to_svg_point(label.position[0], label.position[1], width, height),
            })
            .collect()
    } else {
        Vec::new()
    };

    StandardSvgScene {
        width: map_data.image_width,
        height: map_data.image_height,
        draw_scale: map_data.draw_scale,
        style,
        slope_segments,
        river_paths,
        contour_paths,
        border_paths,
        city_markers,
        town_markers,
        labels,
    }
}

pub fn render_standard_svg_scene(scene: &StandardSvgScene) -> String {
    let mut svg = String::with_capacity(
        1024 + scene.slope_segments.len() * 24
            + (scene.river_paths.len() + scene.contour_paths.len() + scene.border_paths.len()) * 48,
    );

    write!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" preserveAspectRatio=\"xMidYMid meet\">",
        scene.width, scene.height, scene.width, scene.height
    )
    .unwrap();
    write!(
        svg,
        "<rect width=\"{}\" height=\"{}\" fill=\"white\"/>",
        scene.width, scene.height
    )
    .unwrap();

    if !scene.slope_segments.is_empty() {
        svg.push_str(
            "<path fill=\"none\" stroke=\"rgba(0,0,0,0.75)\" stroke-linecap=\"round\" d=\"",
        );
        append_scene_segments(&mut svg, &scene.slope_segments);
        write!(
            svg,
            "\" stroke-width=\"{}\"/>",
            compact(scene.style.slope_line_width)
        )
        .unwrap();
    }

    if !scene.border_paths.is_empty() {
        write!(
            svg,
            "<g fill=\"none\" stroke=\"white\" stroke-width=\"{}\">",
            compact(scene.style.border_line_width)
        )
        .unwrap();
        append_scene_paths(&mut svg, &scene.border_paths);
        svg.push_str("</g>");
        write!(
            svg,
            "<g fill=\"none\" stroke=\"black\" stroke-width=\"{}\" stroke-dasharray=\"{},{}\" stroke-linecap=\"butt\" stroke-linejoin=\"bevel\">",
            compact(scene.style.border_line_width),
            compact(3.0 * scene.draw_scale),
            compact(4.0 * scene.draw_scale)
        )
        .unwrap();
        append_scene_paths(&mut svg, &scene.border_paths);
        svg.push_str("</g>");
    }

    if !scene.river_paths.is_empty() {
        write!(
            svg,
            "<g fill=\"none\" stroke=\"black\" stroke-width=\"{}\">",
            compact(scene.style.river_line_width)
        )
        .unwrap();
        append_scene_paths(&mut svg, &scene.river_paths);
        svg.push_str("</g>");
    }

    if !scene.contour_paths.is_empty() {
        write!(
            svg,
            "<g fill=\"none\" stroke=\"black\" stroke-width=\"{}\" stroke-linecap=\"round\" stroke-linejoin=\"round\">",
            compact(scene.style.contour_line_width)
        )
        .unwrap();
        append_scene_paths(&mut svg, &scene.contour_paths);
        svg.push_str("</g>");
    }

    if !scene.city_markers.is_empty() {
        svg.push_str("<g>");
        for (outer, inner) in &scene.city_markers {
            write!(
                svg,
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"white\" stroke=\"black\" stroke-width=\"{}\"/>",
                compact(outer.center.x),
                compact(outer.center.y),
                compact(outer.radius),
                compact(scene.style.slope_line_width)
            )
            .unwrap();
            write!(
                svg,
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"black\"/>",
                compact(inner.center.x),
                compact(inner.center.y),
                compact(inner.radius)
            )
            .unwrap();
        }
        svg.push_str("</g>");
    }

    if !scene.town_markers.is_empty() {
        write!(
            svg,
            "<g fill=\"white\" stroke=\"black\" stroke-width=\"{}\">",
            compact(scene.style.slope_line_width)
        )
        .unwrap();
        for town in &scene.town_markers {
            write!(
                svg,
                "<circle cx=\"{}\" cy=\"{}\" r=\"{}\"/>",
                compact(town.center.x),
                compact(town.center.y),
                compact(town.radius)
            )
            .unwrap();
        }
        svg.push_str("</g>");
    }

    if !scene.labels.is_empty() {
        svg.push_str("<g>");
        for label in &scene.labels {
            let text = escape_text(&label.text);
            write!(
                svg,
                "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\" fill=\"white\" stroke=\"white\" stroke-width=\"3\" stroke-linejoin=\"round\" paint-order=\"stroke fill\" text-anchor=\"start\" dominant-baseline=\"alphabetic\">{}</text>",
                compact(label.anchor.x),
                compact(label.anchor.y),
                label.font_family,
                label.font_size,
                text
            )
            .unwrap();
            write!(
                svg,
                "<text x=\"{}\" y=\"{}\" font-family=\"{}\" font-size=\"{}\" fill=\"black\" text-anchor=\"start\" dominant-baseline=\"alphabetic\">{}</text>",
                compact(label.anchor.x),
                compact(label.anchor.y),
                label.font_family,
                label.font_size,
                text
            )
            .unwrap();
        }
        svg.push_str("</g>");
    }

    svg.push_str("</svg>");
    svg
}

fn build_svg_paths(paths: &[Vec<f64>], width: f64, height: f64) -> Vec<SvgPath> {
    paths
        .iter()
        .filter_map(|path| {
            let points: Vec<SvgPoint> = path
                .chunks_exact(2)
                .map(|point| normalize_to_svg_point(point[0], point[1], width, height))
                .collect();
            if points.is_empty() {
                None
            } else {
                Some(SvgPath { points })
            }
        })
        .collect()
}

fn normalize_to_svg_point(nx: f64, ny: f64, width: f64, height: f64) -> SvgPoint {
    SvgPoint {
        x: nx * width,
        y: height - ny * height,
    }
}

fn append_scene_paths(output: &mut String, paths: &[SvgPath]) {
    for path in paths {
        if path.points.is_empty() {
            continue;
        }
        output.push_str("<path d=\"");
        for (index, point) in path.points.iter().enumerate() {
            if index == 0 {
                output.push('M');
            } else {
                output.push('L');
            }
            output.push_str(&compact(point.x));
            output.push(' ');
            output.push_str(&compact(point.y));
        }
        output.push_str("\"/>");
    }
}

fn append_scene_segments(output: &mut String, segments: &[SvgSegment]) {
    for segment in segments {
        output.push('M');
        output.push_str(&compact(segment.start.x));
        output.push(' ');
        output.push_str(&compact(segment.start.y));
        output.push('L');
        output.push_str(&compact(segment.end.x));
        output.push(' ');
        output.push_str(&compact(segment.end.y));
    }
}

fn resolve_svg_font_family(fontface: &str) -> &'static str {
    match fontface {
        "Times New Roman" => "serif",
        _ => "serif",
    }
}

fn compact(value: f64) -> String {
    if value.fract().abs() < 1e-6 {
        format!("{:.0}", value)
    } else {
        format!("{:.3}", value)
    }
}

fn escape_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
