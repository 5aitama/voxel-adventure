struct Input {
    @builtin(global_invocation_id) pos: vec3<u32>,
}

struct Uniforms {
    /// The size of the render texture.
    texture_size: vec2<f32>,
}

@group(0) @binding(0)
var render_texture: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<uniform> uniforms: Uniforms;

@group(0) @binding(2)
var<storage, read_write> voxel_buf: array<u32>;

@group(0) @binding(3)
var<storage, read> octree_buf: array<u32>;

// fn slabs(ray: Ray, bounding_box: BoundingBox, hit_point: ptr<function, vec3<f32>>) -> bool {
//     if (all(ray.origin >= bounding_box.min && ray.origin < bounding_box.max)) {
//         return true;
//     }

//     let ta = (bounding_box.min - ray.origin - 0.0001) * ray.direction_inv;
//     let tb = (bounding_box.max - ray.origin - 0.0001) * ray.direction_inv;

//     let tmin = min(ta, tb);
//     let tmax = max(ta, tb);

//     // The distance at where the ray enter in the AABB
//     let ray_in = max(max(tmin.x, tmin.y), tmin.z);

//     // The distance at where the ray exit in the AABB
//     let ray_out = min(min(tmax.x, tmax.y), tmax.z);

//     // Recalculate the hit point
//     *hit_point = ray.origin + ray.direction * ray_in;

//     return ray_in < ray_out && ray_in >= 0.0;
// }

const CHUNK_SIZE = 64u;
const PI = 3.14159265359;

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
    direction_inv: vec3<f32>,
    length_inv: f32,
}

fn new_ray(origin: vec3<f32>, direction: vec3<f32>) -> Ray {
    return Ray(
        origin,
        direction,
        1.0 / direction,
        1.0 / length(direction)
    );
}

fn hasVoxel(pos: vec3<f32>, out_voxel: ptr<function, u32>) -> bool {
    // Check the boundary
    if (!all(pos >= vec3f(0.0) && pos < vec3f(CHUNK_SIZE))) {
        return false;
    }

    let pos_index = vec3<u32>(pos);
    let index = pos_index.x +
                pos_index.y * CHUNK_SIZE +
                pos_index.z * CHUNK_SIZE * CHUNK_SIZE;

    let voxel = (voxel_buf[index / 2] >> (16 * (index % 2))) & 0xFFFFu;

    if (voxel == 0) {
        return false;
    }

    (*out_voxel) = voxel;

    return true;
}

fn extract_voxel_color(voxel: u32) -> vec3<f32> {
    // Extracting each color components of the voxel
    // and normalize them. The color format is RGB565.
    let r = f32( voxel          & 0x1Fu) / f32(0x1Fu);
    let g = f32((voxel >> 0x5u) & 0x3Fu) / f32(0x3Fu);
    let b = f32((voxel >> 0xBu) & 0x1Fu) / f32(0x1Fu);

    // Normalize the color.
    return vec3f(r, g, b);
}

struct RayHit {
    /// The hit point.
    point: vec3<f32>,
    /// The normal at the hit point.
    normal: vec3<f32>,
    /// The voxel hited
    voxel: u32,
}

fn ray_trace(ray: Ray, hit: ptr<function, RayHit>, iteration: i32) -> bool {
    var mapPos     = vec3f(floor(ray.origin));
	let deltaDist  = abs(vec3f(length(ray.direction)) * ray.direction_inv);
	let rayStep    = vec3f(sign(ray.direction));
	var sideDist   = (sign(ray.direction) * (mapPos - ray.origin) + (sign(ray.direction) * 0.5) + 0.5) * deltaDist;
	var mask       = vec3f(0.0);
	var normal     = vec3f(0.0);
	var voxel      = 0u;
	let chunk_max  = vec3f(CHUNK_SIZE);
	let chunk_min  = vec3f(0.0);

	for (var i = 0; i < iteration; i++) {
		if (sideDist.x < sideDist.y) {
			if (sideDist.x < sideDist.z) {
				sideDist.x += deltaDist.x;
				mapPos.x += rayStep.x;
				mask = vec3f(1.0, 0.0, 0.0);
				normal = vec3f(-rayStep.x, 0.0, 0.0);
			} else {
				sideDist.z += deltaDist.z;
				mapPos.z += rayStep.z;
				mask = vec3f(0.0, 0.0, 1.0);
				normal = vec3f(0.0, 0.0, -rayStep.z);
			}
		} else {
			if (sideDist.y < sideDist.z) {
				sideDist.y += deltaDist.y;
				mapPos.y += rayStep.y;
				mask = vec3f(0.0, 1.0, 0.0);
				normal = vec3f(0.0, -rayStep.y, 0.0);
			} else {
				sideDist.z += deltaDist.z;
				mapPos.z += rayStep.z;
				mask = vec3f(0.0, 0.0, 1.0);
				normal = vec3f(0.0, 0.0, -rayStep.z);
			}
		}

		if (hasVoxel(mapPos, &voxel))  {
		    // The distance from the ray origin to the hit point.
		    let d = length(mask * (sideDist - deltaDist)) * ray.length_inv;
			// The hit point (in world space)
		    let point = (ray.origin + ray.direction * d);

			(*hit).point  = point;
			(*hit).normal = normal;
			(*hit).voxel  = voxel;

			return true;
		}
	}

	return false;
}

