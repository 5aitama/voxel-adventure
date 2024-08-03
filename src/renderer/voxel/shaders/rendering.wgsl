@group(0) @binding(0)
var in_tex: texture_2d<f32>;

@group(0) @binding(1)
var in_tex_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct FragmentInput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> FragmentInput {
    var output: FragmentInput;

    output.clip_position = vec4<f32>(input.position, 0.0, 1.0);
    output.uv = input.uv;

    return output;
}

@fragment
fn fs_main(input: FragmentInput) -> @location(0) vec4<f32> {
    return textureSample(in_tex, in_tex_sampler, input.uv);
}