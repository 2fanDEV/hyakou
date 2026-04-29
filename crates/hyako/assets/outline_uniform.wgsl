struct Camera {
    view_projection_matrix: mat4x4<f32>
}

struct Model {
    model_matrix: mat4x4<f32>,
}

struct Outline {
    color: vec4<f32>,
    thickness: f32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normals: vec3<f32>,
    @location(3) colors: vec4<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;
@group(1) @binding(0)
var<uniform> model: Model;
@group(2) @binding(0)
var<uniform> outline: Outline;

@vertex
fn vs_main(mesh: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    var normal = vec3<f32>(0.0, 0.0, 1.0);
    let normal_length = length(mesh.normals);
    if normal_length > 0.0001 {
        normal = mesh.normals / normal_length;
    }
    let expanded_position = mesh.position + normal * outline.thickness;
    out.clip_position = camera.view_projection_matrix * model.model_matrix * vec4<f32>(expanded_position, 1.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return outline.color;
}
