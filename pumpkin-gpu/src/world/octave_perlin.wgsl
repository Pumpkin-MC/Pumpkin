// GPU port of pumpkin-util's OctavePerlinNoiseSampler (see
// pumpkin-util/src/noise/perlin.rs and pumpkin-util/src/math/mod.rs).
//
// Deliberately runs in f32, not f64 like the CPU reference: this is the accepted
// tradeoff for the GPU path (speed over bit-exact vanilla parity). Everything else
// mirrors the CPU algorithm term-for-term, including operation order, so the two
// implementations stay comparable and any divergence beyond f32/f64 precision is a bug.

struct OctaveParams {
    x_origin: f32,
    y_origin: f32,
    z_origin: f32,
    amplitude: f32,
    persistence: f32,
    lacunarity: f32,
    _pad0: f32,
    _pad1: f32,
}

struct Dims {
    num_points: u32,
    num_octaves: u32,
}

@group(0) @binding(0) var<uniform> dims: Dims;
@group(0) @binding(1) var<storage, read> octaves: array<OctaveParams>;
// Flattened [octave][0..256] permutation tables, one 256-entry table per octave.
@group(0) @binding(2) var<storage, read> permutations: array<u32>;
// Flattened [point][x,y,z].
@group(0) @binding(3) var<storage, read> points: array<f32>;
@group(0) @binding(4) var<storage, read_write> out_density: array<f32>;

const GRADIENTS: array<vec3<f32>, 16> = array<vec3<f32>, 16>(
    vec3<f32>(1.0, 1.0, 0.0), vec3<f32>(-1.0, 1.0, 0.0),
    vec3<f32>(1.0, -1.0, 0.0), vec3<f32>(-1.0, -1.0, 0.0),
    vec3<f32>(1.0, 0.0, 1.0), vec3<f32>(-1.0, 0.0, 1.0),
    vec3<f32>(1.0, 0.0, -1.0), vec3<f32>(-1.0, 0.0, -1.0),
    vec3<f32>(0.0, 1.0, 1.0), vec3<f32>(0.0, -1.0, 1.0),
    vec3<f32>(0.0, 1.0, -1.0), vec3<f32>(0.0, -1.0, -1.0),
    vec3<f32>(1.0, 1.0, 0.0), vec3<f32>(0.0, -1.0, 1.0),
    vec3<f32>(-1.0, 1.0, 0.0), vec3<f32>(0.0, -1.0, -1.0),
);

fn perlin_fade(t: f32) -> f32 {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn grad(hash: u32, x: f32, y: f32, z: f32) -> f32 {
    let g = GRADIENTS[hash & 15u];
    return g.x * x + g.y * y + g.z * z;
}

fn perm_at(octave_index: u32, input: i32) -> u32 {
    let idx = octave_index * 256u + u32(input & 0xFF);
    return permutations[idx];
}

fn lerp1(delta: f32, start: f32, end: f32) -> f32 {
    return start + delta * (end - start);
}

fn lerp2(dx: f32, dy: f32, v00: f32, v10: f32, v01: f32, v11: f32) -> f32 {
    return lerp1(dy, lerp1(dx, v00, v10), lerp1(dx, v01, v11));
}

fn lerp3(
    dx: f32, dy: f32, dz: f32,
    v000: f32, v100: f32, v010: f32, v110: f32,
    v001: f32, v101: f32, v011: f32, v111: f32,
) -> f32 {
    return lerp1(dz, lerp2(dx, dy, v000, v100, v010, v110), lerp2(dx, dy, v001, v101, v011, v111));
}

// Mirrors PerlinNoiseSampler::sample, specialized for the y_scale == 0 case (the only
// case OctavePerlinNoiseSampler::sample ever calls it with), so fade_local_y == local_y.
fn perlin_sample(octave_index: u32, x: f32, y: f32, z: f32) -> f32 {
    let x_floor = floor(x);
    let y_floor = floor(y);
    let z_floor = floor(z);

    let local_x = x - x_floor;
    let local_y = y - y_floor;
    let local_z = z - z_floor;

    let xi = i32(x_floor);
    let yi = i32(y_floor);
    let zi = i32(z_floor);

    let i = perm_at(octave_index, xi);
    let j = perm_at(octave_index, xi + 1);
    let k = perm_at(octave_index, i32(i) + yi);
    let l = perm_at(octave_index, i32(i) + yi + 1);
    let m = perm_at(octave_index, i32(j) + yi);
    let n = perm_at(octave_index, i32(j) + yi + 1);

    let d = grad(perm_at(octave_index, i32(k) + zi), local_x, local_y, local_z);
    let e = grad(perm_at(octave_index, i32(m) + zi), local_x - 1.0, local_y, local_z);
    let f = grad(perm_at(octave_index, i32(l) + zi), local_x, local_y - 1.0, local_z);
    let g = grad(perm_at(octave_index, i32(n) + zi), local_x - 1.0, local_y - 1.0, local_z);
    let h = grad(perm_at(octave_index, i32(k) + zi + 1), local_x, local_y, local_z - 1.0);
    let o = grad(perm_at(octave_index, i32(m) + zi + 1), local_x - 1.0, local_y, local_z - 1.0);
    let p = grad(perm_at(octave_index, i32(l) + zi + 1), local_x, local_y - 1.0, local_z - 1.0);
    let q = grad(perm_at(octave_index, i32(n) + zi + 1), local_x - 1.0, local_y - 1.0, local_z - 1.0);

    let r = perlin_fade(local_x);
    let s = perlin_fade(local_y);
    let t = perlin_fade(local_z);

    return lerp3(r, s, t, d, e, f, g, h, o, p, q);
}

fn maintain_precision(value: f32) -> f32 {
    let period = 3.3554432e7;
    return value - floor(value / period + 0.5) * period;
}

@compute @workgroup_size(64)
fn sample_octaves(@builtin(global_invocation_id) gid: vec3<u32>) {
    let point_index = gid.x;
    if (point_index >= dims.num_points) {
        return;
    }

    let px = points[point_index * 3u];
    let py = points[point_index * 3u + 1u];
    let pz = points[point_index * 3u + 2u];

    var total: f32 = 0.0;
    for (var oct: u32 = 0u; oct < dims.num_octaves; oct = oct + 1u) {
        let params = octaves[oct];

        let mapped_x = maintain_precision(px * params.lacunarity);
        let mapped_y = maintain_precision(py * params.lacunarity);
        let mapped_z = maintain_precision(pz * params.lacunarity);

        let sample = perlin_sample(
            oct,
            mapped_x + params.x_origin,
            mapped_y + params.y_origin,
            mapped_z + params.z_origin,
        );

        total = total + params.amplitude * sample * params.persistence;
    }

    out_density[point_index] = total;
}
