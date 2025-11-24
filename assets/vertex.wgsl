struct Camera { 
    view_projection_matrix: mat4x4<f32>
}

struct ModelMatrixPC {
    m: mat4x4<f32>
}

struct Light {
    position: vec3<f32>,
    color: vec3<f32>,
}

@group(1) @binding(0)
var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normals: vec3<f32>,
    @location(3) colors: vec4<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) position: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) normals: vec3<f32>,
};

/*@group(0) @binding(0)
var texture: texture_depth_2d;
@group(0) @binding(1)
var sampler_s: sampler_comparison; */
@group(0) @binding(0)
var<uniform> camera: Camera;
var<push_constant> model_matrix: ModelMatrixPC;

@vertex
fn vs_main(
    mesh: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = mesh.tex_coords;
    out.normals = mesh.normals;
    out.position = mesh.position;
    out.clip_position =  camera.view_projection_matrix * model_matrix.m * vec4<f32>(mesh.position, 1.0);
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var position = light.position;
    var color = light.color;
    var diffuse_power = 0.3;
    var distance = length(position);
    position = position / distance;
    distance = distance * distance;

    var NdotL = max(dot(position, in.normals), 0.0);
    var viewDir = normalize(-in.position);
    var diffuse_intensity = saturate(NdotL);
    var diffuse = diffuse_intensity * color * diffuse_power / distance;
    var H = normalize(position + viewDir);
    var NdotH = max(dot(H, in.normals), 0.0);
    var specular_intensity = pow(saturate(NdotH), 2.0);
    var specular = specular_intensity * color * 1.0 / distance;

    return vec4<f32>(specular, 1.0) + vec4(diffuse, 1.0) *  vec4<f32>(1.0, 1.0, 1.0, 1.0);
}