// Interpreter for the flattened density-function graph produced by graph.rs.
//
// The instruction list is topologically ordered (every node's inputs have lower
// indices), so a single linear pass evaluates the whole graph. Each invocation owns
// one sample point and walks the full instruction list for it.
//
// Intermediate values live in a storage buffer laid out as [node][point], not
// [point][node]: adjacent invocations then touch adjacent addresses on every step,
// which is what keeps the accesses coalesced.
//
// Opcode values must stay in sync with `OpCode` in graph.rs.

const OP_CONSTANT: u32 = 0u;
const OP_PASSTHROUGH: u32 = 1u;
const OP_LINEAR_ADD: u32 = 2u;
const OP_LINEAR_MUL: u32 = 3u;
const OP_UNARY_ABS: u32 = 4u;
const OP_UNARY_SQUARE: u32 = 5u;
const OP_UNARY_CUBE: u32 = 6u;
const OP_UNARY_HALF_NEGATIVE: u32 = 7u;
const OP_UNARY_QUARTER_NEGATIVE: u32 = 8u;
const OP_UNARY_SQUEEZE: u32 = 9u;
const OP_UNARY_INVERT: u32 = 10u;
const OP_CLAMP: u32 = 11u;
const OP_BINARY_ADD: u32 = 12u;
const OP_BINARY_MUL: u32 = 13u;
const OP_BINARY_MIN: u32 = 14u;
const OP_BINARY_MAX: u32 = 15u;
const OP_CLAMPED_Y_GRADIENT: u32 = 16u;
// Samples this node's own DoublePerlin sampler at the point scaled by
// (param0 = xz_scale, param1 = y_scale); input0 carries the sampler index.
const OP_NOISE: u32 = 17u;
const OP_SHIFT_A: u32 = 18u;
const OP_SHIFT_B: u32 = 19u;
const OP_SHIFTED_NOISE: u32 = 20u;
const OP_SPLINE: u32 = 21u;
const OP_INTERPOLATED_NOISE: u32 = 22u;
const OP_BEARDIFIER: u32 = 23u;
const OP_RANGE_CHOICE: u32 = 24u;
const OP_INTERVAL_SELECT: u32 = 25u;

struct Instruction {
    opcode: u32,
    input0: u32,
    input1: u32,
    input2: u32,
    sampler_index: u32,
    // Opcode-specific operands that are NOT scratch indices.
    aux0: u32,
    aux1: u32,
    _pad2: u32,
    param0: f32,
    param1: f32,
    param2: f32,
    param3: f32,
}

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

