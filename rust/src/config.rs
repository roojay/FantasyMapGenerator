//! 命令行配置
//!
//! 定义地图生成器的所有命令行参数和配置选项。
//! 使用 clap 库进行参数解析。
//!
//! # 参考来源
//! - 原始 C++ 实现: src/config.h, src/config.cpp

use clap::Parser;

/// 地图生成器的配置参数
///
/// 包含所有可配置的地图生成选项，从随机种子到渲染设置。
///
/// # 使用示例
/// ```bash
/// # 使用随机种子生成地图
/// map_generation --timeseed --cities 5 --towns 10
///
/// # 使用固定种子（可重现）
/// map_generation --seed 12345 --resolution 0.08
///
/// # 禁用某些特性
/// map_generation --no-rivers --no-labels
/// ```
#[derive(Parser, Debug)]
#[command(name = "map_generation", about = "Fantasy Map Generator")]
pub struct Config {
    // ===================================
    // 随机数设置
    // ===================================
    /// 随机数种子
    ///
    /// 使用相同的种子会生成完全相同的地图。
    /// 这对于调试和分享地图非常有用。
    #[arg(long, short = 's', default_value = "0")]
    pub seed: u32,

    /// 使用当前时间作为种子
    ///
    /// 如果启用，会忽略 --seed 参数，
    /// 使用系统时间生成随机种子。
    #[arg(long, default_value = "false")]
    pub timeseed: bool,

    // ===================================
    // 输出设置
    // ===================================
    /// 输出文件名（不含扩展名）
    ///
    /// 生成的 JSON 文件会自动添加 .json 扩展名。
    /// 建议使用 examples/ 目录存放输出文件。
    #[arg(long, short = 'o', default_value = "examples/output")]
    pub output: String,

    // ===================================
    // 地形生成参数
    // ===================================
    /// 分辨率（Poisson 圆盘采样的最小距离）
    ///
    /// 较小的值会生成更多的采样点，产生更详细的地图，
    /// 但也会增加计算时间。
    ///
    /// 推荐值：0.05 - 0.15
    #[arg(long, short = 'r', default_value = "0.08")]
    pub resolution: f64,

    /// 侵蚀量（-1 表示随机）
    ///
    /// 控制地形侵蚀的强度。
    /// 较大的值会产生更多的河谷和平滑的地形。
    ///
    /// 推荐值：0.2 - 0.35
    #[arg(long, short = 'e', default_value = "-1.0")]
    pub erosion_amount: f64,

    /// 侵蚀迭代次数
    ///
    /// 侵蚀算法的迭代次数。
    /// 更多的迭代会产生更平滑的地形。
    #[arg(long, default_value = "3")]
    pub erosion_steps: i32,

    // ===================================
    // 城市和城镇设置
    // ===================================
    /// 城市数量（-1 表示随机）
    ///
    /// 随机时会在 3-7 之间选择。
    #[arg(long, short = 'c', default_value = "-1")]
    pub cities: i32,

    /// 城镇数量（-1 表示随机）
    ///
    /// 随机时会在 8-25 之间选择。
    #[arg(long, short = 't', default_value = "-1")]
    pub towns: i32,

    // ===================================
    // 渲染设置
    // ===================================
    /// 图像尺寸（格式："宽度:高度" 或 "宽度x高度"）
    ///
    /// 例如："1920:1080" 或 "3840:2160"
    #[arg(long, default_value = "1920:1080")]
    pub size: String,

    /// 绘制缩放比例
    ///
    /// 控制地图元素（如河流、边界）的线条粗细。
    /// 较大的值会产生更粗的线条。
    #[arg(long, default_value = "1.0")]
    pub draw_scale: f64,

    /// 禁用坡度阴影
    ///
    /// 如果启用，地图将不显示地形坡度的阴影效果。
    #[arg(long, default_value = "false")]
    pub no_slopes: bool,

    /// 禁用河流
    ///
    /// 如果启用，地图将不显示河流。
    #[arg(long, default_value = "false")]
    pub no_rivers: bool,

    /// 禁用等高线
    ///
    /// 如果启用，地图将不显示等高线。
    #[arg(long, default_value = "false")]
    pub no_contour: bool,

    /// 禁用领土边界
    ///
    /// 如果启用，地图将不显示城市领土的边界线。
    #[arg(long, default_value = "false")]
    pub no_borders: bool,

    /// 禁用城市标记
    ///
    /// 如果启用，地图将不显示城市的图标。
    #[arg(long, default_value = "false")]
    pub no_cities: bool,

    /// 禁用城镇标记
    ///
    /// 如果启用，地图将不显示城镇的图标。
    #[arg(long, default_value = "false")]
    pub no_towns: bool,

    /// 禁用所有文字标签
    ///
    /// 如果启用，地图将不显示任何文字标签（城市名、地区名等）。
    #[arg(long, default_value = "false")]
    pub no_labels: bool,

    /// 禁用地区标签
    ///
    /// 如果启用，地图将不显示地区名称标签（如海洋、山脉等）。
    #[arg(long, default_value = "false")]
    pub no_arealabels: bool,

    /// 显示绘图支持信息并退出
    ///
    /// 用于检查系统是否支持地图渲染功能。
    #[arg(long, default_value = "false")]
    pub drawing_supported: bool,

    /// 启用详细输出
    ///
    /// 显示地图生成过程的详细信息。
    #[arg(long, short = 'v', default_value = "false")]
    pub verbose: bool,

    /// 禁用 PNG 渲染（仅生成 JSON）
    ///
    /// 如果启用，只生成 JSON 数据文件，不渲染 PNG 图像。
    /// 注意：此选项仅在编译时启用 render feature 时有效。
    #[cfg(feature = "render")]
    #[arg(long, default_value = "false")]
    pub no_render: bool,
}

impl Config {
    /// 解析图像尺寸字符串
    ///
    /// 支持两种格式：
    /// - "宽度:高度"（C++ 版本兼容格式）
    /// - "宽度x高度"（常见格式）
    ///
    /// # 返回
    /// (宽度, 高度) 元组，解析失败时返回默认值 (1920, 1080)
    pub fn image_size(&self) -> (u32, u32) {
        // 支持冒号和 x 两种分隔符
        let sep = if self.size.contains(':') { ':' } else { 'x' };
        let parts: Vec<&str> = self.size.split(sep).collect();

        if parts.len() == 2 {
            let w = parts[0].parse().unwrap_or(1920);
            let h = parts[1].parse().unwrap_or(1080);
            (w, h)
        } else {
            // 解析失败，返回默认值
            (1920, 1080)
        }
    }
}
