//! 渲染配置和常量定义
//!
//! 集中管理所有渲染相关的配置参数和常量。

/// 颜色定义（RGBA 格式，范围 [0.0, 1.0]）
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// 创建新颜色
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// 转换为数组格式
    pub const fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// 白色
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);

    /// 黑色
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    /// 半透明黑色
    pub const BLACK_TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.75);
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self {
        color.to_array()
    }
}

impl From<Color> for wgpu::Color {
    fn from(color: Color) -> Self {
        wgpu::Color {
            r: color.r as f64,
            g: color.g as f64,
            b: color.b as f64,
            a: color.a as f64,
        }
    }
}

/// 渲染样式配置
///
/// 定义地图各元素的颜色和尺寸。
#[derive(Debug, Clone)]
pub struct RenderStyle {
    /// 背景颜色
    pub background_color: Color,

    /// 坡度阴影颜色
    pub slope_color: Color,

    /// 河流颜色
    pub river_color: Color,

    /// 等高线颜色
    pub contour_color: Color,

    /// 边界颜色
    pub border_color: Color,

    /// 标记颜色
    pub marker_color: Color,

    /// 文字颜色
    pub text_color: Color,

    /// 坡度线宽（像素）
    pub slope_line_width: f32,

    /// 河流线宽（像素）
    pub river_line_width: f32,

    /// 等高线线宽（像素）
    pub contour_line_width: f32,

    /// 边界线宽（像素）
    pub border_line_width: f32,

    /// 城市标记外圆半径（像素）
    pub city_marker_outer_radius: f32,

    /// 城市标记内圆半径（像素）
    pub city_marker_inner_radius: f32,

    /// 城镇标记半径（像素）
    pub town_marker_radius: f32,
}

impl Default for RenderStyle {
    /// 默认渲染样式
    ///
    /// 对应 Python 版本的默认配置。
    fn default() -> Self {
        Self {
            background_color: Color::WHITE,
            slope_color: Color::BLACK_TRANSPARENT,
            river_color: Color::BLACK,
            contour_color: Color::BLACK,
            border_color: Color::BLACK,
            marker_color: Color::BLACK,
            text_color: Color::BLACK,
            slope_line_width: 1.0,
            river_line_width: 2.5,
            contour_line_width: 1.5,
            border_line_width: 6.0,
            city_marker_outer_radius: 10.0,
            city_marker_inner_radius: 5.0,
            town_marker_radius: 5.0,
        }
    }
}

impl RenderStyle {
    /// 应用缩放比例
    ///
    /// 根据 draw_scale 参数缩放所有尺寸。
    pub fn with_scale(&self, scale: f32) -> Self {
        Self {
            slope_line_width: self.slope_line_width * scale,
            river_line_width: self.river_line_width * scale,
            contour_line_width: self.contour_line_width * scale,
            border_line_width: self.border_line_width * scale,
            city_marker_outer_radius: self.city_marker_outer_radius * scale,
            city_marker_inner_radius: self.city_marker_inner_radius * scale,
            town_marker_radius: self.town_marker_radius * scale,
            ..*self
        }
    }
}

/// 圆形渲染配置
#[derive(Debug, Clone, Copy)]
pub struct CircleConfig {
    /// 圆形线段数量（越多越平滑）
    pub segments: u32,
}

impl Default for CircleConfig {
    fn default() -> Self {
        Self {
            segments: 32, // 32 个线段足够平滑
        }
    }
}
