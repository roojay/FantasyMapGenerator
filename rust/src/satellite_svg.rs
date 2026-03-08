use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine as _;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use serde::Deserialize;
use std::fmt::Write;

#[derive(Clone, Copy)]
struct TerrainColor {
    r: f64,
    g: f64,
    b: f64,
}

#[derive(Deserialize)]
struct MapRasterF32 {
    width: u32,
    height: u32,
    data: Vec<f32>,
}

#[derive(Deserialize)]
struct MapRasterU8 {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

#[derive(Deserialize)]
struct MapLabel {
    text: String,
    fontface: String,
    fontsize: i32,
    position: [f64; 2],
}

#[derive(Deserialize)]
struct MapData {
    image_width: u32,
    image_height: u32,
    #[serde(default)]
    city: Vec<f64>,
    #[serde(default)]
    contour: Vec<Vec<f64>>,
    #[serde(default)]
    label: Vec<MapLabel>,
    #[serde(default)]
    land_polygons: Vec<Vec<f64>>,
    #[serde(default)]
    river: Vec<Vec<f64>>,
    #[serde(default)]
    slope: Vec<f64>,
    #[serde(default)]
    territory: Vec<Vec<f64>>,
    #[serde(default)]
    town: Vec<f64>,
    heightmap: Option<MapRasterF32>,
    land_mask: Option<MapRasterU8>,
}

#[derive(Deserialize)]
pub struct SatelliteSvgLayers {
    #[serde(default)]
    pub slope: bool,
    pub river: bool,
    #[serde(default)]
    pub contour: bool,
    pub border: bool,
    pub city: bool,
    pub town: bool,
    pub label: bool,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(default)]
pub struct SatelliteSvgOptions {
    pub max_embedded_image_size: u32,
    pub jpeg_quality: u8,
}

impl Default for SatelliteSvgOptions {
    fn default() -> Self {
        Self {
            max_embedded_image_size: 1600,
            jpeg_quality: 82,
        }
    }
}

struct SlopeSample {
    slope_x: f64,
    slope_y: f64,
    magnitude: f64,
}

pub fn build_satellite_svg(map_json: &str, layers_json: &str) -> Result<String, String> {
    build_satellite_svg_with_options(map_json, layers_json, None)
}

pub fn build_satellite_svg_with_options(
    map_json: &str,
    layers_json: &str,
    options_json: Option<&str>,
) -> Result<String, String> {
    let map_data: MapData =
        serde_json::from_str(map_json).map_err(|err| format!("Failed to parse map data: {err}"))?;
    let layers: SatelliteSvgLayers = serde_json::from_str(layers_json)
        .map_err(|err| format!("Failed to parse layer config: {err}"))?;
    let options = match options_json {
        Some(json) if !json.trim().is_empty() => serde_json::from_str(json)
            .map_err(|err| format!("Failed to parse satellite SVG options: {err}"))?,
        _ => SatelliteSvgOptions::default(),
    };
    build_satellite_svg_from_data(&map_data, &layers, options)
}

fn build_satellite_svg_from_data(
    map_data: &MapData,
    layers: &SatelliteSvgLayers,
    options: SatelliteSvgOptions,
) -> Result<String, String> {
    let heightmap = map_data
        .heightmap
        .as_ref()
        .ok_or_else(|| "Heightmap data is required for satellite rendering".to_string())?;
    let land_mask = map_data
        .land_mask
        .as_ref()
        .ok_or_else(|| "Land mask data is required for satellite rendering".to_string())?;

    let width = map_data.image_width;
    let height = map_data.image_height;
    let (embedded_width, embedded_height) =
        compute_embedded_image_size(width, height, options.max_embedded_image_size);
    let terrain_image = encode_jpeg_data_url(
        embedded_width,
        embedded_height,
        &generate_terrain_rgba(
            heightmap,
            land_mask,
            embedded_width,
            embedded_height,
            texture_seed(heightmap),
        ),
        options.jpeg_quality,
    )?;
    let ocean_image = encode_jpeg_data_url(
        embedded_width,
        embedded_height,
        &generate_ocean_rgba(embedded_width, embedded_height),
        options.jpeg_quality,
    )?;
    let use_vector_land_clip = !map_data.land_polygons.is_empty();
    let land_mask_image = if use_vector_land_clip {
        None
    } else {
        Some(encode_png_data_url(
            embedded_width,
            embedded_height,
            &generate_land_mask_rgba(land_mask, embedded_width, embedded_height),
        )?)
    };
    let land_paint_attr = if use_vector_land_clip {
        "clip-path=\"url(#land-clip)\""
    } else {
        "mask=\"url(#land-mask)\""
    };

    let mut svg = String::with_capacity((width * height / 2) as usize);
    write!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">",
        width, height, width, height
    )
    .unwrap();
    svg.push_str(generate_defs_start());
    if use_vector_land_clip {
        svg.push_str("<clipPath id=\"land-clip\" clipPathUnits=\"userSpaceOnUse\">");
        append_svg_closed_path(
            &mut svg,
            &map_data.land_polygons,
            width as f64,
            height as f64,
        );
        svg.push_str("</clipPath>");
    } else {
        svg.push_str("<mask id=\"land-mask\">");
        write!(
            svg,
            "<image href=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\"/>",
            land_mask_image.as_deref().unwrap_or_default(),
            width,
            height
        )
        .unwrap();
        svg.push_str("</mask>");
    }
    svg.push_str("</defs>");
    write!(
        svg,
        "<image href=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\"/>",
        ocean_image, width, height
    )
    .unwrap();
    write!(svg, "<g {}>", land_paint_attr).unwrap();
    write!(
        svg,
        "<image href=\"{}\" width=\"{}\" height=\"{}\" preserveAspectRatio=\"none\"/>",
        terrain_image, width, height
    )
    .unwrap();
    write!(
        svg,
        "<rect width=\"{}\" height=\"{}\" fill=\"#f8eacb\" fill-opacity=\".08\"/>",
        width, height
    )
    .unwrap();
    svg.push_str("</g>");

