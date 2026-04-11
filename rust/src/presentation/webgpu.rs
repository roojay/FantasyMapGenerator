use crate::presentation::{
    default_layer_metadata, PresentationOutputKind, PresentationPluginCapabilities,
    PresentationPluginMetadata, RenderDataPlugin,
};
use crate::MapDrawData;
use serde::Serialize;

const TERRAIN_ELEVATION_SCALE: f32 = 64.0;
const WATER_DEPTH_SCALE: f32 = 10.0;
const OVERLAY_HEIGHT_OFFSET: f32 = 1.2;
const LABEL_HEIGHT_OFFSET: f32 = 2.4;
const INTERACTIVE_TERRAIN_MAX_DIMENSION: u32 = 1024;
const INTERACTIVE_TERRAIN_MAX_TEXELS: u32 = 1_000_000;
const INTERACTIVE_TEXTURE_MAX_DIMENSION: u32 = 2048;
const INTERACTIVE_TEXTURE_MAX_TEXELS: u32 = 4_000_000;

#[derive(Clone, Debug, Serialize)]
pub struct WebGpuSceneMetadata {
    pub image_width: u32,
    pub image_height: u32,
    pub draw_scale: f64,
    pub terrain_width: u32,
    pub terrain_height: u32,
    pub texture_width: u32,
    pub texture_height: u32,
    pub elevation_scale: f32,
    pub city_count: u32,
    pub town_count: u32,
    pub river_count: u32,
    pub territory_count: u32,
    pub label_count: u32,
}

