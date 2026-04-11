//! WASM 绑定模块
//!
//! 提供 JavaScript 可调用的 WASM 接口，用于在浏览器中生成地图。

use crate::map_generator::MapLabelDrawData;
use crate::presentation::webgpu::{WebGpuPresentationConfig, WebGpuScenePlugin};
use crate::presentation::RenderDataPlugin;
use crate::standard_svg;
use crate::{Extents2d, GlibcRand, MapDrawData, MapExportOptions, MapGenerator};
use serde::Serialize;
use std::sync::OnceLock;
use wasm_bindgen::prelude::*;

const TERRAIN_ELEVATION_SCALE: f32 = 64.0;
const WATER_DEPTH_SCALE: f32 = 10.0;
const OVERLAY_HEIGHT_OFFSET: f32 = 1.2;
const LABEL_HEIGHT_OFFSET: f32 = 2.4;
const INTERACTIVE_RASTER_MAX_DIMENSION: u32 = 2048;
const INTERACTIVE_RASTER_MAX_TEXELS: u32 = 4_000_000;

fn cached_city_name_buckets() -> &'static Vec<Vec<String>> {
    static CITY_NAME_BUCKETS: OnceLock<Vec<Vec<String>>> = OnceLock::new();

    CITY_NAME_BUCKETS.get_or_init(|| {
        let city_data = include_str!("citydata/countrycities.json");
        let json: serde_json::Value = serde_json::from_str(city_data).expect("valid JSON");
        json.as_object()
            .expect("city data should be an object")
            .values()
            .filter_map(|value| value.as_array())
            .map(|cities| {
                cities
                    .iter()
                    .filter_map(|value| value.as_str().map(str::to_owned))
                    .collect()
            })
            .collect()
    })
}

#[wasm_bindgen]
pub struct WasmRenderPacket {
    metadata_json: String,
    svg_json: String,
    terrain_positions: Vec<f32>,
    terrain_normals: Vec<f32>,
    terrain_uvs: Vec<f32>,
    terrain_indices: Vec<u32>,
    height_texture: Vec<u8>,
    land_mask_texture: Vec<u8>,
    flux_texture: Vec<u8>,
    terrain_albedo_texture: Vec<u8>,
    roughness_texture: Vec<u8>,
    ao_texture: Vec<u8>,
    water_color_texture: Vec<u8>,
    water_alpha_texture: Vec<u8>,
    coast_glow_texture: Vec<u8>,
    slope_segments: Vec<f32>,
    river_positions: Vec<f32>,
    river_offsets: Vec<u32>,
    contour_positions: Vec<f32>,
    contour_offsets: Vec<u32>,
    border_positions: Vec<f32>,
    border_offsets: Vec<u32>,
    city_positions: Vec<f32>,
    town_positions: Vec<f32>,
    label_bytes: Vec<u8>,
    label_offsets: Vec<u32>,
    label_anchors: Vec<f32>,
    label_sizes: Vec<f32>,
    land_polygon_positions: Vec<f32>,
    land_polygon_offsets: Vec<u32>,
}

#[wasm_bindgen]
impl WasmRenderPacket {
    // PERF: All getters use `std::mem::take` instead of `.clone()` to avoid
    // redundant heap allocation + memcpy. The caller (JS worker) consumes each
    // buffer exactly once and then calls `.free()`, so moving data out is safe
    // and eliminates ~80 MB of needless copies per generation cycle.

    #[wasm_bindgen(getter)]
    pub fn metadata_json(&mut self) -> String {
        std::mem::take(&mut self.metadata_json)
    }

    #[wasm_bindgen(getter)]
    pub fn svg_json(&mut self) -> String {
        std::mem::take(&mut self.svg_json)
    }