    if layers.slope && !map_data.slope.is_empty() {
        write!(
            svg,
            "<g {} stroke=\"#14251a\" stroke-opacity=\".16\" stroke-width=\"1\" fill=\"none\" stroke-linecap=\"round\">",
            land_paint_attr
        )
        .unwrap();
        append_svg_segments(&mut svg, &map_data.slope, width as f64, height as f64);
        svg.push_str("</g>");
    }

    if layers.river && !map_data.river.is_empty() {
        svg.push_str(
            "<g stroke=\"#7ae1ff\" stroke-width=\"2.1\" fill=\"none\" opacity=\".88\" filter=\"url(#river-glow)\" stroke-linecap=\"round\" stroke-linejoin=\"round\">",
        );
        append_svg_path(&mut svg, &map_data.river, width as f64, height as f64);
        svg.push_str("</g>");
    }

    if layers.border && !map_data.territory.is_empty() {
        svg.push_str(
            "<g stroke=\"#dff4d3\" stroke-opacity=\".42\" stroke-width=\"2.2\" fill=\"none\" stroke-dasharray=\"6 4\" stroke-linecap=\"round\" filter=\"url(#border-shadow)\">",
        );
        append_svg_path(&mut svg, &map_data.territory, width as f64, height as f64);
        svg.push_str("</g>");
    }

    if layers.contour && !map_data.contour.is_empty() {
        svg.push_str(
            "<g stroke=\"#9be9f3\" stroke-opacity=\".34\" stroke-width=\"4.8\" fill=\"none\" filter=\"url(#coast-line-glow)\" stroke-linecap=\"round\" stroke-linejoin=\"round\">",
        );
        append_svg_path(&mut svg, &map_data.contour, width as f64, height as f64);
        svg.push_str("</g>");
        svg.push_str(
            "<g stroke=\"#def6f4\" stroke-opacity=\".72\" stroke-width=\"1.6\" fill=\"none\" stroke-linecap=\"round\" stroke-linejoin=\"round\">",
        );
        append_svg_path(&mut svg, &map_data.contour, width as f64, height as f64);
        svg.push_str("</g>");
    }

    if layers.city {
        append_city_markers(&mut svg, map_data, width as f64, height as f64);
    }
    if layers.town {
        append_town_markers(&mut svg, map_data, width as f64, height as f64);
    }
    if layers.label {
        append_labels(&mut svg, map_data, width as f64, height as f64);
    }

    write!(
        svg,
        "<rect width=\"{}\" height=\"{}\" fill=\"#151f2e\" fill-opacity=\".12\"/></svg>",
        width, height
    )
    .unwrap();
    Ok(svg)
}

fn texture_seed(heightmap: &MapRasterF32) -> f64 {
    let mut hash = (heightmap.width as u64)
        .wrapping_mul(73856093)
        .wrapping_add((heightmap.height as u64).wrapping_mul(19349663));
    for (idx, value) in heightmap.data.iter().take(64).enumerate() {
        hash ^= ((*value * 10000.0) as i64 as u64).wrapping_mul((idx as u64 + 1) * 83492791);
    }
    (hash % 1000) as f64 + 0.123
}

fn encode_png_data_url(width: u32, height: u32, rgba: &[u8]) -> Result<String, String> {
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes)
        .write_image(rgba, width, height, ColorType::Rgba8.into())
        .map_err(|err| format!("Failed to encode PNG: {err}"))?;
    Ok(format!("data:image/png;base64,{}", BASE64.encode(bytes)))
}

