//! 字体处理模块
//!
//! 提供字体度量和文本范围计算功能。
//! 使用预先计算的字体度量数据（JSON 格式）来快速计算文本尺寸。
//!
//! # 在地图生成中的应用
//! 用于计算城市名称、地区标签等文本的尺寸，
//! 以便在地图上正确放置和渲染标签。
//!
//! # 字体数据格式
//! JSON 数据结构：
//! ```json
//! {
//!   "FontName": {
//!     "FontSize": {
//!       "Character": [offx, offy, width, height, dx, dy]
//!     }
//!   }
//! }
//! ```
//!
//! # 参考来源
//! - 原始 C++ 实现: src/fontface.h, src/fontface.cpp

use serde_json::Value;
use std::collections::HashMap;

/// 文本范围信息
///
/// 描述文本的边界框和前进距离。
#[derive(Clone, Debug, Default)]
pub struct TextExtents {
    /// X 方向偏移（相对于基线原点）
    pub offx: f64,
    /// Y 方向偏移（相对于基线原点）
    pub offy: f64,
    /// 文本宽度
    pub width: f64,
    /// 文本高度
    pub height: f64,
    /// X 方向前进距离（光标移动距离）
    pub dx: f64,
    /// Y 方向前进距离（通常为 0）
    pub dy: f64,
}

/// 字体管理器
///
/// 管理字体度量数据，提供文本尺寸计算功能。
pub struct FontFace {
    /// 字体数据（JSON 格式）
    data: Value,
    /// 当前字体名称
    font_face: String,
    /// 当前字体大小
    font_size: String,
    /// 默认字体名称
    default_font: String,
    /// 默认字体大小
    default_font_size: String,
}

impl FontFace {
    /// 从 JSON 字符串创建字体管理器
    ///
    /// # 默认字体选择
    /// 1. 如果存在 "Arial" 字体，使用它作为默认字体
    /// 2. 否则使用 JSON 中的第一个字体
    ///
    /// # 参数
    /// * `json_str` - 字体数据 JSON 字符串
    ///
    /// # Panics
    /// 如果 JSON 格式无效会 panic
    ///
    /// # 参考来源
    /// - 原始 C++ 实现: src/fontface.cpp, FontFace 构造函数
    pub fn new(json_str: &str) -> Self {
        let data: Value = serde_json::from_str(json_str).expect("valid fontdata JSON");
        let obj = data.as_object().expect("fontdata is object");

        // 选择默认字体（优先使用 Arial）
        let default_font = if obj.contains_key("Arial") {
            "Arial".to_string()
        } else {
            obj.keys().next().cloned().unwrap_or_default()
        };

        // 选择默认字体大小（使用该字体的第一个可用大小）
        let default_font_size = data[&default_font]
            .as_object()
            .and_then(|o| o.keys().next().cloned())
            .unwrap_or_default();

        FontFace {
            data,
            font_face: default_font.clone(),
            font_size: default_font_size.clone(),
            default_font,
            default_font_size,
        }
    }

    /// 获取当前字体名称
    pub fn get_font_face(&self) -> &str {
        &self.font_face
    }

    /// 获取当前字体大小
    pub fn get_font_size(&self) -> i32 {
        self.font_size.parse().unwrap_or(0)
    }

    /// 设置字体和大小
    ///
    /// # 参数
    /// * `name` - 字体名称
    /// * `size` - 字体大小
    ///
    /// # 返回
    /// 如果字体和大小存在返回 true，否则返回 false（保持原字体）
    ///
    /// # 参考来源
    /// - 原始 C++ 实现: src/fontface.cpp, setFontFaceSize()
    pub fn set_font_face_size(&mut self, name: &str, size: i32) -> bool {
        // 检查字体是否存在
        if self.data.get(name).is_none() { return false; }
        
        let size_str = size.to_string();
        
        // 检查字体大小是否存在
        if self.data[name].get(&size_str).is_none() { return false; }
        
        self.font_face = name.to_string();
        self.font_size = size_str;
        true
    }

    /// 获取文本的整体范围
    ///
    /// 计算整个文本字符串的边界框和前进距离。
    ///
    /// # 算法流程
    /// 1. 如果文本为空，返回默认范围
    /// 2. 如果只有一个字符，返回该字符的范围
    /// 3. 否则，遍历所有字符，累加宽度，计算整体边界框
    ///
    /// # 参数
    /// * `text` - 文本字符串
    ///
    /// # 返回
    /// 文本的整体范围信息
    ///
    /// # 参考来源
    /// - 原始 C++ 实现: src/fontface.cpp, getTextExtents()
    pub fn get_text_extents(&self, text: &str) -> TextExtents {
        if text.is_empty() {
            return TextExtents::default();
        }
        if text.len() == 1 {
            return self.get_char_extents(text.chars().next().unwrap());
        }

        // 使用第一个字符的 X 偏移作为整体偏移
        let first = self.get_char_extents(text.chars().next().unwrap());
        let mut extents = TextExtents::default();
        extents.offx = first.offx;

        // 计算整体的 Y 范围
        let mut ymin = 0.0f64;
        let mut ymax = 0.0f64;
        let mut dx = 0.0f64;
        for c in text.chars() {
            let ce = self.get_char_extents(c);
            ymin = ymin.min(ce.offy);
            ymax = ymax.max(ce.offy + ce.height);
            dx += ce.dx;
        }
        extents.offy = ymin;

        // 计算整体宽度（考虑最后一个字符的实际宽度）
        let last = self.get_char_extents(text.chars().last().unwrap());
        extents.width = dx + extents.offx - (last.dx - last.width);
        extents.height = ymax - ymin;
        extents.dx = dx;
        extents.dy = 0.0;
        extents
    }

    /// 获取每个字符的范围
    ///
    /// 返回文本中每个字符的独立范围信息，
    /// 每个字符的 X 偏移已经调整为相对于文本起点的绝对位置。
    ///
    /// # 参数
    /// * `text` - 文本字符串
    ///
    /// # 返回
    /// 每个字符的范围信息列表
    ///
    /// # 参考来源
    /// - 原始 C++ 实现: src/fontface.cpp, getCharacterExtents()
    pub fn get_character_extents(&self, text: &str) -> Vec<TextExtents> {
        let mut extents = Vec::new();
        let mut x = 0.0f64;
        
        for c in text.chars() {
            let mut ce = self.get_char_extents(c);
            // 调整 X 偏移为绝对位置
            ce.offx += x;
            x += ce.dx;
            extents.push(ce);
        }
        
        extents
    }

    /// 获取单个字符的范围
    ///
    /// 从字体数据中查找字符的度量信息。
    ///
    /// # 参数
    /// * `c` - 字符
    ///
    /// # 返回
    /// 字符的范围信息
    ///
    /// # 参考来源
    /// - 原始 C++ 实现: src/fontface.cpp, getCharExtents()
    fn get_char_extents(&self, c: char) -> TextExtents {
        let key = c.to_string();
        let char_data = &self.data[&self.font_face][&self.font_size][&key];
        let arr = char_data.as_array();
        let mut data = [0.0f64; 6];
        
        // 解析字符度量数据：[offx, offy, width, height, dx, dy]
        if let Some(arr) = arr {
            for (i, v) in arr.iter().enumerate().take(6) {
                data[i] = v.as_f64().unwrap_or(0.0);
            }
        }
        
        TextExtents {
            offx: data[0],
            offy: data[1],
            width: data[2],
            height: data[3],
            dx: data[4],
            dy: data[5],
        }
    }
}