fn pcg3d(value: vec3<u32>) -> vec3<u32>{
    var v = value * 1664525u + 1013904223u;
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    v ^= v >> vec3u(16);
    v.x += v.y * v.z;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    return v;
}

fn random3(f: vec3<f32>) -> vec3<f32> {
    return bitcast<vec3<f32>>((pcg3d(bitcast<vec3<u32>>(f)) & vec3u(0x007FFFFFu)) | vec3u(0x3F800000u)) - 1.0;
}

/**
 * Generate a uniformly distributed random point on the unit-sphere.
 *
 * After:
 * http://mathworld.wolfram.com/SpherePointPicking.html
 */
fn randomSpherePoint(rand: vec3<f32>) -> vec3<f32> {
  let ang1 = (rand.x + 1.0) * PI; // [-1..1) -> [0..2*PI)
  let u = rand.y; // [-1..1), cos and acos(2v-1) cancel each other out, so we arrive at [-1..1)
  let u2 = u * u;
  let sqrt1MinusU2 = sqrt(1.0 - u2);
  let x = sqrt1MinusU2 * cos(ang1);
  let y = sqrt1MinusU2 * sin(ang1);
  let z = u;
  return vec3f(x, y, z);
}

/**
 * Generate a uniformly distributed random point on the unit-hemisphere
 * around the given normal vector.
 *
 * This function can be used to generate reflected rays for diffuse surfaces.
 * Actually, this function can be used to sample reflected rays for ANY surface
 * with an arbitrary BRDF correctly.
 * This is because we always need to solve the integral over the hemisphere of
 * a surface point by using numerical approximation using a sum of many
 * sample directions.
 * It is only with non-lambertian BRDF's that, in theory, we could sample them more
 * efficiently, if we knew in which direction the BRDF reflects the most energy.
 * This would be importance sampling, but care must be taken as to not over-estimate
 * those surfaces, because then our sum for the integral would be greater than the
 * integral itself. This is the inherent problem with importance sampling.
 *
 * The points are uniform over the sphere and NOT over the projected disk
 * of the sphere, so this function cannot be used when sampling a spherical
 * light, where we need to sample the projected surface of the light (i.e. disk)!
 */
fn randomHemispherePoint(rand: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
  /**
   * Generate random sphere point and swap vector along the normal, if it
   * points to the wrong of the two hemispheres.
   * This method provides a uniform distribution over the hemisphere,
   * provided that the sphere distribution is also uniform.
   */
  let v = randomSpherePoint(rand);
  return v * sign(dot(v, n));
}

@compute
@workgroup_size(16, 16)
fn main(input: Input) {
    var uv = (vec2f(input.pos.xy) / uniforms.texture_size.xy) * 2.0 - 1.0;
    let camera_dir = vec3f(0.0, 0.0, 0.8);
	let camera_plane_u = vec3f(1.0, 0.0, 0.0);
	let camera_plane_v = vec3f(0.0, 1.0, 0.0) * uniforms.texture_size.y / uniforms.texture_size.x;

	let ray_pos = vec3f(32.1, 48.1, -32.1);
	let ray_dir = camera_dir + uv.x * camera_plane_u + uv.y * camera_plane_v;

	// Create the ray.
	let ray = new_ray(ray_pos, ray_dir);
	var hit = RayHit();
	var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);

	let light = normalize(vec3f(-0.6, 0.8, -1.0));

	if (ray_trace(ray, &hit, 256)) {
	    let voxel_color = extract_voxel_color(hit.voxel);
		var local_shadow = max(dot(light, hit.normal), 0.1);

	    let ray_shadow = new_ray(hit.point, light);
		var ray_shadow_hit = RayHit();

		if (ray_trace(ray_shadow, &ray_shadow_hit, 256)) {
		    local_shadow = 0.1;
		}

		var acc = 0u;
		var ao_hit = RayHit();

		// Calculate the ambiant occlusion
		for (var i = 0; i < 64; i++) {
		    let random = random3(vec3f(vec2f(input.pos.xy), f32(i)));
		    let random_dir = randomHemispherePoint(random, hit.normal);
    		acc += u32(ray_trace(new_ray(hit.point, random_dir), &ao_hit, 16));
		}

		let m = (64.0 - f32(acc)) * 0.015625;

		color = vec4f(voxel_color * local_shadow * m, 1.0);
	}

    textureStore(render_texture, input.pos.xy, color);
}
