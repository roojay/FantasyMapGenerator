//! WASM 绑定模块
//!
//! 提供 JavaScript 可调用的 WASM 接口，用于在浏览器中生成地图。

use crate::presentation::webgpu::{WebGpuPresentationConfig, WebGpuScenePlugin};
use crate::presentation::RenderDataPlugin;
use crate::standard_svg;
use crate::{Extents2d, GlibcRand, MapDrawData, MapExportOptions, MapGenerator};
use std::sync::OnceLock;
use wasm_bindgen::prelude::*;

const TERRAIN_ELEVATION_SCALE: f32 = 64.0;
const WATER_DEPTH_SCALE: f32 = 10.0;
const OVERLAY_HEIGHT_OFFSET: f32 = 1.2;
const LABEL_HEIGHT_OFFSET: f32 = 2.4;

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
    #[wasm_bindgen(getter)]
    pub fn metadata_json(&self) -> String {
        self.metadata_json.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn svg_json(&self) -> String {
        self.svg_json.clone()
    }

    pub fn terrain_positions(&self) -> Vec<f32> {
        self.terrain_positions.clone()
    }

    pub fn terrain_normals(&self) -> Vec<f32> {
        self.terrain_normals.clone()
    }

    pub fn terrain_uvs(&self) -> Vec<f32> {
        self.terrain_uvs.clone()
    }

    pub fn terrain_indices(&self) -> Vec<u32> {
        self.terrain_indices.clone()
    }

    pub fn height_texture(&self) -> Vec<u8> {
        self.height_texture.clone()
    }

    pub fn land_mask_texture(&self) -> Vec<u8> {
        self.land_mask_texture.clone()
    }

    pub fn flux_texture(&self) -> Vec<u8> {
        self.flux_texture.clone()
    }

    pub fn terrain_albedo_texture(&self) -> Vec<u8> {
        self.terrain_albedo_texture.clone()
    }

    pub fn roughness_texture(&self) -> Vec<u8> {
        self.roughness_texture.clone()
    }

    pub fn ao_texture(&self) -> Vec<u8> {
        self.ao_texture.clone()
    }

    pub fn water_color_texture(&self) -> Vec<u8> {
        self.water_color_texture.clone()
    }

    pub fn water_alpha_texture(&self) -> Vec<u8> {
        self.water_alpha_texture.clone()
    }

    pub fn coast_glow_texture(&self) -> Vec<u8> {
        self.coast_glow_texture.clone()
    }

    pub fn slope_segments(&self) -> Vec<f32> {
        self.slope_segments.clone()
    }

    pub fn river_positions(&self) -> Vec<f32> {
        self.river_positions.clone()
    }

    pub fn river_offsets(&self) -> Vec<u32> {
        self.river_offsets.clone()
    }

    pub fn contour_positions(&self) -> Vec<f32> {
        self.contour_positions.clone()
    }

    pub fn contour_offsets(&self) -> Vec<u32> {
        self.contour_offsets.clone()
    }

    pub fn border_positions(&self) -> Vec<f32> {
        self.border_positions.clone()
    }

    pub fn border_offsets(&self) -> Vec<u32> {
        self.border_offsets.clone()
    }

    pub fn city_positions(&self) -> Vec<f32> {
        self.city_positions.clone()
    }

    pub fn town_positions(&self) -> Vec<f32> {
        self.town_positions.clone()
    }

    pub fn label_bytes(&self) -> Vec<u8> {
        self.label_bytes.clone()
    }

    pub fn label_offsets(&self) -> Vec<u32> {
        self.label_offsets.clone()
    }

    pub fn label_anchors(&self) -> Vec<f32> {
        self.label_anchors.clone()
    }

    pub fn label_sizes(&self) -> Vec<f32> {
        self.label_sizes.clone()
    }

    pub fn land_polygon_positions(&self) -> Vec<f32> {
        self.land_polygon_positions.clone()
    }

    pub fn land_polygon_offsets(&self) -> Vec<u32> {
        self.land_polygon_offsets.clone()
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
        let draw_data = self.generate_draw_data(num_cities, num_towns, true)?;
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
        let draw_data = self.generate_draw_data(num_cities, num_towns, include_raster_data)?;
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
        include_raster_data: bool,
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
            .collect_draw_data_with_options(MapExportOptions {
                include_raster_data,
            }))
    }
}

