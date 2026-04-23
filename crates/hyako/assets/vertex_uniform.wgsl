struct Camera {
    view_projection_matrix: mat4x4<f32>
}

struct Model {
    model_matrix: mat4x4<f32>,
}

struct Material {
    base_color_factor: vec4<f32>,
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

@group(1) @binding(0)
var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normals: vec3<f32>,
    @location(3) colors: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) position: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) normals: vec3<f32>,
    @location(4) colors: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: Camera;
@group(2) @binding(0)
var<uniform> model: Model;
@group(3) @binding(0)
var<uniform> material: Material;
@group(3) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(3) @binding(2)
var base_color_sampler: sampler;

@vertex
fn vs_main(
    mesh: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = mesh.tex_coords;
    out.normals = mesh.normals;
    out.position = mesh.position;
    out.clip_position = camera.view_projection_matrix * model.model_matrix * vec4<f32>(mesh.position, 1.0);
    out.colors = mesh.colors;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = light.transform.translation;
    var color = light.color;
    var diffuse_power = 0.3;
    var distance = length(position);
    position = position / distance;
    distance = distance * distance;

    var NdotL = max(dot(position, in.normals), 0.0);
    var viewDir = normalize(-in.position);
    var diffuse_intensity = clamp(NdotL, 0.0, 1.0);
    var diffuse = diffuse_intensity * color * diffuse_power / distance;
    var H = normalize(position + viewDir);
    var NdotH = max(dot(H, in.normals), 0.0);
    var specular_intensity = pow(clamp(NdotH, 0.0, 1.0), 2.0);
    var specular = specular_intensity * color * 1.0 / distance;
    let sampled_base_color = textureSample(base_color_texture, base_color_sampler, in.tex_coords);
    let base_color = in.colors * material.base_color_factor * sampled_base_color;
    return vec4<f32>(specular, 1.0) + vec4(diffuse, 1.0) * base_color;
}
