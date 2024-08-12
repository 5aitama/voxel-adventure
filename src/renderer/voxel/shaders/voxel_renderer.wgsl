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

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
    direction_inv: vec3<f32>,
}

@compute
@workgroup_size(16, 16)
fn main(input: Input) {
    var uv = (vec2f(input.pos.xy) / uniforms.texture_size.xy) * 2.0 - 1.0;
    let camera_dir = vec3f(0.0, 0.0, 0.8);
	let camera_plane_u = vec3f(1.0, 0.0, 0.0);
	let camera_plane_v = vec3f(0.0, 1.0, 0.0) * uniforms.texture_size.y / uniforms.texture_size.x;

	let ray_pos = vec3f(5.1, 3.5, -10.1);
	let ray_dir = camera_dir + uv.x * camera_plane_u + uv.y * camera_plane_v;

	// Create the ray.
	let ray = Ray(ray_pos, ray_dir, 1.0 / ray_dir);
	var color = vec4<f32>(0.025, 0.025, 0.025, 1.0);

	var mapPos = vec3f(floor(ray.origin));
	var deltaDist = abs(vec3f(length(ray.direction)) * ray.direction_inv);
	var rayStep = vec3f(sign(ray.direction));
	var sideDist = (sign(ray.direction) * (mapPos - ray.origin) + (sign(ray.direction) * 0.5) + 0.5) * deltaDist;
	var mask = vec3f(0.0);
	var normal = vec3f(0.0);
	var dst = 0.0;

	let light = vec3f(5.0, 10.0, -10.0);

	for (var i = 0; i < 128; i++) {
		if (
		    all(mapPos == vec3f( 0.0,  0.0,  0.0)) ||
			all(mapPos == vec3f( 0.0,  1.0,  0.0)) ||
			all(mapPos == vec3f( 1.0,  0.0,  0.0)) ||
			all(mapPos == vec3f( 0.0,  0.0,  1.0)) ||
			all(mapPos == vec3f(-1.0,  0.0,  0.0))
		)  {
		    let d = length(mask * (sideDist - deltaDist)) / length(ray.direction);
		    let hit = (ray.origin + ray.direction * d) - mapPos;
		    let c = hit * dot(normalize(normal), normalize(light - hit));
			color = vec4f(c, 1.0);

			break;
		}

		if (sideDist.x < sideDist.y) {
			if (sideDist.x < sideDist.z) {
				sideDist.x += deltaDist.x;
				mapPos.x += rayStep.x;
				mask = vec3f(1.0, 0.0, 0.0);
				normal = vec3f(-rayStep.x, 0.0, 0.0);
				dst += deltaDist.x;
			}
			else {
				sideDist.z += deltaDist.z;
				mapPos.z += rayStep.z;
				mask = vec3f(0.0, 0.0, 1.0);
				normal = vec3f(0.0, 0.0, -rayStep.z);
				dst += deltaDist.z;
			}
		}
		else {
			if (sideDist.y < sideDist.z) {
				sideDist.y += deltaDist.y;
				mapPos.y += rayStep.y;
				mask = vec3f(0.0, 1.0, 0.0);
				normal = vec3f(0.0, -rayStep.y, 0.0);
				dst += deltaDist.y;
			}
			else {
				sideDist.z += deltaDist.z;
				mapPos.z += rayStep.z;
				mask = vec3f(0.0, 0.0, 1.0);
				normal = vec3f(0.0, 0.0, -rayStep.z);
				dst += deltaDist.z;
			}
		}
	}

    textureStore(render_texture, input.pos.xy, color);
}