    pub fn terrain_positions(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.terrain_positions)
    }

    pub fn terrain_normals(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.terrain_normals)
    }

    pub fn terrain_uvs(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.terrain_uvs)
    }

    pub fn terrain_indices(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.terrain_indices)
    }

    pub fn height_texture(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.height_texture)
    }

    pub fn terrain_albedo_texture(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.terrain_albedo_texture)
    }

    pub fn roughness_texture(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.roughness_texture)
    }

    pub fn ao_texture(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.ao_texture)
    }

    pub fn water_color_texture(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.water_color_texture)
    }

    pub fn water_alpha_texture(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.water_alpha_texture)
    }

    pub fn coast_glow_texture(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.coast_glow_texture)
    }

    pub fn slope_segments(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.slope_segments)
    }

    pub fn river_positions(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.river_positions)
    }

    pub fn river_offsets(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.river_offsets)
    }

    pub fn contour_positions(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.contour_positions)
    }

    pub fn contour_offsets(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.contour_offsets)
    }

    pub fn border_positions(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.border_positions)
    }

    pub fn border_offsets(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.border_offsets)
    }

    pub fn city_positions(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.city_positions)
    }

    pub fn town_positions(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.town_positions)
    }

    pub fn label_bytes(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.label_bytes)
    }

    pub fn label_offsets(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.label_offsets)
    }

    pub fn label_anchors(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.label_anchors)
    }

    pub fn label_sizes(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.label_sizes)
    }

    pub fn land_polygon_positions(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.land_polygon_positions)
    }

    pub fn land_polygon_offsets(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.land_polygon_offsets)
    }
}

/// 设置 panic hook，在浏览器控制台显示 Rust panic 信息
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// WASM 地图生成器包装器
#[wasm_bindgen]
pub struct WasmMapGenerator {
    generator: MapGenerator,
    seed: u32,
}

#[wasm_bindgen]
impl WasmMapGenerator {
    /// 创建新的地图生成器
    ///
    /// # 参数
    /// - `seed`: 随机种子（0 表示使用时间戳）
    /// - `width`: 地图宽度（像素）
    /// - `height`: 地图高度（像素）
    /// - `resolution`: 网格分辨率（0.01-0.2，推荐 0.08）
    #[wasm_bindgen(constructor)]
    pub fn new(
        seed: u32,
        width: u32,
        height: u32,
        resolution: f64,
    ) -> Result<WasmMapGenerator, JsValue> {
        // 使用种子或时间戳
        let actual_seed = if seed == 0 {
            (js_sys::Date::now() as u32) % 1000000
        } else {
            seed
        };

        // 创建地图范围
        let default_extents_height = 20.0;
        let aspect_ratio = width as f64 / height as f64;
        let extents_width = aspect_ratio * default_extents_height;
        let extents = Extents2d::new(0.0, 0.0, extents_width, default_extents_height);

        // 创建随机数生成器
        let mut rng = GlibcRand::new(actual_seed);

        // 预热随机数生成器（与 CLI 保持一致）
        for _ in 0..1000 {
            rng.rand();
        }

        // 创建地图生成器
        let generator = MapGenerator::new(extents, resolution, width, height, rng);

        Ok(WasmMapGenerator {
            generator,
            seed: actual_seed,
        })
    }

    /// 生成完整的地图
    ///
    /// 返回包含地图数据的 JSON 字符串
    #[wasm_bindgen]
    pub fn generate(&mut self, num_cities: i32, num_towns: i32) -> Result<String, JsValue> {
        self.generate_with_options(num_cities, num_towns, true)
    }

    #[wasm_bindgen]
    pub fn generate_render_packet(
        &mut self,
        num_cities: i32,
        num_towns: i32,
    ) -> Result<WasmRenderPacket, JsValue> {
        let draw_data = self.generate_draw_data(
            num_cities,
            num_towns,
            MapExportOptions {
                include_raster_data: true,
                max_raster_dimension: Some(INTERACTIVE_RASTER_MAX_DIMENSION),
                max_raster_texels: Some(INTERACTIVE_RASTER_MAX_TEXELS),
            },
        )?;
        build_render_packet(&draw_data)
    }

    /// 生成完整地图，并允许网页端按需决定是否导出附加栅格数据。
    ///
    /// # 参数
    /// * `num_cities` - 城市数量
    /// * `num_towns` - 城镇数量
    /// * `include_raster_data` - 是否导出附加栅格数据
    ///
    /// # 与原始 C++ 的差异
    /// 原始 C++ 版本没有 WASM 导出层，也没有“按需导出栅格”的调用入口。
    /// 这个接口是本 fork 为浏览器场景新增的能力，用来减少 WASM 与 JS 之间
    /// 的大块 JSON 传输。
    ///
    /// # 性能说明
    /// 普通矢量渲染可将 `include_raster_data` 设为 `false`，
    /// 仅在需要额外栅格数据时再开启。
    #[wasm_bindgen]
    pub fn generate_with_options(
        &mut self,
        num_cities: i32,
        num_towns: i32,
        include_raster_data: bool,
    ) -> Result<String, JsValue> {
        let draw_data = self.generate_draw_data(
            num_cities,
            num_towns,
            MapExportOptions {
                include_raster_data,
                max_raster_dimension: None,
                max_raster_texels: None,
            },
        )?;
        serde_json::to_string(&draw_data).map_err(|err| JsValue::from_str(&err.to_string()))
    }

