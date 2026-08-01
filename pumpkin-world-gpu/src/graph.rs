//! Flattened density-function graph, ready to be evaluated by `graph.wgsl`.
//!
//! Pumpkin already stores density functions as a flat, topologically ordered array
//! (`BaseNoiseRouter::full_component_stack`), where each node references its inputs by
//! index rather than by pointer. That maps almost directly onto a GPU-friendly
//! instruction list, so this module lowers that array into [`Instruction`]s instead of
//! inventing a new flattening pass.
//!
//! Not every node type is supported yet — [`compile`] reports the first one it cannot
//! lower rather than silently emitting something wrong.

use crate::OctaveParams;
use bytemuck::{Pod, Zeroable};
use pumpkin_data::{
    chunk::DoublePerlinNoiseParameters,
    noise_router::{BaseNoiseFunctionComponent, BinaryOperation, LinearOperation, UnaryOperation},
};
use pumpkin_util::{
    noise::perlin::OctavePerlinNoiseSampler, random::xoroshiro128::XoroshiroSplitter,
};
use pumpkin_world::generation::noise::router::proto_noise_router::DoublePerlinNoiseBuilder;

/// Opcodes understood by `graph.wgsl`. Values must stay in sync with the `OP_*`
/// constants in that shader.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum OpCode {
    Constant = 0,
    /// Copies its input; used for cache/interpolator wrappers, which exist only to
    /// avoid recomputation on the CPU and do not affect the value.
    PassThrough = 1,
    LinearAdd = 2,
    LinearMul = 3,
    UnaryAbs = 4,
    UnarySquare = 5,
    UnaryCube = 6,
    UnaryHalfNegative = 7,
    UnaryQuarterNegative = 8,
    UnarySqueeze = 9,
    UnaryInvert = 10,
    Clamp = 11,
    BinaryAdd = 12,
    BinaryMul = 13,
    BinaryMin = 14,
    BinaryMax = 15,
    ClampedYGradient = 16,
    /// Samples this node's own sampler at the point scaled by `param0`/`param1`
    /// (xz/y scale).
    Noise = 17,
    /// Vanilla's `shift_sample_3d(sampler, x, 0, z)`.
    ShiftA = 18,
    /// Vanilla's `shift_sample_3d(sampler, z, x, 0)` — note the rotated arguments.
    ShiftB = 19,
    /// Samples this node's sampler at the point offset by three input nodes;
    /// `input0`/`input1`/`input2` are the x/y/z offsets.
    ShiftedNoise = 20,
}

/// One node of the flattened graph.
///
/// `input0`..`input2` index earlier entries in the same instruction list; unused inputs
/// are set to the node's own index so a buggy read stays in bounds instead of reading
/// past the buffer. `sampler_index` points into the graph's [`SamplerPool`] for the
/// noise-family opcodes and is ignored otherwise.
///
/// Laid out as 48 bytes so `array<Instruction>` stays 16-byte aligned in WGSL; a
/// whole overworld graph is only ~15 KB, so the padding is not worth packing away.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Instruction {
    pub opcode: u32,
    pub input0: u32,
    pub input1: u32,
    pub input2: u32,
    pub sampler_index: u32,
    padding0: u32,
    padding1: u32,
    padding2: u32,
    pub param0: f32,
    pub param1: f32,
    pub param2: f32,
    pub param3: f32,
}

impl Instruction {
    /// Builds a [`OpCode::Noise`] instruction sampling `sampler_index`.
    #[must_use]
    pub const fn noise(index: usize, sampler_index: u32, xz_scale: f32, y_scale: f32) -> Self {
        let mut instruction = Self::new(OpCode::Noise, index);
        instruction.sampler_index = sampler_index;
        instruction.param0 = xz_scale;
        instruction.param1 = y_scale;
        instruction
    }

    /// Builds a bare instruction for a given opcode. Test-only: production callers go
    /// through [`compile`], which also wires up inputs and sampler state.
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn new_for_test(opcode: OpCode, index: usize) -> Self {
        Self::new(opcode, index)
    }