#[derive(Clone, Debug)]
pub struct WebGpuTextureSet {
    pub height: Vec<u8>,
    pub land_mask: Vec<u8>,
    pub flux: Vec<u8>,
    pub terrain_albedo: Vec<u8>,
    pub roughness: Vec<u8>,
    pub ao: Vec<u8>,
    pub water_color: Vec<u8>,
    pub water_alpha: Vec<u8>,
    pub coast_glow: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct WebGpuScenePacket {
    pub metadata: WebGpuSceneMetadata,
    pub terrain_positions: Vec<f32>,
    pub terrain_normals: Vec<f32>,
    pub terrain_uvs: Vec<f32>,
    pub terrain_indices: Vec<u32>,
    pub textures: WebGpuTextureSet,
    pub slope_segments: Vec<f32>,
    pub river_positions: Vec<f32>,
    pub river_offsets: Vec<u32>,
    pub contour_positions: Vec<f32>,
    pub contour_offsets: Vec<u32>,
    pub border_positions: Vec<f32>,
    pub border_offsets: Vec<u32>,
    pub city_positions: Vec<f32>,
    pub town_positions: Vec<f32>,
    pub label_bytes: Vec<u8>,
    pub label_offsets: Vec<u32>,
    pub label_anchors: Vec<f32>,
    pub label_sizes: Vec<f32>,
    pub land_polygon_positions: Vec<f32>,
    pub land_polygon_offsets: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WebGpuPresentationConfig;

#[derive(Default)]
pub struct WebGpuScenePlugin;

impl RenderDataPlugin for WebGpuScenePlugin {
    type Config = WebGpuPresentationConfig;
    type Output = WebGpuScenePacket;
    type Error = String;

    fn build(map_data: &MapDrawData, _config: &Self::Config) -> Result<Self::Output, Self::Error> {
        build_webgpu_scene_packet(map_data)
    }

    fn metadata() -> PresentationPluginMetadata {
        PresentationPluginMetadata {
            id: "webgpu_scene",
            display_name: "WebGPU Scene",
            description:
                "Structured terrain packet for GPU-first rendering with rich raster textures.",
            output_kind: PresentationOutputKind::GpuScenePacket,
            capabilities: PresentationPluginCapabilities {
                supports_layer_config: false,
                supports_direct_svg_export: false,
                requires_raster_data: true,
                requires_heightmap: true,
                requires_land_mask: true,
                embeds_raster_images: false,
            },
            supported_layers: default_layer_metadata(),
            config_sections: Vec::new(),
        }
    }
}

pub fn build_webgpu_scene_packet(map_data: &MapDrawData) -> Result<WebGpuScenePacket, String> {
    let heightmap = map_data
        .heightmap
        .as_ref()
        .ok_or_else(|| "render packet requires heightmap data".to_string())?;
    let land_mask = map_data
        .land_mask
        .as_ref()
        .ok_or_else(|| "render packet requires land_mask data".to_string())?;

    if heightmap.width != land_mask.width || heightmap.height != land_mask.height {
        return Err("heightmap and land_mask dimensions must match".to_string());
    }

    let source_terrain_width = heightmap.width;
    let source_terrain_height = heightmap.height;
    let flux_map = map_data.flux_map.as_ref();

    let top_down_height = to_top_down_f32(
        &heightmap.data,
        source_terrain_width,
        source_terrain_height,
    );
    let top_down_land = to_top_down_u8(
        &land_mask.data,
        source_terrain_width,
        source_terrain_height,
    );
    let top_down_flux = flux_map
        .map(|flux| to_top_down_f32(&flux.data, flux.width, flux.height))
        .unwrap_or_else(|| vec![0.0; (source_terrain_width * source_terrain_height) as usize]);

    // Mesh resolution: capped for geometry budget
    let (terrain_width, terrain_height) =
        clamp_interactive_terrain_size(source_terrain_width, source_terrain_height);

    // Texture resolution: higher cap for visual quality at 4K/8K
    let (texture_width, texture_height) =
        clamp_interactive_texture_size(source_terrain_width, source_terrain_height);

    // --- Mesh-resolution data (for geometry + elevation sampling) ---
    let mesh_height = if terrain_width == source_terrain_width
        && terrain_height == source_terrain_height
    {
        top_down_height.clone()
    } else {
        resample_scalar_grid(
            &top_down_height,
            source_terrain_width,
            source_terrain_height,
            terrain_width,
            terrain_height,
        )
    };
    let mesh_land = if terrain_width == source_terrain_width
        && terrain_height == source_terrain_height
    {
        top_down_land.clone()
    } else {
        resample_mask_grid(
            &top_down_land,
            source_terrain_width,
            source_terrain_height,
            terrain_width,
            terrain_height,
        )
    };
    let elevations = build_elevation_field(
        &mesh_height,
        &mesh_land,
        terrain_width,
        terrain_height,
    );

    let (terrain_positions, terrain_normals, terrain_uvs, terrain_indices) = build_terrain_mesh(
        map_data.image_width,
        map_data.image_height,
        terrain_width,
        terrain_height,
        &elevations,
    );

    // --- Texture-resolution data (for visual detail) ---
    let tex_height = if texture_width == source_terrain_width
        && texture_height == source_terrain_height
    {
        top_down_height
    } else {
        resample_scalar_grid(
            &top_down_height,
            source_terrain_width,
            source_terrain_height,
            texture_width,
            texture_height,
        )
    };
    let tex_land = if texture_width == source_terrain_width
        && texture_height == source_terrain_height
    {
        top_down_land
    } else {
        resample_mask_grid(
            &top_down_land,
            source_terrain_width,
            source_terrain_height,
            texture_width,
            texture_height,
        )
    };
    let tex_flux = if texture_width == source_terrain_width
        && texture_height == source_terrain_height
    {
        top_down_flux
    } else {
        resample_scalar_grid(
            &top_down_flux,
            source_terrain_width,
            source_terrain_height,
            texture_width,
            texture_height,
        )
    };

    let height_texture = encode_scalar_texture(&tex_height, texture_width, texture_height);
    let land_mask_texture = encode_mask_texture(&tex_land, texture_width, texture_height);
    let flux_texture = encode_scalar_texture(&tex_flux, texture_width, texture_height);
    let surface_textures = build_surface_texture_pack(
        &height_texture,
        &flux_texture,
        &land_mask_texture,
        texture_width,
        texture_height,
    );

    let metadata = WebGpuSceneMetadata {
        image_width: map_data.image_width,
        image_height: map_data.image_height,
        draw_scale: map_data.draw_scale,
        terrain_width,
        terrain_height,
        texture_width,
        texture_height,
        elevation_scale: TERRAIN_ELEVATION_SCALE,
        city_count: (map_data.city.len() / 2) as u32,
        town_count: (map_data.town.len() / 2) as u32,
        river_count: map_data.river.len() as u32,
        territory_count: map_data.territory.len() as u32,
        label_count: map_data.label.len() as u32,
    };

    let slope_segments = build_slope_segments(
        &map_data.slope,
        map_data.image_width,
        map_data.image_height,
        terrain_width,
        terrain_height,
        &elevations,
    );
    let (river_positions, river_offsets) = build_path_positions(
        &map_data.river,
        map_data.image_width,
        map_data.image_height,
        terrain_width,
        terrain_height,
        &elevations,
        OVERLAY_HEIGHT_OFFSET + 0.5,
    );
    let (contour_positions, contour_offsets) = build_path_positions(
        &map_data.contour,
        map_data.image_width,
        map_data.image_height,
        terrain_width,
        terrain_height,
        &elevations,
        OVERLAY_HEIGHT_OFFSET,
    );
    let (border_positions, border_offsets) = build_path_positions(
        &map_data.territory,
        map_data.image_width,
        map_data.image_height,
        terrain_width,
        terrain_height,
        &elevations,
        OVERLAY_HEIGHT_OFFSET + 0.25,
    );
    let city_positions = build_point_positions(
        &map_data.city,
        map_data.image_width,
        map_data.image_height,
        terrain_width,
        terrain_height,
        &elevations,
        LABEL_HEIGHT_OFFSET,
    );
    let town_positions = build_point_positions(
        &map_data.town,
        map_data.image_width,
        map_data.image_height,
        terrain_width,
        terrain_height,
        &elevations,
        LABEL_HEIGHT_OFFSET - 0.7,
    );
    let (label_bytes, label_offsets, label_anchors, label_sizes) =
        encode_labels(map_data, terrain_width, terrain_height, &elevations);
    let (land_polygon_positions, land_polygon_offsets) = encode_land_polygons(map_data);

    Ok(WebGpuScenePacket {
        metadata,
        terrain_positions,
        terrain_normals,
        terrain_uvs,
        terrain_indices,
        textures: WebGpuTextureSet {
            height: height_texture,
            land_mask: land_mask_texture,
            flux: flux_texture,
            terrain_albedo: surface_textures.terrain_albedo,
            roughness: surface_textures.roughness,
            ao: surface_textures.ao,
            water_color: surface_textures.water_color,
            water_alpha: surface_textures.water_alpha,
            coast_glow: surface_textures.coast_glow,
        },
        slope_segments,
        river_positions,
        river_offsets,
        contour_positions,
        contour_offsets,
        border_positions,
        border_offsets,
        city_positions,
        town_positions,
        label_bytes,
        label_offsets,
        label_anchors,
        label_sizes,
        land_polygon_positions,
        land_polygon_offsets,
    })
}

fn to_top_down_f32(data: &[f32], width: u32, height: u32) -> Vec<f32> {
    let width = width as usize;
    let height = height as usize;
    let mut output = vec![0.0; data.len()];
    for y in 0..height {
        let src_y = height.saturating_sub(1) - y;
        let dst_row = y * width;
        let src_row = src_y * width;
        output[dst_row..dst_row + width].copy_from_slice(&data[src_row..src_row + width]);
    }
    output
}

fn to_top_down_u8(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let width = width as usize;
    let height = height as usize;
    let mut output = vec![0; data.len()];
    for y in 0..height {
        let src_y = height.saturating_sub(1) - y;
        let dst_row = y * width;
        let src_row = src_y * width;
        output[dst_row..dst_row + width].copy_from_slice(&data[src_row..src_row + width]);
    }
    output
}

fn clamp_interactive_terrain_size(width: u32, height: u32) -> (u32, u32) {
    if width == 0 || height == 0 {
        return (width.max(1), height.max(1));
    }

    let texel_count = u64::from(width) * u64::from(height);
    if width <= INTERACTIVE_TERRAIN_MAX_DIMENSION
        && height <= INTERACTIVE_TERRAIN_MAX_DIMENSION
        && texel_count <= u64::from(INTERACTIVE_TERRAIN_MAX_TEXELS)
    {
        return (width, height);
    }

    let dimension_scale = f64::min(
        INTERACTIVE_TERRAIN_MAX_DIMENSION as f64 / width as f64,
        INTERACTIVE_TERRAIN_MAX_DIMENSION as f64 / height as f64,
    );
    let texel_scale =
        (INTERACTIVE_TERRAIN_MAX_TEXELS as f64 / texel_count as f64).sqrt();
    let scale = dimension_scale.min(texel_scale).min(1.0);

    (
        ((width as f64 * scale).round() as u32).max(1),
        ((height as f64 * scale).round() as u32).max(1),
    )
}

fn clamp_interactive_texture_size(width: u32, height: u32) -> (u32, u32) {
    if width == 0 || height == 0 {
        return (width.max(1), height.max(1));
    }

    let texel_count = u64::from(width) * u64::from(height);
    if width <= INTERACTIVE_TEXTURE_MAX_DIMENSION
        && height <= INTERACTIVE_TEXTURE_MAX_DIMENSION
        && texel_count <= u64::from(INTERACTIVE_TEXTURE_MAX_TEXELS)
    {
        return (width, height);
    }

    let dimension_scale = f64::min(
        INTERACTIVE_TEXTURE_MAX_DIMENSION as f64 / width as f64,
        INTERACTIVE_TEXTURE_MAX_DIMENSION as f64 / height as f64,
    );
    let texel_scale =
        (INTERACTIVE_TEXTURE_MAX_TEXELS as f64 / texel_count as f64).sqrt();
    let scale = dimension_scale.min(texel_scale).min(1.0);

    (
        ((width as f64 * scale).round() as u32).max(1),
        ((height as f64 * scale).round() as u32).max(1),
    )
}

fn sample_scalar_grid(data: &[f32], width: u32, height: u32, x: f32, y: f32) -> f32 {
    let width = width.max(1);
    let height = height.max(1);
    let sample_x = x.clamp(0.0, (width - 1) as f32);
    let sample_y = y.clamp(0.0, (height - 1) as f32);

    let x0 = sample_x.floor() as i32;
    let y0 = sample_y.floor() as i32;
    let x1 = (x0 + 1).min(width.saturating_sub(1) as i32);
    let y1 = (y0 + 1).min(height.saturating_sub(1) as i32);
    let tx = sample_x - x0 as f32;
    let ty = sample_y - y0 as f32;

    let h00 = sample_grid(data, width, height, x0, y0);
    let h10 = sample_grid(data, width, height, x1, y0);
    let h01 = sample_grid(data, width, height, x0, y1);
    let h11 = sample_grid(data, width, height, x1, y1);
    let hx0 = h00 + (h10 - h00) * tx;
    let hx1 = h01 + (h11 - h01) * tx;
    hx0 + (hx1 - hx0) * ty
}

fn resample_scalar_grid(
    data: &[f32],
    source_width: u32,
    source_height: u32,
    target_width: u32,
    target_height: u32,
) -> Vec<f32> {
    let mut output = vec![0.0; (target_width * target_height) as usize];

    for y in 0..target_height {
        let sample_y = if target_height > 1 {
            y as f32 * (source_height.saturating_sub(1)) as f32
                / (target_height.saturating_sub(1)) as f32
        } else {
            0.0
        };

        for x in 0..target_width {
            let sample_x = if target_width > 1 {
                x as f32 * (source_width.saturating_sub(1)) as f32
                    / (target_width.saturating_sub(1)) as f32
            } else {
                0.0
            };

            output[(y * target_width + x) as usize] =
                sample_scalar_grid(data, source_width, source_height, sample_x, sample_y);
        }
    }

    output
}

fn resample_mask_grid(
    data: &[u8],
    source_width: u32,
    source_height: u32,
    target_width: u32,
    target_height: u32,
) -> Vec<u8> {
    let mut output = vec![0; (target_width * target_height) as usize];

    for y in 0..target_height {
        let sample_y = if target_height > 1 {
            ((y as f32 * (source_height.saturating_sub(1)) as f32)
                / (target_height.saturating_sub(1)) as f32)
                .round() as i32
        } else {
            0
        };

        for x in 0..target_width {
            let sample_x = if target_width > 1 {
                ((x as f32 * (source_width.saturating_sub(1)) as f32)
                    / (target_width.saturating_sub(1)) as f32)
                    .round() as i32
            } else {
                0
            };

            let clamped_x = sample_x.clamp(0, source_width.saturating_sub(1) as i32) as usize;
            let clamped_y = sample_y.clamp(0, source_height.saturating_sub(1) as i32) as usize;
            output[(y * target_width + x) as usize] =
                data[clamped_y * source_width as usize + clamped_x];
        }
    }

    output
}

fn build_elevation_field(height: &[f32], land: &[u8], width: u32, height_px: u32) -> Vec<f32> {
    let mut elevations = vec![0.0; (width * height_px) as usize];
    for y in 0..height_px as usize {
        for x in 0..width as usize {
            let idx = y * width as usize + x;
            elevations[idx] = terrain_elevation(height[idx], land[idx] > 0);
        }
    }
    elevations
}

fn build_terrain_mesh(
    image_width: u32,
    image_height: u32,
    terrain_width: u32,
    terrain_height: u32,
    elevations: &[f32],
) -> (Vec<f32>, Vec<f32>, Vec<f32>, Vec<u32>) {
    let vertex_count = (terrain_width * terrain_height) as usize;
    let mut positions = Vec::with_capacity(vertex_count * 3);
    let mut normals = Vec::with_capacity(vertex_count * 3);
    let mut uvs = Vec::with_capacity(vertex_count * 2);
    let mut indices = Vec::with_capacity(((terrain_width - 1) * (terrain_height - 1) * 6) as usize);

    for y in 0..terrain_height {
        let v_top = if terrain_height > 1 {
            y as f32 / (terrain_height - 1) as f32
        } else {
            0.0
        };
        let world_z = (0.5 - v_top) * image_height as f32;
        for x in 0..terrain_width {
            let u = if terrain_width > 1 {
                x as f32 / (terrain_width - 1) as f32
            } else {
                0.0
            };
            let idx = (y * terrain_width + x) as usize;
            let world_x = (u - 0.5) * image_width as f32;
            positions.push(world_x);
            positions.push(elevations[idx]);
            positions.push(world_z);
            let normal = sample_normal(
                elevations,
                terrain_width,
                terrain_height,
                x as i32,
                y as i32,
                image_width,
                image_height,
            );
            normals.extend_from_slice(&normal);
            uvs.push(u);
            uvs.push(1.0 - v_top);
        }
    }

    if terrain_width > 1 && terrain_height > 1 {
        for y in 0..terrain_height - 1 {
            for x in 0..terrain_width - 1 {
                let i0 = y * terrain_width + x;
                let i1 = i0 + 1;
                let i2 = i0 + terrain_width;
                let i3 = i2 + 1;
                indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
            }
        }
    }

    (positions, normals, uvs, indices)
}

fn sample_normal(
    elevations: &[f32],
    width: u32,
    height: u32,
    x: i32,
    y: i32,
    image_width: u32,
    image_height: u32,
) -> [f32; 3] {
    let left = sample_grid(elevations, width, height, x - 1, y);
    let right = sample_grid(elevations, width, height, x + 1, y);
    let up = sample_grid(elevations, width, height, x, y - 1);
    let down = sample_grid(elevations, width, height, x, y + 1);
    let scale_x = (image_width as f32 / width.max(1) as f32).max(1.0);
    let scale_z = (image_height as f32 / height.max(1) as f32).max(1.0);
    let nx = (left - right) / scale_x;
    let nz = (down - up) / scale_z;
    normalize3(nx, 2.0, nz)
}

fn sample_grid(data: &[f32], width: u32, height: u32, x: i32, y: i32) -> f32 {
    let clamped_x = x.clamp(0, width.saturating_sub(1) as i32) as usize;
    let clamped_y = y.clamp(0, height.saturating_sub(1) as i32) as usize;
    data[clamped_y * width as usize + clamped_x]
}

fn normalize3(x: f32, y: f32, z: f32) -> [f32; 3] {
    let length = (x * x + y * y + z * z).sqrt().max(1e-6);
    [x / length, y / length, z / length]
}

fn terrain_elevation(height: f32, is_land: bool) -> f32 {
    if is_land {
        ((height.clamp(0.0, 1.0).powf(1.12) * 0.9) + 0.04) * TERRAIN_ELEVATION_SCALE
    } else {
        -WATER_DEPTH_SCALE + height.clamp(0.0, 1.0) * (WATER_DEPTH_SCALE * 0.35)
    }
}

fn encode_scalar_texture(data: &[f32], _width: u32, _height: u32) -> Vec<u8> {
    // PERF: Single-channel R8 encoding — 4× smaller than the previous RGBA path.
    // The JS side creates a Three.js DataTexture with `RedFormat` to match.
    data.iter()
        .map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8)
        .collect()
}

fn encode_mask_texture(data: &[u8], _width: u32, _height: u32) -> Vec<u8> {
    // PERF: Single-channel R8 encoding — 4× smaller than the previous RGBA path.
    data.iter().map(|v| if *v > 0 { 255 } else { 0 }).collect()
}

struct SurfaceTexturePack {
    terrain_albedo: Vec<u8>,
    roughness: Vec<u8>,
    ao: Vec<u8>,
    water_color: Vec<u8>,
    water_alpha: Vec<u8>,
    coast_glow: Vec<u8>,
}

fn build_surface_texture_pack(
    height_texture: &[u8],
    flux_texture: &[u8],
    land_mask_texture: &[u8],
    width: u32,
    height: u32,
) -> SurfaceTexturePack {
    let pixel_count = (width * height) as usize;
    let mut terrain_albedo = vec![0u8; pixel_count * 4];
    // PERF: Scalar textures use single-channel R8 (1 byte/pixel) instead of
    // RGBA (4 bytes/pixel). For 1024×1024 terrain this saves ~15 MB.
    let mut roughness = vec![0u8; pixel_count];
    let mut ao = vec![0u8; pixel_count];
    let mut water_color = vec![0u8; pixel_count * 4];
    let mut water_alpha = vec![0u8; pixel_count];
    let mut coast_glow = vec![0u8; pixel_count];

    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let rgba_idx = pixel_offset_rgba(width, x as usize, y as usize);
            let r8_idx = (y as usize) * (width as usize) + (x as usize);
            let u = if width > 1 {
                x as f32 / (width - 1) as f32
            } else {
                0.0
            };
            let v = if height > 1 {
                y as f32 / (height - 1) as f32
            } else {
                0.0
            };
            let height_value =
                sample_texture_channel_r8(height_texture, width, height, x, y).clamp(0.0, 1.0);
            let flux_value =
                sample_texture_channel_r8(flux_texture, width, height, x, y).clamp(0.0, 1.0);
            let is_land = sample_mask_r8(land_mask_texture, width, height, x, y);
            let coast = coastal_strength_r8(land_mask_texture, width, height, x, y);
            let left = sample_texture_channel_r8(height_texture, width, height, x - 1, y);
            let right = sample_texture_channel_r8(height_texture, width, height, x + 1, y);
            let up = sample_texture_channel_r8(height_texture, width, height, x, y - 1);
            let down = sample_texture_channel_r8(height_texture, width, height, x, y + 1);
            let relief = clamp01(((right - left).abs() + (down - up).abs()) * 3.4);
            let slope_dx = right - left;
            let slope_dy = down - up;
            let slope_mag = (slope_dx * slope_dx + slope_dy * slope_dy).sqrt();
            let ridge_strength = clamp01(relief * 1.35 + slope_mag * 1.8);
            let moisture =
                clamp01(flux_value * 0.65 + coast * 0.32 + (0.42 - height_value).max(0.0) * 0.18);
            let latitude_cool = 1.0 - ((v - 0.48).abs() * 1.7).clamp(0.0, 1.0);
            let latitude_warm = clamp01(v * 1.18 + (v - 0.58).max(0.0) * 0.55);
            let macro_noise = fbm2(u * 2.6 + 11.7, v * 2.4 + 3.9, 4);
            let forest_noise = fbm2(u * 10.5 + 19.3, v * 10.5 + 7.1, 5);
            let desert_noise = fbm2(u * 4.2 + 5.6, v * 3.7 + 17.2, 4);
            let snow_noise = fbm2(u * 8.1 + 41.7, v * 8.4 + 12.9, 4);
            let mountain_noise = fbm2(u * 6.4 + 2.1, v * 6.1 + 31.4, 4);
            let continentality = clamp01(
                0.48 + macro_noise * 0.28 + (u - 0.35).max(0.0) * 0.18 + (v - 0.55).max(0.0) * 0.16,
            );
            let dryness = clamp01(
                latitude_warm * 0.42
                    + continentality * 0.36
                    + (1.0 - moisture) * 0.52
                    + desert_noise * 0.18
                    - coast * 0.14,
            );
            let forest_amount = clamp01(
                (moisture - 0.22) * 1.15
                    + forest_noise * 0.38
                    + (0.68 - height_value).max(0.0) * 0.14
                    - dryness * 0.52
                    - ridge_strength * 0.12,
            );
            let desert_amount = clamp01(
                (dryness - 0.42) * 1.2
                    + (0.36 - moisture).max(0.0) * 0.55
                    + (0.55 - height_value).max(0.0) * 0.12,
            );
            let snowline = 0.74 - latitude_cool * 0.17 + macro_noise * 0.04
                - moisture * 0.03
                - snow_noise * 0.02;
            let snow_amount = clamp01(
                ((height_value - snowline) / 0.12) + ridge_strength * 0.28 + mountain_noise * 0.08
                    - latitude_warm * 0.08,
            );
            let aspect_light = clamp01(0.5 + (-slope_dx * 0.72 - slope_dy * 0.58) * 2.4);
            let cool_ridge = clamp01(
                ridge_strength
                    * ((height_value - 0.42).max(0.0) * 1.1 + latitude_cool * 0.32)
                    * (1.0 - aspect_light),
            );
            let warm_ridge = clamp01(
                ridge_strength
                    * ((height_value - 0.28).max(0.0) * 0.95 + latitude_warm * 0.22)
                    * aspect_light,
            );

            let mut color = if is_land {
                colorize_land(height_value, flux_value, coast)
            } else {
                colorize_water(height_value, coast)
            };

            if is_land {
                color = [
                    (apply_contrast(color[0] as f32 / 255.0, 1.08) * 255.0).round() as u8,
                    (apply_contrast(color[1] as f32 / 255.0, 1.10) * 255.0).round() as u8,
                    (apply_contrast(color[2] as f32 / 255.0, 1.06) * 255.0).round() as u8,
                ];
                color = lerp_color(
                    color,
                    [86, 110, 53],
                    clamp01((0.58 - (height_value - 0.34).abs()) * 0.58 + flux_value * 0.14),
                );
                color = lerp_color(
                    color,
                    [172, 152, 101],
                    clamp01((0.18 - height_value) * 3.1) * (1.0 - flux_value * 0.7),
                );
                color = lerp_color(
                    color,
                    [98, 96, 101],
                    clamp01(relief * 0.68 + (height_value - 0.58).max(0.0) * 0.6),
                );
                color = lerp_color(
                    color,
                    [242, 244, 247],
                    clamp01((height_value - 0.78) * 3.5 + relief * 0.2),
                );
                color = lerp_color(
                    color,
                    [201, 190, 145],
                    clamp01(coast * 0.18 + (0.1 - height_value).max(0.0) * 2.3),
                );
                color = lerp_color(color, [54, 83, 44], forest_amount * 0.72);
                color = lerp_color(color, [188, 169, 118], desert_amount * 0.78);
                color = lerp_color(color, [104, 114, 136], cool_ridge * 0.6);
                color = lerp_color(color, [164, 136, 94], warm_ridge * 0.42);
                color = lerp_color(color, [246, 248, 250], snow_amount * 0.92);
            } else {
                color = lerp_color(color, [18, 44, 68], 0.34);
            }

            let roughness_value = if is_land {
                clamp01(
                    0.94 - flux_value * 0.18
                        - forest_amount * 0.06
                        - snow_amount * 0.08
                        - warm_ridge * 0.04
                        + desert_amount * 0.06
                        + relief * 0.08,
                )
            } else {
                1.0
            };
            let ao_value = if is_land {
                clamp01(
                    0.92 - relief * 0.42
                        + flux_value * 0.06
                        + (height_value - 0.72).max(0.0) * 0.04
                        - forest_amount * 0.08
                        + cool_ridge * 0.06,
                )
            } else {
                clamp01(0.96 - coast * 0.08)
            };

            write_rgba(&mut terrain_albedo, rgba_idx, color[0], color[1], color[2], 255);

            roughness[r8_idx] = (roughness_value * 255.0).round() as u8;
            ao[r8_idx] = (ao_value * 255.0).round() as u8;

            let water_depth = clamp01(1.0 - height_value);
            let shallow_mix = clamp01(1.0 - water_depth * 1.28);
            let water_base = lerp_color(
                [9, 38, 69],
                [66, 158, 199],
                shallow_mix * 0.8 + coast * 0.18,
            );
            let water_tint = lerp_color(water_base, [138, 218, 242], coast * 0.26);
            let water_opacity = if is_land {
                0.0
            } else {
                clamp01(0.84 - coast * 0.22 + shallow_mix * 0.08)
            };
            let glow_opacity = if is_land {
                0.0
            } else {
                clamp01(coast * 0.78 + shallow_mix * 0.12)
            };

            write_rgba(
                &mut water_color,
                rgba_idx,
                water_tint[0],
                water_tint[1],
                water_tint[2],
                255,
            );

            water_alpha[r8_idx] = (water_opacity * 255.0).round() as u8;
            coast_glow[r8_idx] = (glow_opacity * 255.0).round() as u8;
        }
    }

    SurfaceTexturePack {
        terrain_albedo,
        roughness,
        ao,
        water_color,
        water_alpha,
        coast_glow,
    }
}