    /// 仅生成地形（不包括城市和边界）
    #[wasm_bindgen]
    pub fn generate_terrain_only(&mut self) -> Result<String, JsValue> {
        // 初始化网格
        self.generator.initialize();

        // 生成地形
        self.initialize_heightmap();

        // 侵蚀地形
        self.erode(0.25, 5);

        // 导出为 JSON
        let draw_data = self.generator.collect_draw_data();
        serde_json::to_string(&draw_data).map_err(|err| JsValue::from_str(&err.to_string()))
    }

    /// 获取当前使用的种子
    #[wasm_bindgen]
    pub fn get_seed(&self) -> u32 {
        self.seed
    }

    /// 设置绘制缩放比例
    #[wasm_bindgen]
    pub fn set_draw_scale(&mut self, scale: f64) {
        self.generator.set_draw_scale(scale);
    }

    // 内部辅助方法

    /// 初始化地形高度图
    fn initialize_heightmap(&mut self) {
        let pad = 5.0;
        let extents = self.generator.get_extents();

        let expanded = Extents2d::new(
            extents.minx - pad,
            extents.miny - pad,
            extents.maxx + pad,
            extents.maxy + pad,
        );

        // 随机放置山丘和圆锥
        let n = self.generator.rng_mut().random_double(100.0, 250.0) as i32;
        for _ in 0..n {
            let _px_discard = self
                .generator
                .rng_mut()
                .random_double(expanded.minx, expanded.maxx);
            let px = self
                .generator
                .rng_mut()
                .random_double(expanded.minx, expanded.maxx);
            let py = self
                .generator
                .rng_mut()
                .random_double(expanded.miny, expanded.maxy);
            let r = self.generator.rng_mut().random_double(1.0, 8.0);
            let strength = self.generator.rng_mut().random_double(0.5, 1.5);

            if self.generator.rng_mut().random_double(0.0, 1.0) > 0.5 {
                self.generator.add_hill(px, py, r, strength);
            } else {
                self.generator.add_cone(px, py, r, strength);
            }
        }

        // 可能添加大型圆锥
        if self.generator.rng_mut().random_double(0.0, 1.0) > 0.5 {
            let _px_discard = self
                .generator
                .rng_mut()
                .random_double(expanded.minx, expanded.maxx);
            let px = self
                .generator
                .rng_mut()
                .random_double(expanded.minx, expanded.maxx);
            let py = self
                .generator
                .rng_mut()
                .random_double(expanded.miny, expanded.maxy);
            let r = self.generator.rng_mut().random_double(6.0, 12.0);
            let strength = self.generator.rng_mut().random_double(1.0, 3.0);
            self.generator.add_cone(px, py, r, strength);
        }

        // 可能添加斜坡
        if self.generator.rng_mut().random_double(0.0, 1.0) > 0.1 {
            let angle = self
                .generator
                .rng_mut()
                .random_double(0.0, 2.0 * std::f64::consts::PI);
            let dir_x = angle.sin();
            let dir_y = angle.cos();
            let _lx_discard = self
                .generator
                .rng_mut()
                .random_double(extents.minx, extents.maxx);
            let lx = self
                .generator
                .rng_mut()
                .random_double(extents.minx, extents.maxx);
            let ly = self
                .generator
                .rng_mut()
                .random_double(extents.miny, extents.maxy);
            let slope_width = self.generator.rng_mut().random_double(0.5, 5.0);
            let strength = self.generator.rng_mut().random_double(2.0, 3.0);
            self.generator
                .add_slope(lx, ly, dir_x, dir_y, slope_width, strength);
        }

        // 归一化或圆滑
        if self.generator.rng_mut().random_double(0.0, 1.0) > 0.5 {
            self.generator.normalize();
        } else {
            self.generator.round();
        }

        // 可能松弛
        if self.generator.rng_mut().random_double(0.0, 1.0) > 0.5 {
            self.generator.relax();
        }
    }

