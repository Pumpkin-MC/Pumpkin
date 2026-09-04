use pumpkin_data::noise_router::{
    BinaryData, BinaryOperation, ClampData, LinearData, RoundingData, RoundingOperation, UnaryData,
};
use pumpkin_util::math::vector3::Vector3;

use crate::generation::noise::router::{
    chunk_noise_router::{ChunkNoiseFunctionComponent, StaticChunkNoiseFunctionComponentImpl},
    density_volume::{DensityBuffer, DensityVolume},
};

use super::{NoiseFunctionComponentRange, StaticIndependentChunkNoiseFunctionComponentImpl};

pub struct Constant {
    value: f32,
}

impl Constant {
    pub const fn new(value: f32) -> Self {
        Self { value }
    }
}

impl NoiseFunctionComponentRange for Constant {
    #[inline]
    fn min(&self) -> f32 {
        self.value
    }

    #[inline]
    fn max(&self) -> f32 {
        self.value
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for Constant {
    fn sample(&self, _pos: &Vector3<i32>) -> f32 {
        self.value
    }

    fn sample_volume(&self, buffer: &mut [f32], _volume: &DensityVolume) {
        buffer.fill(self.value);
    }
}

pub struct Linear {
    pub(crate) input_index: usize,
    min_value: f32,
    max_value: f32,
    pub(crate) data: &'static LinearData,
}

impl NoiseFunctionComponentRange for Linear {
    #[inline]
    fn min(&self) -> f32 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f32 {
        self.max_value
    }
}

impl StaticChunkNoiseFunctionComponentImpl for Linear {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
    ) -> f32 {
        let input_density = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_index],
            pos,
        );
        self.data.apply_density(input_density)
    }

    fn sample_volume(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        buffer: &mut [f32],
        volume: &DensityVolume,
    ) {
        ChunkNoiseFunctionComponent::sample_volume_from_stack(
            &mut component_stack[..=self.input_index],
            buffer,
            volume,
        );
        for value in buffer.iter_mut() {
            *value = self.data.apply_density(*value);
        }
    }
}

impl Linear {
    pub const fn new(
        input_index: usize,
        min_value: f32,
        max_value: f32,
        data: &'static LinearData,
    ) -> Self {
        Self {
            input_index,
            min_value,
            max_value,
            data,
        }
    }
}

pub struct Binary {
    pub(crate) input1_index: usize,
    pub(crate) input2_index: usize,
    min_value: f32,
    max_value: f32,
    pub(crate) data: &'static BinaryData,
}

impl NoiseFunctionComponentRange for Binary {
    #[inline]
    fn min(&self) -> f32 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f32 {
        self.max_value
    }
}