fn pixel_offset_rgba(width: u32, x: usize, y: usize) -> usize {
    (y * width as usize + x) * 4
}

fn write_rgba(texture: &mut [u8], offset: usize, r: u8, g: u8, b: u8, a: u8) {
    texture[offset] = r;
    texture[offset + 1] = g;
    texture[offset + 2] = b;
    texture[offset + 3] = a;
}

// --- R8 (single-channel) texture sampling ---------------------------------

fn sample_texture_channel_r8(texture: &[u8], width: u32, height: u32, x: i32, y: i32) -> f32 {
    let clamped_x = x.clamp(0, width.saturating_sub(1) as i32) as usize;
    let clamped_y = y.clamp(0, height.saturating_sub(1) as i32) as usize;
    let idx = clamped_y * width as usize + clamped_x;
    texture[idx] as f32 / 255.0
}

fn sample_mask_r8(texture: &[u8], width: u32, height: u32, x: i32, y: i32) -> bool {
    sample_texture_channel_r8(texture, width, height, x, y) > 0.5
}

fn coastal_strength_r8(mask: &[u8], width: u32, height: u32, x: i32, y: i32) -> f32 {
    let center: f32 = if sample_mask_r8(mask, width, height, x, y) {
        1.0
    } else {
        0.0
    };
    let mut delta = 0.0;
    for oy in -1..=1 {
        for ox in -1..=1 {
            if ox == 0 && oy == 0 {
                continue;
            }
            let sample: f32 = if sample_mask_r8(mask, width, height, x + ox, y + oy) {
                1.0
            } else {
                0.0
            };
            delta += (center - sample).abs();
        }
    }
    clamp01(delta / 8.0)
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn apply_contrast(value: f32, contrast: f32) -> f32 {
    clamp01((value - 0.5) * contrast + 0.5)
}

fn fbm2(x: f32, y: f32, octaves: usize) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 0.5;
    let mut frequency = 1.0;
    let mut total = 0.0;

    for _ in 0..octaves {
        value += value_noise_2d(x * frequency, y * frequency) * amplitude;
        total += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    if total <= 0.0 {
        0.0
    } else {
        value / total
    }
}

fn value_noise_2d(x: f32, y: f32) -> f32 {
    let x0 = x.floor();
    let y0 = y.floor();
    let tx = x - x0;
    let ty = y - y0;

    let v00 = hash2(x0, y0);
    let v10 = hash2(x0 + 1.0, y0);
    let v01 = hash2(x0, y0 + 1.0);
    let v11 = hash2(x0 + 1.0, y0 + 1.0);

    let sx = smoothstep01(tx);
    let sy = smoothstep01(ty);
    let ix0 = v00 + (v10 - v00) * sx;
    let ix1 = v01 + (v11 - v01) * sx;
    ix0 + (ix1 - ix0) * sy
}

fn smoothstep01(t: f32) -> f32 {
    let t = clamp01(t);
    t * t * (3.0 - 2.0 * t)
}

fn hash2(x: f32, y: f32) -> f32 {
    let v = (x * 127.1 + y * 311.7).sin() * 43_758.547;
    v.fract().abs()
}

fn coastal_strength(mask: &[u8], width: u32, height: u32, x: i32, y: i32) -> f32 {
    let center = sample_mask(mask, width, height, x, y);
    let mut delta = 0.0;
    for oy in -1..=1 {
        for ox in -1..=1 {
            if ox == 0 && oy == 0 {
                continue;
            }
            let sample = sample_mask(mask, width, height, x + ox, y + oy);
            delta += (center - sample).abs();
        }
    }
    (delta / 8.0).clamp(0.0, 1.0)
}

fn sample_mask(mask: &[u8], width: u32, height: u32, x: i32, y: i32) -> f32 {
    let clamped_x = x.clamp(0, width.saturating_sub(1) as i32) as usize;
    let clamped_y = y.clamp(0, height.saturating_sub(1) as i32) as usize;
    if mask[clamped_y * width as usize + clamped_x] > 0 {
        1.0
    } else {
        0.0
    }
}

fn colorize_land(height: f32, flux: f32, coast: f32) -> [u8; 3] {
    let base = if height < 0.08 {
        lerp_color([207, 192, 145], [181, 167, 113], height / 0.08)
    } else if height < 0.22 {
        lerp_color([182, 176, 131], [119, 144, 84], (height - 0.08) / 0.14)
    } else if height < 0.48 {
        lerp_color([97, 127, 76], [86, 103, 70], (height - 0.22) / 0.26)
    } else if height < 0.72 {
        lerp_color([112, 110, 90], [142, 138, 118], (height - 0.48) / 0.24)
    } else {
        lerp_color([186, 186, 180], [244, 243, 238], (height - 0.72) / 0.28)
    };
    let river_mix = (flux * 1.6).clamp(0.0, 0.8);
    let coast_mix = (coast * 0.35).clamp(0.0, 0.35);
    let river_tint = [84, 158, 204];
    let coast_tint = [224, 214, 176];
    lerp_color(
        lerp_color(base, river_tint, river_mix),
        coast_tint,
        coast_mix,
    )
}

fn colorize_water(height: f32, coast: f32) -> [u8; 3] {
    let depth = (1.0 - height).clamp(0.0, 1.0);
    let base = lerp_color([34, 82, 126], [10, 34, 74], depth * 0.9);
    let coast_tint = [76, 155, 187];
    lerp_color(base, coast_tint, (coast * 0.75).clamp(0.0, 0.75))
}

fn lerp_color(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        ((a[0] as f32) + (b[0] as f32 - a[0] as f32) * t).round() as u8,
        ((a[1] as f32) + (b[1] as f32 - a[1] as f32) * t).round() as u8,
        ((a[2] as f32) + (b[2] as f32 - a[2] as f32) * t).round() as u8,
    ]
}

