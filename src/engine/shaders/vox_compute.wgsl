struct Uniforms {
    /// The size (in pixels) of the surface on wich the rendering happend.
    screen_size: vec2<u32>,
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

    let nori = vec3<f32>(
        f32(origin.x >= 0.0),
        f32(origin.y >= 0.0),
        f32(origin.z >= 0.0),
    ) * 2.0 - 1.0;

    let dir: vec3<f32> = max(abs(direction), vec3<f32>(0.0001)) * ndir;
    let ori: vec3<f32> = max(abs(origin), vec3<f32>(0.0001)) * nori;

    return Ray(ori, dir, 1.0 / dir);
}

/// Represent a voxel.
struct Voxel {
    /// The minimum bounding edge.
    min: vec3<f32>,
    /// The maximum bounding edge.
    max: vec3<f32>,
    /// The center position of the current [`Voxel`].
    center: vec3<f32>,
    /// The size of the current [`Voxel`].
    size: f32,
}

/// Create a new [`Voxel`]
///
/// # Arguments
///
/// * `center` - The center position of the [`Voxel`].
/// * `size` - The size of the [`Voxel`].
///
fn voxel_new(center: vec3<f32>, size: f32) -> Voxel {
    let extend = vec3<f32>(size * 0.5);
    return Voxel(center - extend, center + extend, center, size);
}

/// Same as the original `sign(...)` function but instead of returning `0`
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

/// Calculate the distance of the point at where the given [`Ray`] enter
/// and exit the given [`Voxel`].
///
/// # Arguments
///
/// * `ray` - The ray.
/// * `voxel` - The voxel.
///
/// # Returns
///
/// A [`vec2<f32>`] where the `x` component is the distance at
/// where the given [`Ray`] enter in the given [`Voxel`] and the
/// `y` component is the distance at where the given [`Ray`] exit
/// the given [`Voxel`].
///
/// ## Note
///
/// You can know if the given [`Ray`] hit the given [`Voxel`] by
/// comparing the `x` and `y` component of the returned value. So
/// if `x` > `y` it means that the given [`Ray`] don't hit the given
/// [`Voxel`] otherwise the given [`Ray`] hit the given [`Voxel`].
///
fn compute_distance(ray: Ray, voxel: Voxel) -> vec2<f32> {
    let v_min = (voxel.min - voxel.center) * i_sign(ray.direction) + voxel.center;
    let v_max = (voxel.max - voxel.center) * i_sign(ray.direction) + voxel.center;

    let c_min = (v_min - ray.origin) * ray.inverse_direction;
    let c_max = (v_max - ray.origin) * ray.inverse_direction;

    let t_min = min(i_max(c_min), i_max(c_max));
    let t_max = max(i_min(c_min), i_min(c_max));

    return vec2<f32>(t_min, t_max);
}

/**
 * Return the normalized direction to march in from the eye point for a single pixel.
 *
 * fieldOfView: vertical field of view in degrees
 * size: resolution of the output image
 * fragCoord: the x,y coordinate of the pixel in the output image
 */
fn ray_direction(field_of_view: f32, size: vec2<f32>, frag_coord: vec2<f32>) -> vec3<f32> {
    let xy = frag_coord - size / 2.0;
    let z = size.y / tan(radians(field_of_view) / 2.0);
    return normalize(vec3<f32>(xy, z));
}


fn node_dir(node: Node, point: vec3<f32>) -> vec3<f32> {
    let dir = vec3<f32>(
        f32(point.x >= node.center.x),
        f32(point.y >= node.center.y),
        f32(point.z >= node.center.z),
    ) * 2.0 - 1.0;

    return dir;
}

/// Represent a [`Node`].
struct Node {
    /// The center of the [`Node`] in 3D space.
    center: vec3<f32>,
    /// The size of the [`Node`].
    size: f32,
}

