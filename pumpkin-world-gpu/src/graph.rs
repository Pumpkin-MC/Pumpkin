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
    noise_router::{
        BaseNoiseFunctionComponent, BaseNoiseRouter, BinaryOperation, LinearOperation, SplineRepr,
        UnaryOperation,
    },
};
use pumpkin_util::{
    math::{block_box::BlockBox, vector3::Vector3},
    noise::perlin::OctavePerlinNoiseSampler,
    random::{legacy_rand::LegacyRand, xoroshiro128::XoroshiroSplitter},
};
use pumpkin_world::generation::{
    GlobalRandomConfig,
    noise::router::{
        density_function::{
            beardifier::{BeardifierJunction, BeardifierStructure, TerrainAdaptation},
            noise::InterpolatedNoiseSampler,
        },
        proto_noise_router::DoublePerlinNoiseBuilder,
    },
};

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
    /// Cubic spline. `input0` is the node giving the location to look up; `aux0`/`aux1`
    /// are the start and length of this spline's run in the graph's point table.
    Spline = 21,
    /// Vanilla's interpolated noise sampler; `sampler_index` selects an entry in the
    /// graph's interpolated-sampler table.
    InterpolatedNoise = 22,
    /// Structure "beard" weighting. Reads no graph inputs — its data is per-chunk and
    /// supplied at dispatch time, not baked into the compiled graph.
    Beardifier = 23,
    /// `input1` when `input0` falls in `[param0, param1)`, otherwise `input2`.
    RangeChoice = 24,
    /// Picks a branch by which threshold `input0` falls under; `aux0`/`aux1` are the
    /// start and length of this node's run in the graph's interval table.
    IntervalSelect = 25,
    /// A `ClampedYGradient` whose value range equals its Y range, i.e. clamped identity
    /// on Y. Lowered separately because computing it as a lerp is not exact: an integer
    /// Y comes back a hair off, and differently per backend, which flips the threshold
    /// comparisons that read it. `param0`/`param1` are the bounds.
    ClampedYIdentity = 26,
}

/// One branch of an [`OpCode::IntervalSelect`].
///
/// The final entry carries an infinite threshold so "below none of them" needs no
/// special case — it matches vanilla's fallback to `thresholds.len()`.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct GpuIntervalEntry {
    pub threshold: f32,
    pub function_node: u32,
}

/// How a structure adapts the terrain around it. Values must match
/// `TerrainAdaptation` in `density_function/beardifier.rs` and the shader.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum GpuTerrainAdaptation {
    None = 0,
    BeardThin = 1,
    BeardBox = 2,
    Bury = 3,
    Encapsulate = 4,
}

impl From<TerrainAdaptation> for GpuTerrainAdaptation {
    fn from(value: TerrainAdaptation) -> Self {
        match value {
            TerrainAdaptation::None => Self::None,
            TerrainAdaptation::BeardThin => Self::BeardThin,
            TerrainAdaptation::BeardBox => Self::BeardBox,
            TerrainAdaptation::Bury => Self::Bury,
            TerrainAdaptation::Encapsulate => Self::Encapsulate,
        }
    }
}

/// One structure's bounding box and adaptation mode, as uploaded to the GPU.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct GpuBeardStructure {
    pub min: [i32; 3],
    pub adaptation: u32,
    pub max: [i32; 3],
    pub ground_level_delta: i32,
}

/// One structure-piece junction, as uploaded to the GPU.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct GpuBeardJunction {
    pub x: i32,
    pub ground_y: i32,
    pub z: i32,
    padding: i32,
}

/// Per-chunk structure data for [`OpCode::Beardifier`].
///
/// Unlike everything else in a [`CompiledGraph`], this is not derived from the seed:
/// it depends on which structures generation placed near the chunk being sampled, so
/// it is supplied per dispatch and a compiled graph can be reused across chunks.
#[derive(Default, Debug, Clone)]
pub struct BeardifierData {
    pub structures: Vec<GpuBeardStructure>,
    pub junctions: Vec<GpuBeardJunction>,
    /// Sampling outside this box contributes nothing; `None` disables the node.
    pub affected_box: Option<(Vector3<i32>, Vector3<i32>)>,
}

impl BeardifierData {
    /// Converts the CPU-side beardifier inputs into GPU-uploadable tables.
    #[must_use]
    pub fn from_cpu(
        structures: &[BeardifierStructure],
        junctions: &[BeardifierJunction],
        affected_box: Option<&BlockBox>,
    ) -> Self {
        Self {
            structures: structures
                .iter()
                .map(|s| GpuBeardStructure {
                    min: [
                        s.bounding_box.min.x,
                        s.bounding_box.min.y,
                        s.bounding_box.min.z,
                    ],
                    adaptation: GpuTerrainAdaptation::from(s.terrain_adaptation) as u32,
                    max: [
                        s.bounding_box.max.x,
                        s.bounding_box.max.y,
                        s.bounding_box.max.z,
                    ],
                    ground_level_delta: s.ground_level_delta,
                })
                .collect(),
            junctions: junctions
                .iter()
                .map(|j| GpuBeardJunction {
                    x: j.x,
                    ground_y: j.ground_y,
                    z: j.z,
                    padding: 0,
                })
                .collect(),
            affected_box: affected_box.map(|b| (b.min, b.max)),
        }
    }
}