fn build_slope_segments(
    data: &[f64],
    image_width: u32,
    image_height: u32,
    terrain_width: u32,
    terrain_height: u32,
    elevations: &[f32],
) -> Vec<f32> {
    let mut positions = Vec::with_capacity((data.len() / 4) * 6);
    for segment in data.chunks_exact(4) {
        let x1 = segment[0] as f32;
        let y1 = segment[1] as f32;
        let x2 = segment[2] as f32;
        let y2 = segment[3] as f32;
        append_world_position(
            &mut positions,
            x1,
            y1,
            image_width,
            image_height,
            terrain_width,
            terrain_height,
            elevations,
            OVERLAY_HEIGHT_OFFSET,
        );
        append_world_position(
            &mut positions,
            x2,
            y2,
            image_width,
            image_height,
            terrain_width,
            terrain_height,
            elevations,
            OVERLAY_HEIGHT_OFFSET,
        );
    }
    positions
}

fn build_path_positions(
    paths: &[Vec<f64>],
    image_width: u32,
    image_height: u32,
    terrain_width: u32,
    terrain_height: u32,
    elevations: &[f32],
    y_offset: f32,
) -> (Vec<f32>, Vec<u32>) {
    let mut positions = Vec::new();
    let mut offsets = Vec::with_capacity(paths.len() + 1);
    offsets.push(0);
    for path in paths {
        for point in path.chunks_exact(2) {
            append_world_position(
                &mut positions,
                point[0] as f32,
                point[1] as f32,
                image_width,
                image_height,
                terrain_width,
                terrain_height,
                elevations,
                y_offset,
            );
        }
        offsets.push((positions.len() / 3) as u32);
    }
    (positions, offsets)
}

