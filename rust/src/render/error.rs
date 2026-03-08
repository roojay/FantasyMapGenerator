//! 渲染错误类型定义
//!
//! 提供更精确的错误信息，便于调试和错误处理。

use std::fmt;

/// 渲染错误类型
///
/// 定义渲染过程中可能出现的各种错误。
#[derive(Debug)]
pub enum RenderError {
    /// WebGPU 适配器初始化失败
    AdapterNotFound,

    /// WebGPU 设备创建失败
    DeviceCreationFailed(String),

    /// 缓冲区映射失败
    BufferMapFailed,

    /// 图像创建失败
    ImageCreationFailed,

    /// 图像保存失败
    ImageSaveFailed(std::io::Error),

    /// JSON 解析失败
    JsonParseFailed(serde_json::Error),

    /// 无效的 JSON 数据格式
    InvalidJsonFormat(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AdapterNotFound => {
                write!(f, "未找到合适的 WebGPU 适配器。请确保系统支持 WebGPU。")
            }
            Self::DeviceCreationFailed(msg) => {
                write!(f, "WebGPU 设备创建失败: {}", msg)
            }
            Self::BufferMapFailed => {
                write!(f, "缓冲区映射失败")
            }
            Self::ImageCreationFailed => {
                write!(f, "图像创建失败：数据格式不正确")
            }
            Self::ImageSaveFailed(err) => {
                write!(f, "图像保存失败: {}", err)
            }
            Self::JsonParseFailed(err) => {
                write!(f, "JSON 解析失败: {}", err)
            }
            Self::InvalidJsonFormat(msg) => {
                write!(f, "无效的 JSON 数据格式: {}", msg)
            }
        }
    }
}

impl std::error::Error for RenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ImageSaveFailed(err) => Some(err),
            Self::JsonParseFailed(err) => Some(err),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for RenderError {
    fn from(err: serde_json::Error) -> Self {
        Self::JsonParseFailed(err)
    }
}

impl From<std::io::Error> for RenderError {
    fn from(err: std::io::Error) -> Self {
        Self::ImageSaveFailed(err)
    }
}

impl From<image::ImageError> for RenderError {
    fn from(err: image::ImageError) -> Self {
        match err {
            image::ImageError::IoError(io_err) => Self::ImageSaveFailed(io_err),
            _ => Self::ImageSaveFailed(std::io::Error::new(
                std::io::ErrorKind::Other,
                err.to_string(),
            )),
        }
    }
}

/// 渲染结果类型别名
pub type RenderResult<T> = Result<T, RenderError>;