fn encode_jpeg_data_url(
    width: u32,
    height: u32,
    rgba: &[u8],
    quality: u8,
) -> Result<String, String> {
    let mut bytes = Vec::new();
    let rgb = rgba_to_rgb(rgba);
    JpegEncoder::new_with_quality(&mut bytes, quality.clamp(1, 100))
        .encode(&rgb, width, height, ColorType::Rgb8.into())
        .map_err(|err| format!("Failed to encode JPEG: {err}"))?;
    Ok(format!("data:image/jpeg;base64,{}", BASE64.encode(bytes)))
}

fn rgba_to_rgb(rgba: &[u8]) -> Vec<u8> {
    let mut rgb = Vec::with_capacity(rgba.len().saturating_mul(3) / 4);
    for chunk in rgba.chunks_exact(4) {
        rgb.extend_from_slice(&chunk[..3]);
    }
    rgb
}

fn compute_embedded_image_size(width: u32, height: u32, max_size: u32) -> (u32, u32) {
    let max_size = max_size.max(256);
    let longest_edge = width.max(height);
    if longest_edge <= max_size {
        return (width.max(1), height.max(1));
    }

    let scale = max_size as f64 / longest_edge as f64;
    (
        ((width as f64 * scale).round() as u32).max(1),
        ((height as f64 * scale).round() as u32).max(1),
    )
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = clamp01((x - edge0) / (edge1 - edge0).abs().max(0.0001));
    t * t * (3.0 - 2.0 * t)
}

fn simple_noise(x: f64, y: f64, seed: f64) -> f64 {
    let n = (x * 12.9898 + y * 78.233 + seed).sin() * 43758.5453;
    (n - n.floor()) * 2.0 - 1.0
}

fn lerp_color(a: TerrainColor, b: TerrainColor, t: f64) -> TerrainColor {
    let t = clamp01(t);
    TerrainColor {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
    }
}

fn get_mask_nearest(land_mask: &MapRasterU8, x: f64, y: f64) -> f64 {
    let sx = x
        .round()
        .clamp(0.0, (land_mask.width.saturating_sub(1)) as f64) as usize;
    let sy = y
        .round()
        .clamp(0.0, (land_mask.height.saturating_sub(1)) as f64) as usize;
    land_mask
        .data
        .get(sy * land_mask.width as usize + sx)
        .copied()
        .unwrap_or(0) as f64
}

fn sample_mask_average(land_mask: &MapRasterU8, x: f64, y: f64, radius: f64) -> f64 {
    let offsets = [
        (radius, 0.0),
        (-radius, 0.0),
        (0.0, radius),
        (0.0, -radius),
        (radius * 0.7, radius * 0.7),
        (-radius * 0.7, radius * 0.7),
        (radius * 0.7, -radius * 0.7),
        (-radius * 0.7, -radius * 0.7),
    ];

    let mut sum = 0.0;
    for (dx, dy) in offsets {
        sum += get_mask_nearest(land_mask, x + dx, y + dy);
    }
    sum / offsets.len() as f64
}

fn fantasy_terrain_color(
    height: f64,
    coastalness: f64,
    inland: f64,
    ridge: f64,
    latitude: f64,
    biome_mask: f64,
) -> TerrainColor {
    let beach = TerrainColor {
        r: 224.0,
        g: 212.0,
        b: 153.0,
    };
    let meadow = TerrainColor {
        r: 158.0,
        g: 196.0,
        b: 76.0,
    };
    let fertile = TerrainColor {
        r: 97.0,
        g: 157.0,
        b: 58.0,
    };
    let forest = TerrainColor {
        r: 48.0,
        g: 102.0,
        b: 43.0,
    };
    let grove = TerrainColor {
        r: 77.0,
        g: 130.0,
        b: 52.0,
    };
    let plateau = TerrainColor {
        r: 161.0,
        g: 147.0,
        b: 79.0,
    };
    let dryland = TerrainColor {
        r: 198.0,
        g: 173.0,
        b: 89.0,
    };
    let prairie = TerrainColor {
        r: 191.0,
        g: 205.0,
        b: 82.0,
    };
    let rock = TerrainColor {
        r: 132.0,
        g: 134.0,
        b: 128.0,
    };
    let cold_rock = TerrainColor {
        r: 119.0,
        g: 126.0,
        b: 137.0,
    };
    let tundra = TerrainColor {
        r: 178.0,
        g: 194.0,
        b: 168.0,
    };
    let snow = TerrainColor {
        r: 247.0,
        g: 249.0,
        b: 252.0,
    };

    let wetness = clamp01(coastalness * 0.82 + (1.0 - inland) * 0.14 + latitude * 0.06);
    let continental_dry = inland * (1.0 - wetness * 0.74);
    let coolness = smoothstep(0.68, 1.0, latitude);
    let lush_patch = smoothstep(0.52, 0.82, biome_mask + wetness * 0.24 - inland * 0.08);
    let dry_patch = smoothstep(0.56, 0.86, (1.0 - biome_mask) + continental_dry * 0.28);

    let mut lowland = lerp_color(meadow, fertile, wetness);
    lowland = lerp_color(lowland, prairie, dry_patch * (1.0 - wetness) * 0.72);
    let mut wooded = lerp_color(
        lowland,
        forest,
        smoothstep(0.22, 0.78, wetness + inland * 0.16),
    );
    wooded = lerp_color(wooded, grove, lush_patch * 0.46);

    let terrace_strength = smoothstep(0.28, 0.74, height) * (0.18 + ridge * 0.16);
    let terraced_height =
        height * (1.0 - terrace_strength) + ((height * 7.0).floor() / 7.0) * terrace_strength;

    let mut base = lerp_color(beach, lowland, smoothstep(0.02, 0.07, terraced_height));
    base = lerp_color(base, wooded, smoothstep(0.08, 0.24, terraced_height));
    base = lerp_color(
        base,
        lerp_color(plateau, dryland, continental_dry),
        smoothstep(0.20, 0.48, terraced_height) * (0.45 + continental_dry * 0.58),
    );
    base = lerp_color(
        base,
        tundra,
        smoothstep(0.42, 0.70, terraced_height) * coolness * 0.34,
    );
    base = lerp_color(
        base,
        lerp_color(rock, cold_rock, ridge),
        smoothstep(0.48, 0.76, terraced_height + ridge * 0.08),
    );

    let snowline = 0.76 - coolness * 0.10;
    lerp_color(
        base,
        snow,
        smoothstep(
            snowline,
            (snowline + 0.12).min(0.96),
            terraced_height + ridge * 0.05,
        ),
    )
}

