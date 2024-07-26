struct Vertex {
    @location(0) position: vec3<f32>,
    @location(1) i_color: vec4<f32>,
};

struct VertexOutput {
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    out.color = vertex.color;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}