    const fn new(opcode: OpCode, index: usize) -> Self {
        let own = index as u32;
        Self {
            opcode: opcode as u32,
            input0: own,
            input1: own,
            input2: own,
            sampler_index: 0,
            padding0: 0,
            padding1: 0,
            padding2: 0,
            param0: 0.0,
            param1: 0.0,
            param2: 0.0,
            param3: 0.0,
        }
    }
}

/// GPU-side descriptor of one `DoublePerlinNoiseSampler`: two runs of octaves in the
/// shared pool, plus the amplitude applied to their sum.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct SamplerRef {
    pub first_start: u32,
    pub first_count: u32,
    pub second_start: u32,
    pub second_count: u32,
    pub amplitude: f32,
    padding0: f32,
    padding1: f32,
    padding2: f32,
}

/// Every noise sampler referenced by a compiled graph, flattened into GPU-uploadable
/// tables.
///
/// Each `Noise`-family node owns its own seeded sampler, so instructions index into
/// [`Self::samplers`] rather than carrying sampler state inline.
#[derive(Default, Debug)]
pub struct SamplerPool {
    pub samplers: Vec<SamplerRef>,
    /// Octave parameters for every sampler, concatenated.
    pub octaves: Vec<OctaveParams>,
    /// 256-entry permutation table per octave, concatenated in the same order.
    pub permutations: Vec<u32>,
}

impl SamplerPool {
    /// Appends a double-Perlin sampler, returning the index instructions should use.
    pub fn push_double_perlin(
        &mut self,
        first: &OctavePerlinNoiseSampler,
        second: &OctavePerlinNoiseSampler,
        amplitude: f64,
    ) -> u32 {
        let first_start = self.push_octaves(first);
        let second_start = self.push_octaves(second);

        let index = self.samplers.len() as u32;
        self.samplers.push(SamplerRef {
            first_start,
            first_count: first.samplers.len() as u32,
            second_start,
            second_count: second.samplers.len() as u32,
            amplitude: amplitude as f32,
            padding0: 0.0,
            padding1: 0.0,
            padding2: 0.0,
        });
        index
    }

    fn push_octaves(&mut self, sampler: &OctavePerlinNoiseSampler) -> u32 {
        let start = self.octaves.len() as u32;
        for data in &sampler.samplers {
            let (x_origin, y_origin, z_origin) = data.sampler.origin();
            self.octaves.push(OctaveParams::new(
                x_origin as f32,
                y_origin as f32,
                z_origin as f32,
                data.amplitude as f32,
                data.persistence as f32,
                data.lacunarity as f32,
            ));
            self.permutations
                .extend(data.sampler.permutation().iter().map(|&b| u32::from(b)));
        }
        start
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.samplers.is_empty()
    }
}

/// A density-function node that [`compile`] does not know how to lower yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedNode {
    pub index: usize,
    pub name: &'static str,
}

impl std::fmt::Display for UnsupportedNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "density function node {} at index {} is not supported by the GPU backend yet",
            self.name, self.index
        )
    }
}

impl std::error::Error for UnsupportedNode {}

/// A compiled graph plus the sampler tables its instructions index into.
#[derive(Debug)]
pub struct CompiledGraph {
    pub instructions: Vec<Instruction>,
    pub samplers: SamplerPool,
}

