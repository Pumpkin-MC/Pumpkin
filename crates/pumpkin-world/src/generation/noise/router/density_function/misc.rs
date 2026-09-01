use std::sync::Arc;

use pumpkin_data::noise_router::{
    Axis, ClampedYGradientData, DistanceMetric, DistanceToPointData, GradientData, RangeChoiceData,
    Tiling,
};
use pumpkin_util::{
    math::{clamped_map, vector3::Vector3},
    noise::simplex::SimplexNoiseSampler,
    random::{RandomImpl, legacy_rand::LegacyRand},
};

use crate::generation::noise::router::{
    chunk_density_function::ChunkNoiseFunctionSampleOptions,
    chunk_noise_router::{ChunkNoiseFunctionComponent, StaticChunkNoiseFunctionComponentImpl},
};

use super::{
    IndexToNoisePos, NoiseFunctionComponentRange, StaticIndependentChunkNoiseFunctionComponentImpl,
};

pub struct EndIsland {
    sampler: Arc<SimplexNoiseSampler>,
}

impl EndIsland {
    pub fn new(seed: u64) -> Self {
        let mut rand = LegacyRand::from_seed(seed);
        rand.skip(17292);
        Self {
            sampler: Arc::new(SimplexNoiseSampler::new(&mut rand)),
        }
    }

    fn sample_2d(sampler: &SimplexNoiseSampler, x: i32, z: i32) -> f32 {
        let i = x / 2;
        let j = z / 2;
        let k = x % 2;
        let l = z % 2;

        let mut f = -100.0f32;

        for m in -12..=12 {
            for n in -12..=12 {
                let o = (i + m) as i64;
                let p = (j + n) as i64;

                if (o * o + p * p) > 4096 && sampler.sample_2d(o as f64, p as f64) < -0.9 {
                    let g = (o as f32).abs().mul_add(3439.0, (p as f32).abs() * 147.0) % 13.0 + 9.0;
                    let h = (k - m * 2) as f32;
                    let q = (l - n * 2) as f32;
                    let r = h.hypot(q).mul_add(-g, 100.0);
                    let s = r.clamp(-100.0, 80.0);

                    f = f.max(s);
                }
            }
        }

        f
    }
}

// These values are hardcoded from java
impl NoiseFunctionComponentRange for EndIsland {
    #[inline]
    fn min(&self) -> f32 {
        -0.84375
    }

    #[inline]
    fn max(&self) -> f32 {
        0.5625
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for EndIsland {
    fn sample(&self, pos: &Vector3<i32>) -> f32 {
        (Self::sample_2d(&self.sampler, pos.x / 8, pos.z / 8) - 8.0) / 128.0
    }
}

pub struct IntervalSelect {
    pub input_index: usize,
    pub thresholds: &'static [f32],
    pub functions_indices: &'static [usize],
    min_value: f32,
    max_value: f32,
}

impl IntervalSelect {
    pub const fn new(
        input_index: usize,
        thresholds: &'static [f32],
        functions_indices: &'static [usize],
        min_value: f32,
        max_value: f32,
    ) -> Self {
        Self {
            input_index,
            thresholds,
            functions_indices,
            min_value,
            max_value,
        }
    }
}

impl StaticChunkNoiseFunctionComponentImpl for IntervalSelect {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f32 {
        let input_val = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_index],
            pos,
            sample_options,
        );

        let mut selected_index = self.thresholds.len();
        for (i, &threshold) in self.thresholds.iter().enumerate() {
            if input_val < threshold {
                selected_index = i;
                break;
            }
        }

        let func_index = self.functions_indices[selected_index];
        ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=func_index],
            pos,
            sample_options,
        )
    }

    fn fill(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        array: &mut [f32],
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        ChunkNoiseFunctionComponent::fill_from_stack(
            &mut component_stack[..=self.input_index],
            array,
            mapper,
            sample_options,
        );

        array.iter_mut().enumerate().for_each(|(index, value)| {
            let mut selected_index = self.thresholds.len();
            for (i, &threshold) in self.thresholds.iter().enumerate() {
                if *value < threshold {
                    selected_index = i;
                    break;
                }
            }

            let func_index = self.functions_indices[selected_index];
            let pos = mapper.at(index, Some(sample_options));
            *value = ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=func_index],
                &pos,
                sample_options,
            );
        });
    }
}

