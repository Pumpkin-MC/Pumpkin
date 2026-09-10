use pumpkin_data::noise_router::WrapperType;
use pumpkin_util::math::vector3::Vector3;

use super::density_volume::DensityVolume;

pub(crate) mod beardifier;
pub(crate) mod math;
pub(crate) mod misc;
pub(crate) mod noise;
pub(crate) mod spline;

#[cfg(test)]
mod test;
// Helper functions for deserializing unique density functions for testing
#[cfg(test)]
mod test_deserializer;

pub trait NoiseFunctionComponentRange {
    fn min(&self) -> f32;
    fn max(&self) -> f32;
}

pub trait StaticIndependentChunkNoiseFunctionComponentImpl: NoiseFunctionComponentRange {
    fn sample(&self, pos: &Vector3<i32>) -> f32;

    fn sample_volume(&self, buffer: &mut [f32], volume: &DensityVolume) {
        volume.fill_with(buffer, |pos| self.sample(pos));
    }
}

pub struct Wrapper {
    pub input_index: usize,
    pub wrapper_type: WrapperType,
    min_value: f32,
    max_value: f32,
}

impl Wrapper {
    #[must_use]
    pub const fn new(
        input_index: usize,
        wrapper_type: WrapperType,
        min_value: f32,
        max_value: f32,
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
    fn min(&self) -> f32 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f32 {
        self.max_value
    }
}

#[derive(Clone)]
pub struct PassThrough {
    input_index: usize,
    min_value: f32,
    max_value: f32,
}

impl PassThrough {
    #[must_use]
    pub const fn new(input_index: usize, min_value: f32, max_value: f32) -> Self {
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
    fn min(&self) -> f32 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f32 {
        self.max_value
    }
}