fn get_height_bilinear(heightmap: &MapRasterF32, x: f64, y: f64) -> f64 {
    let x = x.clamp(0.0, heightmap.width.saturating_sub(1) as f64 - 0.001);
    let y = y.clamp(0.0, heightmap.height.saturating_sub(1) as f64 - 0.001);
    let x0 = x.floor() as usize;
    let y0 = y.floor() as usize;
    let x1 = (x0 + 1).min(heightmap.width.saturating_sub(1) as usize);
    let y1 = (y0 + 1).min(heightmap.height.saturating_sub(1) as usize);
    let fx = x - x0 as f64;
    let fy = y - y0 as f64;

    let idx00 = y0 * heightmap.width as usize + x0;
    let idx10 = y0 * heightmap.width as usize + x1;
    let idx01 = y1 * heightmap.width as usize + x0;
    let idx11 = y1 * heightmap.width as usize + x1;

    let h00 = *heightmap.data.get(idx00).unwrap_or(&0.0) as f64;
    let h10 = *heightmap.data.get(idx10).unwrap_or(&0.0) as f64;
    let h01 = *heightmap.data.get(idx01).unwrap_or(&0.0) as f64;
    let h11 = *heightmap.data.get(idx11).unwrap_or(&0.0) as f64;

    let h0 = h00 * (1.0 - fx) + h10 * fx;
    let h1 = h01 * (1.0 - fx) + h11 * fx;
    h0 * (1.0 - fy) + h1 * fy
}

fn calculate_slope_bilinear(heightmap: &MapRasterF32, x: f64, y: f64) -> SlopeSample {
    let delta = 1.0;
    let h_left = get_height_bilinear(heightmap, x - delta, y);
    let h_right = get_height_bilinear(heightmap, x + delta, y);
    let h_up = get_height_bilinear(heightmap, x, y - delta);
    let h_down = get_height_bilinear(heightmap, x, y + delta);
    let slope_x = (h_right - h_left) / (2.0 * delta);
    let slope_y = (h_down - h_up) / (2.0 * delta);
    let magnitude = (slope_x * slope_x + slope_y * slope_y).sqrt();
    SlopeSample {
        slope_x,
        slope_y,
        magnitude,
    }
}

fn to_u8(value: f64) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