/// Lowers a component stack into GPU instructions, building each noise node's sampler
/// from `base_random_deriver` exactly as `ProtoNoiseRouter` does on the CPU, so both
/// paths use the same seeded state.
///
/// # Errors
/// Returns [`UnsupportedNode`] for the first node whose type has no opcode yet, so
/// callers can fall back to the CPU path instead of generating wrong terrain.
pub fn compile(
    stack: &[BaseNoiseFunctionComponent],
    base_random_deriver: &XoroshiroSplitter,
) -> Result<CompiledGraph, UnsupportedNode> {
    let mut out = Vec::with_capacity(stack.len());
    let mut samplers = SamplerPool::default();

    for (index, component) in stack.iter().enumerate() {
        if let Some(instruction) =
            lower_noise_family(component, index, &mut samplers, base_random_deriver)
        {
            out.push(instruction);
            continue;
        }

        let instruction = match component {
            BaseNoiseFunctionComponent::Constant { value } => {
                let mut i = Instruction::new(OpCode::Constant, index);
                i.param0 = *value as f32;
                i
            }
            // Mirrors proto_noise_router.rs: while pumpkin-world has no blender, these
            // collapse to constants. If the blender lands, both paths must change together.
            BaseNoiseFunctionComponent::BlendAlpha => {
                let mut i = Instruction::new(OpCode::Constant, index);
                i.param0 = 1.0;
                i
            }
            BaseNoiseFunctionComponent::BlendOffset => {
                let mut i = Instruction::new(OpCode::Constant, index);
                i.param0 = 0.0;
                i
            }
            // Wrappers are pure caching in the CPU implementation, so on the GPU —
            // where every point is recomputed in parallel anyway — they are identity.
            BaseNoiseFunctionComponent::Wrapper { input_index, .. }
            | BaseNoiseFunctionComponent::BlendDensity { input_index } => {
                let mut i = Instruction::new(OpCode::PassThrough, index);
                i.input0 = *input_index as u32;
                i
            }
            BaseNoiseFunctionComponent::Linear { input_index, data } => {
                let op = match data.operation {
                    LinearOperation::Add => OpCode::LinearAdd,
                    LinearOperation::Mul => OpCode::LinearMul,
                };
                let mut i = Instruction::new(op, index);
                i.input0 = *input_index as u32;
                i.param0 = data.argument as f32;
                i
            }
            BaseNoiseFunctionComponent::Unary { input_index, data } => {
                let op = match data.operation {
                    UnaryOperation::Abs => OpCode::UnaryAbs,
                    UnaryOperation::Square => OpCode::UnarySquare,
                    UnaryOperation::Cube => OpCode::UnaryCube,
                    UnaryOperation::HalfNegative => OpCode::UnaryHalfNegative,
                    UnaryOperation::QuarterNegative => OpCode::UnaryQuarterNegative,
                    UnaryOperation::Squeeze => OpCode::UnarySqueeze,
                    UnaryOperation::Invert => OpCode::UnaryInvert,
                };
                let mut i = Instruction::new(op, index);
                i.input0 = *input_index as u32;
                i
            }
            BaseNoiseFunctionComponent::Clamp { input_index, data } => {
                let mut i = Instruction::new(OpCode::Clamp, index);
                i.input0 = *input_index as u32;
                i.param0 = data.min_value as f32;
                i.param1 = data.max_value as f32;
                i
            }
            BaseNoiseFunctionComponent::Binary {
                argument1_index,
                argument2_index,
                data,
            } => {
                let op = match data.operation {
                    BinaryOperation::Add => OpCode::BinaryAdd,
                    BinaryOperation::Mul => OpCode::BinaryMul,
                    BinaryOperation::Min => OpCode::BinaryMin,
                    BinaryOperation::Max => OpCode::BinaryMax,
                };
                let mut i = Instruction::new(op, index);
                i.input0 = *argument1_index as u32;
                i.input1 = *argument2_index as u32;
                i
            }
            BaseNoiseFunctionComponent::ClampedYGradient { data } => {
                let mut i = Instruction::new(OpCode::ClampedYGradient, index);
                i.param0 = data.from_y as f32;
                i.param1 = data.to_y as f32;
                i.param2 = data.from_value as f32;
                i.param3 = data.to_value as f32;
                i
            }
            other => {
                return Err(UnsupportedNode {
                    index,
                    name: node_name(other),
                });
            }
        };
        out.push(instruction);
    }

    Ok(CompiledGraph {
        instructions: out,
        samplers,
    })
}