fn build_point_positions(
    data: &[f64],
    image_width: u32,
    image_height: u32,
    terrain_width: u32,
    terrain_height: u32,
    elevations: &[f32],
    y_offset: f32,
) -> Vec<f32> {
    let mut positions = Vec::with_capacity((data.len() / 2) * 3);
    for point in data.chunks_exact(2) {
        append_world_position(
            &mut positions,
            point[0] as f32,
            point[1] as f32,
            image_width,
            image_height,
            terrain_width,
            terrain_height,
            elevations,
            y_offset,
        );
    }
    positions
}

fn encode_labels(
    draw_data: &MapDrawData,
    terrain_width: u32,
    terrain_height: u32,
    elevations: &[f32],
) -> (Vec<u8>, Vec<u32>, Vec<f32>, Vec<f32>) {
    let mut label_bytes = Vec::new();
    let mut label_offsets = Vec::with_capacity(draw_data.label.len() + 1);
    let mut label_anchors = Vec::with_capacity(draw_data.label.len() * 3);
    let mut label_sizes = Vec::with_capacity(draw_data.label.len());
    label_offsets.push(0);

    for label in &draw_data.label {
        label_bytes.extend_from_slice(label.text.as_bytes());
        label_offsets.push(label_bytes.len() as u32);
        append_world_position(
            &mut label_anchors,
            label.position[0] as f32,
            label.position[1] as f32,
            draw_data.image_width,
            draw_data.image_height,
            terrain_width,
            terrain_height,
            elevations,
            LABEL_HEIGHT_OFFSET,
        );
        label_sizes.push(label.fontsize as f32);
    }

    (label_bytes, label_offsets, label_anchors, label_sizes)
}