// One DoublePerlin sampler: two runs of octaves in the shared pool.
struct SamplerRef {
    first_start: u32,
    first_count: u32,
    second_start: u32,
    second_count: u32,
    amplitude: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

// vec3 is 16-byte aligned in WGSL, so the scalars are interleaved to fill the gaps
// that would otherwise be padding. Keep this layout in sync with GraphDims in lib.rs.
struct Dims {
    num_points: u32,
    num_instructions: u32,
    // Per-chunk beardifier inputs; see BeardifierData in graph.rs.
    num_structures: u32,
    num_junctions: u32,
    affected_min: vec3<i32>,
    has_affected_box: u32,
    affected_max: vec3<i32>,
    // 0 means "emit the last instruction only".
    num_outputs: u32,
}

struct BeardStructure {
    min: vec3<i32>,
    adaptation: u32,
    max: vec3<i32>,
    ground_level_delta: i32,
}

struct BeardJunction {
    x: i32,
    ground_y: i32,
    z: i32,
    _pad: i32,
}

@group(0) @binding(0) var<uniform> dims: Dims;
@group(0) @binding(1) var<storage, read> instructions: array<Instruction>;
// Flattened [point][x,y,z].
@group(0) @binding(2) var<storage, read> points: array<f32>;
// Scratch for intermediate node values, laid out [node * num_points + point].
@group(0) @binding(3) var<storage, read_write> scratch: array<f32>;
@group(0) @binding(4) var<storage, read_write> out_density: array<f32>;
@group(0) @binding(5) var<storage, read> samplers: array<SamplerRef>;
@group(0) @binding(6) var<storage, read> octaves: array<OctaveParams>;
// 256 entries per octave, in the same order as `octaves`.
@group(0) @binding(7) var<storage, read> permutations: array<u32>;
@group(0) @binding(8) var<storage, read> spline_points: array<SplinePoint>;
@group(0) @binding(9) var<storage, read> interpolated: array<InterpolatedRef>;
@group(0) @binding(10) var<storage, read> beard_structures: array<BeardStructure>;
@group(0) @binding(11) var<storage, read> beard_junctions: array<BeardJunction>;
@group(0) @binding(12) var<storage, read> interval_entries: array<IntervalEntry>;
// Instruction indices to emit, in the order the caller expects them.
@group(0) @binding(13) var<storage, read> outputs: array<u32>;

struct IntervalEntry {
    threshold: f32,
    function_node: u32,
}

struct InterpolatedRef {
    lower_start: u32,
    lower_count: u32,
    upper_start: u32,
    upper_count: u32,
    noise_start: u32,
    noise_count: u32,
    xz_multiplier: f32,
    y_multiplier: f32,
    xz_factor: f32,
    y_factor: f32,
    smear: f32,
    _pad: f32,
}

struct SplinePoint {
    location: f32,
    derivative: f32,
    value_node: u32,
    _pad: u32,
}

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
    return permutations[octave_index * 256u + u32(input & 0xFF)];
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

// Mirrors PerlinNoiseSampler::sample. `local_y` and `fade_local_y` differ only when a
// non-zero vertical scale quantized the Y component (see perlin_sample_no_fade).
fn perlin_core(
    octave_index: u32,
    xi: i32, yi: i32, zi: i32,
    local_x: f32, local_y: f32, local_z: f32,
    fade_local_y: f32,
) -> f32 {

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

    return lerp3(
        perlin_fade(local_x), perlin_fade(fade_local_y), perlin_fade(local_z),
        d, e, f, g, h, o, p, q,
    );
}

// Mirrors PerlinNoiseSampler::sample_no_fade, including the vertical quantization that
// only kicks in for a non-zero y_scale.
fn perlin_sample_no_fade(
    octave_index: u32,
    x: f32, y: f32, z: f32,
    y_scale: f32, y_max: f32,
) -> f32 {
    let params = octaves[octave_index];
    let true_x = x + params.x_origin;
    let true_y = y + params.y_origin;
    let true_z = z + params.z_origin;

    let x_floor = floor(true_x);
    let y_floor = floor(true_y);
    let z_floor = floor(true_z);

    let x_dec = true_x - x_floor;
    let y_dec = true_y - y_floor;
    let z_dec = true_z - z_floor;

    var y_noise: f32 = 0.0;
    if (y_scale != 0.0) {
        var raw_y_dec = y_dec;
        if (y_max >= 0.0 && y_max < y_dec) {
            raw_y_dec = y_max;
        }
        y_noise = floor(raw_y_dec / y_scale + 1e-7) * y_scale;
    }

    return perlin_core(
        octave_index,
        i32(x_floor), i32(y_floor), i32(z_floor),
        x_dec, y_dec - y_noise, z_dec,
        y_dec,
    );
}

// Sums one octave run against the fractions 1, 1/2, 1/4, ... applied in reverse octave
// order, as InterpolatedNoiseSampler::sample does.
fn interpolated_run(
    start: u32, count: u32,
    x: f32, y: f32, z: f32,
    y_scale: f32, y_max_base: f32,
) -> f32 {
    var total: f32 = 0.0;
    var fraction: f32 = 1.0;
    for (var i: u32 = 0u; i < count && i < 16u; i = i + 1u) {
        // Reverse order: fraction 1 pairs with the last octave.
        let octave_index = start + (count - 1u - i);
        total = total + perlin_sample_no_fade(
            octave_index,
            maintain_precision(x * fraction),
            maintain_precision(y * fraction),
            maintain_precision(z * fraction),
            y_scale * fraction,
            y_max_base * fraction,
        ) / fraction;
        fraction = fraction * 0.5;
    }
    return total;
}

// Mirrors InterpolatedNoiseSampler::sample in density_function/noise.rs.
fn interpolated_sample(index: u32, px: f32, py: f32, pz: f32) -> f32 {
    let s = interpolated[index];

    let d = px * s.xz_multiplier;
    let e = py * s.y_multiplier;
    let f = pz * s.xz_multiplier;

    let g = d / s.xz_factor;
    let h = e / s.y_factor;
    let i = f / s.xz_factor;

    let k = s.smear / s.y_factor;

    let n = interpolated_run(s.noise_start, s.noise_count, g, h, i, k, h);
    let q = (n / 10.0 + 1.0) * 0.5;

    var l: f32 = 0.0;
    if (q < 1.0) {
        l = interpolated_run(s.lower_start, s.lower_count, d, e, f, s.smear, e);
    }
    var m: f32 = 0.0;
    if (q > 0.0) {
        m = interpolated_run(s.upper_start, s.upper_count, d, e, f, s.smear, e);
    }

    return clamp_lerp(l / 512.0, m / 512.0, q) / 128.0;
}

// The 24x24x24 table vanilla precomputes is exactly this closed form, so it is
// evaluated directly instead of uploaded.
fn beard_contribution(dx: i32, dy: i32, dz: i32, y_to_ground: i32) -> f32 {
    let xi = dx + 12;
    let yi = dy + 12;
    let zi = dz + 12;
    if (xi < 0 || xi >= 24 || yi < 0 || yi >= 24 || zi < 0 || zi >= 24) {
        return 0.0;
    }

    let fx = f32(dx);
    let fz = f32(dz);
    let dy_with_offset = f32(y_to_ground) + 0.5;
    let distance_sqr = fx * fx + dy_with_offset * dy_with_offset + fz * fz;
    let value = -dy_with_offset / sqrt(distance_sqr / 2.0) / 2.0;

    // Uses `dy`, while `value` uses `y_to_ground`; they differ for BeardBox.
    let kernel_dy = f32(dy) + 0.5;
    let kernel_sqr = fx * fx + kernel_dy * kernel_dy + fz * fz;
    return value * exp(-kernel_sqr / 16.0);
}

// Equivalent to Mth.clampedMap(distance, 0, 6, 1, 0).
fn bury_contribution(dx: f32, dy: f32, dz: f32) -> f32 {
    let distance = sqrt(dx * dx + dy * dy + dz * dz);
    if (distance < 0.0) { return 1.0; }
    if (distance > 6.0) { return 0.0; }
    return 1.0 - distance / 6.0;
}

// Mirrors Beardifier::sample in density_function/beardifier.rs.
fn beardifier_sample(x: i32, y: i32, z: i32) -> f32 {
    if (dims.has_affected_box == 0u) {
        return 0.0;
    }
    let lo = dims.affected_min;
    let hi = dims.affected_max;
    if (x < lo.x || x > hi.x || y < lo.y || y > hi.y || z < lo.z || z > hi.z) {
        return 0.0;
    }

    var weight: f32 = 0.0;

    for (var i: u32 = 0u; i < dims.num_structures; i = i + 1u) {
        let s = beard_structures[i];

        let dx = max(0, max(s.min.x - x, x - s.max.x));
        let dz = max(0, max(s.min.z - z, z - s.max.z));
        let ground_y = s.min.y + s.ground_level_delta;
        let dy_to_ground = y - ground_y;

        var dy: i32 = 0;
        if (s.adaptation == 1u || s.adaptation == 3u) {
            dy = dy_to_ground;
        } else if (s.adaptation == 2u) {
            dy = max(0, max(ground_y - y, y - s.max.y));
        } else if (s.adaptation == 4u) {
            dy = max(0, max(s.min.y - y, y - s.max.y));
        }

        if (s.adaptation == 1u || s.adaptation == 2u) {
            weight = weight + beard_contribution(dx, dy, dz, dy_to_ground) * 0.8;
        } else if (s.adaptation == 3u) {
            weight = weight + bury_contribution(f32(dx), f32(dy) / 2.0, f32(dz));
        } else if (s.adaptation == 4u) {
            weight = weight
                + bury_contribution(f32(dx) / 2.0, f32(dy) / 2.0, f32(dz) / 2.0) * 0.8;
        }
    }

    for (var i: u32 = 0u; i < dims.num_junctions; i = i + 1u) {
        let j = beard_junctions[i];
        let dy = y - j.ground_y;
        weight = weight + beard_contribution(x - j.x, dy, z - j.z, dy) * 0.4;
    }

    return weight;
}

fn clamp_lerp(start: f32, end: f32, delta: f32) -> f32 {
    if (delta < 0.0) { return start; }
    if (delta > 1.0) { return end; }
    return lerp1(delta, start, end);
}

fn maintain_precision(value: f32) -> f32 {
    let period = 3.3554432e7;
    return value - floor(value / period + 0.5) * period;
}

fn octave_sample(start: u32, count: u32, x: f32, y: f32, z: f32) -> f32 {
    var total: f32 = 0.0;
    for (var i: u32 = 0u; i < count; i = i + 1u) {
        let octave_index = start + i;
        let params = octaves[octave_index];

        let sample = perlin_sample_no_fade(
            octave_index,
            maintain_precision(x * params.lacunarity),
            maintain_precision(y * params.lacunarity),
            maintain_precision(z * params.lacunarity),
            0.0,
            0.0,
        );
        total = total + params.amplitude * sample * params.persistence;
    }
    return total;
}

// Mirrors DoublePerlinNoiseSampler::sample.
fn double_perlin_sample(sampler_index: u32, x: f32, y: f32, z: f32) -> f32 {
    let s = samplers[sampler_index];
    let scale = 1.0181268882175227;
    let first = octave_sample(s.first_start, s.first_count, x, y, z);
    let second = octave_sample(s.second_start, s.second_count, x * scale, y * scale, z * scale);
    return (first + second) * s.amplitude;
}

fn spline_outside_range(knot: SplinePoint, location: f32, last_known: f32) -> f32 {
    if (knot.derivative == 0.0) {
        return last_known;
    }
    return knot.derivative * (location - knot.location) + last_known;
}

// Mirrors Spline::sample in density_function/spline.rs. Knot values are read from
// already-computed instruction outputs, so this never recurses.
fn sample_spline(start: u32, count: u32, location: f32, point_index: u32) -> f32 {
    if (count == 0u) {
        return 0.0;
    }

    // Equivalent to partition_point(|p| location >= p.location); locations ascend.
    var above: u32 = 0u;
    for (var k: u32 = 0u; k < count; k = k + 1u) {
        if (location >= spline_points[start + k].location) {
            above = k + 1u;
        }
    }

    if (above == 0u) {
        let knot = spline_points[start];
        let value = scratch[knot.value_node * dims.num_points + point_index];
        return spline_outside_range(knot, location, value);
    }
    if (above == count) {
        let knot = spline_points[start + count - 1u];
        let value = scratch[knot.value_node * dims.num_points + point_index];
        return spline_outside_range(knot, location, value);
    }

    let lower = spline_points[start + above - 1u];
    let upper = spline_points[start + above];
    let lower_value = scratch[lower.value_node * dims.num_points + point_index];
    let upper_value = scratch[upper.value_node * dims.num_points + point_index];

    let dist = upper.location - lower.location;
    let x_scale = (location - lower.location) / dist;

    let delta = upper_value - lower_value;
    let extrapolated_lower = lower.derivative * dist - delta;
    let extrapolated_upper = -upper.derivative * dist + delta;

    let cubic = (x_scale * (1.0 - x_scale))
        * lerp1(x_scale, extrapolated_lower, extrapolated_upper);
    return cubic + lerp1(x_scale, lower_value, upper_value);
}

// Mirrors shift_sample_3d in density_function/noise.rs.
fn shift_sample_3d(sampler_index: u32, x: f32, y: f32, z: f32) -> f32 {
    return double_perlin_sample(sampler_index, x * 0.25, y * 0.25, z * 0.25) * 4.0;
}

// f32 mirror of pumpkin_util::math::clamped_map.
fn clamped_map(value: f32, old_start: f32, old_end: f32, new_start: f32, new_end: f32) -> f32 {
    let delta = (value - old_start) / (old_end - old_start);
    if (delta < 0.0) {
        return new_start;
    }
    if (delta > 1.0) {
        return new_end;
    }
    return new_start + delta * (new_end - new_start);
}

@compute @workgroup_size(64)
fn evaluate_graph(@builtin(global_invocation_id) gid: vec3<u32>) {
    let point_index = gid.x;
    if (point_index >= dims.num_points) {
        return;
    }

    let px = points[point_index * 3u];
    let py = points[point_index * 3u + 1u];
    let pz = points[point_index * 3u + 2u];

    for (var i: u32 = 0u; i < dims.num_instructions; i = i + 1u) {
        let instruction = instructions[i];
        let a = scratch[instruction.input0 * dims.num_points + point_index];
        let b = scratch[instruction.input1 * dims.num_points + point_index];
        let c = scratch[instruction.input2 * dims.num_points + point_index];

        var result: f32 = 0.0;
        switch instruction.opcode {
            case 0u: { result = instruction.param0; }
            case 1u: { result = a; }
            case 2u: { result = a + instruction.param0; }
            case 3u: { result = a * instruction.param0; }
            case 4u: { result = abs(a); }
            case 5u: { result = a * a; }
            case 6u: { result = a * a * a; }
            case 7u: {
                if (a > 0.0) { result = a; } else { result = a * 0.5; }
            }
            case 8u: {
                if (a > 0.0) { result = a; } else { result = a * 0.25; }
            }
            case 9u: {
                let c = clamp(a, -1.0, 1.0);
                result = c / 2.0 - c * c * c / 24.0;
            }
            case 10u: {
                // Matches the CPU path's 1/0 -> +inf rather than clamping it away.
                if (a == 0.0) {
                    result = bitcast<f32>(0x7f800000u);
                } else {
                    result = 1.0 / a;
                }
            }
            case 11u: { result = clamp(a, instruction.param0, instruction.param1); }
            case 12u: { result = a + b; }
            case 13u: {
                // Vanilla short-circuits on a == 0 so the second input is never sampled;
                // reproduced here so 0 * inf stays 0 instead of becoming NaN.
                if (a == 0.0) { result = 0.0; } else { result = a * b; }
            }
            case 14u: { result = min(a, b); }
            case 15u: { result = max(a, b); }
            case 16u: {
                result = clamped_map(
                    py,
                    instruction.param0,
                    instruction.param1,
                    instruction.param2,
                    instruction.param3,
                );
            }
            case 17u: {
                result = double_perlin_sample(
                    instruction.sampler_index,
                    px * instruction.param0,
                    py * instruction.param1,
                    pz * instruction.param0,
                );
            }
            case 18u: {
                result = shift_sample_3d(instruction.sampler_index, px, 0.0, pz);
            }
            case 19u: {
                // Vanilla passes (z, x, 0) here, not (x, y, z); the rotation is deliberate.
                result = shift_sample_3d(instruction.sampler_index, pz, px, 0.0);
            }
            case 20u: {
                result = double_perlin_sample(
                    instruction.sampler_index,
                    px * instruction.param0 + a,
                    py * instruction.param1 + b,
                    pz * instruction.param0 + c,
                );
            }
            case 21u: {
                result = sample_spline(instruction.aux0, instruction.aux1, a, point_index);
            }
            case 22u: {
                result = interpolated_sample(instruction.sampler_index, px, py, pz);
            }
            case 23u: {
                result = beardifier_sample(i32(px), i32(py), i32(pz));
            }
            case 24u: {
                if (instruction.param0 <= a && a < instruction.param1) {
                    result = b;
                } else {
                    result = c;
                }
            }
            case 25u: {
                // The last entry has an infinite threshold, so this always matches.
                result = 0.0;
                for (var k: u32 = 0u; k < instruction.aux1; k = k + 1u) {
                    let entry = interval_entries[instruction.aux0 + k];
                    if (a < entry.threshold) {
                        result = scratch[entry.function_node * dims.num_points + point_index];
                        break;
                    }
                }
            }
            default: { result = 0.0; }
        }

        scratch[i * dims.num_points + point_index] = result;
    }

    if (dims.num_outputs == 0u) {
        out_density[point_index] =
            scratch[(dims.num_instructions - 1u) * dims.num_points + point_index];
        return;
    }

    // Results are grouped by output, matching the [output][point] layout the reader
    // expects, so each output's values stay contiguous.
    for (var i: u32 = 0u; i < dims.num_outputs; i = i + 1u) {
        out_density[i * dims.num_points + point_index] =
            scratch[outputs[i] * dims.num_points + point_index];
    }
}