/// Lowers the noise-family nodes, which all need a seeded sampler registered in
/// `samplers`. Returns `None` for any other node type.
fn lower_noise_family(
    component: &BaseNoiseFunctionComponent,
    index: usize,
    samplers: &mut SamplerPool,
    base_random_deriver: &XoroshiroSplitter,
) -> Option<Instruction> {
    let instruction = match component {
        BaseNoiseFunctionComponent::Noise { data } => {
            let sampler_index = push_sampler(samplers, base_random_deriver, &data.noise_id);
            Instruction::noise(
                index,
                sampler_index,
                data.xz_scale as f32,
                data.y_scale as f32,
            )
        }
        BaseNoiseFunctionComponent::ShiftA { noise_id } => {
            let mut i = Instruction::new(OpCode::ShiftA, index);
            i.sampler_index = push_sampler(samplers, base_random_deriver, noise_id);
            i
        }
        BaseNoiseFunctionComponent::ShiftB { noise_id } => {
            let mut i = Instruction::new(OpCode::ShiftB, index);
            i.sampler_index = push_sampler(samplers, base_random_deriver, noise_id);
            i
        }
        BaseNoiseFunctionComponent::ShiftedNoise {
            shift_x_index,
            shift_y_index,
            shift_z_index,
            data,
        } => {
            let mut i = Instruction::new(OpCode::ShiftedNoise, index);
            i.input0 = *shift_x_index as u32;
            i.input1 = *shift_y_index as u32;
            i.input2 = *shift_z_index as u32;
            i.sampler_index = push_sampler(samplers, base_random_deriver, &data.noise_id);
            i.param0 = data.xz_scale as f32;
            i.param1 = data.y_scale as f32;
            i
        }
        _ => return None,
    };
    Some(instruction)
}

/// Builds a seeded sampler the same way `ProtoNoiseRouter` does and adds it to `pool`.
fn push_sampler(
    pool: &mut SamplerPool,
    base_random_deriver: &XoroshiroSplitter,
    noise_id: &DoublePerlinNoiseParameters,
) -> u32 {
    let sampler = DoublePerlinNoiseBuilder::get_noise_sampler_for_id(base_random_deriver, noise_id);
    let (first, second) = sampler.samplers();
    pool.push_double_perlin(first, second, sampler.amplitude())
}

const fn node_name(component: &BaseNoiseFunctionComponent) -> &'static str {
    match component {
        BaseNoiseFunctionComponent::Beardifier => "Beardifier",
        BaseNoiseFunctionComponent::BlendAlpha => "BlendAlpha",
        BaseNoiseFunctionComponent::BlendOffset => "BlendOffset",
        BaseNoiseFunctionComponent::BlendDensity { .. } => "BlendDensity",
        BaseNoiseFunctionComponent::FindTopSurface { .. } => "FindTopSurface",
        BaseNoiseFunctionComponent::EndIslands => "EndIslands",
        BaseNoiseFunctionComponent::Noise { .. } => "Noise",
        BaseNoiseFunctionComponent::ShiftA { .. } => "ShiftA",
        BaseNoiseFunctionComponent::ShiftB { .. } => "ShiftB",
        BaseNoiseFunctionComponent::ShiftedNoise { .. } => "ShiftedNoise",
        BaseNoiseFunctionComponent::InterpolatedNoiseSampler { .. } => "InterpolatedNoiseSampler",
        BaseNoiseFunctionComponent::IntervalSelect { .. } => "IntervalSelect",
        BaseNoiseFunctionComponent::Wrapper { .. } => "Wrapper",
        BaseNoiseFunctionComponent::Constant { .. } => "Constant",
        BaseNoiseFunctionComponent::ClampedYGradient { .. } => "ClampedYGradient",
        BaseNoiseFunctionComponent::Binary { .. } => "Binary",
        BaseNoiseFunctionComponent::Linear { .. } => "Linear",
        BaseNoiseFunctionComponent::Unary { .. } => "Unary",
        BaseNoiseFunctionComponent::Clamp { .. } => "Clamp",
        BaseNoiseFunctionComponent::RangeChoice { .. } => "RangeChoice",
        BaseNoiseFunctionComponent::Spline { .. } => "Spline",
    }
}