fn build_render_packet(draw_data: &MapDrawData) -> Result<WasmRenderPacket, JsValue> {
    let svg_draw_data = MapDrawData {
        image_width: draw_data.image_width,
        image_height: draw_data.image_height,
        draw_scale: draw_data.draw_scale,
        contour: draw_data.contour.clone(),
        river: draw_data.river.clone(),
        slope: draw_data.slope.clone(),
        city: draw_data.city.clone(),
        town: draw_data.town.clone(),
        territory: draw_data.territory.clone(),
        label: draw_data.label.clone(),
        heightmap: None,
        flux_map: None,
        land_mask: None,
        land_polygons: None,
    };
    let svg_json = serde_json::to_string(&svg_draw_data)
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

fn encode_scalar_texture(data: &[f32], width: u32, height: u32) -> Vec<u8> {
    let mut texture = Vec::with_capacity((width * height * 4) as usize);
    for value in data {
        let encoded = (value.clamp(0.0, 1.0) * 255.0).round() as u8;
        texture.extend_from_slice(&[encoded, encoded, encoded, 255]);
    }
    texture
}

fn encode_mask_texture(data: &[u8], width: u32, height: u32) -> Vec<u8> {
    let mut texture = Vec::with_capacity((width * height * 4) as usize);
    for value in data {
        let encoded = if *value > 0 { 255 } else { 0 };
        texture.extend_from_slice(&[encoded, encoded, encoded, 255]);
    }
    texture
}

fn encode_albedo_texture(
    height: &[f32],
    land: &[u8],
    flux: &[f32],
    width: u32,
    height_px: u32,
) -> Vec<u8> {
    let mut texture = Vec::with_capacity((width * height_px * 4) as usize);
    for y in 0..height_px as usize {
        for x in 0..width as usize {
            let idx = y * width as usize + x;
            let land_value = land[idx] > 0;
            let height_value = height[idx].clamp(0.0, 1.0);
            let flux_value = flux.get(idx).copied().unwrap_or(0.0).clamp(0.0, 1.0);
            let coast = coastal_strength(land, width, height_px, x as i32, y as i32);
            let [r, g, b] = if land_value {
                colorize_land(height_value, flux_value, coast)
            } else {
                colorize_water(height_value, coast)
            };
            texture.extend_from_slice(&[r, g, b, 255]);
        }
    }
    texture
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
    albedo_texture: &[u8],
    height_texture: &[u8],
    flux_texture: &[u8],
    land_mask_texture: &[u8],
    width: u32,
    height: u32,
) -> SurfaceTexturePack {
    let pixel_count = (width * height) as usize;
    let mut terrain_albedo = vec![0u8; pixel_count * 4];
    let mut roughness = vec![0u8; pixel_count * 4];
    let mut ao = vec![0u8; pixel_count * 4];
    let mut water_color = vec![0u8; pixel_count * 4];
    let mut water_alpha = vec![0u8; pixel_count * 4];
    let mut coast_glow = vec![0u8; pixel_count * 4];

    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let idx = pixel_offset_rgba(width, x as usize, y as usize);
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
                sample_texture_channel_rgba(height_texture, width, height, x, y).clamp(0.0, 1.0);
            let flux_value =
                sample_texture_channel_rgba(flux_texture, width, height, x, y).clamp(0.0, 1.0);
            let is_land = sample_mask_rgba(land_mask_texture, width, height, x, y);
            let coast = coastal_strength_rgba(land_mask_texture, width, height, x, y);
            let left = sample_texture_channel_rgba(height_texture, width, height, x - 1, y);
            let right = sample_texture_channel_rgba(height_texture, width, height, x + 1, y);
            let up = sample_texture_channel_rgba(height_texture, width, height, x, y - 1);
            let down = sample_texture_channel_rgba(height_texture, width, height, x, y + 1);
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

            let mut color = [
                albedo_texture[idx],
                albedo_texture[idx + 1],
                albedo_texture[idx + 2],
            ];

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

            write_rgba(&mut terrain_albedo, idx, color[0], color[1], color[2], 255);

            let roughness_encoded = (roughness_value * 255.0).round() as u8;
            write_rgba(
                &mut roughness,
                idx,
                roughness_encoded,
                roughness_encoded,
                roughness_encoded,
                255,
            );

            let ao_encoded = (ao_value * 255.0).round() as u8;
            write_rgba(&mut ao, idx, ao_encoded, ao_encoded, ao_encoded, 255);

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
                idx,
                water_tint[0],
                water_tint[1],
                water_tint[2],
                255,
            );

            let water_alpha_encoded = (water_opacity * 255.0).round() as u8;
            write_rgba(
                &mut water_alpha,
                idx,
                water_alpha_encoded,
                water_alpha_encoded,
                water_alpha_encoded,
                255,
            );

            let glow_encoded = (glow_opacity * 255.0).round() as u8;
            write_rgba(
                &mut coast_glow,
                idx,
                glow_encoded,
                glow_encoded,
                glow_encoded,
                255,
            );
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

fn sample_texture_channel_rgba(texture: &[u8], width: u32, height: u32, x: i32, y: i32) -> f32 {
    let clamped_x = x.clamp(0, width.saturating_sub(1) as i32) as usize;
    let clamped_y = y.clamp(0, height.saturating_sub(1) as i32) as usize;
    let idx = pixel_offset_rgba(width, clamped_x, clamped_y);
    texture[idx] as f32 / 255.0
}

fn sample_mask_rgba(texture: &[u8], width: u32, height: u32, x: i32, y: i32) -> bool {
    sample_texture_channel_rgba(texture, width, height, x, y) > 0.5
}

fn coastal_strength_rgba(mask: &[u8], width: u32, height: u32, x: i32, y: i32) -> f32 {
    let center: f32 = if sample_mask_rgba(mask, width, height, x, y) {
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
            let sample: f32 = if sample_mask_rgba(mask, width, height, x + ox, y + oy) {
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
