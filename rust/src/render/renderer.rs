//! WebGPU 渲染器核心实现

use serde_json::Value;
use wgpu::util::DeviceExt;

use super::config::{CircleConfig, Color, RenderStyle};
use super::error::{RenderError, RenderResult};
use super::primitives::*;
use super::text::TextRenderer;

/// WebGPU 地图渲染器
///
/// 管理 WebGPU 设备、纹理和渲染管线。
///
/// # 示例
///
/// ```rust,no_run
/// use fantasy_map_generator::render::MapRenderer;
/// use serde_json::json;
///
/// let data = json!({
///     "image_width": 1920,
///     "image_height": 1080,
///     "draw_scale": 1.0,
///     "slope": [],
///     "river": [],
///     // ... 其他数据
/// });
///
/// let mut renderer = MapRenderer::new(1920, 1080)?;
/// renderer.render(&data)?;
/// renderer.save_png("output.png")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct MapRenderer {
    width: u32,
    height: u32,
    device: wgpu::Device,
    queue: wgpu::Queue,
    texture: wgpu::Texture,
    line_pipeline: wgpu::RenderPipeline,
    circle_pipeline: wgpu::RenderPipeline,
    text_renderer: TextRenderer,
    style: RenderStyle,
    circle_config: CircleConfig,
}

impl MapRenderer {
    /// 创建新的渲染器
    ///
    /// 初始化 WebGPU 设备、纹理和渲染管线。
    ///
    /// # 参数
    /// * `width` - 图像宽度（像素）
    /// * `height` - 图像高度（像素）
    ///
    /// # 错误
    /// - `RenderError::AdapterNotFound` - 未找到合适的 WebGPU 适配器
    /// - `RenderError::DeviceCreationFailed` - 设备创建失败
    pub fn new(width: u32, height: u32) -> RenderResult<Self> {
        Self::with_style(width, height, RenderStyle::default())
    }

    /// 使用自定义样式创建渲染器
    ///
    /// # 参数
    /// * `width` - 图像宽度（像素）
    /// * `height` - 图像高度（像素）
    /// * `style` - 渲染样式配置
    pub fn with_style(width: u32, height: u32, style: RenderStyle) -> RenderResult<Self> {
        // 初始化 WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .ok_or(RenderError::AdapterNotFound)?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Map Renderer Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ))
        .map_err(|e| RenderError::DeviceCreationFailed(e.to_string()))?;

        // 创建渲染目标纹理
        let texture = Self::create_render_texture(&device, width, height);

        // 创建渲染管线
        let line_pipeline = Self::create_line_pipeline(&device);
        let circle_pipeline = Self::create_circle_pipeline(&device);

        let text_renderer = TextRenderer::new(&device, &queue, width, height)?;

        Ok(Self {
            width,
            height,
            device,
            queue,
            texture,
            line_pipeline,
            circle_pipeline,
            text_renderer,
            style,
            circle_config: CircleConfig::default(),
        })
    }

