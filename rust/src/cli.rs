//! 命令行应用程序入口
//!
//! 处理命令行参数，协调地图生成的各个步骤。
//!
//! # 地图生成流程
//! 1. 解析命令行参数
//! 2. 初始化随机数生成器
//! 3. 创建地图生成器
//! 4. 生成不规则网格（Voronoi 图）
//! 5. 生成地形高度图
//! 6. 侵蚀地形
//! 7. 放置城市和城镇
//! 8. 输出地图数据
//!
//! # 参考来源
//! - M. O'Leary, "Generating fantasy maps", https://mewo2.com/notes/terrain/
//! - 原始 C++ 实现: src/main.cpp

use anyhow::Result;
use clap::Parser;

use crate::{Config, Extents2d, GlibcRand, MapGenerator};

/// 运行地图生成 CLI 应用程序
///
/// 这是主要的应用程序入口点，协调整个地图生成流程。
///
/// # 返回
/// - `Ok(())` - 地图生成成功
/// - `Err(...)` - 发生错误（文件写入失败等）
///
/// # 参考来源
/// - 原始 C++ 实现: src/main.cpp, main()
pub fn run() -> Result<()> {
    // ===================================
    // 1. 解析命令行参数
    // ===================================
    let cfg = Config::parse();

    // ===================================
    // 2. 初始化随机数生成器
    // ===================================
    let seed = if cfg.timeseed {
        // 使用系统时间作为种子
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32
    } else {
        cfg.seed
    };

    eprintln!("Generating map with seed value: {}", seed);

    let mut rng = GlibcRand::new(seed);
    
    // 预热随机数生成器
    // 这与 C++ 版本保持一致，确保相同的随机数序列
    for _ in 0..1000 {
        rng.rand();
    }

    // ===================================
    // 3. 计算地图范围
    // ===================================
    let (img_w, img_h) = cfg.image_size();
    
    // 地图的逻辑高度固定为 20.0
    // 宽度根据图像宽高比计算，保持正确的纵横比
    let default_extents_height = 20.0f64;
    let aspect = img_w as f64 / img_h as f64;
    let extents_width = aspect * default_extents_height;
    let extents = Extents2d::new(0.0, 0.0, extents_width, default_extents_height);

    // ===================================
    // 4. 创建地图生成器
    // ===================================
    let mut map = MapGenerator::new(extents, cfg.resolution, img_w, img_h, rng);
    map.set_draw_scale(cfg.draw_scale);

    // 根据命令行参数禁用某些特性
    if cfg.no_slopes { map.disable_slopes(); }
    if cfg.no_rivers { map.disable_rivers(); }
    if cfg.no_contour { map.disable_contour(); }
    if cfg.no_borders { map.disable_borders(); }
    if cfg.no_cities { map.disable_cities(); }
    if cfg.no_towns { map.disable_towns(); }
    if cfg.no_labels { map.disable_labels(); }
    if cfg.no_arealabels { map.disable_area_labels(); }

    // ===================================
    // 5. 生成不规则网格
    // ===================================
    eprintln!("Initializing map generator...");
    map.initialize();

    // ===================================
    // 6. 生成地形高度图
    // ===================================
    eprintln!("Initializing height map...");
    initialize_heightmap(&mut map);

    // ===================================
    // 7. 侵蚀地形
    // ===================================
    let erosion_steps = cfg.erosion_steps;
    let erosion_amount = if cfg.erosion_amount >= 0.0 {
        cfg.erosion_amount
    } else {
        // 如果未指定，随机选择侵蚀量
        map.rng_mut().random_double(0.2, 0.35)
    };
    eprintln!("Eroding height map by {} over {} iterations...", erosion_amount, erosion_steps);
    erode(&mut map, erosion_amount, erosion_steps);

    // ===================================
    // 8. 放置城市和城镇
    // ===================================
    let num_cities = if cfg.cities >= 0 { cfg.cities } else { map.rng_mut().random_range(3, 7) };
    let num_towns = if cfg.towns >= 0 { cfg.towns } else { map.rng_mut().random_range(8, 25) };

    let num_labels = 2 * num_cities + num_towns;
    let label_names = get_label_names(num_labels as usize, map.rng_mut());

    eprintln!("Generating {} cities...", num_cities);
    let mut label_idx = label_names.len();
    for _ in 0..num_cities {
        if label_idx >= 2 {
            label_idx -= 2;
            let city_name = label_names[label_idx + 1].clone();
            let territory_name = label_names[label_idx].to_uppercase();
            map.add_city(city_name, territory_name);
        }
    }

    eprintln!("Generating {} towns...", num_towns);
    for _ in 0..num_towns {
        if label_idx >= 1 {
            label_idx -= 1;
            let town_name = label_names[label_idx].clone();
            map.add_town(town_name);
        }
    }

    eprintln!("Generating map draw data...");
    let draw_data = map.get_draw_data();

    let outfile = if cfg.output.ends_with(".json") {
        cfg.output.clone()
    } else {
        format!("{}.json", cfg.output)
    };

    std::fs::write(&outfile, draw_data.as_bytes())?;
    eprintln!("Wrote map draw data to file: {}", outfile);

    // ===================================
    // 9. 渲染地图（如果启用了 render feature）
    // ===================================
    #[cfg(feature = "render")]
    {
        if !cfg.no_render {
            let png_file = if cfg.output.ends_with(".json") {
                cfg.output.replace(".json", ".png")
            } else {
                format!("{}.png", cfg.output)
            };
            
            eprintln!("Rendering map to PNG...");
            render_map(&draw_data, &png_file)?;
            eprintln!("Wrote map image to file: {}", png_file);
        }
    }

    Ok(())
}

