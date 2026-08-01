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

use bytemuck::{Pod, Zeroable};
use pumpkin_data::noise_router::{
    BaseNoiseFunctionComponent, BinaryOperation, LinearOperation, UnaryOperation,
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
}

/// One node of the flattened graph.
///
/// `input0`/`input1` index earlier entries in the same instruction list; unused inputs
/// are set to the node's own index so a buggy read stays in bounds instead of reading
/// past the buffer.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Debug)]
pub struct Instruction {
    pub opcode: u32,
    pub input0: u32,
    pub input1: u32,
    /// Padding so the struct matches the WGSL `Instruction` layout; never read.
    pub padding: u32,
    pub param0: f32,
    pub param1: f32,
    pub param2: f32,
    pub param3: f32,
}

impl Instruction {
    const fn new(opcode: OpCode, index: usize) -> Self {
        let own = index as u32;
        Self {
            opcode: opcode as u32,
            input0: own,
            input1: own,
            padding: 0,
            param0: 0.0,
            param1: 0.0,
            param2: 0.0,
            param3: 0.0,
        }
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

/// Lowers a component stack into GPU instructions.
///
/// # Errors
/// Returns [`UnsupportedNode`] for the first node whose type has no opcode yet, so
/// callers can fall back to the CPU path instead of generating wrong terrain.
pub fn compile(stack: &[BaseNoiseFunctionComponent]) -> Result<Vec<Instruction>, UnsupportedNode> {
    let mut out = Vec::with_capacity(stack.len());

    for (index, component) in stack.iter().enumerate() {
        let instruction = match component {
            BaseNoiseFunctionComponent::Constant { value } => {
                let mut i = Instruction::new(OpCode::Constant, index);
                i.param0 = *value as f32;
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

    Ok(out)
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
pub fn evaluate_cpu(instructions: &[Instruction], x: f32, y: f32, z: f32) -> f32 {
    let mut values = vec![0.0f32; instructions.len()];

    for (index, instruction) in instructions.iter().enumerate() {
        let a = values[instruction.input0 as usize];
        let b = values[instruction.input1 as usize];
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
            _ => 0.0,
        };
        let _ = (x, z);
    }

    values.last().copied().unwrap_or(0.0)
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
    use super::{Instruction, OpCode, compile, evaluate_cpu};
    use pumpkin_data::noise_router::{
        NETHER_BASE_NOISE_ROUTER, OVERWORLD_BASE_NOISE_ROUTER, UnaryOperation,
    };

    fn instruction(opcode: OpCode, index: usize) -> Instruction {
        Instruction::new(opcode, index)
    }

    /// The real graphs still contain node types with no opcode yet (noise, splines,
    /// ...). Compilation must say so explicitly rather than emitting a graph that
    /// silently evaluates to something wrong.
    #[test]
    fn real_routers_report_unsupported_nodes_instead_of_miscompiling() {
        for stack in [
            OVERWORLD_BASE_NOISE_ROUTER.noise.full_component_stack,
            NETHER_BASE_NOISE_ROUTER.noise.full_component_stack,
        ] {
            let err = compile(stack).expect_err("noise/spline nodes are not supported yet");
            assert!(
                stack.len() > err.index,
                "reported index must point into the stack"
            );
        }
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
        assert!((evaluate_cpu(&graph, 0.0, -1000.0, 0.0) - 2.5).abs() < 1e-6);

        // Above to_y the gradient saturates at to_value (1.0) -> *4 -> squeeze(1.0)
        // clamps to 1.0 -> 1/2 - 1/24, so the sum is 3.0 + ~0.4583, clamped to 2.5.
        assert!((evaluate_cpu(&graph, 0.0, 1000.0, 0.0) - 2.5).abs() < 1e-6);
    }

    #[test]
    fn unary_invert_matches_cpu_semantics_for_zero() {
        let mut constant = instruction(OpCode::Constant, 0);
        constant.param0 = 0.0;
        let mut invert = instruction(OpCode::UnaryInvert, 1);
        invert.input0 = 0;

        assert!(evaluate_cpu(&[constant, invert], 0.0, 0.0, 0.0).is_infinite());
        let _ = UnaryOperation::Invert;
    }
}
