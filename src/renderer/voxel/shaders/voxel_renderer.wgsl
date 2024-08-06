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
fn bb_new(min: vec3<f32>, max: vec3<f32>) -> BoundingBox {
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

@group(0) @binding(3)
var<storage, read> octree_buf: array<u32>;

fn get_octree(index: u32) -> u32 {
    return (octree_buf[index / 4u] >> ((index % 4u) * 8u)) & 0xffu;
}

fn slabs(ray: Ray, bounding_box: BoundingBox, hit_point: ptr<function, vec3<f32>>) -> bool {
    if (all(ray.origin >= bounding_box.min && ray.origin < bounding_box.max)) {
        return true;
    }

    let ta = (bounding_box.min - ray.origin - 0.0001) * ray.direction_inv;
    let tb = (bounding_box.max - ray.origin - 0.0001) * ray.direction_inv;

    let tmin = min(ta, tb);
    let tmax = max(ta, tb);

    // The distance at where the ray enter in the AABB
    let ray_in = max(max(tmin.x, tmin.y), tmin.z);

    // The distance at where the ray exit in the AABB
    let ray_out = min(min(tmax.x, tmax.y), tmax.z);

    // Recalculate the hit point
    *hit_point = ray.origin + ray.direction * ray_in;

    return ray_in < ray_out && ray_in >= 0.0;
}

fn contains(b: BoundingBox, p: vec3<f32>) -> bool {
    return all(p >= b.min && p < b.max);
}

fn subdivide(b: BoundingBox) -> array<BoundingBox, 8> {
    let s = b.max / 2.0;

    let offsets = array<vec3<f32>, 8>(
        vec3f(0.0, 0.0, 0.0),
        vec3f(0.0, 0.0, s.z),
        vec3f(0.0, s.y, 0.0),
        vec3f(0.0, s.y, s.z),

        vec3f(s.x, 0.0, 0.0),
        vec3f(s.x, 0.0, s.z),
        vec3f(s.x, s.y, 0.0),
        vec3f(s.x, s.y, s.z),
    );

    let childs = array<BoundingBox, 8>(
        BoundingBox(b.min + offsets[0], offsets[0] + s),
        BoundingBox(b.min + offsets[1], offsets[1] + s),
        BoundingBox(b.min + offsets[2], offsets[2] + s),
        BoundingBox(b.min + offsets[3], offsets[3] + s),

        BoundingBox(b.min + offsets[4], offsets[4] + s),
        BoundingBox(b.min + offsets[5], offsets[5] + s),
        BoundingBox(b.min + offsets[6], offsets[6] + s),
        BoundingBox(b.min + offsets[7], offsets[7] + s),
    );

    return childs;
}

fn to_index(pos: vec3<f32>) -> u32 {
    let p = vec3<u32>(pos);
    return p.x + p.y * 128 + p.z * 128 * 128;
}

@compute
@workgroup_size(16, 16)
fn main(input: Input) {
    let uv = (2.0 * vec2f(input.pos.xy) - uniforms.texture_size) / uniforms.texture_size.y;

	let rayPos = vec3<f32>(32.0, 64.0, -64.0);
	let rayDir = (vec4<f32>(normalize(vec3f(uv, 1.0)), 1.0)).xyz;

	var bounding_box = BoundingBox(vec3<f32>(0.0), vec3<f32>(256.0));

	var ray: Ray = Ray(rayPos, rayDir, 1.0 / rayDir);
	var hit_point = vec3<f32>(0.0, 0.0, 0.0);

	var color = vec4<f32>(0.0);
	let point = vec3<f32>(0.0);
	var octree_offset: u32 = 0;

	for(var i = 0; i <= 16; i++) {
	    let size = (bounding_box.max - bounding_box.min);

		if(size.x == 1.0) {
            color = vec4<f32>(vec3<f32>(8.0 / f32(i)), 1.0);
            break;
		}

		let s = subdivide(bounding_box);

		if (slabs(ray, s[0], &hit_point)) {
		    bounding_box = s[0];
			continue;
		}

		color = vec4<f32>(vec3<f32>(f32(i) / 16.0), 1.0);

		break;
		// The ray hit the block s[..]
		// if (slabs(ray, s[0], &hit_point)) {
		//     if (((get_octree(octree_offset) >> 0u) & 1u) == 0u) { break; }
		// 	octree_offset = octree_offset * 8u + 1u;
		// 	bounding_box = s[0];
		// }

		// else if (slabs(ray, s[1], &hit_point)) {
		//     if (((get_octree(octree_offset) >> 1u) & 0x1u) == 0u) { break; }
		// 	octree_offset = octree_offset * 8u + 2u;
		// 	bounding_box = s[1];
		// 	continue;
		// }

		// else if (slabs(ray, s[2], &hit_point)) {
		//     if (((get_octree(octree_offset) >> 2u) & 0x1u) == 0u) { break; }
		// 	octree_offset = octree_offset * 8u + 3u;
		// 	bounding_box = s[2];
		// 	continue;
		// }

		// else if (slabs(ray, s[3], &hit_point)) {
		//     if (((get_octree(octree_offset) >> 3u) & 0x1u) == 0u) { break; }
		// 	octree_offset = octree_offset * 8u + 4u;
		// 	bounding_box = s[3];
		// 	continue;
		// }

		// else if (slabs(ray, s[4], &hit_point)) {
		//     if (((get_octree(octree_offset) >> 4u) & 0x1u) == 0u) { break; }
		// 	octree_offset = octree_offset * 8u + 5u;
		// 	bounding_box = s[4];
		// 	continue;
		// }

		// else if (slabs(ray, s[5], &hit_point)) {
		//     if (((get_octree(octree_offset) >> 5u) & 0x1u) == 0u) { break; }
		// 	octree_offset = octree_offset * 8u + 6u;
		// 	bounding_box = s[5];
		// 	continue;
		// }

		// else if (slabs(ray, s[6], &hit_point)) {
		//     if (((get_octree(octree_offset) >> 6u) & 0x1u) == 0u) { break; }
		// 	octree_offset = octree_offset * 8u + 7u;
		// 	bounding_box = s[6];
		// 	continue;
		// }

		// else if (slabs(ray, s[7], &hit_point)) {
		//     if (((get_octree(octree_offset) >> 7u) & 0x1u) == 0u) { break; }
		// 	octree_offset = octree_offset * 8u + 8u;
		// 	bounding_box = s[7];
		// 	continue;
		// }

		// else {
		//     // break;
		// }

        // if (contains(s[0], point) && slabs(ray, s[0], &hit_point)) { bounding_box = s[0]; continue; }
        // if (contains(s[1], point) && slabs(ray, s[1], &hit_point)) { bounding_box = s[1]; continue; }
        // if (contains(s[2], point) && slabs(ray, s[2], &hit_point)) { bounding_box = s[2]; continue; }
        // if (contains(s[3], point) && slabs(ray, s[3], &hit_point)) { bounding_box = s[3]; continue; }
        // if (contains(s[4], point) && slabs(ray, s[4], &hit_point)) { bounding_box = s[4]; continue; }
        // if (contains(s[5], point) && slabs(ray, s[5], &hit_point)) { bounding_box = s[5]; continue; }
        // if (contains(s[6], point) && slabs(ray, s[6], &hit_point)) { bounding_box = s[6]; continue; }
        // if (contains(s[7], point) && slabs(ray, s[7], &hit_point)) { bounding_box = s[7]; continue; }
        // break;
	}

    textureStore(render_texture, input.pos.xy, color);
}