/// 初始化地形高度图
///
/// 通过随机放置山丘、圆锥和斜坡来生成初始地形。
/// 这个函数的实现与 C++ 版本完全一致，包括随机数调用顺序。
///
/// # 算法流程
/// 1. 在扩展区域内随机放置 100-250 个地形特征（山丘或圆锥）
/// 2. 50% 概率添加一个大型圆锥（火山）
/// 3. 90% 概率添加一个斜坡（山脉）
/// 4. 50% 概率进行归一化或圆滑处理
/// 5. 50% 概率进行松弛平滑
///
/// # 为什么要扩展区域
/// 在地图边界外放置地形特征，可以避免边界处的不自然截断。
///
/// # 随机数兼容性
/// 注意：代码中的 `_px_discard` 等变量是为了与 C++ 版本保持
/// 完全相同的随机数调用顺序，确保相同种子生成相同的地图。
///
/// # 参数
/// * `map` - 地图生成器
///
/// # 参考来源
/// - 原始 C++ 实现: src/main.cpp, initializeHeightmap()
fn initialize_heightmap(map: &mut MapGenerator) {
    let pad = 5.0f64;
    let extents = map.get_extents();
    
    // 扩展地图边界，在边界外也放置地形特征
    let expanded = Extents2d::new(
        extents.minx - pad, extents.miny - pad,
        extents.maxx + pad, extents.maxy + pad,
    );

    // ===================================
    // 1. 随机放置山丘和圆锥
    // ===================================
    let n = map.rng_mut().random_double(100.0, 250.0) as i32;
    for _ in 0..n {
        // 注意：这里调用两次 random_double 是为了与 C++ 版本保持一致
        // 第一次调用的结果被丢弃，只使用第二次的结果
        let _px_discard = map.rng_mut().random_double(expanded.minx, expanded.maxx);
        let px = map.rng_mut().random_double(expanded.minx, expanded.maxx);
        let py = map.rng_mut().random_double(expanded.miny, expanded.maxy);
        let r = map.rng_mut().random_double(1.0, 8.0);
        let strength = map.rng_mut().random_double(0.5, 1.5);
        
        // 50% 概率选择山丘或圆锥
        if map.rng_mut().random_double(0.0, 1.0) > 0.5 {
            map.add_hill(px, py, r, strength);
        } else {
            map.add_cone(px, py, r, strength);
        }
    }

    // ===================================
    // 2. 可能添加一个大型圆锥（火山）
    // ===================================
    if map.rng_mut().random_double(0.0, 1.0) > 0.5 {
        let _px_discard = map.rng_mut().random_double(expanded.minx, expanded.maxx);
        let px = map.rng_mut().random_double(expanded.minx, expanded.maxx);
        let py = map.rng_mut().random_double(expanded.miny, expanded.maxy);
        let r = map.rng_mut().random_double(6.0, 12.0);
        let strength = map.rng_mut().random_double(1.0, 3.0);
        map.add_cone(px, py, r, strength);
    }

    // ===================================
    // 3. 可能添加一个斜坡（山脉）
    // ===================================
    if map.rng_mut().random_double(0.0, 1.0) > 0.1 {
        let angle = map.rng_mut().random_double(0.0, 2.0 * std::f64::consts::PI);
        let dir_x = angle.sin();
        let dir_y = angle.cos();
        let _lx_discard = map.rng_mut().random_double(extents.minx, extents.maxx);
        let lx = map.rng_mut().random_double(extents.minx, extents.maxx);
        let ly = map.rng_mut().random_double(extents.miny, extents.maxy);
        let slope_width = map.rng_mut().random_double(0.5, 5.0);
        let strength = map.rng_mut().random_double(2.0, 3.0);
        map.add_slope(lx, ly, dir_x, dir_y, slope_width, strength);
    }

    // ===================================
    // 4. 归一化或圆滑处理
    // ===================================
    if map.rng_mut().random_double(0.0, 1.0) > 0.5 {
        map.normalize();
    } else {
        map.round();
    }

    // ===================================
    // 5. 可能进行松弛平滑
    // ===================================
    if map.rng_mut().random_double(0.0, 1.0) > 0.5 {
        map.relax();
    }
}