impl StaticChunkNoiseFunctionComponentImpl for Binary {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
    ) -> f32 {
        let input1_density = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input1_index],
            pos,
        );

        match self.data.operation {
            BinaryOperation::Add => {
                let input2_density = ChunkNoiseFunctionComponent::sample_from_stack(
                    &mut component_stack[..=self.input2_index],
                    pos,
                );
                input1_density + input2_density
            }
            BinaryOperation::Mul => {
                if input1_density == 0.0 {
                    0.0
                } else {
                    let input2_density = ChunkNoiseFunctionComponent::sample_from_stack(
                        &mut component_stack[..=self.input2_index],
                        pos,
                    );
                    input1_density * input2_density
                }
            }
            BinaryOperation::Min => {
                let input2_min = component_stack[self.input2_index].min();

                if input1_density < input2_min {
                    input1_density
                } else {
                    let input2_density = ChunkNoiseFunctionComponent::sample_from_stack(
                        &mut component_stack[..=self.input2_index],
                        pos,
                    );

                    input1_density.min(input2_density)
                }
            }
            BinaryOperation::Max => {
                let input2_max = component_stack[self.input2_index].max();

                if input1_density > input2_max {
                    input1_density
                } else {
                    let input2_density = ChunkNoiseFunctionComponent::sample_from_stack(
                        &mut component_stack[..=self.input2_index],
                        pos,
                    );

                    input1_density.max(input2_density)
                }
            }
            BinaryOperation::Sub => {
                let input2_density = ChunkNoiseFunctionComponent::sample_from_stack(
                    &mut component_stack[..=self.input2_index],
                    pos,
                );
                input1_density - input2_density
            }
            BinaryOperation::Div => {
                if input1_density == 0.0 {
                    0.0
                } else {
                    let input2_density = ChunkNoiseFunctionComponent::sample_from_stack(
                        &mut component_stack[..=self.input2_index],
                        pos,
                    );
                    input1_density / input2_density
                }
            }
            BinaryOperation::Pow => {
                let input2_density = ChunkNoiseFunctionComponent::sample_from_stack(
                    &mut component_stack[..=self.input2_index],
                    pos,
                );
                f64::from(input1_density).powf(f64::from(input2_density)) as f32
            }
        }
    }

    fn sample_volume(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        buffer: &mut [f32],
        volume: &DensityVolume,
    ) {
        ChunkNoiseFunctionComponent::sample_volume_from_stack(
            &mut component_stack[..=self.input1_index],
            buffer,
            volume,
        );
        let mut input2 = DensityBuffer::acquire(volume);
        ChunkNoiseFunctionComponent::sample_volume_from_stack(
            &mut component_stack[..=self.input2_index],
            &mut input2,
            volume,
        );
        match self.data.operation {
            BinaryOperation::Mul => {
                for (value, input2) in buffer.iter_mut().zip(input2.iter()) {
                    *value *= input2;
                }
            }
            BinaryOperation::Div => {
                for (value, input2) in buffer.iter_mut().zip(input2.iter()) {
                    *value /= input2;
                }
            }
            BinaryOperation::Pow => {
                for (value, input2) in buffer.iter_mut().zip(input2.iter()) {
                    *value = f64::from(*value).powf(f64::from(*input2)) as f32;
                }
            }
            _ => {
                for (value, input2) in buffer.iter_mut().zip(input2.iter()) {
                    *value = self.data.apply_density(*value, *input2);
                }
            }
        }
    }
}

impl Binary {
    pub const fn new(
        input1_index: usize,
        input2_index: usize,
        min_value: f32,
        max_value: f32,
        data: &'static BinaryData,
    ) -> Self {
        Self {
            input1_index,
            input2_index,
            min_value,
            max_value,
            data,
        }
    }
}

pub struct Unary {
    pub(crate) input_index: usize,
    min_value: f32,
    max_value: f32,
    pub(crate) data: &'static UnaryData,
}

impl NoiseFunctionComponentRange for Unary {
    #[inline]
    fn min(&self) -> f32 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f32 {
        self.max_value
    }
}

impl StaticChunkNoiseFunctionComponentImpl for Unary {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
    ) -> f32 {
        let input_density = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_index],
            pos,
        );
        self.data.apply_density(input_density)
    }

    fn sample_volume(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        buffer: &mut [f32],
        volume: &DensityVolume,
    ) {
        ChunkNoiseFunctionComponent::sample_volume_from_stack(
            &mut component_stack[..=self.input_index],
            buffer,
            volume,
        );
        for value in buffer.iter_mut() {
            *value = self.data.apply_density(*value);
        }
    }
}

impl Unary {
    pub const fn new(
        input_index: usize,
        min_value: f32,
        max_value: f32,
        data: &'static UnaryData,
    ) -> Self {
        Self {
            input_index,
            min_value,
            max_value,
            data,
        }
    }
}

pub struct Clamp {
    pub(crate) input_index: usize,
    pub(crate) data: &'static ClampData,
}

impl Clamp {
    pub const fn new(input_index: usize, data: &'static ClampData) -> Self {
        Self { input_index, data }
    }
}

impl NoiseFunctionComponentRange for Clamp {
    #[inline]
    fn min(&self) -> f32 {
        self.data.min_value
    }

    #[inline]
    fn max(&self) -> f32 {
        self.data.max_value
    }
}

impl StaticChunkNoiseFunctionComponentImpl for Clamp {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
    ) -> f32 {
        let input_density = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_index],
            pos,
        );
        self.data.apply_density(input_density)
    }

    fn sample_volume(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        buffer: &mut [f32],
        volume: &DensityVolume,
    ) {
        ChunkNoiseFunctionComponent::sample_volume_from_stack(
            &mut component_stack[..=self.input_index],
            buffer,
            volume,
        );
        for value in buffer.iter_mut() {
            *value = self.data.apply_density(*value);
        }
    }
}