impl NoiseFunctionComponentRange for IntervalSelect {
    #[inline]
    fn min(&self) -> f32 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f32 {
        self.max_value
    }
}

pub struct ClampedYGradient {
    data: &'static ClampedYGradientData,
}

impl ClampedYGradient {
    pub const fn new(data: &'static ClampedYGradientData) -> Self {
        Self { data }
    }
}

impl NoiseFunctionComponentRange for ClampedYGradient {
    #[inline]
    fn min(&self) -> f32 {
        self.data.from_value.min(self.data.to_value)
    }

    #[inline]
    fn max(&self) -> f32 {
        self.data.from_value.max(self.data.to_value)
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for ClampedYGradient {
    fn sample(&self, pos: &Vector3<i32>) -> f32 {
        clamped_map(
            pos.y as f32,
            self.data.from_y,
            self.data.to_y,
            self.data.from_value,
            self.data.to_value,
        )
    }

    fn fill(&self, array: &mut [f32], mapper: &impl IndexToNoisePos) {
        array.iter_mut().enumerate().for_each(|(index, value)| {
            let pos = mapper.at(index, None);
            *value = clamped_map(
                pos.y as f32,
                self.data.from_y,
                self.data.to_y,
                self.data.from_value,
                self.data.to_value,
            );
        });
    }
}

pub struct Gradient {
    data: &'static GradientData,
}

impl Gradient {
    pub const fn new(data: &'static GradientData) -> Self {
        Self { data }
    }
}

impl NoiseFunctionComponentRange for Gradient {
    #[inline]
    fn min(&self) -> f32 {
        self.data.from_value.min(self.data.to_value)
    }

    #[inline]
    fn max(&self) -> f32 {
        self.data.from_value.max(self.data.to_value)
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for Gradient {
    fn sample(&self, pos: &Vector3<i32>) -> f32 {
        let coordinate = match self.data.axis {
            Axis::X => pos.x,
            Axis::Y => pos.y,
            Axis::Z => pos.z,
        };
        let coordinate_range = self.data.to_coordinate - self.data.from_coordinate;
        let coordinate_factor =
            (self.data.to_value - self.data.from_value) / (coordinate_range as f32);
        match self.data.tiling {
            Tiling::ClampToEdge => {
                let min_coordinate = self.data.from_coordinate.min(self.data.to_coordinate);
                let max_coordinate = self.data.from_coordinate.max(self.data.to_coordinate);
                let relative_coordinate =
                    coordinate.clamp(min_coordinate, max_coordinate) - self.data.from_coordinate;
                self.data.from_value + relative_coordinate as f32 * coordinate_factor
            }
            Tiling::Repeat => {
                let relative_coordinate = coordinate - self.data.from_coordinate;
                self.data.from_value
                    + relative_coordinate.rem_euclid(coordinate_range) as f32 * coordinate_factor
            }
            Tiling::MirroredRepeat => {
                let relative_coordinate = coordinate - self.data.from_coordinate;
                let tile_index = relative_coordinate.div_euclid(coordinate_range);
                let local_coordinate = relative_coordinate - tile_index * coordinate_range;
                if (tile_index & 1) == 0 {
                    self.data.from_value + local_coordinate as f32 * coordinate_factor
                } else {
                    self.data.from_value
                        + (coordinate_range - local_coordinate) as f32 * coordinate_factor
                }
            }
        }
    }

    fn fill(&self, array: &mut [f32], mapper: &impl IndexToNoisePos) {
        array.iter_mut().enumerate().for_each(|(index, value)| {
            let pos = mapper.at(index, None);
            *value = self.sample(&pos);
        });
    }
}

pub struct DistanceToPoint {
    data: &'static DistanceToPointData,
}

impl DistanceToPoint {
    pub const fn new(data: &'static DistanceToPointData) -> Self {
        Self { data }
    }
}

impl NoiseFunctionComponentRange for DistanceToPoint {
    #[inline]
    fn min(&self) -> f32 {
        0.0
    }

    #[inline]
    fn max(&self) -> f32 {
        f32::INFINITY
    }
}

impl StaticIndependentChunkNoiseFunctionComponentImpl for DistanceToPoint {
    fn sample(&self, pos: &Vector3<i32>) -> f32 {
        let dx = (pos.x - self.data.point[0]) as f32;
        let dy = (pos.y - self.data.point[1]) as f32;
        let dz = (pos.z - self.data.point[2]) as f32;
        match self.data.metric {
            DistanceMetric::Euclidean => (dx * dx + dy * dy + dz * dz).sqrt(),
            DistanceMetric::EuclideanSquared => dx * dx + dy * dy + dz * dz,
            DistanceMetric::Manhattan => dx.abs() + dy.abs() + dz.abs(),
            DistanceMetric::Chebyshev => dx.abs().max(dy.abs()).max(dz.abs()),
        }
    }

    fn fill(&self, array: &mut [f32], mapper: &impl IndexToNoisePos) {
        array.iter_mut().enumerate().for_each(|(index, value)| {
            let pos = mapper.at(index, None);
            *value = self.sample(&pos);
        });
    }
}

pub struct Slice {
    pub(crate) input_index: usize,
    pub(crate) axis: Axis,
    pub(crate) coordinate: i32,
    min_value: f32,
    max_value: f32,
}

impl Slice {
    pub const fn new(
        input_index: usize,
        axis: Axis,
        coordinate: i32,
        min_value: f32,
        max_value: f32,
    ) -> Self {
        Self {
            input_index,
            axis,
            coordinate,
            min_value,
            max_value,
        }
    }
}

impl NoiseFunctionComponentRange for Slice {
    #[inline]
    fn min(&self) -> f32 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f32 {
        self.max_value
    }
}

impl StaticChunkNoiseFunctionComponentImpl for Slice {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f32 {
        let slice_pos = match self.axis {
            Axis::X => Vector3::new(self.coordinate, pos.y, pos.z),
            Axis::Y => Vector3::new(pos.x, self.coordinate, pos.z),
            Axis::Z => Vector3::new(pos.x, pos.y, self.coordinate),
        };
        ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_index],
            &slice_pos,
            sample_options,
        )
    }

    fn fill(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        array: &mut [f32],
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        for (index, value) in array.iter_mut().enumerate() {
            let pos = mapper.at(index, Some(sample_options));
            *value = self.sample(component_stack, &pos, sample_options);
        }
    }
}

pub struct RangeChoice {
    pub(crate) input_index: usize,
    pub(crate) when_in_index: usize,
    pub(crate) when_out_index: usize,
    pub(crate) data: &'static RangeChoiceData,
    min_value: f32,
    max_value: f32,
}

impl RangeChoice {
    pub const fn new(
        input_index: usize,
        when_in_index: usize,
        when_out_index: usize,
        min_value: f32,
        max_value: f32,
        data: &'static RangeChoiceData,
    ) -> Self {
        Self {
            input_index,
            when_in_index,
            when_out_index,
            data,
            min_value,
            max_value,
        }
    }
}

impl NoiseFunctionComponentRange for RangeChoice {
    #[inline]
    fn min(&self) -> f32 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f32 {
        self.max_value
    }
}

impl StaticChunkNoiseFunctionComponentImpl for RangeChoice {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f32 {
        let input_sample = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_index],
            pos,
            sample_options,
        );

        if self.data.min_inclusive <= input_sample && input_sample < self.data.max_exclusive {
            ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=self.when_in_index],
                pos,
                sample_options,
            )
        } else {
            ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=self.when_out_index],
                pos,
                sample_options,
            )
        }
    }

    fn fill(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        array: &mut [f32],
        mapper: &impl IndexToNoisePos,
        sample_options: &mut ChunkNoiseFunctionSampleOptions,
    ) {
        ChunkNoiseFunctionComponent::fill_from_stack(
            &mut component_stack[..=self.input_index],
            array,
            mapper,
            sample_options,
        );

        array.iter_mut().enumerate().for_each(|(index, value)| {
            let pos = mapper.at(index, Some(sample_options));
            *value = if self.data.min_inclusive <= *value && *value < self.data.max_exclusive {
                ChunkNoiseFunctionComponent::sample_from_stack(
                    &mut component_stack[..=self.when_in_index],
                    &pos,
                    sample_options,
                )
            } else {
                ChunkNoiseFunctionComponent::sample_from_stack(
                    &mut component_stack[..=self.when_out_index],
                    &pos,
                    sample_options,
                )
            };
        });
    }
}