/// 侵蚀地形
///
/// 通过多次迭代应用侵蚀算法，模拟水流对地形的侵蚀作用。
/// 侵蚀会产生河谷、平滑山脉，使地形看起来更自然。
///
/// # 算法原理
/// 将总侵蚀量分散到多次迭代中，每次应用少量侵蚀。
/// 这比一次性应用大量侵蚀产生更好的效果。
///
/// # 参数
/// * `map` - 地图生成器
/// * `amount` - 总侵蚀量
/// * `iterations` - 迭代次数
///
/// # 参考来源
/// - 原始 C++ 实现: src/main.cpp, erode()
fn erode(map: &mut MapGenerator, amount: f64, iterations: i32) {
    for _ in 0..iterations {
        // 每次迭代应用部分侵蚀量
        map.erode(amount / iterations as f64);
    }
    
    // 侵蚀后调整海平面到中位数
    // 这样可以保持陆地和海洋的比例
    map.set_sea_level_to_median();
}

/// 获取城市和地区的名称
///
/// 从内嵌的城市数据 JSON 文件中随机选择指定数量的城市名称。
/// 这些名称将用于城市、城镇和领土的命名。
///
/// # 算法流程
/// 1. 解析 JSON 数据（按国家组织的城市列表）
/// 2. 随机选择国家，收集该国的城市名称
/// 3. 重复直到收集到足够的名称
/// 4. 打乱名称顺序（Fisher-Yates 洗牌算法）
/// 5. 截取所需数量的名称
///
/// # 为什么按国家组织
/// 这样可以生成具有地域特色的名称组合，
/// 使同一领土内的城市名称风格一致。
///
/// # 参数
/// * `num` - 需要的名称数量
/// * `rng` - 随机数生成器
///
/// # 返回
/// 随机选择并打乱的城市名称列表
///
/// # 参考来源
/// - 原始 C++ 实现: src/main.cpp, getLabelNames()
fn get_label_names(num: usize, rng: &mut GlibcRand) -> Vec<String> {
    // 加载内嵌的城市数据
    let city_data = include_str!("citydata/countrycities.json");
    let json: serde_json::Value = serde_json::from_str(city_data).expect("valid JSON");
    let obj = json.as_object().unwrap();
    let countries: Vec<String> = obj.keys().cloned().collect();
    let mut cities: Vec<String> = Vec::new();

    // 随机选择国家，收集城市名称
    while cities.len() < num {
        let rand_idx = rng.rand() as usize % countries.len();
        let country = &countries[rand_idx];
        if let Some(arr) = json[country].as_array() {
            for v in arr {
                if let Some(s) = v.as_str() {
                    cities.push(s.to_string());
                }
            }
        }
    }

    // Fisher-Yates 洗牌算法（与 C++ 实现一致）
    for i in (0..cities.len().saturating_sub(1)).rev() {
        let j = rng.rand() as usize % (i + 1);
        cities.swap(i, j);
    }

    // 截取所需数量
    cities.truncate(num);
    cities
}

/// 渲染地图到 PNG 文件
///
/// 使用 WebGPU 渲染器将地图数据渲染为 PNG 图像。
///
/// # 参数
/// * `draw_data` - JSON 格式的绘图数据
/// * `output_path` - 输出 PNG 文件路径
///
/// # 返回
/// - `Ok(())` - 渲染成功
/// - `Err(...)` - 渲染失败
#[cfg(feature = "render")]
fn render_map(draw_data: &str, output_path: &str) -> Result<()> {
    use crate::render::render_map as do_render;
    
    // 使用便捷函数渲染
    do_render(draw_data, output_path)
        .map_err(|e| anyhow::anyhow!("渲染失败: {}", e))?;
    
    Ok(())
}
