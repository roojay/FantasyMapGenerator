//! WebGPU 地图渲染模块
//!
//! 使用 WebGPU 渲染幻想地图，替代原 C++ 版本的 Python/Cairo 渲染。
//!
//! # 渲染流程
//!
//! 1. 解析 JSON 绘图数据
//! 2. 初始化 WebGPU 设备和纹理
//! 3. 绘制背景
//! 4. 绘制坡度阴影（线段）
//! 5. 绘制领土边界（路径）
//! 6. 绘制河流（路径）
//! 7. 绘制等高线（路径）
//! 8. 绘制城市标记（圆形）
//! 9. 绘制城镇标记（圆形）
//! 10. 绘制文字标签
//! 11. 保存为 PNG 图像
//!
//! # 示例
//!
//! ```rust,no_run
//! use fantasy_map_generator::render::{render_map, MapRenderer, RenderStyle};
//! use serde_json::json;
//!
//! // 方式 1: 使用便捷函数
//! let json_data = r#"{"image_width": 1920, "image_height": 1080, ...}"#;
//! render_map(json_data, "output.png").unwrap();
//!
//! // 方式 2: 使用渲染器（更灵活）
//! let data = json!({
//!     "image_width": 1920,
//!     "image_height": 1080,
//!     "draw_scale": 1.0,
//!     // ... 其他数据
//! });
//!
//! let mut renderer = MapRenderer::new(1920, 1080).unwrap();
//! renderer.render(&data).unwrap();
//! renderer.save_png("output.png").unwrap();
//!
//! // 方式 3: 使用自定义样式
//! let style = RenderStyle::default();
//! let mut renderer = MapRenderer::with_style(1920, 1080, style).unwrap();
//! renderer.render(&data).unwrap();
//! renderer.save_png("output.png").unwrap();
//! ```
//!
//! # 与 Python/Cairo 版本的对应关系
//!
//! - Cairo Context → WebGPU RenderPass
//! - Cairo Surface → WebGPU Texture
//! - Cairo 路径绘制 → WebGPU 线条渲染
//! - Cairo 文字渲染 → ab_glyph 字体光栅化（未实现）
//!
//! # 参考来源
//! - 原始 Python 实现: src/render/rendermap.py
//! - 原始 C++ 实现: src/render.cpp

mod config;
mod error;
mod primitives;
mod renderer;
mod text;

#[cfg(test)]
mod tests;

// 公共导出
pub use config::{CircleConfig, Color, RenderStyle};
pub use error::{RenderError, RenderResult};
pub use renderer::MapRenderer;

use serde_json::Value;

/// 渲染地图到 PNG 文件
///
/// 这是主要的公共接口，对应 Python 版本的 draw_map 函数。
///
/// # 参数
/// * `json_data` - 地图绘图数据（JSON 格式）
/// * `output_path` - 输出 PNG 文件路径
///
/// # 返回
/// - `Ok(())` - 渲染成功
/// - `Err(RenderError)` - 渲染失败
///
/// # 示例
/// ```rust,no_run
/// use fantasy_map_generator::render::render_map;
///
/// let json_data = r#"{
///     "image_width": 1920,
///     "image_height": 1080,
///     "draw_scale": 1.0,
///     "slope": [],
///     "river": [],
///     "contour": [],
///     "territory": [],
///     "city": [],
///     "town": [],
///     "label": []
/// }"#;
///
/// render_map(json_data, "output.png").unwrap();
/// ```
///
/// # 参考来源
/// - 原始 Python 实现: rendermap.py, draw_map()
pub fn render_map(json_data: &str, output_path: &str) -> RenderResult<()> {
    let data: Value = serde_json::from_str(json_data)?;

    let width = data["image_width"].as_u64().unwrap_or(1920) as u32;
    let height = data["image_height"].as_u64().unwrap_or(1080) as u32;

    let mut renderer = MapRenderer::new(width, height)?;
    renderer.render(&data)?;
    renderer.save_png(output_path)?;

    Ok(())
}
