struct Camera {
    view_projection_matrix: mat4x4<f32>
}

struct Model {
    model_matrix: mat4x4<f32>,
}

struct Transform {
    translation: vec3<f32>,
    rotation: vec4<f32>,
    scale: vec3<f32>
}

struct Light {
    transform: Transform,
    color: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;
@group(1) @binding(0)
var<uniform> light: Light;
@group(2) @binding(0)
var<uniform> model: Model;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normals: vec3<f32>,
    @location(3) colors: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    input_vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    var scale: f32 = 0.25;
    out.clip_position = camera.view_projection_matrix * model.model_matrix * vec4<f32>(input_vertex.position * scale + light.transform.translation, 1.0);
    out.color = light.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