    /// 侵蚀地形
    fn erode(&mut self, amount: f64, iterations: i32) {
        for _ in 0..iterations {
            self.generator.erode(amount / iterations as f64);
        }
        self.generator.set_sea_level_to_median();
    }

    /// 获取城市名称
    fn get_label_names(&mut self, num: usize) -> Vec<String> {
        let city_buckets = cached_city_name_buckets();
        let mut cities: Vec<String> = Vec::new();

        while cities.len() < num {
            let rand_idx = self.generator.rng_mut().rand() as usize % city_buckets.len();
            for city in &city_buckets[rand_idx] {
                cities.push(city.clone());
            }
        }

        // Fisher-Yates 洗牌
        for i in (0..cities.len().saturating_sub(1)).rev() {
            let j = self.generator.rng_mut().rand() as usize % (i + 1);
            cities.swap(i, j);
        }

        cities.truncate(num);
        cities
    }

    fn generate_draw_data(
        &mut self,
        num_cities: i32,
        num_towns: i32,
        export_options: MapExportOptions,
    ) -> Result<MapDrawData, JsValue> {
        // PERF: 将栅格导出显式化，避免普通网页路径总是携带大型 height/flux/mask 数组。
        self.generator.initialize();
        self.initialize_heightmap();
        self.erode(0.25, 5);

        let label_names = self.get_label_names((2 * num_cities + num_towns) as usize);
        let mut label_idx = label_names.len();

        for _ in 0..num_cities {
            if label_idx >= 2 {
                label_idx -= 2;
                let city_name = label_names[label_idx + 1].clone();
                let territory_name = label_names[label_idx].to_uppercase();
                self.generator.add_city(city_name, territory_name);
            }
        }

        for _ in 0..num_towns {
            if label_idx >= 1 {
                label_idx -= 1;
                let town_name = label_names[label_idx].clone();
                self.generator.add_town(town_name);
            }
        }

        Ok(self
            .generator
            .collect_draw_data_with_options(export_options))
    }
}

/// Borrow-based view of `MapDrawData` for SVG JSON serialization.
/// Avoids deep-cloning all vector fields (~several MB) by borrowing from
/// the original draw data and excluding raster-only fields.
#[derive(Serialize)]
struct SvgDrawDataRef<'a> {
    image_width: u32,
    image_height: u32,
    draw_scale: f64,
    contour: &'a [Vec<f64>],
    river: &'a [Vec<f64>],
    slope: &'a [f64],
    city: &'a [f64],
    town: &'a [f64],
    territory: &'a [Vec<f64>],
    label: &'a [MapLabelDrawData],
}

