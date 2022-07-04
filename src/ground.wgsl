struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

struct FrameUniform {
    frame: u32,
};
@group(0) @binding(0)
var<uniform> frame: FrameUniform;

struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) position: vec3<f32>,
};

// Vertex Shader
@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    out.color = model.color;
    let offset = vec3<f32>(0.0, 0.0, f32(frame.frame) * (0.03 * 4.0));
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position + offset, 1.0);
    out.position = out.clip_position.xyz;
    return out;
}

// Fragment Shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.1, 1.0, 1.0 - ((in.position.z - 10.0) * 0.1));
}