fn encode_land_polygons(draw_data: &MapDrawData) -> (Vec<f32>, Vec<u32>) {
    let polygons = draw_data
        .land_polygons
        .as_ref()
        .map_or(&[][..], |value| value.as_slice());
    let mut positions = Vec::new();
    let mut offsets = Vec::with_capacity(polygons.len() + 1);
    offsets.push(0);

    for polygon in polygons {
        if polygon.len() < 6 || polygon.len() % 2 != 0 {
            continue;
        }

        positions.extend(polygon.iter().map(|value| *value as f32));
        offsets.push(positions.len() as u32);
    }

    (positions, offsets)
}

fn append_world_position(
    output: &mut Vec<f32>,
    normalized_x: f32,
    normalized_y: f32,
    image_width: u32,
    image_height: u32,
    terrain_width: u32,
    terrain_height: u32,
    elevations: &[f32],
    y_offset: f32,
) {
    let world_x = (normalized_x - 0.5) * image_width as f32;
    let world_z = (normalized_y - 0.5) * image_height as f32;
    let terrain_y = sample_elevation(
        elevations,
        terrain_width,
        terrain_height,
        normalized_x,
        1.0 - normalized_y,
    );
    output.push(world_x);
    output.push(terrain_y + y_offset);
    output.push(world_z);
}

