// Vertex shader

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) vert_col: vec3<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.vert_col = vec3<f32>(
        max(0.0, 1.0 - f32(in_vertex_index)), 
        max(0.0, min(f32(in_vertex_index), 2.0 - f32(in_vertex_index))),
        max(0.0, min(f32(in_vertex_index) - 1.0, 3.0 - f32(in_vertex_index))));
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.vert_col.r, in.vert_col.g, in.vert_col.b, 1.0);
}
