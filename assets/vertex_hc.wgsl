struct Uniform {
    view_projection_matrix: mat4x4<f32>
}

struct ModelMatrixPC {
    m: mat4x4<f32>
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normals: vec3<f32>,
    @location(3) colors: vec4<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@group(0) @binding(0)
var texture: texture_depth_2d;
@group(0) @binding(1)
var sampler_s: sampler_comparison;
@group(0) @binding(2)
var<uniform> uniforms: Uniform;
var<push_constant> model_matrix: ModelMatrixPC;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position =  uniforms.view_projection_matrix * model_matrix.m * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv: vec2<f32> = in.tex_coords; 

    let reference_depth: f32 = 0.5; 
    
    let visibility: f32 = textureSampleCompare(
        texture, 
        sampler_s, 
        uv, 
        reference_depth 
    );
    
    return vec4<f32>(visibility, visibility, visibility, 1.0);
}