pub struct Lerp {
    pub(crate) alpha_index: usize,
    pub(crate) first_index: usize,
    pub(crate) second_index: usize,
    min_value: f32,
    max_value: f32,
}

impl Lerp {
    pub const fn new(
        alpha_index: usize,
        first_index: usize,
        second_index: usize,
        min_value: f32,
        max_value: f32,
    ) -> Self {
        Self {
            alpha_index,
            first_index,
            second_index,
            min_value,
            max_value,
        }
    }
}

impl NoiseFunctionComponentRange for Lerp {
    #[inline]
    fn min(&self) -> f32 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f32 {
        self.max_value
    }
}

impl StaticChunkNoiseFunctionComponentImpl for Lerp {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
    ) -> f32 {
        let alpha = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.alpha_index],
            pos,
        );
        if alpha == 0.0 {
            return ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=self.first_index],
                pos,
            );
        }
        if alpha == 1.0 {
            return ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=self.second_index],
                pos,
            );
        }
        let first = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.first_index],
            pos,
        );
        let second = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.second_index],
            pos,
        );
        first + alpha * (second - first)
    }

    fn sample_volume(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        buffer: &mut [f32],
        volume: &DensityVolume,
    ) {
        ChunkNoiseFunctionComponent::sample_volume_from_stack(
            &mut component_stack[..=self.alpha_index],
            buffer,
            volume,
        );
        let mut first = DensityBuffer::acquire(volume);
        ChunkNoiseFunctionComponent::sample_volume_from_stack(
            &mut component_stack[..=self.first_index],
            &mut first,
            volume,
        );
        let mut second = DensityBuffer::acquire(volume);
        ChunkNoiseFunctionComponent::sample_volume_from_stack(
            &mut component_stack[..=self.second_index],
            &mut second,
            volume,
        );
        for ((value, first), second) in buffer.iter_mut().zip(first.iter()).zip(second.iter()) {
            let alpha = *value;
            *value = if alpha == 0.0 {
                *first
            } else if alpha == 1.0 {
                *second
            } else {
                first + alpha * (second - first)
            };
        }
    }
}

pub struct Rounding {
    pub(crate) input_index: usize,
    pub(crate) multiple_index: usize,
    min_value: f32,
    max_value: f32,
    pub(crate) data: &'static RoundingData,
}

impl Rounding {
    pub const fn new(
        input_index: usize,
        multiple_index: usize,
        min_value: f32,
        max_value: f32,
        data: &'static RoundingData,
    ) -> Self {
        Self {
            input_index,
            multiple_index,
            min_value,
            max_value,
            data,
        }
    }
}

impl NoiseFunctionComponentRange for Rounding {
    #[inline]
    fn min(&self) -> f32 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f32 {
        self.max_value
    }
}

#[inline]
#[must_use]
pub fn round_to_integer(input: f32, op: RoundingOperation) -> f32 {
    match op {
        RoundingOperation::Floor => input.floor(),
        RoundingOperation::Round => (input + 0.5).floor(),
        RoundingOperation::Ceil => input.ceil(),
        RoundingOperation::Truncate => {
            if input > 0.0 {
                input.floor()
            } else {
                input.ceil()
            }
        }
    }
}

impl StaticChunkNoiseFunctionComponentImpl for Rounding {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
    ) -> f32 {
        let input = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_index],
            pos,
        );
        let multiple = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.multiple_index],
            pos,
        );
        if multiple == 0.0 {
            input
        } else {
            round_to_integer(input / multiple, self.data.operation) * multiple
        }
    }

    fn sample_volume(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        buffer: &mut [f32],
        volume: &DensityVolume,
    ) {
        ChunkNoiseFunctionComponent::sample_volume_from_stack(
            &mut component_stack[..=self.input_index],
            buffer,
            volume,
        );
        let mut multiple = DensityBuffer::acquire(volume);
        ChunkNoiseFunctionComponent::sample_volume_from_stack(
            &mut component_stack[..=self.multiple_index],
            &mut multiple,
            volume,
        );
        for (value, multiple) in buffer.iter_mut().zip(multiple.iter()) {
            if *multiple != 0.0 {
                *value = round_to_integer(*value / multiple, self.data.operation) * multiple;
            }
        }
    }
}
