struct Node {
    center: vec3<f32>,
    size: f32,
}

/// Represent a [`Ray`] [`Hit`].
struct Hit {
    /// The distance of the point at where the [`Ray`] enter.
    t_min: f32,
    /// The distance of the point at where the [`Ray`] exit.
    t_max: f32,
}

struct Stack {
    node: Node,
    hit: Hit,
}

/// Represent a Ray.
struct Ray {
    /// The position at where the [`Ray`] start.
    origin: vec3<f32>,
    /// The direction *(normalized)* of the [`Ray`].
    direction: vec3<f32>,
    /// The inverse direction *(normalized)* of the [`Ray`].
    inverse_direction: vec3<f32>,
}

const BIAS: f32 = 0.01;

/// Create a new [`Ray`].
///
/// # Arguments
///
/// * `origin` - The position at where the [`Ray`] start.
/// * `direction` - The direction *(normalized)* of the [`Ray`].
///
fn ray_new(origin: vec3<f32>, direction: vec3<f32>) -> Ray {
    let ndir = vec3<f32>(
        f32(direction.x >= 0.0),
        f32(direction.y >= 0.0),
        f32(direction.z >= 0.0),
    ) * 2.0 - 1.0;

    let dir: vec3<f32> = max(abs(direction), vec3<f32>(0.01)) * ndir;
    return Ray(origin, dir, 1.0 / dir);
}

// Same as the original `sign(...)` function but instead of returning `0`
/// if a component is equal to `0` it will return `-1`.
///
/// # Arguments
///
/// * `v` - The [`vec3<f32>`].
///
fn i_sign(v: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        f32(v.x >= 0.0),
        f32(v.y >= 0.0),
        f32(v.z >= 0.0)
    ) * 2.0 - 1.0;
}

/// Return the largest component of a given [`vec3<f32>`].
///
/// # Arguments
///
/// * `v` - The [`vec3<f32>`].
///
fn i_max(v: vec3<f32>) -> f32 {
    return max(v.x, max(v.y, v.z));
}

/// Return the lowest component of a given [`vec3<f32>`].
///
/// # Arguments
///
/// * `v` - The [`vec3<f32>`].
///
fn i_min(v: vec3<f32>) -> f32 {
    return min(v.x, min(v.y, v.z));
}


/// Test if a given [`Ray`] hit a given [`Node`]. If the test passed it
/// will write the result into the given [`Hit`] structure.
///
/// # Arguments
///
/// * `ray` - The ray.
/// * `node` - The node.
/// * `hit` - A pointer to the hit result.
///
fn calculate_time(ray: Ray, center: vec3<f32>, size: f32) -> vec2f {
    let s = size * 0.5;
    let ray_dir_sign = i_sign(ray.direction);

    let v_min = -s * ray_dir_sign + center;
    let v_max =  s * ray_dir_sign + center;

    let c_min = (v_min - ray.origin) * ray.inverse_direction;
    let c_max = (v_max - ray.origin) * ray.inverse_direction;

    let t_min = min(i_max(c_min), i_max(c_max));
    let t_max = max(i_min(c_min), i_min(c_max));

    return vec2f(t_min, t_max);
}

const FOV = 120.0;
const PREC_Z = 1.0 / tan(radians(FOV) * 0.5);

/// Compute the ray direction from fragment coordinates.
///
/// # Arguments
///
/// * `fov` - The field of view in degrees.
/// * `size` - The size of the surface in pixels.
/// * `frag_coord` - The fragment coordinates.
///
fn ray_direction(size: vec2<f32>, frag_coord: vec2<f32>) -> vec3<f32> {
    let xy = frag_coord - size * 0.5;
    let z = size.y * PREC_Z;
    return normalize(vec3<f32>(xy, z));
}

struct Uniforms {
    /// The size (in pixels) of the surface on wich the rendering happend.
    screen_size: vec2<u32>,
}

/// The uniforms.
@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

/// The output texture.
@group(0) @binding(1)
var out_tex: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(2)
var<storage> sparse_voxel_octree_data: array<u32>;

const NODE_MIN_SIZE = 8.0;

fn read_svo_byte(offset: u32) -> u32 {
    let ipos: u32 = offset / 4u;
    let val_u32: u32 = sparse_voxel_octree_data[ipos];
    let shift: u32 = 8u * (offset % 4u);
    return (val_u32 >> shift) & 0xFFu;
}

