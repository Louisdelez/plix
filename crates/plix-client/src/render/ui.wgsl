// Plix UI shader for screen-space 2D rendering
// T031: UI vertex/fragment shader for screen-space quads

struct VertexInput {
    @location(0) position: vec2<f32>,  // Screen position in clip space (-1 to 1)
    @location(1) color: vec4<f32>,     // RGBA color with alpha
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Position is already in clip space (-1 to 1), just add z=0 and w=1
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
