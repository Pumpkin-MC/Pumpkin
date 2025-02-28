use crate::{
    generation::noise_router::{
        chunk_density_function::ChunkNoiseFunctionSampleOptions,
        chunk_noise_router::{ChunkNoiseFunctionComponent, StaticChunkNoiseFunctionComponentImpl},
    },
    noise_router::density_function_ast::{
        BinaryData, BinaryOperation, ClampData, LinearData, LinearOperation, UnaryData,
        UnaryOperation,
    },
};

use super::{
    IndexToNoisePos, NoiseFunctionComponentRange, NoisePos,
    StaticIndependentChunkNoiseFunctionComponentImpl,
};

#[derive(Clone)]
pub struct Constant {
    value: f64,
}

impl Constant {
    pub fn new(value: f64) -> Self {
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
    fn sample(&self, _pos: &impl NoisePos) -> f64 {
        self.value
    }

    fn fill(&self, array: &mut [f64], _mapper: &impl IndexToNoisePos) {
        array.fill(self.value);
    }
}

#[derive(Clone)]
pub struct Linear {
    pub input_index: usize,
    pub operation: LinearOperation,
    pub argument: f64,
    min_value: f64,
    max_value: f64,
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
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        let input_density = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_index],
            pos,
            sample_options,
        );
        self.apply_density(input_density)
    }
}

impl Linear {
    pub fn new(input_index: usize, data: &LinearData) -> Self {
        Self {
            input_index,
            operation: data.operation,
            argument: data.argument.0,
            min_value: data.min_value.0,
            max_value: data.max_value.0,
        }
    }

    #[inline]
    pub fn apply_density(&self, density: f64) -> f64 {
        match self.operation {
            LinearOperation::Add => density + self.argument,
            LinearOperation::Mul => density * self.argument,
        }
    }
}

#[derive(Clone)]
pub struct Binary {
    pub input1_index: usize,
    pub input2_index: usize,
    pub operation: BinaryOperation,
    pub min_value: f64,
    pub max_value: f64,
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
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        let input1_density = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input1_index],
            pos,
            sample_options,
        );

        match self.operation {
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

        match self.operation {
            BinaryOperation::Add => {
                array.iter_mut().enumerate().for_each(|(index, value)| {
                    let pos = mapper.at(index, Some(sample_options));
                    let density2 = ChunkNoiseFunctionComponent::sample_from_stack(
                        &mut component_stack[..=self.input2_index],
                        &pos,
                        sample_options,
                    );
                    *value += density2;
                });
            }
            BinaryOperation::Mul => {
                array.iter_mut().enumerate().for_each(|(index, value)| {
                    if *value != 0.0 {
                        let pos = mapper.at(index, Some(sample_options));
                        let density2 = ChunkNoiseFunctionComponent::sample_from_stack(
                            &mut component_stack[..=self.input2_index],
                            &pos,
                            sample_options,
                        );
                        *value *= density2;
                    }
                });
            }
            BinaryOperation::Min => {
                let input2_min = component_stack[self.input2_index].min();
                array.iter_mut().enumerate().for_each(|(index, value)| {
                    if *value > input2_min {
                        let pos = mapper.at(index, Some(sample_options));
                        let density2 = ChunkNoiseFunctionComponent::sample_from_stack(
                            &mut component_stack[..=self.input2_index],
                            &pos,
                            sample_options,
                        );
                        *value = value.min(density2);
                    }
                });
            }
            BinaryOperation::Max => {
                let input2_max = component_stack[self.input2_index].max();
                array.iter_mut().enumerate().for_each(|(index, value)| {
                    if *value < input2_max {
                        let pos = mapper.at(index, Some(sample_options));
                        let density2 = ChunkNoiseFunctionComponent::sample_from_stack(
                            &mut component_stack[..=self.input2_index],
                            &pos,
                            sample_options,
                        );
                        *value = value.max(density2);
                    }
                });
            }
        }
    }
}

impl Binary {
    pub fn new(input1_index: usize, input2_index: usize, data: &BinaryData) -> Self {
        Self {
            input1_index,
            input2_index,
            operation: data.operation,
            min_value: data.min_value.0,
            max_value: data.max_value.0,
        }
    }
}

#[derive(Clone)]
pub struct Unary {
    pub(crate) input_index: usize,
    pub operation: UnaryOperation,
    pub min_value: f64,
    pub max_value: f64,
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
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        let input_density = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_index],
            pos,
            sample_options,
        );
        self.apply_density(input_density)
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
        array.iter_mut().for_each(|value| {
            *value = self.apply_density(*value);
        });
    }
}

impl Unary {
    pub fn new(input_index: usize, data: &UnaryData) -> Self {
        Self {
            input_index,
            operation: data.operation,
            min_value: data.min_value.0,
            max_value: data.max_value.0,
        }
    }

    #[inline]
    pub fn apply_density(&self, density: f64) -> f64 {
        match self.operation {
            UnaryOperation::Abs => density.abs(),
            UnaryOperation::Square => density * density,
            UnaryOperation::Cube => density * density * density,
            UnaryOperation::HalfNegative => {
                if density > 0.0 {
                    density
                } else {
                    density * 0.5
                }
            }
            UnaryOperation::QuarterNegative => {
                if density > 0.0 {
                    density
                } else {
                    density * 0.25
                }
            }
            UnaryOperation::Squeeze => {
                let clamped = density.clamp(-1.0, 1.0);
                clamped / 2.0 - clamped * clamped * clamped / 24.0
            }
        }
    }
}

#[derive(Clone)]
pub struct Clamp {
    pub input_index: usize,
    pub min_value: f64,
    pub max_value: f64,
}

impl Clamp {
    pub fn new(input_index: usize, data: &ClampData) -> Self {
        Self {
            input_index,
            min_value: data.min_value.0,
            max_value: data.max_value.0,
        }
    }

    #[inline]
    pub fn apply_density(&self, density: f64) -> f64 {
        density.clamp(self.min_value, self.max_value)
    }
}

impl NoiseFunctionComponentRange for Clamp {
    #[inline]
    fn min(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f64 {
        self.max_value
    }
}

impl StaticChunkNoiseFunctionComponentImpl for Clamp {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        let input_density = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_index],
            pos,
            sample_options,
        );
        self.apply_density(input_density)
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
        array.iter_mut().for_each(|value| {
            *value = self.apply_density(*value);
        });
    }
}