/// GPU-side descriptor of an `InterpolatedNoiseSampler`.
///
/// Unlike [`SamplerRef`], the three octave runs here are sampled with per-octave
/// fractions (1, 1/2, 1/4, ...) applied in reverse order, and with a non-zero vertical
/// scale, so they cannot reuse the double-Perlin path.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct InterpolatedRef {
    pub lower_start: u32,
    pub lower_count: u32,
    pub upper_start: u32,
    pub upper_count: u32,
    pub noise_start: u32,
    pub noise_count: u32,
    /// `scaled_xz_scale * 684.412`.
    pub xz_multiplier: f32,
    /// `scaled_y_scale * xz_factor / y_factor * 684.412`.
    pub y_multiplier: f32,
    pub xz_factor: f32,
    pub y_factor: f32,
    /// `y_multiplier * smear_scale_multiplier`.
    pub smear: f32,
    padding: f32,
}

/// One knot of a spline, as uploaded to the GPU.
///
/// `value_node` is the instruction whose output supplies this knot's value. Nested
/// splines are emitted as their own instructions, so evaluation never recurses.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct GpuSplinePoint {
    pub location: f32,
    pub derivative: f32,
    pub value_node: u32,
    padding: u32,
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
    /// Opcode-specific extra operands that are NOT scratch indices (unlike
    /// `input0`..`input2`, which the shader always dereferences).
    pub aux0: u32,
    pub aux1: u32,
    aux2: u32,
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
            aux0: 0,
            aux1: 0,
            aux2: 0,
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

impl GpuSplinePoint {
    /// Builds a knot whose value comes from instruction `value_node`.
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn new(location: f32, derivative: f32, value_node: u32) -> Self {
        Self {
            location,
            derivative,
            value_node,
            padding: 0,
        }
    }
}

/// Every noise sampler referenced by a compiled graph, flattened into GPU-uploadable
/// tables.
///
/// Each `Noise`-family node owns its own seeded sampler, so instructions index into
/// [`Self::samplers`] rather than carrying sampler state inline.
#[derive(Default, Debug)]
pub struct SamplerPool {
    pub samplers: Vec<SamplerRef>,
    pub interpolated: Vec<InterpolatedRef>,
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

