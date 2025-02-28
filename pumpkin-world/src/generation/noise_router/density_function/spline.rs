use crate::{
    generation::{
        noise::lerp,
        noise_router::{
            chunk_density_function::ChunkNoiseFunctionSampleOptions,
            chunk_noise_router::{
                ChunkNoiseFunctionComponent, StaticChunkNoiseFunctionComponentImpl,
            },
        },
    },
    noise_router::density_function_ast::SplineData,
};

use super::{NoiseFunctionComponentRange, NoisePos};

#[derive(Clone)]
pub enum SplineValue {
    Spline(Spline),
    Fixed(f32),
}

impl SplineValue {
    #[inline]
    fn sample(
        &self,
        pos: &impl NoisePos,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f32 {
        match self {
            Self::Fixed(fixed) => *fixed,
            Self::Spline(spline) => spline.sample(pos, component_stack, sample_options),
        }
    }
}

#[derive(Clone)]
pub struct SplinePoint {
    pub location: f32,
    pub value: SplineValue,
    pub derivative: f32,
}

impl SplinePoint {
    pub fn new(location: f32, value: SplineValue, derivative: f32) -> Self {
        Self {
            location,
            value,
            derivative,
        }
    }

    fn sample_outside_range(&self, sample_location: f32, last_known_sample: f32) -> f32 {
        if self.derivative == 0f32 {
            last_known_sample
        } else {
            self.derivative * (sample_location - self.location) + last_known_sample
        }
    }
}

/// Returns the smallest usize between min..max that does not match the predicate
fn binary_walk(min: usize, max: usize, pred: impl Fn(usize) -> bool) -> usize {
    let mut i = max - min;
    let mut min = min;
    while i > 0 {
        let j = i / 2;
        let k = min + j;
        if pred(k) {
            i = j;
        } else {
            min = k + 1;
            i -= j + 1;
        }
    }
    min
}

pub enum Range {
    In(usize),
    Below,
}

#[derive(Clone)]
pub struct Spline {
    pub input_index: usize,
    pub points: Box<[SplinePoint]>,
}

impl Spline {
    pub fn new(input_index: usize, points: Box<[SplinePoint]>) -> Self {
        Self {
            input_index,
            points,
        }
    }

    fn find_index_for_location(&self, loc: f32) -> Range {
        let index_greater_than_x =
            binary_walk(0, self.points.len(), |i| loc < self.points[i].location);
        if index_greater_than_x == 0 {
            Range::Below
        } else {
            Range::In(index_greater_than_x - 1)
        }
    }

    fn sample(
        &self,
        pos: &impl NoisePos,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f32 {
        let location = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_index],
            pos,
            sample_options,
        ) as f32;

        match self.find_index_for_location(location) {
            Range::In(index) => {
                if index == self.points.len() - 1 {
                    let last_known_sample =
                        self.points[index]
                            .value
                            .sample(pos, component_stack, sample_options);
                    self.points[index].sample_outside_range(location, last_known_sample)
                } else {
                    let lower_point = &self.points[index];
                    let upper_point = &self.points[index + 1];

                    let lower_value =
                        lower_point
                            .value
                            .sample(pos, component_stack, sample_options);
                    let upper_value =
                        upper_point
                            .value
                            .sample(pos, component_stack, sample_options);

                    // Use linear interpolation (-ish cuz of derivatives) to derivate a point between two points
                    let x_scale = (location - lower_point.location)
                        / (upper_point.location - lower_point.location);
                    let extrapolated_lower_value = lower_point.derivative
                        * (upper_point.location - lower_point.location)
                        - (upper_value - lower_value);
                    let extrapolated_upper_value = -upper_point.derivative
                        * (upper_point.location - lower_point.location)
                        + (upper_value - lower_value);

                    (x_scale * (1f32 - x_scale))
                        * lerp(x_scale, extrapolated_lower_value, extrapolated_upper_value)
                        + lerp(x_scale, lower_value, upper_value)
                }
            }
            Range::Below => {
                let last_known_sample =
                    self.points[0]
                        .value
                        .sample(pos, component_stack, sample_options);
                self.points[0].sample_outside_range(location, last_known_sample)
            }
        }
    }
}

#[derive(Clone)]
pub struct SplineFunction {
    pub spline: Spline,
    pub min_value: f64,
    pub max_value: f64,
}

impl SplineFunction {
    pub fn new(spline: Spline, data: &SplineData) -> Self {
        Self {
            spline,
            min_value: data.min_value.0,
            max_value: data.max_value.0,
        }
    }
}

impl StaticChunkNoiseFunctionComponentImpl for SplineFunction {
    #[inline]
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        self.spline.sample(pos, component_stack, sample_options) as f64
    }
}

impl NoiseFunctionComponentRange for SplineFunction {
    #[inline]
    fn min(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f64 {
        self.max_value
    }
}