fn generate_terrain_rgba(
    heightmap: &MapRasterF32,
    land_mask: &MapRasterU8,
    target_width: u32,
    target_height: u32,
    seed: f64,
) -> Vec<u8> {
    let mut rgba = vec![0u8; target_width as usize * target_height as usize * 4];
    let scale_x =
        (heightmap.width.saturating_sub(1)) as f64 / (target_width.saturating_sub(1).max(1)) as f64;
    let scale_y = (heightmap.height.saturating_sub(1)) as f64
        / (target_height.saturating_sub(1).max(1)) as f64;
    let mask_scale_x =
        (land_mask.width.saturating_sub(1)) as f64 / (target_width.saturating_sub(1).max(1)) as f64;
    let mask_scale_y = (land_mask.height.saturating_sub(1)) as f64
        / (target_height.saturating_sub(1).max(1)) as f64;

    let light_dir = (0.45f64, 0.55f64, 0.72f64);
    let light_magnitude =
        (light_dir.0 * light_dir.0 + light_dir.1 * light_dir.1 + light_dir.2 * light_dir.2).sqrt();

    for y in 0..target_height {
        for x in 0..target_width {
            let i = (y as usize * target_width as usize + x as usize) * 4;
            let src_x = x as f64 * scale_x;
            let src_y = (target_height - 1 - y) as f64 * scale_y;
            let height = get_height_bilinear(heightmap, src_x, src_y);
            let slope = calculate_slope_bilinear(heightmap, src_x, src_y);
            let far_slope =
                calculate_slope_bilinear(heightmap, src_x * 0.92 + 2.0, src_y * 0.92 + 2.0);

            let mask_x = x as f64 * mask_scale_x;
            let mask_y = (target_height - 1 - y) as f64 * mask_scale_y;
            let coast_average = sample_mask_average(land_mask, mask_x, mask_y, 3.0);
            let inland_average = sample_mask_average(land_mask, mask_x, mask_y, 18.0);
            let continental_average = sample_mask_average(land_mask, mask_x, mask_y, 36.0);
            let coastalness = clamp01(1.0 - (coast_average - 0.5).abs() * 2.0);
            let inland = clamp01((inland_average * 0.55 + continental_average * 0.45 - 0.35) * 1.5);
            let ridge = clamp01(slope.magnitude * 1.8 + far_slope.magnitude * 0.9 - 0.08);
            let latitude = 1.0 - y as f64 / target_height.saturating_sub(1).max(1) as f64;
            let warp_x = simple_noise(x as f64 * 0.012, y as f64 * 0.012, seed + 300.0) * 18.0;
            let warp_y = simple_noise(x as f64 * 0.012, y as f64 * 0.012, seed + 301.0) * 18.0;
            let biome_mask = clamp01(
                simple_noise(
                    (x as f64 + warp_x) * 0.018,
                    (y as f64 + warp_y) * 0.018,
                    seed + 330.0,
                ) * 0.45
                    + simple_noise(
                        (x as f64 + warp_x) * 0.055,
                        (y as f64 + warp_y) * 0.055,
                        seed + 360.0,
                    ) * 0.25
                    + 0.5,
            );
            let macro_variation = clamp01(
                simple_noise(x as f64 * 0.006, y as f64 * 0.006, seed + 410.0) * 0.5
                    + simple_noise(x as f64 * 0.014, y as f64 * 0.014, seed + 420.0) * 0.2
                    + 0.5,
            );

            let color =
                fantasy_terrain_color(height, coastalness, inland, ridge, latitude, biome_mask);
            let normal_x = -(slope.slope_x * 18.0 + far_slope.slope_x * 9.0);
            let normal_y = -(slope.slope_y * 18.0 + far_slope.slope_y * 9.0);
            let normal_z = 1.0;
            let normal_mag =
                (normal_x * normal_x + normal_y * normal_y + normal_z * normal_z).sqrt();
            let dot = (normal_x * light_dir.0 + normal_y * light_dir.1 + normal_z * light_dir.2)
                / (normal_mag * light_magnitude);
            let diffuse = (dot * 0.68 + 0.34).clamp(0.34, 1.0);
            let ridge_accent = 1.0 + ridge * 0.10;
            let valley_cool = 1.0 - (inland * 0.03 + slope.magnitude * 0.04).min(0.08);
            let noise1 = simple_noise(x as f64 * 0.07, y as f64 * 0.07, seed) * 10.0;
            let noise2 = simple_noise(x as f64 * 0.24, y as f64 * 0.24, seed + 100.0) * 5.0;
            let noise3 = simple_noise(x as f64 * 0.95, y as f64 * 0.95, seed + 200.0) * 2.0;
            let total_noise = noise1 + noise2 + noise3;
            let vignette = 1.0
                - (((x as f64 / target_width.max(1) as f64) - 0.5)
                    .hypot((y as f64 / target_height.max(1) as f64) - 0.5)
                    * 0.10)
                    .min(0.08);
            let shading_factor = diffuse * ridge_accent * valley_cool * vignette;
            let coast_glow = coastalness * smoothstep(0.02, 0.10, height) * (1.0 - inland * 0.5);
            let warm_shift = smoothstep(0.58, 0.92, macro_variation) * 0.10;
            let cool_shift = smoothstep(0.08, 0.34, macro_variation) * 0.08;

            rgba[i] = to_u8(
                color.r * shading_factor * (1.0 + warm_shift * 0.6) * (1.0 - cool_shift * 0.06)
                    + total_noise * 0.72
                    + coast_glow * 18.0,
            );
            rgba[i + 1] = to_u8(
                color.g * shading_factor * (1.0 + warm_shift * 0.24) * (1.0 + cool_shift * 0.06)
                    + total_noise * 0.64
                    + coast_glow * 20.0,
            );
            rgba[i + 2] = to_u8(
                color.b * shading_factor * (1.0 - warm_shift * 0.10) * (1.0 + cool_shift * 0.12)
                    + total_noise * 0.54,
            );
            rgba[i + 3] = 255;
        }
    }

    rgba
}

