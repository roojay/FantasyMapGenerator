//! WASM 绑定模块
//!
//! 提供 JavaScript 可调用的 WASM 接口，用于在浏览器中生成地图。

use crate::satellite_svg;
use crate::{Extents2d, GlibcRand, MapExportOptions, MapGenerator};
use wasm_bindgen::prelude::*;

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

    /// 生成完整地图，并允许网页端按需决定是否导出附加栅格数据。
    ///
    /// # 参数
    /// * `num_cities` - 城市数量
    /// * `num_towns` - 城镇数量
    /// * `include_raster_data` - 是否导出供卫星风格渲染使用的栅格数据
    ///
    /// # 与原始 C++ 的差异
    /// 原始 C++ 版本没有 WASM 导出层，也没有“按需导出栅格”的调用入口。
    /// 这个接口是本 fork 为浏览器场景新增的能力，用来减少 WASM 与 JS 之间
    /// 的大块 JSON 传输。
    ///
    /// # 性能说明
    /// 普通矢量渲染可将 `include_raster_data` 设为 `false`，
    /// 只有卫星风格或调试栅格数据时才需要导出附加数组。
    #[wasm_bindgen]
    pub fn generate_with_options(
        &mut self,
        num_cities: i32,
        num_towns: i32,
        include_raster_data: bool,
    ) -> Result<String, JsValue> {
        // PERF: 将栅格导出显式化，避免普通网页路径总是携带大型 height/flux/mask 数组。
        // 初始化网格
        self.generator.initialize();

        // 生成地形
        self.initialize_heightmap();

        // 侵蚀地形
        self.erode(0.25, 5);

        // 生成城市和城镇
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

        // 导出为 JSON
        let json = self.generator.get_draw_data_with_options(MapExportOptions {
            include_raster_data,
        });
        Ok(json)
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
        let json = self.generator.get_draw_data();
        Ok(json)
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
        let city_data = include_str!("citydata/countrycities.json");
        let json: serde_json::Value = serde_json::from_str(city_data).expect("valid JSON");
        let obj = json.as_object().unwrap();
        let countries: Vec<String> = obj.keys().cloned().collect();
        let mut cities: Vec<String> = Vec::new();

        while cities.len() < num {
            let rand_idx = self.generator.rng_mut().rand() as usize % countries.len();
            let country = &countries[rand_idx];
            if let Some(arr) = json[country].as_array() {
                for v in arr {
                    if let Some(s) = v.as_str() {
                        cities.push(s.to_string());
                    }
                }
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
}

/// 简化的地图生成函数（用于快速测试）
#[wasm_bindgen]
pub fn generate_map_simple(seed: u32, width: u32, height: u32) -> Result<String, JsValue> {
    let mut generator = WasmMapGenerator::new(seed, width, height, 0.08)?;
    generator.generate(5, 10)
}

/// 根据导出的地图 JSON 和图层配置直接在 Rust 侧生成卫星风格 SVG。
#[wasm_bindgen]
pub fn build_satellite_svg(map_json: &str, layers_json: &str) -> Result<String, JsValue> {
    satellite_svg::build_satellite_svg(map_json, layers_json).map_err(|err| JsValue::from_str(&err))
}

/// 根据导出的地图 JSON、图层配置和优化选项生成卫星风格 SVG。
#[wasm_bindgen]
pub fn build_satellite_svg_with_options(
    map_json: &str,
    layers_json: &str,
    options_json: &str,
) -> Result<String, JsValue> {
    satellite_svg::build_satellite_svg_with_options(map_json, layers_json, Some(options_json))
        .map_err(|err| JsValue::from_str(&err))
}