/// CPU reference interpreter for [`Instruction`]s.
///
/// Mirrors `graph.wgsl` operation for operation (in f32, like the shader) so the two
/// can be compared directly in tests. It deliberately does *not* reimplement the
/// pumpkin-world CPU semantics independently — those live in
/// `pumpkin-world/src/generation/noise/router/density_function/`, and this exists to
/// pin down what the shader should produce.
#[must_use]
pub fn evaluate_cpu(
    instructions: &[Instruction],
    pool: &SamplerPool,
    x: f32,
    y: f32,
    z: f32,
) -> f32 {
    let mut values = vec![0.0f32; instructions.len()];

    for (index, instruction) in instructions.iter().enumerate() {
        let a = values[instruction.input0 as usize];
        let b = values[instruction.input1 as usize];
        let c = values[instruction.input2 as usize];
        let p = instruction;

        values[index] = match p.opcode {
            x if x == OpCode::Constant as u32 => p.param0,
            x if x == OpCode::PassThrough as u32 => a,
            x if x == OpCode::LinearAdd as u32 => a + p.param0,
            x if x == OpCode::LinearMul as u32 => a * p.param0,
            x if x == OpCode::UnaryAbs as u32 => a.abs(),
            x if x == OpCode::UnarySquare as u32 => a * a,
            x if x == OpCode::UnaryCube as u32 => a * a * a,
            x if x == OpCode::UnaryHalfNegative as u32 => {
                if a > 0.0 {
                    a
                } else {
                    a * 0.5
                }
            }
            x if x == OpCode::UnaryQuarterNegative as u32 => {
                if a > 0.0 {
                    a
                } else {
                    a * 0.25
                }
            }
            x if x == OpCode::UnarySqueeze as u32 => {
                let c = a.clamp(-1.0, 1.0);
                c / 2.0 - c * c * c / 24.0
            }
            x if x == OpCode::UnaryInvert as u32 => {
                if a == 0.0 {
                    f32::INFINITY
                } else {
                    1.0 / a
                }
            }
            x if x == OpCode::Clamp as u32 => a.clamp(p.param0, p.param1),
            x if x == OpCode::BinaryAdd as u32 => a + b,
            x if x == OpCode::BinaryMul as u32 => {
                if a == 0.0 {
                    0.0
                } else {
                    a * b
                }
            }
            x if x == OpCode::BinaryMin as u32 => a.min(b),
            x if x == OpCode::BinaryMax as u32 => a.max(b),
            x if x == OpCode::ClampedYGradient as u32 => {
                clamped_map(y, p.param0, p.param1, p.param2, p.param3)
            }
            op if op == OpCode::Noise as u32 => double_perlin_sample(
                pool,
                p.sampler_index as usize,
                x * p.param0,
                y * p.param1,
                z * p.param0,
            ),
            op if op == OpCode::ShiftA as u32 => {
                shift_sample_3d(pool, p.sampler_index as usize, x, 0.0, z)
            }
            // Vanilla feeds (z, x, 0) here, not (x, y, z); the rotation is deliberate.
            op if op == OpCode::ShiftB as u32 => {
                shift_sample_3d(pool, p.sampler_index as usize, z, x, 0.0)
            }
            op if op == OpCode::ShiftedNoise as u32 => double_perlin_sample(
                pool,
                p.sampler_index as usize,
                x * p.param0 + a,
                y * p.param1 + b,
                z * p.param0 + c,
            ),
            _ => 0.0,
        };
    }

    values.last().copied().unwrap_or(0.0)
}

/// f32 mirror of `shift_sample_3d` in
/// `pumpkin-world/src/generation/noise/router/density_function/noise.rs`.
fn shift_sample_3d(pool: &SamplerPool, sampler_index: usize, x: f32, y: f32, z: f32) -> f32 {
    double_perlin_sample(pool, sampler_index, x * 0.25, y * 0.25, z * 0.25) * 4.0
}