    /// 创建渲染目标纹理
    fn create_render_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Render Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    }

    /// 创建线条渲染管线
    fn create_line_pipeline(device: &wgpu::Device) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Line Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/line.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Line Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Line Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[LineVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
    }

    /// 创建圆形渲染管线
    fn create_circle_pipeline(device: &wgpu::Device) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Circle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/circle.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Circle Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Circle Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[CircleVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
    }

    /// 渲染地图
    ///
    /// 按照正确的顺序绘制所有地图元素。
    ///
    /// # 参数
    /// * `data` - JSON 绘图数据
    ///
    /// # 错误
    /// - `RenderError::InvalidJsonFormat` - JSON 数据格式不正确
    pub fn render(&mut self, data: &Value) -> RenderResult<()> {
        let draw_scale = data["draw_scale"].as_f64().unwrap_or(1.0) as f32;
        let style = self.style.with_scale(draw_scale);

        let view = self
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // 1. 清空背景
        self.clear_background(&view, &mut encoder, style.background_color);

        // 2. 绘制坡度阴影
        if let Some(slopes) = data.get("slope") {
            self.draw_segments(slopes, style.slope_color, &view, &mut encoder)?;
        }

        // 3. 绘制领土边界（先白色底，再黑色线）
        if let Some(territory) = data.get("territory") {
            self.draw_paths(territory, style.background_color, &view, &mut encoder)?;
            self.draw_paths(territory, style.border_color, &view, &mut encoder)?;
        }

        // 4. 绘制河流
        if let Some(rivers) = data.get("river") {
            self.draw_paths(rivers, style.river_color, &view, &mut encoder)?;
        }

        // 5. 绘制等高线
        if let Some(contours) = data.get("contour") {
            self.draw_paths(contours, style.contour_color, &view, &mut encoder)?;
        }

        // 6. 绘制城市标记
        if let Some(cities) = data.get("city") {
            self.draw_city_markers(
                cities,
                style.city_marker_outer_radius,
                style.city_marker_inner_radius,
                style.marker_color,
                style.background_color,
                &view,
                &mut encoder,
            )?;
        }

        // 7. 绘制城镇标记
        if let Some(towns) = data.get("town") {
            self.draw_town_markers(
                towns,
                style.town_marker_radius,
                style.marker_color,
                &view,
                &mut encoder,
            )?;
        }

        // 8. 绘制文字标签
        if let Some(labels) = data.get("label") {
            self.draw_labels(labels, style.text_color, &view, &mut encoder)?;
        }

        self.queue.submit(Some(encoder.finish()));

        Ok(())
    }

    /// 清空背景
    fn clear_background(
        &self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        color: Color,
    ) {
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(color.into()),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }

    /// 绘制线段（用于坡度阴影）
    fn draw_segments(
        &self,
        data: &Value,
        color: Color,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> RenderResult<()> {
        let segments = data
            .as_array()
            .ok_or_else(|| RenderError::InvalidJsonFormat("slope 应为数组".to_string()))?;

        if segments.is_empty() {
            return Ok(());
        }

        let vertices = self.parse_segments(segments, color)?;
        if vertices.is_empty() {
            return Ok(());
        }

        self.draw_line_vertices(&vertices, "Segment", view, encoder);
        Ok(())
    }

    /// 解析线段数据
    fn parse_segments(&self, segments: &[Value], color: Color) -> RenderResult<Vec<LineVertex>> {
        let mut vertices = Vec::new();
        let color_array = color.to_array();

        for i in (0..segments.len()).step_by(4) {
            if i + 3 >= segments.len() {
                break;
            }

            let x1 = segments[i].as_f64().unwrap_or(0.0) as f32;
            let y1 = 1.0 - segments[i + 1].as_f64().unwrap_or(0.0) as f32;
            let x2 = segments[i + 2].as_f64().unwrap_or(0.0) as f32;
            let y2 = 1.0 - segments[i + 3].as_f64().unwrap_or(0.0) as f32;

            vertices.push(LineVertex::new(x1, y1, color_array));
            vertices.push(LineVertex::new(x2, y2, color_array));
        }

        Ok(vertices)
    }

    /// 绘制路径（用于河流、等高线、边界）
    fn draw_paths(
        &self,
        data: &Value,
        color: Color,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> RenderResult<()> {
        let paths = data
            .as_array()
            .ok_or_else(|| RenderError::InvalidJsonFormat("路径应为数组".to_string()))?;

        if paths.is_empty() {
            return Ok(());
        }

        let vertices = self.parse_paths(paths, color)?;
        if vertices.is_empty() {
            return Ok(());
        }

        self.draw_line_vertices(&vertices, "Path", view, encoder);
        Ok(())
    }

    /// 解析路径数据
    fn parse_paths(&self, paths: &[Value], color: Color) -> RenderResult<Vec<LineVertex>> {
        let mut all_vertices = Vec::new();
        let color_array = color.to_array();

        for path in paths {
            let points = path
                .as_array()
                .ok_or_else(|| RenderError::InvalidJsonFormat("路径点应为数组".to_string()))?;

            if points.len() < 4 {
                continue;
            }

            for i in (0..points.len() - 2).step_by(2) {
                let x1 = points[i].as_f64().unwrap_or(0.0) as f32;
                let y1 = 1.0 - points[i + 1].as_f64().unwrap_or(0.0) as f32;
                let x2 = points[i + 2].as_f64().unwrap_or(0.0) as f32;
                let y2 = 1.0 - points[i + 3].as_f64().unwrap_or(0.0) as f32;

                all_vertices.push(LineVertex::new(x1, y1, color_array));
                all_vertices.push(LineVertex::new(x2, y2, color_array));
            }
        }

        Ok(all_vertices)
    }

    /// 绘制线条顶点（公共方法）
    fn draw_line_vertices(
        &self,
        vertices: &[LineVertex],
        label: &str,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Vertex Buffer", label)),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&format!("{} Pass", label)),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.line_pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..vertices.len() as u32, 0..1);
    }

    /// 绘制城市标记（双圆环）
    fn draw_city_markers(
        &self,
        data: &Value,
        outer_radius: f32,
        inner_radius: f32,
        marker_color: Color,
        background_color: Color,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> RenderResult<()> {
        let positions = data
            .as_array()
            .ok_or_else(|| RenderError::InvalidJsonFormat("城市位置应为数组".to_string()))?;

        if positions.is_empty() {
            return Ok(());
        }

        let mut vertices = Vec::new();

        for i in (0..positions.len()).step_by(2) {
            if i + 1 >= positions.len() {
                break;
            }

            let x = positions[i].as_f64().unwrap_or(0.0) as f32;
            let y = 1.0 - positions[i + 1].as_f64().unwrap_or(0.0) as f32;

            // 外圆
            vertices.extend(self.create_circle_vertices(
                x,
                y,
                outer_radius / self.width as f32,
                marker_color,
            ));

            // 内圆
            vertices.extend(self.create_circle_vertices(
                x,
                y,
                inner_radius / self.width as f32,
                background_color,
            ));
        }

        if !vertices.is_empty() {
            self.draw_circle_vertices(&vertices, "City Marker", view, encoder);
        }

        Ok(())
    }

    /// 绘制城镇标记（单圆）
    fn draw_town_markers(
        &self,
        data: &Value,
        radius: f32,
        color: Color,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> RenderResult<()> {
        let positions = data
            .as_array()
            .ok_or_else(|| RenderError::InvalidJsonFormat("城镇位置应为数组".to_string()))?;

        if positions.is_empty() {
            return Ok(());
        }

        let mut vertices = Vec::new();

        for i in (0..positions.len()).step_by(2) {
            if i + 1 >= positions.len() {
                break;
            }

            let x = positions[i].as_f64().unwrap_or(0.0) as f32;
            let y = 1.0 - positions[i + 1].as_f64().unwrap_or(0.0) as f32;

            vertices.extend(self.create_circle_vertices(x, y, radius / self.width as f32, color));
        }

        if !vertices.is_empty() {
            self.draw_circle_vertices(&vertices, "Town Marker", view, encoder);
        }

        Ok(())
    }

    /// 创建圆形顶点
    fn create_circle_vertices(
        &self,
        cx: f32,
        cy: f32,
        radius: f32,
        color: Color,
    ) -> Vec<CircleVertex> {
        let segments = self.circle_config.segments;
        let mut vertices = Vec::with_capacity((segments * 3) as usize);
        let color_array = color.to_array();

        for i in 0..segments {
            let angle1 = 2.0 * std::f32::consts::PI * i as f32 / segments as f32;
            let angle2 = 2.0 * std::f32::consts::PI * (i + 1) as f32 / segments as f32;

            // 中心点
            vertices.push(CircleVertex::new(cx, cy, color_array));
            // 第一个边缘点
            vertices.push(CircleVertex::new(
                cx + radius * angle1.cos(),
                cy + radius * angle1.sin(),
                color_array,
            ));
            // 第二个边缘点
            vertices.push(CircleVertex::new(
                cx + radius * angle2.cos(),
                cy + radius * angle2.sin(),
                color_array,
            ));
        }

        vertices
    }

    /// 绘制圆形顶点（公共方法）
    fn draw_circle_vertices(
        &self,
        vertices: &[CircleVertex],
        label: &str,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{} Vertex Buffer", label)),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&format!("{} Pass", label)),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.circle_pipeline);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.draw(0..vertices.len() as u32, 0..1);
    }

    /// 绘制文字标签（占位实现）
    fn draw_labels(
        &mut self,
        data: &Value,
        _color: Color,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> RenderResult<()> {
        let labels = data
            .as_array()
            .ok_or_else(|| RenderError::InvalidJsonFormat("标签应为数组".to_string()))?;

        for label in labels {
            let text = label["text"].as_str().unwrap_or("");
            let x = label["position"][0].as_f64().unwrap_or(0.0) as f32;
            let y = 1.0 - label["position"][1].as_f64().unwrap_or(0.0) as f32;
            let font_size = label["fontsize"].as_i64().unwrap_or(12) as f32;

            // 占位实现：不渲染文字
            self.text_renderer.draw_text(
                text,
                x * self.width as f32,
                y * self.height as f32,
                font_size,
                [0.0, 0.0, 0.0, 1.0],
                view,
                encoder,
            )?;
        }

        Ok(())
    }

    /// 保存为 PNG 文件
    ///
    /// 将渲染结果保存为 PNG 图像文件。
    ///
    /// # 参数
    /// * `path` - 输出文件路径
    ///
    /// # 错误
    /// - `RenderError::BufferMapFailed` - 缓冲区映射失败
    /// - `RenderError::ImageCreationFailed` - 图像创建失败
    /// - `RenderError::ImageSaveFailed` - 图像保存失败
    pub fn save_png(&self, path: &str) -> RenderResult<()> {
        // WGPU requires bytes_per_row to be aligned to COPY_BYTES_PER_ROW_ALIGNMENT (256)
        let unpadded_bytes_per_row = 4 * self.width;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = (unpadded_bytes_per_row + align - 1) / align * align;

        // 创建缓冲区用于读取纹理数据
        let buffer_size = (padded_bytes_per_row * self.height) as wgpu::BufferAddress;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Copy Encoder"),
            });

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(Some(encoder.finish()));

        // 读取缓冲区数据
        let buffer_slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv()
            .unwrap()
            .map_err(|_| RenderError::BufferMapFailed)?;

        let data = buffer_slice.get_mapped_range();

        // If there's padding, we need to remove it when creating the image
        let image = if padded_bytes_per_row != unpadded_bytes_per_row {
            let mut unpadded_data =
                Vec::with_capacity((unpadded_bytes_per_row * self.height) as usize);
            for row in 0..self.height {
                let start = (row * padded_bytes_per_row) as usize;
                let end = start + unpadded_bytes_per_row as usize;
                unpadded_data.extend_from_slice(&data[start..end]);
            }
            image::RgbaImage::from_raw(self.width, self.height, unpadded_data)
        } else {
            image::RgbaImage::from_raw(self.width, self.height, data.to_vec())
        };

        let image = image.ok_or(RenderError::ImageCreationFailed)?;
        image.save(path)?;

        Ok(())
    }
}
