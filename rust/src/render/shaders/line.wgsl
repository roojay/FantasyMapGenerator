// 线条渲染着色器
//
// 用于绘制河流、等高线、边界等线条元素

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // 将归一化坐标 [0, 1] 转换为 NDC [-1, 1]
    let ndc_x = input.position.x * 2.0 - 1.0;
    let ndc_y = input.position.y * 2.0 - 1.0;
    
    output.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    output.color = input.color;
    
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