fn generate_ocean_rgba(target_width: u32, target_height: u32) -> Vec<u8> {
    let mut rgba = vec![0u8; target_width as usize * target_height as usize * 4];
    let seed = 913.37;

    let deep = TerrainColor {
        r: 16.0,
        g: 78.0,
        b: 146.0,
    };
    let mid = TerrainColor {
        r: 20.0,
        g: 126.0,
        b: 197.0,
    };
    let bright = TerrainColor {
        r: 84.0,
        g: 219.0,
        b: 235.0,
    };

    for y in 0..target_height {
        for x in 0..target_width {
            let i = (y as usize * target_width as usize + x as usize) * 4;
            let nx = x as f64 / target_width.saturating_sub(1).max(1) as f64;
            let ny = y as f64 / target_height.saturating_sub(1).max(1) as f64;
            let radial = (nx - 0.5).hypot(ny - 0.48);
            let wave = simple_noise(nx * 6.0, ny * 6.0, seed) * 0.5
                + simple_noise(nx * 20.0, ny * 20.0, seed + 20.0) * 0.25;
            let current =
                ((nx * 10.0 + ny * 6.0) * std::f64::consts::PI + wave * 4.0).sin() * 0.5 + 0.5;
            let depth = clamp01(0.25 + radial * 1.35 + wave * 0.18);

            let mut color = lerp_color(bright, mid, depth);
            color = lerp_color(color, deep, smoothstep(0.25, 0.95, depth));
            color = lerp_color(color, bright, current * (1.0 - depth) * 0.24);

            let mist = simple_noise(nx * 3.5, ny * 3.5, seed + 90.0) * 18.0
                + simple_noise(nx * 14.0, ny * 14.0, seed + 120.0) * 6.0;
            let vignette = 1.0 - (radial * 0.18).min(0.10);

            rgba[i] = to_u8(color.r * vignette + mist);
            rgba[i + 1] = to_u8(color.g * vignette + mist);
            rgba[i + 2] = to_u8(color.b * vignette + mist * 1.1);
            rgba[i + 3] = 255;
        }
    }

    rgba
}

fn generate_land_mask_rgba(
    land_mask: &MapRasterU8,
    target_width: u32,
    target_height: u32,
) -> Vec<u8> {
    let mut rgba = vec![0u8; target_width as usize * target_height as usize * 4];
    let scale_x = land_mask.width as f64 / target_width.max(1) as f64;
    let scale_y = land_mask.height as f64 / target_height.max(1) as f64;

    for y in 0..target_height {
        let src_y = (((target_height - 1 - y) as f64 * scale_y).floor() as u32)
            .min(land_mask.height.saturating_sub(1));
        for x in 0..target_width {
            let src_x =
                ((x as f64 * scale_x).floor() as u32).min(land_mask.width.saturating_sub(1));
            let idx = (y as usize * target_width as usize + x as usize) * 4;
            let mask_idx = src_y as usize * land_mask.width as usize + src_x as usize;
            let v = if land_mask.data.get(mask_idx).copied().unwrap_or(0) > 0 {
                255
            } else {
                0
            };
            rgba[idx] = v;
            rgba[idx + 1] = v;
            rgba[idx + 2] = v;
            rgba[idx + 3] = 255;
        }
    }

    rgba
}

fn generate_defs_start() -> &'static str {
    "<defs>\
      <filter id=\"terrain-shadow\" x=\"-50%\" y=\"-50%\" width=\"200%\" height=\"200%\">\
        <feGaussianBlur in=\"SourceAlpha\" stdDeviation=\"2\" result=\"blur\"/>\
        <feOffset in=\"blur\" dx=\"1\" dy=\"1\" result=\"offsetBlur\"/>\
        <feComposite in=\"SourceGraphic\" in2=\"offsetBlur\" operator=\"over\"/>\
      </filter>\
      <filter id=\"river-glow\">\
        <feGaussianBlur stdDeviation=\"1.8\" result=\"coloredBlur\"/>\
        <feMerge><feMergeNode in=\"coloredBlur\"/><feMergeNode in=\"SourceGraphic\"/></feMerge>\
      </filter>\
      <filter id=\"border-shadow\">\
        <feDropShadow dx=\"0.4\" dy=\"0.4\" stdDeviation=\"0.9\" flood-opacity=\"0.22\"/>\
      </filter>\
      <filter id=\"coast-line-glow\">\
        <feGaussianBlur stdDeviation=\"2.1\" result=\"blur\"/>\
        <feMerge><feMergeNode in=\"blur\"/><feMergeNode in=\"SourceGraphic\"/></feMerge>\
      </filter>"
}

