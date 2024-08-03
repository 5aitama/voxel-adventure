struct Input {
    @builtin(global_invocation_id) pos: vec3<u32>,
}

struct Uniforms {
    /// The size of the render texture.
    texture_size: vec2<f32>,
}

struct BoundingBox {
    /// The minimum edge of the bounding box.
    min: vec3<f32>,
    /// The maximum edge of the bounding box.
    max: vec3<f32>,
}

/// Create a new `BoundingBox`.
///
/// # Argument
///
/// * `min` - The minimum bounding box edge.
/// * `max` - The maximum bounding box edge.
fn bb_new(min: vec3<f32>, max: vec3<f32>) {
    return BoundingBox(min, max);
}

struct Ray {
    /// The ray origin.
    origin: vec3<f32>,
    /// The ray direction.
    direction: vec3<f32>,
    /// The inverse ray direction.
    direction_inv: vec3<f32>,
}

/// Create a new `Ray`
/// 
/// # Arguments
///
/// `origin` - The ray origin.
/// `direction` - The ray direction.
fn ray_new(origin: vec3<f32>, direction: vec3<f32>) -> Ray {
    return Ray(origin, direction, 1.0 / direction);
}

@group(0) @binding(0)
var render_texture: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<uniform> uniforms: Uniforms;

@group(0) @binding(2)
var<storage, read_write> voxel_buf: array<u32>;

fn slabs()

@compute
@workgroup_size(16, 16)
fn main(input: Input) {
    let normalized_screen_pos = vec2f(input.pos.xy) / uniforms.texture_size;
    textureStore(render_texture, input.pos.xy, vec4f(normalized_screen_pos, 0.0, 1.0));
}