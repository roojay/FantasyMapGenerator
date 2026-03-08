//! 渲染模块单元测试

#[cfg(test)]
mod tests {
    use super::super::*;
    use serde_json::json;

    #[test]
    fn test_color_creation() {
        let color = Color::new(1.0, 0.5, 0.0, 1.0);
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_color_to_array() {
        let color = Color::WHITE;
        let array = color.to_array();
        assert_eq!(array, [1.0, 1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::WHITE.to_array(), [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(Color::BLACK.to_array(), [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(Color::BLACK_TRANSPARENT.to_array(), [0.0, 0.0, 0.0, 0.75]);
    }

    #[test]
    fn test_render_style_default() {
        let style = RenderStyle::default();
        assert_eq!(style.slope_line_width, 1.0);
        assert_eq!(style.river_line_width, 2.5);
        assert_eq!(style.city_marker_outer_radius, 10.0);
    }

    #[test]
    fn test_render_style_with_scale() {
        let style = RenderStyle::default();
        let scaled = style.with_scale(2.0);

        assert_eq!(scaled.slope_line_width, 2.0);
        assert_eq!(scaled.river_line_width, 5.0);
        assert_eq!(scaled.city_marker_outer_radius, 20.0);
    }

    #[test]
    fn test_circle_config_default() {
        let config = CircleConfig::default();
        assert_eq!(config.segments, 32);
    }

    #[test]
    fn test_render_error_display() {
        let err = RenderError::AdapterNotFound;
        let msg = format!("{}", err);
        assert!(msg.contains("WebGPU"));
    }

    #[test]
    fn test_json_parse_error_conversion() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let render_err: RenderError = json_err.into();

        match render_err {
            RenderError::JsonParseFailed(_) => {}
            _ => panic!("Expected JsonParseFailed"),
        }
    }

    #[test]
    fn test_invalid_json_format() {
        let data = json!({
            "image_width": 100,
            "image_height": 100,
            "draw_scale": 1.0,
            "slope": "not an array" // 应该是数组
        });

        // 注意：这个测试需要 WebGPU 支持，在 CI 环境中可能失败
        // 实际测试应该 mock WebGPU 或使用集成测试
        if let Ok(mut renderer) = MapRenderer::new(100, 100) {
            let result = renderer.render(&data);
            assert!(result.is_err());

            if let Err(RenderError::InvalidJsonFormat(msg)) = result {
                assert!(msg.contains("数组"));
            }
        }
    }
}