    /// Registers an interpolated sampler's three octave runs, returning its index.
    pub fn push_interpolated(&mut self, sampler: &InterpolatedNoiseSampler) -> u32 {
        let (lower, upper, noise) = sampler.octave_samplers();
        let lower_start = self.push_octaves(lower);
        let upper_start = self.push_octaves(upper);
        let noise_start = self.push_octaves(noise);

        let index = self.interpolated.len() as u32;
        let data = sampler.data();
        self.interpolated.push(InterpolatedRef {
            lower_start,
            lower_count: lower.samplers.len() as u32,
            upper_start,
            upper_count: upper.samplers.len() as u32,
            noise_start,
            noise_count: noise.samplers.len() as u32,
            xz_multiplier: (data.scaled_xz_scale * 684.412) as f32,
            y_multiplier: sampler.y_multiplier() as f32,
            xz_factor: data.xz_factor as f32,
            y_factor: data.y_factor as f32,
            smear: (sampler.y_multiplier() * data.smear_scale_multiplier) as f32,
            padding: 0.0,
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

/// Which instruction holds each of the router's named outputs.
///
/// The block-state samplers (aquifer, ore veins) need several of these at the same
/// position, so the GPU emits them all in one pass and the CPU does the branchy,
/// stateful decision work with the results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouterOutputs {
    pub barrier_noise: u32,
    pub fluid_level_floodedness_noise: u32,
    pub fluid_level_spread_noise: u32,
    pub lava_noise: u32,
    pub erosion: u32,
    pub depth: u32,
    pub final_density: u32,
    pub vein_toggle: u32,
    pub vein_ridged: u32,
    pub vein_gap: u32,
}

impl RouterOutputs {
    /// The outputs in the order [`CompiledGraph::outputs`] emits them.
    #[must_use]
    pub const fn as_slots(&self) -> [u32; 10] {
        [
            self.barrier_noise,
            self.fluid_level_floodedness_noise,
            self.fluid_level_spread_noise,
            self.lava_noise,
            self.erosion,
            self.depth,
            self.final_density,
            self.vein_toggle,
            self.vein_ridged,
            self.vein_gap,
        ]
    }
}

/// Position of each named output within one point's slice of the result, matching the
/// order of [`RouterOutputs::as_slots`].
pub mod output_slot {
    pub const BARRIER_NOISE: usize = 0;
    pub const FLUID_LEVEL_FLOODEDNESS_NOISE: usize = 1;
    pub const FLUID_LEVEL_SPREAD_NOISE: usize = 2;
    pub const LAVA_NOISE: usize = 3;
    pub const EROSION: usize = 4;
    pub const DEPTH: usize = 5;
    pub const FINAL_DENSITY: usize = 6;
    pub const VEIN_TOGGLE: usize = 7;
    pub const VEIN_RIDGED: usize = 8;
    pub const VEIN_GAP: usize = 9;
    pub const COUNT: usize = 10;
}

/// A compiled graph plus the sampler tables its instructions index into.
#[derive(Debug, Default)]
pub struct CompiledGraph {
    pub instructions: Vec<Instruction>,
    pub samplers: SamplerPool,
    /// All spline knots, concatenated; `Spline` instructions index runs of this.
    pub spline_points: Vec<GpuSplinePoint>,
    /// All interval branches, concatenated; `IntervalSelect` instructions index runs.
    pub interval_entries: Vec<GpuIntervalEntry>,
    /// Instruction indices whose values are written out, in `output_slot` order. Empty
    /// means "just the last instruction", which is what the standalone graph tests use.
    pub outputs: Vec<u32>,
    /// Maps a node's index in the source component stack to its instruction index.
    /// They differ because splines emit instructions for their nested values.
    pub stack_to_instruction: Vec<u32>,
}

impl CompiledGraph {
    /// How many values [`crate::PreparedGraph::evaluate`] returns per point.
    #[must_use]
    pub fn outputs_per_point(&self) -> usize {
        self.outputs.len().max(1)
    }
}

/// Lowers a component stack into GPU instructions.
///
/// Every sampler is seeded from `random_config` exactly as `ProtoNoiseRouter` does on
/// the CPU, so both paths share state. The full config is needed rather than just the
/// deriver: `InterpolatedNoiseSampler` is built from the raw seed on legacy dimensions
/// and from the deriver everywhere else.
///
/// # Errors
/// Returns [`UnsupportedNode`] for the first node whose type has no opcode yet, so
/// callers can fall back to the CPU path instead of generating wrong terrain.
pub fn compile(
    stack: &[BaseNoiseFunctionComponent],
    random_config: &GlobalRandomConfig,
) -> Result<CompiledGraph, UnsupportedNode> {
    let mut out: Vec<Instruction> = Vec::with_capacity(stack.len());
    let mut samplers = SamplerPool::default();
    let mut spline_points: Vec<GpuSplinePoint> = Vec::new();
    let mut interval_entries: Vec<GpuIntervalEntry> = Vec::new();
    // Splines emit extra instructions for their nested values, so an original stack
    // index no longer equals its instruction index; every input goes through this map.
    let mut map: Vec<u32> = vec![0; stack.len()];

    for (stack_index, component) in stack.iter().enumerate() {
        // `own` is an index into `out`; `stack_index` is an index into `stack`. They
        // diverge as soon as a spline emits instructions for its nested values.
        let own = out.len();

        if let Some(mut instruction) =
            lower_noise_family(component, own, &mut samplers, random_config)
        {
            remap_inputs(&mut instruction, component, &map, own);
            out.push(instruction);
            map[stack_index] = own as u32;
            continue;
        }

        if let BaseNoiseFunctionComponent::Spline { spline } = component {
            map[stack_index] = emit_spline(spline, &mut out, &mut spline_points, &map);
            continue;
        }

        if let BaseNoiseFunctionComponent::IntervalSelect {
            input_index,
            thresholds,
            functions_indices,
        } = component
        {
            let start = interval_entries.len() as u32;
            for (i, &function) in functions_indices.iter().enumerate() {
                interval_entries.push(GpuIntervalEntry {
                    // One more branch than thresholds: the last is the fallback.
                    threshold: thresholds.get(i).map_or(f32::INFINITY, |&t| t as f32),
                    function_node: map[function],
                });
            }

            let mut i = Instruction::new(OpCode::IntervalSelect, own);
            i.input0 = map[*input_index];
            i.aux0 = start;
            i.aux1 = functions_indices.len() as u32;
            out.push(i);
            map[stack_index] = own as u32;
            continue;
        }

        let instruction = lower_simple(component, stack_index, own, &map)?;
        out.push(instruction);
        map[stack_index] = own as u32;
    }

    Ok(CompiledGraph {
        instructions: out,
        samplers,
        spline_points,
        interval_entries,
        outputs: Vec::new(),
        stack_to_instruction: map,
    })
}

const fn unary_opcode(operation: UnaryOperation) -> OpCode {
    match operation {
        UnaryOperation::Abs => OpCode::UnaryAbs,
        UnaryOperation::Square => OpCode::UnarySquare,
        UnaryOperation::Cube => OpCode::UnaryCube,
        UnaryOperation::HalfNegative => OpCode::UnaryHalfNegative,
        UnaryOperation::QuarterNegative => OpCode::UnaryQuarterNegative,
        UnaryOperation::Squeeze => OpCode::UnarySqueeze,
        UnaryOperation::Invert => OpCode::UnaryInvert,
    }
}

const fn binary_opcode(operation: BinaryOperation) -> OpCode {
    match operation {
        BinaryOperation::Add => OpCode::BinaryAdd,
        BinaryOperation::Mul => OpCode::BinaryMul,
        BinaryOperation::Min => OpCode::BinaryMin,
        BinaryOperation::Max => OpCode::BinaryMax,
    }
}

/// Lowers the arithmetic, constant and wrapper nodes — everything that needs neither a
/// sampler nor spline flattening.
fn lower_simple(
    component: &BaseNoiseFunctionComponent,
    stack_index: usize,
    own: usize,
    map: &[u32],
) -> Result<Instruction, UnsupportedNode> {
    let index = own;
    let instruction = match component {
        // Reads no graph inputs: its data arrives per dispatch, not in the graph.
        BaseNoiseFunctionComponent::Beardifier => Instruction::new(OpCode::Beardifier, index),
        BaseNoiseFunctionComponent::RangeChoice {
            input_index,
            when_in_range_index,
            when_out_range_index,
            data,
        } => {
            let mut i = Instruction::new(OpCode::RangeChoice, index);
            i.input0 = map[*input_index];
            i.input1 = map[*when_in_range_index];
            i.input2 = map[*when_out_range_index];
            i.param0 = data.min_inclusive as f32;
            i.param1 = data.max_exclusive as f32;
            i
        }
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
            i.input0 = map[*input_index];
            i
        }
        BaseNoiseFunctionComponent::Linear { input_index, data } => {
            let op = match data.operation {
                LinearOperation::Add => OpCode::LinearAdd,
                LinearOperation::Mul => OpCode::LinearMul,
            };
            let mut i = Instruction::new(op, index);
            i.input0 = map[*input_index];
            i.param0 = data.argument as f32;
            i
        }
        BaseNoiseFunctionComponent::Unary { input_index, data } => {
            let op = unary_opcode(data.operation);
            let mut i = Instruction::new(op, index);
            i.input0 = map[*input_index];
            i
        }
        BaseNoiseFunctionComponent::Clamp { input_index, data } => {
            let mut i = Instruction::new(OpCode::Clamp, index);
            i.input0 = map[*input_index];
            i.param0 = data.min_value as f32;
            i.param1 = data.max_value as f32;
            i
        }
        BaseNoiseFunctionComponent::Binary {
            argument1_index,
            argument2_index,
            data,
        } => {
            let op = binary_opcode(data.operation);
            let mut i = Instruction::new(op, index);
            i.input0 = map[*argument1_index];
            i.input1 = map[*argument2_index];
            i
        }
        BaseNoiseFunctionComponent::ClampedYGradient { data } => {
            // The overworld uses this as plain clamped identity on Y; lowering that
            // case to a clamp keeps it exact for integer Y on both backends.
            let is_identity = data.from_y == data.from_value && data.to_y == data.to_value;
            let op = if is_identity {
                OpCode::ClampedYIdentity
            } else {
                OpCode::ClampedYGradient
            };
            let mut i = Instruction::new(op, index);
            i.param0 = data.from_y as f32;
            i.param1 = data.to_y as f32;
            i.param2 = data.from_value as f32;
            i.param3 = data.to_value as f32;
            i
        }
        other => {
            return Err(UnsupportedNode {
                index: stack_index,
                name: node_name(other),
            });
        }
    };
    Ok(instruction)
}

/// Rewrites the shift-noise inputs, which `lower_noise_family` fills with original
/// stack indices, into instruction indices.
fn remap_inputs(
    instruction: &mut Instruction,
    component: &BaseNoiseFunctionComponent,
    map: &[u32],
    own: usize,
) {
    if let BaseNoiseFunctionComponent::ShiftedNoise {
        shift_x_index,
        shift_y_index,
        shift_z_index,
        ..
    } = component
    {
        instruction.input0 = map[*shift_x_index];
        instruction.input1 = map[*shift_y_index];
        instruction.input2 = map[*shift_z_index];
    } else {
        let own = own as u32;
        instruction.input0 = own;
        instruction.input1 = own;
        instruction.input2 = own;
    }
}

/// Emits `repr` (and, depth-first, everything nested inside it) as instructions,
/// returning the index of the instruction holding its value.
///
/// Nested splines become ordinary instructions rather than a runtime recursion, which
/// WGSL cannot express. The GPU evaluates every branch instead of only the two a given
/// point needs, but it already evaluates every instruction for every point, so nothing
/// is actually wasted.
fn emit_spline(
    repr: &SplineRepr,
    out: &mut Vec<Instruction>,
    spline_points: &mut Vec<GpuSplinePoint>,
    map: &[u32],
) -> u32 {
    match repr {
        SplineRepr::Fixed { value } => {
            let own = out.len();
            let mut i = Instruction::new(OpCode::Constant, own);
            i.param0 = *value;
            out.push(i);
            own as u32
        }
        SplineRepr::Standard {
            location_function_index,
            points,
        } => {
            // Emit nested values first so their instructions precede this one.
            let knots: Vec<GpuSplinePoint> = points
                .iter()
                .map(|point| GpuSplinePoint {
                    location: point.location,
                    derivative: point.derivative,
                    value_node: emit_spline(point.value, out, spline_points, map),
                    padding: 0,
                })
                .collect();

            let start = spline_points.len() as u32;
            let count = knots.len() as u32;
            spline_points.extend(knots);

            let own = out.len();
            let mut i = Instruction::new(OpCode::Spline, own);
            i.input0 = map[*location_function_index];
            i.aux0 = start;
            i.aux1 = count;
            out.push(i);
            own as u32
        }
    }
}

/// Lowers the noise-family nodes, which all need a seeded sampler registered in
/// `samplers`. Returns `None` for any other node type.
fn lower_noise_family(
    component: &BaseNoiseFunctionComponent,
    index: usize,
    samplers: &mut SamplerPool,
    random_config: &GlobalRandomConfig,
) -> Option<Instruction> {
    let base_random_deriver = &random_config.base_random_deriver;
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
        BaseNoiseFunctionComponent::InterpolatedNoiseSampler { data } => {
            // Mirrors proto_noise_router.rs: legacy dimensions (the Nether) seed this
            // from the raw world seed, everything else from the terrain deriver.
            let sampler = if random_config.legacy_random_source {
                let mut legacy = LegacyRand::from_seed(random_config.seed);
                InterpolatedNoiseSampler::new(data, &mut legacy)
            } else {
                let mut random = base_random_deriver.split_string("minecraft:terrain");
                InterpolatedNoiseSampler::new(data, &mut random)
            };
            let mut i = Instruction::new(OpCode::InterpolatedNoise, index);
            i.sampler_index = samplers.push_interpolated(&sampler);
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

/// Lowers a whole router, recording where each of its named outputs landed.
///
/// Prefer this over [`compile`] when the caller needs more than the final density —
/// the aquifer and ore-vein samplers each need several outputs at the same position.
///
/// # Errors
/// Same as [`compile`].
pub fn compile_router(
    router: &BaseNoiseRouter,
    random_config: &GlobalRandomConfig,
) -> Result<CompiledGraph, UnsupportedNode> {
    let mut compiled = compile(router.full_component_stack, random_config)?;

    // compile() maps stack indices to instruction indices; recover that map by
    // re-running the same lowering is wasteful, so compile() records it for us.
    let map = &compiled.stack_to_instruction;
    let outputs = RouterOutputs {
        barrier_noise: map[router.barrier_noise],
        fluid_level_floodedness_noise: map[router.fluid_level_floodedness_noise],
        fluid_level_spread_noise: map[router.fluid_level_spread_noise],
        lava_noise: map[router.lava_noise],
        erosion: map[router.erosion],
        depth: map[router.depth],
        final_density: map[router.final_density],
        vein_toggle: map[router.vein_toggle],
        vein_ridged: map[router.vein_ridged],
        vein_gap: map[router.vein_gap],
    };
    compiled.outputs = outputs.as_slots().to_vec();
    Ok(compiled)
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
    compiled: &CompiledGraph,
    beardifier: &BeardifierData,
    x: f32,
    y: f32,
    z: f32,
) -> f32 {
    let last = compiled.instructions.len().saturating_sub(1) as u32;
    evaluate_cpu_node(compiled, beardifier, last, x, y, z)
}

/// Evaluates the graph and returns one specific instruction's value, for callers that
/// need a named router output rather than the final density.
#[must_use]
pub fn evaluate_cpu_node(
    compiled: &CompiledGraph,
    beardifier: &BeardifierData,
    node: u32,
    x: f32,
    y: f32,
    z: f32,
) -> f32 {
    let instructions = &compiled.instructions;
    let mut values = vec![0.0f32; instructions.len()];

    for (index, instruction) in instructions.iter().enumerate() {
        let a = values[instruction.input0 as usize];
        let b = values[instruction.input1 as usize];
        let c = values[instruction.input2 as usize];
        let p = instruction;

        values[index] = eval_opcode(
            p,
            EvalInputs { a, b, c },
            &values,
            compiled,
            beardifier,
            [x, y, z],
        );
    }

    values.get(node as usize).copied().unwrap_or(0.0)
}

/// The three already-computed input values an instruction may read.
#[derive(Clone, Copy)]
struct EvalInputs {
    a: f32,
    b: f32,
    c: f32,
}

/// Dispatches one instruction. Split from `evaluate_cpu` so each stays readable.
fn eval_opcode(
    p: &Instruction,
    inputs: EvalInputs,
    values: &[f32],
    compiled: &CompiledGraph,
    beardifier: &BeardifierData,
    point: [f32; 3],
) -> f32 {
    let EvalInputs { a, b, c } = inputs;
    let [x, y, z] = point;

    match p.opcode {
        op if op == OpCode::Constant as u32 => p.param0,
        op if op == OpCode::PassThrough as u32 => a,
        op if op == OpCode::LinearAdd as u32 => a + p.param0,
        op if op == OpCode::LinearMul as u32 => a * p.param0,
        op if op == OpCode::UnaryAbs as u32 => a.abs(),
        op if op == OpCode::UnarySquare as u32 => a * a,
        op if op == OpCode::UnaryCube as u32 => a * a * a,
        op if op == OpCode::UnaryHalfNegative as u32 => {
            if a > 0.0 {
                a
            } else {
                a * 0.5
            }
        }
        op if op == OpCode::UnaryQuarterNegative as u32 => {
            if a > 0.0 {
                a
            } else {
                a * 0.25
            }
        }
        op if op == OpCode::UnarySqueeze as u32 => {
            let clamped = a.clamp(-1.0, 1.0);
            clamped / 2.0 - clamped * clamped * clamped / 24.0
        }
        op if op == OpCode::UnaryInvert as u32 => 1.0 / a,
        op if op == OpCode::Clamp as u32 => a.clamp(p.param0, p.param1),
        op if op == OpCode::BinaryAdd as u32 => a + b,
        op if op == OpCode::BinaryMul as u32 => a * b,
        op if op == OpCode::BinaryMin as u32 => a.min(b),
        op if op == OpCode::BinaryMax as u32 => a.max(b),
        op if op == OpCode::ClampedYGradient as u32 => {
            clamped_map(y, p.param0, p.param1, p.param2, p.param3)
        }
        op if op == OpCode::ClampedYIdentity as u32 => y.clamp(p.param0, p.param1),
        op if op == OpCode::Spline as u32 => sample_spline(
            &compiled.spline_points,
            p.aux0 as usize,
            p.aux1 as usize,
            a,
            values,
        ),
        op if op == OpCode::Beardifier as u32 => {
            beardifier_sample(beardifier, x as i32, y as i32, z as i32)
        }
        op if op == OpCode::RangeChoice as u32 => {
            if p.param0 <= a && a < p.param1 {
                b
            } else {
                c
            }
        }
        op if op == OpCode::IntervalSelect as u32 => compiled
            .interval_entries
            .get(p.aux0 as usize..(p.aux0 + p.aux1) as usize)
            .unwrap_or_default()
            .iter()
            .find(|entry| a < entry.threshold)
            .map_or(0.0, |entry| values[entry.function_node as usize]),
        _ => eval_noise_opcode(p, inputs, &compiled.samplers, point),
    }
}

/// The noise-family opcodes, split out to keep the main dispatch readable.
fn eval_noise_opcode(
    p: &Instruction,
    EvalInputs { a, b, c }: EvalInputs,
    pool: &SamplerPool,
    [x, y, z]: [f32; 3],
) -> f32 {
    match p.opcode {
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
        op if op == OpCode::InterpolatedNoise as u32 => {
            interpolated_sample(pool, p.sampler_index as usize, x, y, z)
        }
        _ => 0.0,
    }
}

/// f32 mirror of `Spline::sample` in
/// `pumpkin-world/src/generation/noise/router/density_function/spline.rs`.
///
/// Knot values are read from already-computed instruction outputs rather than being
/// evaluated here, which is what removes the recursion.
fn sample_spline(
    spline_points: &[GpuSplinePoint],
    start: usize,
    count: usize,
    location: f32,
    values: &[f32],
) -> f32 {
    if count == 0 {
        return 0.0;
    }
    let knots = &spline_points[start..start + count];
    let knot_value = |knot: &GpuSplinePoint| values[knot.value_node as usize];

    // Same as points.partition_point(|p| location >= p.location); locations ascend.
    let above = knots.iter().filter(|k| location >= k.location).count();

    if above == 0 {
        let knot = &knots[0];
        return sample_outside_range(knot, location, knot_value(knot));
    }
    if above == count {
        let knot = &knots[count - 1];
        return sample_outside_range(knot, location, knot_value(knot));
    }

    let lower = &knots[above - 1];
    let upper = &knots[above];
    let lower_value = knot_value(lower);
    let upper_value = knot_value(upper);

    let dist = upper.location - lower.location;
    let x_scale = (location - lower.location) / dist;

    let delta = upper_value - lower_value;
    let extrapolated_lower = lower.derivative * dist - delta;
    let extrapolated_upper = -upper.derivative * dist + delta;

    let cubic =
        (x_scale * (1.0 - x_scale)) * lerp_f32(x_scale, extrapolated_lower, extrapolated_upper);
    cubic + lerp_f32(x_scale, lower_value, upper_value)
}

fn sample_outside_range(knot: &GpuSplinePoint, location: f32, last_known: f32) -> f32 {
    if knot.derivative == 0.0 {
        last_known
    } else {
        knot.derivative * (location - knot.location) + last_known
    }
}

fn lerp_f32(delta: f32, start: f32, end: f32) -> f32 {
    start + delta * (end - start)
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
        let sample = perlin_sample_no_fade(
            pool,
            octave_index,
            maintain_precision(x * params.lacunarity),
            maintain_precision(y * params.lacunarity),
            maintain_precision(z * params.lacunarity),
            0.0,
            0.0,
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

/// f32 mirror of `PerlinNoiseSampler::sample_no_fade`, including the vertical
/// quantization that only applies for a non-zero `y_scale`.
fn perlin_sample_no_fade(
    pool: &SamplerPool,
    octave_index: usize,
    x: f32,
    y: f32,
    z: f32,
    y_scale: f32,
    y_max: f32,
) -> f32 {
    let Some(params) = pool.octaves.get(octave_index) else {
        return 0.0;
    };
    let true_x = x + params.x_origin;
    let true_y = y + params.y_origin;
    let true_z = z + params.z_origin;

    let x_dec = true_x - true_x.floor();
    let y_dec = true_y - true_y.floor();
    let z_dec = true_z - true_z.floor();

    let y_noise = if y_scale == 0.0 {
        0.0
    } else {
        let raw = if y_max >= 0.0 && y_max < y_dec {
            y_max
        } else {
            y_dec
        };
        (raw / y_scale + 1e-7).floor() * y_scale
    };

    perlin_core(
        pool,
        octave_index,
        [
            true_x.floor() as i32,
            true_y.floor() as i32,
            true_z.floor() as i32,
        ],
        [x_dec, y_dec - y_noise, z_dec],
        y_dec,
    )
}

#[expect(clippy::many_single_char_names)]
fn perlin_core(
    pool: &SamplerPool,
    octave_index: usize,
    lattice: [i32; 3],
    local: [f32; 3],
    fade_local_y: f32,
) -> f32 {
    let [xi, yi, zi] = lattice;
    let [local_x, local_y, local_z] = local;

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
        perlin_fade(fade_local_y),
        perlin_fade(local_z),
        [d, e, f, g, h, o, p, q],
    )
}

/// f32 mirror of one octave run in `InterpolatedNoiseSampler::sample`: fractions
/// 1, 1/2, 1/4, ... paired with octaves in reverse order.
fn interpolated_run(
    pool: &SamplerPool,
    start: u32,
    count: u32,
    point: [f32; 3],
    y_scale: f32,
    y_max_base: f32,
) -> f32 {
    let mut total = 0.0f32;
    let mut fraction = 1.0f32;
    for i in 0..count.min(16) {
        let octave_index = (start + (count - 1 - i)) as usize;
        total += perlin_sample_no_fade(
            pool,
            octave_index,
            maintain_precision(point[0] * fraction),
            maintain_precision(point[1] * fraction),
            maintain_precision(point[2] * fraction),
            y_scale * fraction,
            y_max_base * fraction,
        ) / fraction;
        fraction *= 0.5;
    }
    total
}

/// f32 mirror of `InterpolatedNoiseSampler::sample`.
fn interpolated_sample(pool: &SamplerPool, index: usize, x: f32, y: f32, z: f32) -> f32 {
    let Some(s) = pool.interpolated.get(index) else {
        return 0.0;
    };

    let d = x * s.xz_multiplier;
    let e = y * s.y_multiplier;
    let f = z * s.xz_multiplier;

    let g = d / s.xz_factor;
    let h = e / s.y_factor;
    let i = f / s.xz_factor;
    let k = s.smear / s.y_factor;

    let n = interpolated_run(pool, s.noise_start, s.noise_count, [g, h, i], k, h);
    let q = f32::midpoint(n / 10.0, 1.0);

    let lower = if q >= 1.0 {
        0.0
    } else {
        interpolated_run(pool, s.lower_start, s.lower_count, [d, e, f], s.smear, e)
    };
    let upper = if q <= 0.0 {
        0.0
    } else {
        interpolated_run(pool, s.upper_start, s.upper_count, [d, e, f], s.smear, e)
    };

    clamped_lerp_f32(lower / 512.0, upper / 512.0, q) / 128.0
}

/// f32 mirror of `Beardifier::sample` in `density_function/beardifier.rs`.
fn beardifier_sample(data: &BeardifierData, x: i32, y: i32, z: i32) -> f32 {
    let Some((min, max)) = data.affected_box else {
        return 0.0;
    };
    if x < min.x || x > max.x || y < min.y || y > max.y || z < min.z || z > max.z {
        return 0.0;
    }

    let mut weight = 0.0f32;

    for s in &data.structures {
        let dx = 0.max((s.min[0] - x).max(x - s.max[0]));
        let dz = 0.max((s.min[2] - z).max(z - s.max[2]));
        let ground_y = s.min[1] + s.ground_level_delta;
        let dy_to_ground = y - ground_y;

        let dy = match s.adaptation {
            1 | 3 => dy_to_ground,
            2 => 0.max((ground_y - y).max(y - s.max[1])),
            4 => 0.max((s.min[1] - y).max(y - s.max[1])),
            _ => 0,
        };

        weight += match s.adaptation {
            1 | 2 => beard_contribution(dx, dy, dz, dy_to_ground) * 0.8,
            3 => bury_contribution(dx as f32, dy as f32 / 2.0, dz as f32),
            4 => bury_contribution(dx as f32 / 2.0, dy as f32 / 2.0, dz as f32 / 2.0) * 0.8,
            _ => 0.0,
        };
    }

    for j in &data.junctions {
        let dy = y - j.ground_y;
        weight += beard_contribution(x - j.x, dy, z - j.z, dy) * 0.4;
    }

    weight
}

/// The kernel vanilla precomputes into a 24x24x24 table is just this closed form, so
/// it is evaluated directly rather than uploaded.
fn beard_contribution(dx: i32, dy: i32, dz: i32, y_to_ground: i32) -> f32 {
    const RADIUS: i32 = 12;
    const SIZE: i32 = 24;

    let (xi, yi, zi) = (dx + RADIUS, dy + RADIUS, dz + RADIUS);
    if !(0..SIZE).contains(&xi) || !(0..SIZE).contains(&yi) || !(0..SIZE).contains(&zi) {
        return 0.0;
    }

    let dy_with_offset = y_to_ground as f32 + 0.5;
    let distance_sqr = (dx as f32).powi(2) + dy_with_offset.powi(2) + (dz as f32).powi(2);
    let value = -dy_with_offset * (distance_sqr / 2.0).sqrt().recip() / 2.0;

    // The table entry works out to exp(-(dx^2 + (dy + 0.5)^2 + dz^2) / 16). Note this
    // uses `dy`, while `value` above uses `y_to_ground`; they differ for BeardBox.
    let kernel_dy = dy as f32 + 0.5;
    let kernel_sqr = (dx as f32).powi(2) + kernel_dy.powi(2) + (dz as f32).powi(2);
    value * (-kernel_sqr / 16.0).exp()
}

/// Equivalent to `Mth.clampedMap(distance, 0, 6, 1, 0)`.
fn bury_contribution(dx: f32, dy: f32, dz: f32) -> f32 {
    let distance = dx.mul_add(dx, dy.mul_add(dy, dz * dz)).sqrt();
    if distance < 0.0 {
        1.0
    } else if distance > 6.0 {
        0.0
    } else {
        1.0 - distance / 6.0
    }
}

fn clamped_lerp_f32(start: f32, end: f32, delta: f32) -> f32 {
    if delta < 0.0 {
        start
    } else if delta > 1.0 {
        end
    } else {
        lerp_f32(delta, start, end)
    }
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
    use super::{
        BeardifierData, CompiledGraph, Instruction, OpCode, SamplerPool, compile, evaluate_cpu,
    };
    use pumpkin_data::noise_router::{
        BaseNoiseFunctionComponent, NETHER_BASE_NOISE_ROUTER, OVERWORLD_BASE_NOISE_ROUTER,
        UnaryOperation,
    };
    use pumpkin_util::random::xoroshiro128::{Xoroshiro, XoroshiroSplitter};
    use pumpkin_world::generation::GlobalRandomConfig;

    fn instruction(opcode: OpCode, index: usize) -> Instruction {
        Instruction::new(opcode, index)
    }

    /// Wraps a bare instruction list so tests can call `evaluate_cpu` without
    /// building the sampler and table plumbing a real compile produces.
    fn bare_graph(instructions: &[Instruction]) -> CompiledGraph {
        let instructions = instructions.to_vec();
        CompiledGraph {
            instructions,
            ..Default::default()
        }
    }

    pub fn test_deriver() -> XoroshiroSplitter {
        Xoroshiro::from_seed(42).next_splitter()
    }

    pub fn test_random_config() -> GlobalRandomConfig {
        GlobalRandomConfig::new(42, false)
    }

    /// Both shipped routers lower end to end. If a future node type appears without an
    /// opcode, compilation must report it rather than emit a graph that silently
    /// evaluates to something wrong.
    #[test]
    fn real_routers_lower_end_to_end() {
        let config = test_random_config();

        for (name, stack) in [
            (
                "overworld",
                OVERWORLD_BASE_NOISE_ROUTER.noise.full_component_stack,
            ),
            (
                "nether",
                NETHER_BASE_NOISE_ROUTER.noise.full_component_stack,
            ),
        ] {
            let compiled = compile(stack, &config)
                .unwrap_or_else(|e| panic!("{name} router should lower fully, but hit {e}"));
            assert!(
                compiled.instructions.len() >= stack.len(),
                "{name}: each node lowers to at least one instruction"
            );
        }
    }

    /// A node type with no opcode has to be reported, not silently miscompiled.
    #[test]
    fn compile_reports_unsupported_nodes() {
        let config = test_random_config();
        // EndIslands has no opcode; it stands in for any future unlowered node.
        let stack = [BaseNoiseFunctionComponent::EndIslands];
        let err = compile(&stack, &config).expect_err("EndIslands has no opcode");
        assert_eq!(err.index, 0);
        assert_eq!(err.name, "EndIslands");
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
        let graph = CompiledGraph {
            instructions: vec![noise],
            samplers: pool,
            ..Default::default()
        };

        let mut max_diff = 0.0f64;
        for i in 0..500 {
            let x = f64::from(i) * 1.7;
            let y = f64::from(i) * -0.9;
            let z = f64::from(i) * 2.3;

            let expected = cpu_sampler.sample(x, y, z);
            let actual = evaluate_cpu(
                &graph,
                &BeardifierData::default(),
                x as f32,
                y as f32,
                z as f32,
            );
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
            (evaluate_cpu(
                &bare_graph(&graph),
                &BeardifierData::default(),
                0.0,
                -1000.0,
                0.0
            ) - 2.5)
                .abs()
                < 1e-6
        );

        // Above to_y the gradient saturates at to_value (1.0) -> *4 -> squeeze(1.0)
        // clamps to 1.0 -> 1/2 - 1/24, so the sum is 3.0 + ~0.4583, clamped to 2.5.
        assert!(
            (evaluate_cpu(
                &bare_graph(&graph),
                &BeardifierData::default(),
                0.0,
                1000.0,
                0.0
            ) - 2.5)
                .abs()
                < 1e-6
        );
    }

    #[test]
    fn unary_invert_matches_cpu_semantics_for_zero() {
        let mut constant = instruction(OpCode::Constant, 0);
        constant.param0 = 0.0;
        let mut invert = instruction(OpCode::UnaryInvert, 1);
        invert.input0 = 0;

        assert!(
            evaluate_cpu(
                &bare_graph(&[constant, invert]),
                &BeardifierData::default(),
                0.0,
                0.0,
                0.0
            )
            .is_infinite()
        );
        let _ = UnaryOperation::Invert;
    }
}
