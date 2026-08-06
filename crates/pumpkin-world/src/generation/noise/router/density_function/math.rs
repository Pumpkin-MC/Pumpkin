use std::cell::RefCell;

use pumpkin_data::noise_router::{BinaryData, BinaryOperation, ClampData, LinearData, UnaryData};
use pumpkin_util::math::vector3::Vector3;

use crate::generation::noise::router::{
    chunk_density_function::ChunkNoiseFunctionSampleOptions,
    chunk_noise_router::{ChunkNoiseFunctionComponent, StaticChunkNoiseFunctionComponentImpl},
};

thread_local! {
    static SCRATCH_BUFFER: RefCell<Vec<f64>> = const { RefCell::new(Vec::new()) };
}

use super::{
    IndexToNoisePos, NoiseFunctionComponentRange, StaticIndependentChunkNoiseFunctionComponentImpl,
};

pub struct Constant {
    value: f64,
}

impl Constant {
    pub const fn new(value: f64) -> Self {
        Self { value }
    }
}

impl NoiseFunctionComponentRange for Constant {
    #[inline]
    fn min(&self) -> f64 {
        self.value
    }

    #[inline]
    fn max(&self) -> f64 {
        self.value
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for Constant {
    fn sample(&self, _pos: &Vector3<i32>) -> f64 {
        self.value
    }

    fn fill(&self, array: &mut [f64], _mapper: &impl IndexToNoisePos) {
        array.fill(self.value);
    }
}

pub struct Linear {
    pub(crate) input_index: usize,
    min_value: f64,
    max_value: f64,
    pub(crate) data: &'static LinearData,
}

impl NoiseFunctionComponentRange for Linear {
    #[inline]
    fn min(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f64 {
        self.max_value
    }
}

impl StaticChunkNoiseFunctionComponentImpl for Linear {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        let input_density = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_index],
            pos,
            sample_options,
        );
        self.data.apply_density(input_density)
    }
}

impl Linear {
    pub const fn new(
        input_index: usize,
        min_value: f64,
        max_value: f64,
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
    min_value: f64,
    max_value: f64,
    pub(crate) data: &'static BinaryData,
}

impl NoiseFunctionComponentRange for Binary {
    #[inline]
    fn min(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f64 {
        self.max_value
    }
}

impl StaticChunkNoiseFunctionComponentImpl for Binary {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        let input1_density = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input1_index],
            pos,
            sample_options,
        );

        match self.data.operation {
            BinaryOperation::Add => {
                let input2_density = ChunkNoiseFunctionComponent::sample_from_stack(
                    &mut component_stack[..=self.input2_index],
                    pos,
                    sample_options,
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
                        sample_options,
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
                        sample_options,
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
                        sample_options,
                    );

                    input1_density.max(input2_density)
                }
            }
        }
    }

    fn fill(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        array: &mut [f64],
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        ChunkNoiseFunctionComponent::fill_from_stack(
            &mut component_stack[..=self.input1_index],
            array,
            mapper,
            sample_options,
        );

        let len = array.len();
        match self.data.operation {
            BinaryOperation::Add => {
                SCRATCH_BUFFER.with(|scratch_cell| {
                    let mut scratch = scratch_cell.borrow_mut();
                    if scratch.len() < len {
                        scratch.resize(len, 0.0);
                    }
                    let scratch_slice = &mut scratch[..len];

                    ChunkNoiseFunctionComponent::fill_from_stack(
                        &mut component_stack[..=self.input2_index],
                        scratch_slice,
                        mapper,
                        sample_options,
                    );

                    for (a, b) in array.iter_mut().zip(scratch_slice.iter()) {
                        *a += b;
                    }
                });
            }
            BinaryOperation::Mul => {
                for (index, value) in array.iter_mut().enumerate() {
                    if *value != 0.0 {
                        let pos = mapper.at(index, Some(sample_options));
                        let density2 = ChunkNoiseFunctionComponent::sample_from_stack(
                            &mut component_stack[..=self.input2_index],
                            &pos,
                            sample_options,
                        );
                        *value *= density2;
                    }
                }
            }
            BinaryOperation::Min => {
                let input2_min = component_stack[self.input2_index].min();
                for (index, value) in array.iter_mut().enumerate() {
                    if *value >= input2_min {
                        let pos = mapper.at(index, Some(sample_options));
                        let density2 = ChunkNoiseFunctionComponent::sample_from_stack(
                            &mut component_stack[..=self.input2_index],
                            &pos,
                            sample_options,
                        );
                        *value = value.min(density2);
                    }
                }
            }
            BinaryOperation::Max => {
                let input2_max = component_stack[self.input2_index].max();
                for (index, value) in array.iter_mut().enumerate() {
                    if *value <= input2_max {
                        let pos = mapper.at(index, Some(sample_options));
                        let density2 = ChunkNoiseFunctionComponent::sample_from_stack(
                            &mut component_stack[..=self.input2_index],
                            &pos,
                            sample_options,
                        );
                        *value = value.max(density2);
                    }
                }
            }
        }
    }
}

impl Binary {
    pub const fn new(
        input1_index: usize,
        input2_index: usize,
        min_value: f64,
        max_value: f64,
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
    min_value: f64,
    max_value: f64,
    pub(crate) data: &'static UnaryData,
}

impl NoiseFunctionComponentRange for Unary {
    #[inline]
    fn min(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f64 {
        self.max_value
    }
}

impl StaticChunkNoiseFunctionComponentImpl for Unary {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        let input_density = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_index],
            pos,
            sample_options,
        );
        self.data.apply_density(input_density)
    }

    fn fill(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        array: &mut [f64],
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        ChunkNoiseFunctionComponent::fill_from_stack(
            &mut component_stack[..=self.input_index],
            array,
            mapper,
            sample_options,
        );
        for value in array.iter_mut() {
            *value = self.data.apply_density(*value);
        }
    }
}

impl Unary {
    pub const fn new(
        input_index: usize,
        min_value: f64,
        max_value: f64,
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
    fn min(&self) -> f64 {
        self.data.min_value
    }

    #[inline]
    fn max(&self) -> f64 {
        self.data.max_value
    }
}

impl StaticChunkNoiseFunctionComponentImpl for Clamp {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        let input_density = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_index],
            pos,
            sample_options,
        );
        self.data.apply_density(input_density)
    }

    fn fill(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        array: &mut [f64],
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        ChunkNoiseFunctionComponent::fill_from_stack(
            &mut component_stack[..=self.input_index],
            array,
            mapper,
            sample_options,
        );
        for value in array.iter_mut() {
            *value = self.data.apply_density(*value);
        }
    }
}