fn sample_elevation(
    elevations: &[f32],
    width: u32,
    height: u32,
    normalized_x: f32,
    normalized_y_top: f32,
) -> f32 {
    let width_f = width.max(1) as f32;
    let height_f = height.max(1) as f32;
    let sample_x = (normalized_x.clamp(0.0, 1.0) * (width_f - 1.0)).clamp(0.0, width_f - 1.0);
    let sample_y = (normalized_y_top.clamp(0.0, 1.0) * (height_f - 1.0)).clamp(0.0, height_f - 1.0);

    let x0 = sample_x.floor() as i32;
    let y0 = sample_y.floor() as i32;
    let x1 = (x0 + 1).min(width.saturating_sub(1) as i32);
    let y1 = (y0 + 1).min(height.saturating_sub(1) as i32);
    let tx = sample_x - x0 as f32;
    let ty = sample_y - y0 as f32;

    let h00 = sample_grid(elevations, width, height, x0, y0);
    let h10 = sample_grid(elevations, width, height, x1, y0);
    let h01 = sample_grid(elevations, width, height, x0, y1);
    let h11 = sample_grid(elevations, width, height, x1, y1);
    let hx0 = h00 + (h10 - h00) * tx;
    let hx1 = h01 + (h11 - h01) * tx;
    hx0 + (hx1 - hx0) * ty
}

#[cfg(test)]
mod tests {
    use super::{clamp_interactive_terrain_size, resample_scalar_grid};

    #[test]
    fn keeps_small_interactive_terrain_size_unchanged() {
        assert_eq!(clamp_interactive_terrain_size(960, 540), (960, 540));
    }

    #[test]
    fn clamps_large_interactive_terrain_size() {
        let (width, height) = clamp_interactive_terrain_size(1920, 1080);
        assert!(width < 1920);
        assert!(height < 1080);
        assert!(width <= 1024);
        assert!(height <= 1024);
        assert!(u64::from(width) * u64::from(height) <= 1_000_000);
    }

    #[test]
    fn resamples_scalar_grid_to_target_size() {
        let source = vec![0.0, 1.0, 2.0, 3.0];
        let sampled = resample_scalar_grid(&source, 2, 2, 4, 4);

        assert_eq!(sampled.len(), 16);
        assert!((sampled[0] - 0.0).abs() < 1e-6);
        assert!((sampled[15] - 3.0).abs() < 1e-6);
    }
}