fn append_svg_path(svg: &mut String, paths: &[Vec<f64>], width: f64, height: f64) {
    append_svg_path_with_options(svg, paths, width, height, false);
}

fn append_svg_closed_path(svg: &mut String, paths: &[Vec<f64>], width: f64, height: f64) {
    append_svg_path_with_options(svg, paths, width, height, true);
}

fn append_svg_path_with_options(
    svg: &mut String,
    paths: &[Vec<f64>],
    width: f64,
    height: f64,
    close_each: bool,
) {
    let mut wrote = false;
    for path in paths {
        if path.len() < 2 || path.len() % 2 != 0 {
            continue;
        }
        if !wrote {
            svg.push_str("<path d=\"");
            wrote = true;
        }
        for index in (0..path.len()).step_by(2) {
            let x = path[index] * width;
            let y = (1.0 - path[index + 1]) * height;
            if index == 0 {
                svg.push('M');
                write_compact_number(svg, x);
                svg.push(',');
                write_compact_number(svg, y);
            } else {
                svg.push('L');
                write_compact_number(svg, x);
                svg.push(',');
                write_compact_number(svg, y);
            }
        }
        if close_each {
            svg.push('Z');
        }
    }
    if wrote {
        svg.push_str("\"/>");
    }
}

fn append_svg_segments(svg: &mut String, segments: &[f64], width: f64, height: f64) {
    if segments.len() < 4 {
        return;
    }

    let mut wrote = false;
    for index in (0..segments.len()).step_by(4) {
        if index + 3 >= segments.len() {
            break;
        }
        if !wrote {
            svg.push_str("<path d=\"");
            wrote = true;
        }
        let x1 = segments[index] * width;
        let y1 = (1.0 - segments[index + 1]) * height;
        let x2 = segments[index + 2] * width;
        let y2 = (1.0 - segments[index + 3]) * height;
        svg.push('M');
        write_compact_number(svg, x1);
        svg.push(',');
        write_compact_number(svg, y1);
        svg.push('L');
        write_compact_number(svg, x2);
        svg.push(',');
        write_compact_number(svg, y2);
    }
    if wrote {
        svg.push_str("\"/>");
    }
}

fn append_city_markers(svg: &mut String, map_data: &MapData, width: f64, height: f64) {
    if map_data.city.is_empty() {
        return;
    }
    svg.push_str("<g>");
    svg.push_str("<g fill=\"#1c372a\" fill-opacity=\".86\" filter=\"url(#terrain-shadow)\">");
    for index in (0..map_data.city.len()).step_by(2) {
        if index + 1 >= map_data.city.len() {
            break;
        }
        let x = map_data.city[index] * width;
        let y = (1.0 - map_data.city[index + 1]) * height;
        append_circle(svg, x, y, 10.0);
    }
    svg.push_str("</g><g fill=\"#f9fadd\" fill-opacity=\".96\">");
    for index in (0..map_data.city.len()).step_by(2) {
        if index + 1 >= map_data.city.len() {
            break;
        }
        let x = map_data.city[index] * width;
        let y = (1.0 - map_data.city[index + 1]) * height;
        append_circle(svg, x, y, 5.0);
    }
    svg.push_str("</g></g>");
}

fn append_town_markers(svg: &mut String, map_data: &MapData, width: f64, height: f64) {
    if map_data.town.is_empty() {
        return;
    }
    svg.push_str("<g fill=\"#f2f8cc\" fill-opacity=\".92\" filter=\"url(#terrain-shadow)\">");
    for index in (0..map_data.town.len()).step_by(2) {
        if index + 1 >= map_data.town.len() {
            break;
        }
        let x = map_data.town[index] * width;
        let y = (1.0 - map_data.town[index + 1]) * height;
        append_circle(svg, x, y, 5.0);
    }
    svg.push_str("</g>");
}

fn append_labels(svg: &mut String, map_data: &MapData, width: f64, height: f64) {
    if map_data.label.is_empty() {
        return;
    }
    svg.push_str("<g text-anchor=\"start\" dominant-baseline=\"alphabetic\">");
    for label in &map_data.label {
        let x = label.position[0] * width;
        let y = (1.0 - label.position[1]) * height;
        let font_family = if label.fontface == "Times New Roman" {
            "serif"
        } else {
            label.fontface.as_str()
        };
        let safe_text = escape_xml(&label.text);
        write!(
            svg,
            "<g font-family=\"{}\" font-size=\"{}\"><text x=\"",
            font_family, label.fontsize,
        )
        .unwrap();
        write_compact_number(svg, x);
        svg.push_str("\" y=\"");
        write_compact_number(svg, y);
        write!(
            svg,
            "\" fill=\"#edf6f6\" fill-opacity=\".94\" stroke=\"#132b41\" stroke-opacity=\".68\" stroke-width=\"2.4\" stroke-linejoin=\"round\" paint-order=\"stroke fill\">{}</text><text x=\"",
            safe_text,
        )
        .unwrap();
        write_compact_number(svg, x);
        svg.push_str("\" y=\"");
        write_compact_number(svg, y);
        write!(
            svg,
            "\" fill=\"#ecf5f5\" fill-opacity=\".92\">{}</text></g>",
            safe_text
        )
        .unwrap();
    }
    svg.push_str("</g>");
}