/// Represent a [`Ray`] [`Hit`].
struct Hit {
    /// The distance of the point at where
    /// the [`Ray`] enter.
    t_min: f32,
    /// The distance of the point at where
    /// the [`Ray`] exit.
    t_max: f32,
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
fn ray_hit_node(ray: Ray, node: Node, hit: ptr<function, Hit>) -> bool {
    let v_min = (node.size * -0.5) * i_sign(ray.direction) + node.center;
    let v_max = (node.size *  0.5) * i_sign(ray.direction) + node.center;

    let c_min = (v_min - ray.origin) * ray.inverse_direction;
    let c_max = (v_max - ray.origin) * ray.inverse_direction;

    let t_min = min(i_max(c_min), i_max(c_max));
    let t_max = max(i_min(c_min), i_min(c_max));

    *hit = Hit(t_min, t_max);

    return t_min < t_max;
}

@group(0) @binding(0)
/// The uniforms.
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
/// The output texture.
var out_tex: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(2)
var<storage, read_write> stack_buf: array<Stack>;

@compute
@workgroup_size(16, 16)
/// The entry point for the voxel compute shader.
///
/// # Arguments
///
/// * `screen` - The position of the pixel.
fn main(@builtin(global_invocation_id) screen: vec3<u32>) {
    let ray_pos = vec3<f32>(-128.0, 0.0, -200.0);
    let ray_dir = ray_direction(120.0, vec2<f32>(uniforms.screen_size), vec2<f32>(screen.xy));
    let ray = ray_new(ray_pos, ray_dir);

    let node_min_size = 1.0;
    
    var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    var current_node: Node = node_new(
        vec3<f32>(0.0),
        128.0
    );

    var current_node_hit = 

    var voxel: Voxel = voxel_new(vec3<f32>(0.0, 0.0, 0.0), 128.0);
    var dist: vec2<f32> = compute_distance(ray, voxel);
    var stack_index = 0;
    var popped = false;

    // If the ray don't hit the voxel we can
    // return the sky color.
    if (dist.x >= dist.y) {
        color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
        textureStore(out_tex, screen.xy, color);
        return;
    }

    for (var i = 0; i < 1000000; i++) {
        
        // The first step to do is to retrieve the
        // voxel that the ray hit. Then from the hited
        // voxel we calculate in wich subvoxel the ray
        // hit is, until we reach a minimum subvoxel size.

        // The position at where the ray hit `voxel`
        let hit_pos = ray.origin + ray.direction * dist.x;
        // Calculate the center position of the subvoxel at where the ray hit.
        let sub_voxel_center = subvoxel_center_pos(hit_pos, voxel);

        if (voxel.size > max_size && !popped) {
            // Create a new voxel wich is the hitted subvoxel.
            let sub_voxel: Voxel = voxel_new(sub_voxel_center, voxel.size * 0.5);

            // Push the actual voxel and dist into the stack.
            stack_buf[stack_index] = Stack(voxel, dist);
            stack_index ++;

            // Set the actual voxel to the subvoxel
            voxel = sub_voxel;
            // Set the actual dist to the subvoxel dist.
            dist = compute_distance(ray, voxel);

            continue;
        }

        if (!popped) {
            if (voxel.size == 1.0) {
                color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
            }
        }

        if (dist.y >= stack_buf[stack_index - 2].dist.y) {

        }

        break;

        // if (dist.y >= stack_buf[stack_index - 1].dist.y) {
        //     popped = false;

        //     let parent = stack_buf[stack_index - 1];
        //     stack_index --;

        //     if (stack_index == 0) {
        //         break;
        //     }

        //     let pmax = ray.origin + ray.direction * parent.dist.y + 0.001;
        //     let next_voxel_dir = vec3<f32>(
        //         f32(pmax.x >= stack_buf[stack_index - 1].voxel.center.x),
        //         f32(pmax.y >= stack_buf[stack_index - 1].voxel.center.y),
        //         f32(pmax.z >= stack_buf[stack_index - 1].voxel.center.z),
        //     ) * 2.0 - 1.0;
        //     let next_voxel_center =
        //         stack_buf[stack_index - 1].voxel.center + next_voxel_dir * parent.voxel.size * 0.5;

        //     if (next_voxel_center.x == parent.voxel.center.x &&
        //         next_voxel_center.y == parent.voxel.center.y &&
        //         next_voxel_center.z == parent.voxel.center.z)
        //     {
        //         popped = true;
        //     }

        //     let next_voxel = voxel_new(next_voxel_center, parent.voxel.size);
        //     voxel = next_voxel;
        //     dist = compute_distance(ray, voxel);

        //     continue;
        // }

        // let pmax = ray.origin + ray.direction * dist.y + 0.001;
        // let next_voxel_dir = vec3<f32>(
        //     f32(pmax.x >= stack_buf[stack_index - 1].voxel.center.x),
        //     f32(pmax.y >= stack_buf[stack_index - 1].voxel.center.y),
        //     f32(pmax.z >= stack_buf[stack_index - 1].voxel.center.z),
        // ) * 2.0 - 1.0;


        // let next_voxel_center = stack_buf[stack_index - 1].voxel.center + next_voxel_dir * voxel.size * 0.5;
        // voxel = voxel_new(next_voxel_center, voxel.size);
        // dist = compute_distance(ray, voxel);
    }

    textureStore(out_tex, screen.xy, color);
}