/// f32 mirror of `DoublePerlinNoiseSampler::sample`, reading from the flattened pool.
fn double_perlin_sample(pool: &SamplerPool, sampler_index: usize, x: f32, y: f32, z: f32) -> f32 {
    const SCALE: f32 = 1.018_126_9;

    let Some(sampler) = pool.samplers.get(sampler_index) else {
        return 0.0;
    };

    let first = octave_sample(pool, sampler.first_start, sampler.first_count, x, y, z);
    let second = octave_sample(
        pool,
        sampler.second_start,
        sampler.second_count,
        x * SCALE,
        y * SCALE,
        z * SCALE,
    );
    (first + second) * sampler.amplitude
}

fn octave_sample(pool: &SamplerPool, start: u32, count: u32, x: f32, y: f32, z: f32) -> f32 {
    let mut total = 0.0f32;
    for i in 0..count {
        let octave_index = (start + i) as usize;
        let Some(params) = pool.octaves.get(octave_index) else {
            continue;
        };
        let sample = perlin_sample(
            pool,
            octave_index,
            maintain_precision(x * params.lacunarity) + params.x_origin,
            maintain_precision(y * params.lacunarity) + params.y_origin,
            maintain_precision(z * params.lacunarity) + params.z_origin,
        );
        total += params.amplitude * sample * params.persistence;
    }
    total
}

fn maintain_precision(value: f32) -> f32 {
    const PERIOD: f32 = 3.355_443_2e7;
    value - (value / PERIOD + 0.5).floor() * PERIOD
}

fn perlin_fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn perm_at(pool: &SamplerPool, octave_index: usize, input: i32) -> u32 {
    pool.permutations
        .get(octave_index * 256 + (input & 0xFF) as usize)
        .copied()
        .unwrap_or(0)
}

fn grad(hash: u32, x: f32, y: f32, z: f32) -> f32 {
    // Same 16-entry table as pumpkin_util::noise::GRADIENTS.
    const GRADIENTS: [[f32; 3]; 16] = [
        [1.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0],
        [1.0, -1.0, 0.0],
        [-1.0, -1.0, 0.0],
        [1.0, 0.0, 1.0],
        [-1.0, 0.0, 1.0],
        [1.0, 0.0, -1.0],
        [-1.0, 0.0, -1.0],
        [0.0, 1.0, 1.0],
        [0.0, -1.0, 1.0],
        [0.0, 1.0, -1.0],
        [0.0, -1.0, -1.0],
        [1.0, 1.0, 0.0],
        [0.0, -1.0, 1.0],
        [-1.0, 1.0, 0.0],
        [0.0, -1.0, -1.0],
    ];
    let g = GRADIENTS[(hash & 15) as usize];
    g[0] * x + g[1] * y + g[2] * z
}

