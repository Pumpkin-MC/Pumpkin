use std::fmt::Debug;

use enum_dispatch::enum_dispatch;
use pumpkin_data::noise_router::WrapperType;

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

pub trait NoisePos: Debug {
    fn x(&self) -> i32;
    fn y(&self) -> i32;
    fn z(&self) -> i32;
}

#[derive(Debug)]
pub struct UnblendedNoisePos {
    x: i32,
    y: i32,
    z: i32,
}

impl UnblendedNoisePos {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

impl NoisePos for UnblendedNoisePos {
    #[inline]
    fn x(&self) -> i32 {
        self.x
    }

    #[inline]
    fn y(&self) -> i32 {
        self.y
    }

    #[inline]
    fn z(&self) -> i32 {
        self.z
    }
}

pub trait IndexToNoisePos {
    fn at(
        &self,
        index: usize,
        sample_options: Option<&mut ChunkNoiseFunctionSampleOptions>,
    ) -> impl NoisePos + 'static;
}

#[enum_dispatch]
pub trait NoiseFunctionComponentRange {
    fn min(&self) -> f64;
    fn max(&self) -> f64;
}

#[enum_dispatch]
pub trait StaticIndependentChunkNoiseFunctionComponentImpl: NoiseFunctionComponentRange {
    fn sample(&self, pos: &impl NoisePos) -> f64;
    fn fill(&self, array: &mut [f64], mapper: &impl IndexToNoisePos) {
        array.iter_mut().enumerate().for_each(|(index, value)| {
            let pos = mapper.at(index, None);
            *value = self.sample(&pos);
        });
    }
}

#[derive(Clone)]
pub struct Wrapper {
    input_index: usize,
    wrapper_type: WrapperType,
    min_value: f64,
    max_value: f64,
}

impl Wrapper {
    pub fn new(
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

    pub fn input_index(&self) -> usize {
        self.input_index
    }

    pub fn wrapper_type(&self) -> WrapperType {
        self.wrapper_type
    }
}

impl NoiseFunctionComponentRange for Wrapper {
    fn min(&self) -> f64 {
        self.min_value
    }

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
    pub fn new(input_index: usize, min_value: f64, max_value: f64) -> Self {
        Self {
            input_index,
            min_value,
            max_value,
        }
    }

    pub fn input_index(&self) -> usize {
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
