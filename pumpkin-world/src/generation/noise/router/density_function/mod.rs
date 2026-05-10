use enum_dispatch::enum_dispatch;
use pumpkin_data::noise_router::WrapperType;
use pumpkin_util::math::vector3::Vector3;

// These are for enum_dispatch
use super::chunk_density_function::{
    ChunkNoiseFunctionSampleOptions, ChunkSpecificNoiseFunctionComponent,
};

pub(crate) mod math;
pub(crate) mod misc;
pub(crate) mod noise;
pub(crate) mod spline;

#[cfg(test)]
mod test;
// Helper functions for deserializing unique density functions for testing
#[cfg(test)]
mod test_deserializer;

pub trait IndexToNoisePos {
    fn at(
        &self,
        index: usize,
        sample_options: Option<&mut ChunkNoiseFunctionSampleOptions>,
    ) -> Vector3<i32>;
}

#[enum_dispatch]
pub trait NoiseFunctionComponentRange {
    fn min(&self) -> f64;
    fn max(&self) -> f64;
}

#[enum_dispatch]
pub trait StaticIndependentChunkNoiseFunctionComponentImpl: NoiseFunctionComponentRange {
    fn sample(&self, pos: &Vector3<i32>) -> f64;
    fn fill(&self, array: &mut [f64], mapper: &impl IndexToNoisePos) {
        array.iter_mut().enumerate().for_each(|(index, value)| {
            let pos = mapper.at(index, None);
            *value = self.sample(&pos);
        });
    }
}

#[derive(Clone)]
pub struct Wrapper {
    pub input_index: usize,
    pub wrapper_type: WrapperType,
    min_value: f64,
    max_value: f64,
}

impl Wrapper {
    #[must_use]
    pub const fn new(
        input_index: usize,
        wrapper_type: WrapperType,
        min_value: f64,
        max_value: f64,
    ) -> Self {
        Self {
            input_index,
            wrapper_type,
            min_value,
            max_value,
        }
    }
}

impl NoiseFunctionComponentRange for Wrapper {
    #[inline]
    fn min(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f64 {
        self.max_value
    }
}

#[derive(Clone)]
pub struct PassThrough {
    input_index: usize,
    min_value: f64,
    max_value: f64,
}

impl PassThrough {
    #[must_use]
    pub const fn new(input_index: usize, min_value: f64, max_value: f64) -> Self {
        Self {
            input_index,
            min_value,
            max_value,
        }
    }

    #[must_use]
    pub const fn input_index(&self) -> usize {
        self.input_index
    }
}

impl NoiseFunctionComponentRange for PassThrough {
    #[inline]
    fn min(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f64 {
        self.max_value
    }
}