#[expect(clippy::many_single_char_names)]
fn perlin_sample(pool: &SamplerPool, octave_index: usize, x: f32, y: f32, z: f32) -> f32 {
    let x_floor = x.floor();
    let y_floor = y.floor();
    let z_floor = z.floor();

    let local_x = x - x_floor;
    let local_y = y - y_floor;
    let local_z = z - z_floor;

    let xi = x_floor as i32;
    let yi = y_floor as i32;
    let zi = z_floor as i32;

    let i = perm_at(pool, octave_index, xi) as i32;
    let j = perm_at(pool, octave_index, xi + 1) as i32;
    let k = perm_at(pool, octave_index, i + yi) as i32;
    let l = perm_at(pool, octave_index, i + yi + 1) as i32;
    let m = perm_at(pool, octave_index, j + yi) as i32;
    let n = perm_at(pool, octave_index, j + yi + 1) as i32;

    let d = grad(
        perm_at(pool, octave_index, k + zi),
        local_x,
        local_y,
        local_z,
    );
    let e = grad(
        perm_at(pool, octave_index, m + zi),
        local_x - 1.0,
        local_y,
        local_z,
    );
    let f = grad(
        perm_at(pool, octave_index, l + zi),
        local_x,
        local_y - 1.0,
        local_z,
    );
    let g = grad(
        perm_at(pool, octave_index, n + zi),
        local_x - 1.0,
        local_y - 1.0,
        local_z,
    );
    let h = grad(
        perm_at(pool, octave_index, k + zi + 1),
        local_x,
        local_y,
        local_z - 1.0,
    );
    let o = grad(
        perm_at(pool, octave_index, m + zi + 1),
        local_x - 1.0,
        local_y,
        local_z - 1.0,
    );
    let p = grad(
        perm_at(pool, octave_index, l + zi + 1),
        local_x,
        local_y - 1.0,
        local_z - 1.0,
    );
    let q = grad(
        perm_at(pool, octave_index, n + zi + 1),
        local_x - 1.0,
        local_y - 1.0,
        local_z - 1.0,
    );

    lerp3(
        perlin_fade(local_x),
        perlin_fade(local_y),
        perlin_fade(local_z),
        [d, e, f, g, h, o, p, q],
    )
}

fn lerp1(delta: f32, start: f32, end: f32) -> f32 {
    start + delta * (end - start)
}

fn lerp3(dx: f32, dy: f32, dz: f32, v: [f32; 8]) -> f32 {
    let lo = lerp1(dy, lerp1(dx, v[0], v[1]), lerp1(dx, v[2], v[3]));
    let hi = lerp1(dy, lerp1(dx, v[4], v[5]), lerp1(dx, v[6], v[7]));
    lerp1(dz, lo, hi)
}

