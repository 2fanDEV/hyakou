struct Camera {
    view_projection_matrix: mat4x4<f32>
}

struct PushConstants {
    model_matrix: mat4x4<f32>,      // bytes 0-64 (vertex stage)
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
var<push_constant> pc: PushConstants;
@group(1) @binding(0)
var<uniform> light: Light;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    //this is just for testing purposes
    var scale: f32 = 0.25;
    out.clip_position =  camera.view_projection_matrix * pc.model_matrix * vec4<f32>(model.position * scale + light.transform.translation, 1.0);
    out.color = light.color;
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}