fn build_render_packet(draw_data: &MapDrawData) -> Result<WasmRenderPacket, JsValue> {
    // PERF: Use a borrow-based reference struct instead of cloning all vector
    // fields. The previous code cloned contour, river, slope, city, town,
    // territory and label just to set the raster fields to None.
    let svg_ref = SvgDrawDataRef {
        image_width: draw_data.image_width,
        image_height: draw_data.image_height,
        draw_scale: draw_data.draw_scale,
        contour: &draw_data.contour,
        river: &draw_data.river,
        slope: &draw_data.slope,
        city: &draw_data.city,
        town: &draw_data.town,
        territory: &draw_data.territory,
        label: &draw_data.label,
    };
    let svg_json = serde_json::to_string(&svg_ref)
        .map_err(|err| JsValue::from_str(&err.to_string()))?;
    let packet = WebGpuScenePlugin::build(draw_data, &WebGpuPresentationConfig::default())
        .map_err(|err| JsValue::from_str(&err))?;
    let metadata_json = serde_json::to_string(&packet.metadata)
        .map_err(|err| JsValue::from_str(&err.to_string()))?;

    Ok(WasmRenderPacket {
        metadata_json,
        svg_json,
        terrain_positions: packet.terrain_positions,
        terrain_normals: packet.terrain_normals,
        terrain_uvs: packet.terrain_uvs,
        terrain_indices: packet.terrain_indices,
        height_texture: packet.textures.height,
        land_mask_texture: packet.textures.land_mask,
        flux_texture: packet.textures.flux,
        terrain_albedo_texture: packet.textures.terrain_albedo,
        roughness_texture: packet.textures.roughness,
        ao_texture: packet.textures.ao,
        water_color_texture: packet.textures.water_color,
        water_alpha_texture: packet.textures.water_alpha,
        coast_glow_texture: packet.textures.coast_glow,
        slope_segments: packet.slope_segments,
        river_positions: packet.river_positions,
        river_offsets: packet.river_offsets,
        contour_positions: packet.contour_positions,
        contour_offsets: packet.contour_offsets,
        border_positions: packet.border_positions,
        border_offsets: packet.border_offsets,
        city_positions: packet.city_positions,
        town_positions: packet.town_positions,
        label_bytes: packet.label_bytes,
        label_offsets: packet.label_offsets,
        label_anchors: packet.label_anchors,
        label_sizes: packet.label_sizes,
        land_polygon_positions: packet.land_polygon_positions,
        land_polygon_offsets: packet.land_polygon_offsets,
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

/// 简化的地图生成函数（用于快速测试）
#[wasm_bindgen]
pub fn generate_map_simple(seed: u32, width: u32, height: u32) -> Result<String, JsValue> {
    let mut generator = WasmMapGenerator::new(seed, width, height, 0.08)?;
    generator.generate(5, 10)
}

/// 根据导出的地图 JSON 和图层配置在 Rust 侧生成标准原始地图 SVG。
#[wasm_bindgen]
pub fn build_map_svg(map_json: &str, layers_json: &str) -> Result<String, JsValue> {
    standard_svg::build_map_svg(map_json, layers_json).map_err(|err| JsValue::from_str(&err))
}

/// 返回 presentation 插件的统一 capability/config metadata。
#[wasm_bindgen]
pub fn presentation_plugin_metadata_json() -> Result<String, JsValue> {
    serde_json::to_string(&crate::presentation::presentation_plugin_metadata())
        .map_err(|err| JsValue::from_str(&err.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{presentation_plugin_metadata_json, WasmMapGenerator};

    #[test]
    fn render_packet_contains_geometry_and_texture_buffers() {
        let mut generator = WasmMapGenerator::new(12345, 512, 288, 0.08).unwrap();
        generator.set_draw_scale(1.0);

        let packet = generator.generate_render_packet(5, 10).unwrap();
        let metadata: serde_json::Value = serde_json::from_str(&packet.metadata_json).unwrap();

        assert_eq!(metadata["image_width"].as_u64(), Some(512));
        assert_eq!(metadata["image_height"].as_u64(), Some(288));
        assert!(packet.terrain_positions.len() > 0);
        assert_eq!(packet.terrain_positions.len(), packet.terrain_normals.len());
        assert_eq!(
            packet.terrain_uvs.len() / 2,
            packet.terrain_positions.len() / 3
        );
        assert!(packet.terrain_indices.len() > 0);
        assert_eq!(packet.height_texture.len(), packet.land_mask_texture.len());
        assert_eq!(packet.height_texture.len(), packet.flux_texture.len());
        assert_eq!(
            packet.height_texture.len(),
            packet.terrain_albedo_texture.len()
        );
        assert_eq!(packet.height_texture.len(), packet.roughness_texture.len());
        assert_eq!(packet.height_texture.len(), packet.ao_texture.len());
        assert_eq!(
            packet.height_texture.len(),
            packet.water_color_texture.len()
        );
        assert_eq!(
            packet.height_texture.len(),
            packet.water_alpha_texture.len()
        );
        assert_eq!(packet.height_texture.len(), packet.coast_glow_texture.len());
        assert!(packet.label_offsets.len() > 1);
        assert_eq!(packet.label_offsets.len() - 1, packet.label_sizes.len());
        assert!(!packet.land_polygon_offsets.is_empty());
        assert_eq!(
            packet.land_polygon_positions.len() % 2,
            0,
            "land polygon positions should be xy pairs"
        );
    }

    #[test]
    fn plugin_metadata_json_contains_expected_plugins() {
        let metadata_json = presentation_plugin_metadata_json().unwrap();
        let metadata: serde_json::Value = serde_json::from_str(&metadata_json).unwrap();
        let plugins = metadata.as_array().expect("metadata should be an array");

        assert!(plugins.iter().any(|plugin| plugin["id"] == "standard_svg"));
        assert!(plugins.iter().any(|plugin| plugin["id"] == "webgpu_scene"));
    }
}