/// f32 mirror of `pumpkin_util::math::clamped_map`.
fn clamped_map(value: f32, old_start: f32, old_end: f32, new_start: f32, new_end: f32) -> f32 {
    let delta = (value - old_start) / (old_end - old_start);
    if delta < 0.0 {
        new_start
    } else if delta > 1.0 {
        new_end
    } else {
        new_start + delta * (new_end - new_start)
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::{Instruction, OpCode, SamplerPool, compile, evaluate_cpu};
    use pumpkin_data::noise_router::{
        NETHER_BASE_NOISE_ROUTER, OVERWORLD_BASE_NOISE_ROUTER, UnaryOperation,
    };
    use pumpkin_util::random::xoroshiro128::{Xoroshiro, XoroshiroSplitter};

    fn instruction(opcode: OpCode, index: usize) -> Instruction {
        Instruction::new(opcode, index)
    }

    pub fn test_deriver() -> XoroshiroSplitter {
        Xoroshiro::from_seed(42).next_splitter()
    }

    /// The real graphs still contain node types with no opcode yet (splines, shifted
    /// noise, ...). Compilation must say so explicitly rather than emitting a graph
    /// that silently evaluates to something wrong.
    #[test]
    fn real_routers_report_unsupported_nodes_instead_of_miscompiling() {
        let deriver = test_deriver();
        for stack in [
            OVERWORLD_BASE_NOISE_ROUTER.noise.full_component_stack,
            NETHER_BASE_NOISE_ROUTER.noise.full_component_stack,
        ] {
            let err =
                compile(stack, &deriver).expect_err("spline/shift nodes are not supported yet");
            assert!(
                stack.len() > err.index,
                "reported index must point into the stack"
            );
        }
    }

    /// The GPU-side Noise opcode must reproduce the real CPU `DoublePerlinNoiseSampler`
    /// it was built from, within f32 precision.
    #[test]
    fn noise_opcode_matches_real_cpu_sampler() {
        use pumpkin_data::chunk::DoublePerlinNoiseParameters;
        use pumpkin_world::generation::noise::perlin::DoublePerlinNoiseSampler;
        use pumpkin_world::generation::noise::router::proto_noise_router::DoublePerlinNoiseBuilder;

        // A representative multi-octave noise; any seeded sampler exercises the same
        // path. Fields are (id, first_octave, amplitudes, lo, hi, amplitude).
        const AMPLITUDES: &[f64] = &[1.0, 1.0];
        let params = DoublePerlinNoiseParameters::new(
            0,
            -7,
            AMPLITUDES,
            0x5F3B_1A77,
            0x91E4_C2D0,
            DoublePerlinNoiseSampler::get_amplitude(AMPLITUDES),
        );
        let deriver = test_deriver();
        let cpu_sampler = DoublePerlinNoiseBuilder::get_noise_sampler_for_id(&deriver, &params);

        let mut pool = SamplerPool::default();
        let (first, second) = cpu_sampler.samplers();
        let index = pool.push_double_perlin(first, second, cpu_sampler.amplitude());

        let mut noise = instruction(OpCode::Noise, 0);
        noise.input0 = index;
        noise.param0 = 1.0;
        noise.param1 = 1.0;
        let graph = [noise];

        let mut max_diff = 0.0f64;
        for i in 0..500 {
            let x = f64::from(i) * 1.7;
            let y = f64::from(i) * -0.9;
            let z = f64::from(i) * 2.3;

            let expected = cpu_sampler.sample(x, y, z);
            let actual = evaluate_cpu(&graph, &pool, x as f32, y as f32, z as f32);
            max_diff = max_diff.max((expected - f64::from(actual)).abs());
        }

        assert!(
            max_diff < 1e-3,
            "Noise opcode diverged from the CPU sampler by {max_diff}"
        );
    }

    /// A hand-built graph covering the supported opcodes, so the CPU reference can be
    /// diffed against the shader in lib.rs's GPU test.
    #[must_use]
    pub fn sample_graph() -> Vec<Instruction> {
        let mut constant = instruction(OpCode::Constant, 0);
        constant.param0 = 3.0;

        let mut gradient = instruction(OpCode::ClampedYGradient, 1);
        gradient.param0 = -64.0;
        gradient.param1 = 64.0;
        gradient.param2 = 0.0;
        gradient.param3 = 1.0;

        let mut scaled = instruction(OpCode::LinearMul, 2);
        scaled.input0 = 1;
        scaled.param0 = 4.0;

        let mut squeezed = instruction(OpCode::UnarySqueeze, 3);
        squeezed.input0 = 2;

        let mut sum = instruction(OpCode::BinaryAdd, 4);
        sum.input0 = 0;
        sum.input1 = 3;

        let mut clamped = instruction(OpCode::Clamp, 5);
        clamped.input0 = 4;
        clamped.param0 = -1.5;
        clamped.param1 = 2.5;

        vec![constant, gradient, scaled, squeezed, sum, clamped]
    }

    #[test]
    fn cpu_reference_evaluates_supported_opcodes() {
        let graph = sample_graph();

        // y below from_y clamps the gradient to from_value (0.0), so the result is
        // constant(3.0) + squeeze(0.0) = 3.0, then clamped to the node's 2.5 ceiling.
        assert!(
            (evaluate_cpu(&graph, &SamplerPool::default(), 0.0, -1000.0, 0.0) - 2.5).abs() < 1e-6
        );

        // Above to_y the gradient saturates at to_value (1.0) -> *4 -> squeeze(1.0)
        // clamps to 1.0 -> 1/2 - 1/24, so the sum is 3.0 + ~0.4583, clamped to 2.5.
        assert!(
            (evaluate_cpu(&graph, &SamplerPool::default(), 0.0, 1000.0, 0.0) - 2.5).abs() < 1e-6
        );
    }

    #[test]
    fn unary_invert_matches_cpu_semantics_for_zero() {
        let mut constant = instruction(OpCode::Constant, 0);
        constant.param0 = 0.0;
        let mut invert = instruction(OpCode::UnaryInvert, 1);
        invert.input0 = 0;

        assert!(
            evaluate_cpu(&[constant, invert], &SamplerPool::default(), 0.0, 0.0, 0.0).is_infinite()
        );
        let _ = UnaryOperation::Invert;
    }
}