fn append_circle(svg: &mut String, cx: f64, cy: f64, r: f64) {
    svg.push_str("<circle cx=\"");
    write_compact_number(svg, cx);
    svg.push_str("\" cy=\"");
    write_compact_number(svg, cy);
    svg.push_str("\" r=\"");
    write_compact_number(svg, r);
    svg.push_str("\"/>");
}

fn write_compact_number(svg: &mut String, value: f64) {
    let rounded = (value * 10.0).round() / 10.0;
    let mut text = format!("{rounded:.1}");
    if text.ends_with(".0") {
        text.truncate(text.len() - 2);
    }
    if text.starts_with("0.") {
        text.remove(0);
    } else if text.starts_with("-0.") {
        text.remove(1);
    }
    svg.push_str(&text);
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::{build_satellite_svg, build_satellite_svg_with_options};
    use serde_json::json;

    #[test]
    fn uses_vector_land_clip_and_renders_contour_and_slope_layers() {
        let map = json!({
            "image_width": 16,
            "image_height": 12,
            "contour": [[0.1, 0.1, 0.9, 0.1, 0.9, 0.8, 0.1, 0.8, 0.1, 0.1]],
            "slope": [0.2, 0.2, 0.35, 0.35],
            "land_polygons": [[0.1, 0.1, 0.9, 0.1, 0.9, 0.8, 0.1, 0.8]],
            "heightmap": {
                "width": 2,
                "height": 2,
                "data": [0.1, 0.4, 0.7, 0.9]
            },
            "land_mask": {
                "width": 2,
                "height": 2,
                "data": [0, 1, 1, 1]
            }
        });
        let layers = json!({
            "slope": true,
            "river": false,
            "contour": true,
            "border": false,
            "city": false,
            "town": false,
            "label": false
        });

        let svg = build_satellite_svg(&map.to_string(), &layers.to_string()).unwrap();

        assert!(svg.contains("clipPath id=\"land-clip\""));
        assert!(svg.contains("clip-path=\"url(#land-clip)\""));
        assert!(svg.contains("stroke=\"#14251a\""));
        assert!(svg.contains("stroke=\"#def6f4\""));
        assert!(svg.contains("data:image/jpeg;base64,"));
        assert!(!svg.contains("mask id=\"land-mask\""));
    }

    #[test]
    fn falls_back_to_raster_mask_when_land_polygons_are_missing() {
        let map = json!({
            "image_width": 8,
            "image_height": 8,
            "heightmap": {
                "width": 2,
                "height": 2,
                "data": [0.2, 0.3, 0.4, 0.5]
            },
            "land_mask": {
                "width": 2,
                "height": 2,
                "data": [0, 1, 1, 0]
            }
        });
        let layers = json!({
            "river": false,
            "border": false,
            "city": false,
            "town": false,
            "label": false
        });

        let svg = build_satellite_svg(&map.to_string(), &layers.to_string()).unwrap();

        assert!(svg.contains("mask id=\"land-mask\""));
        assert!(!svg.contains("clipPath id=\"land-clip\""));
    }

    #[test]
    fn smaller_embedded_images_produce_smaller_svg_payloads() {
        let map = json!({
            "image_width": 2400,
            "image_height": 1200,
            "heightmap": {
                "width": 4,
                "height": 4,
                "data": [
                    0.1, 0.2, 0.3, 0.4,
                    0.2, 0.3, 0.4, 0.5,
                    0.3, 0.4, 0.5, 0.6,
                    0.4, 0.5, 0.6, 0.7
                ]
            },
            "land_mask": {
                "width": 4,
                "height": 4,
                "data": [
                    0, 0, 1, 1,
                    0, 1, 1, 1,
                    0, 1, 1, 1,
                    0, 0, 1, 1
                ]
            }
        });
        let layers = json!({
            "river": false,
            "border": false,
            "city": false,
            "town": false,
            "label": false
        });

        let export_svg = build_satellite_svg(&map.to_string(), &layers.to_string()).unwrap();
        let preview_svg = build_satellite_svg_with_options(
            &map.to_string(),
            &layers.to_string(),
            Some(r#"{"max_embedded_image_size":512,"jpeg_quality":60}"#),
        )
        .unwrap();

        assert!(preview_svg.len() < export_svg.len());
    }
}
