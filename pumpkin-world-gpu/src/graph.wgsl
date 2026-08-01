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

struct Instruction {
    opcode: u32,
    input0: u32,
    input1: u32,
    _pad: u32,
    param0: f32,
    param1: f32,
    param2: f32,
    param3: f32,
}

struct Dims {
    num_points: u32,
    num_instructions: u32,
}

@group(0) @binding(0) var<uniform> dims: Dims;
@group(0) @binding(1) var<storage, read> instructions: array<Instruction>;
// Flattened [point][x,y,z].
@group(0) @binding(2) var<storage, read> points: array<f32>;
// Scratch for intermediate node values, laid out [node * num_points + point].
@group(0) @binding(3) var<storage, read_write> scratch: array<f32>;
@group(0) @binding(4) var<storage, read_write> out_density: array<f32>;

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

    let py = points[point_index * 3u + 1u];

    for (var i: u32 = 0u; i < dims.num_instructions; i = i + 1u) {
        let instruction = instructions[i];
        let a = scratch[instruction.input0 * dims.num_points + point_index];
        let b = scratch[instruction.input1 * dims.num_points + point_index];

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
            default: { result = 0.0; }
        }

        scratch[i * dims.num_points + point_index] = result;
    }

    out_density[point_index] = scratch[(dims.num_instructions - 1u) * dims.num_points + point_index];
}