fn direction_to_index(dir: vec3<f32>) -> u32 {
    let bits = vec3<u32>(dir > vec3f(0.0));
    return (bits.x | (bits.y << 1) | (bits.z << 2));
}

fn svo_read(offset: u32) -> bool {
    let value: u32 = sparse_voxel_octree_data[ offset / 32u];
    let shift: u32 = offset % 32u;
    return ((value >> shift) & 0x1u) == 0x1u;
}

const ROOT_NODE_SIZE: u32 = 512u;
const ROOT_NODE_DEPTH: u32 = 6u;

/// The entry point for the voxel compute shader.
///
/// # Arguments
///
/// * `screen` - The position of the pixel.
@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) screen: vec3<u32>, @builtin(local_invocation_id) thread_id: vec3<u32>) {
    // The actual ray.
    let ray = ray_new(
        vec3<f32>(-380.0, 100.0, -800.0),
        ray_direction(vec2<f32>(uniforms.screen_size), vec2<f32>(screen.xy)),
    );

    // The output color.
    var color = vec3<f32>(0.0);

    // The stack contains all previous nodes.
    var stack: array<Node, (ROOT_NODE_DEPTH + 1u)>;
    var stack_len = 0;

    var offset = 0u;
    var offsets: array<u32, ROOT_NODE_DEPTH>;
    var offsets_len = 0;

    stack[0] = Node(vec3f(0.01, 0.01, 0.01), f32(ROOT_NODE_SIZE));
    stack_len ++;

    var dist = calculate_time(ray, stack[0].center, stack[0].size);

    // var dist = calculate_time(ray, root_node.center, root_node.size);
    let root_dist = dist;
    var depth = 0u;

    // Mean that the ray don't hit the root node.
    if (dist.x + BIAS >= dist.y) {
        textureStore(out_tex, screen.xy, vec4f(vec3f(0.0), 1.0));
        return;
    }

    loop {
        let cur_node = stack[stack_len - 1];
        // The position of the point at where the ray hit the current voxel.
        let p_in = ray.origin + ray.direction * (dist.x + BIAS);
        // The direction of the next voxel (child) calculated from the current voxel (parent).
        let p_dir = vec3f(p_in >= cur_node.center) * 2.0 - 1.0;
        // The center position of the next voxel (child).
        let p_center = cur_node.center + p_dir * cur_node.size * 0.25;
        // The distance at where the ray exit the next voxel (child).
        let t_max = calculate_time(ray, p_center, cur_node.size * 0.5).y;

        // The index of the next voxel (child) calculated from the current voxel (parent).
        let idx = direction_to_index(p_dir);

        // Calculate the offset of the next voxel (child) in the SVO.
        let local_offset = (1u << (3u * (ROOT_NODE_DEPTH - depth)) - 1u) / 7u;
        // The offset of the next voxel (child) in the SVO.
        let new_offset = offset + local_offset * idx;

        // This operation is the PUSH operation. It consist of just set the
        // next voxel (child) as the current voxel (parent) for the next
        // iteration loop. A PUSH operation can be executed if the distance
        // at where the ray exit the current voxel (parent) is not equal to the
        // maximum distance (t_max) and the offset of the next voxel (child)
        // in the SVO was set to 0x1.
        if (dist.x < t_max && svo_read(new_offset)) {

            let child_node =  Node(p_center, cur_node.size * 0.5);
            let is_transparent = true;

            if (child_node.size == 8.0) {
                if (is_transparent) {
                    color = (cur_node.center + 255.5 - 8.0) * 0.001953125;
                } else {
                    // 1 / 512 = 0.001953125
                    color = (cur_node.center + 255.5 - 8.0) * 0.001953125;
                    break;
                }
            } else {
                stack[stack_len] =  child_node;
                dist.y = calculate_time(ray, p_center, cur_node.size * 0.5).y;
                stack_len ++;

                depth ++;

                offset = new_offset;
                offsets[offsets_len] = new_offset - local_offset * idx;
                offsets_len ++;
                offset ++;

                continue;
            }
        }

        dist.x = t_max;

        if (dist.x + BIAS >= root_dist.y) {
            break;
        }

        if (stack_len > 1) {
            offset = offsets[offsets_len - 1];
            offsets_len --;
            stack_len --;
            depth --;
        }
    }

    textureStore(out_tex, screen.xy, vec4<f32>(color, 1.0));
}
