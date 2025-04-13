use pumpkin_util::math::lerp;

use crate::generation::noise_router::{
    chunk_density_function::ChunkNoiseFunctionSampleOptions,
    chunk_noise_router::{ChunkNoiseFunctionComponent, StaticChunkNoiseFunctionComponentImpl},
    proto_noise_router::ProtoNoiseFunctionComponent,
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

    #[inline]
    fn calculate_min_and_max(&self, component_stack: &[ProtoNoiseFunctionComponent]) -> (f32, f32) {
        match self {
            Self::Fixed(fixed) => (*fixed, *fixed),
            Self::Spline(spline) => spline.calculate_min_and_max(component_stack),
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

    fn calculate_min_and_max(&self, component_stack: &[ProtoNoiseFunctionComponent]) -> (f32, f32) {
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;

        let input_function = &component_stack[self.input_index];
        let input_max = input_function.max() as f32;
        let input_min = input_function.min() as f32;

        let first_point = self.points.first().expect("A spline with no values?");
        if input_min < first_point.location {
            let (point_min, point_max) = first_point.value.calculate_min_and_max(component_stack);
            let sample_min = first_point.sample_outside_range(input_min, point_min);
            let sample_max = first_point.sample_outside_range(input_min, point_max);

            min = min.min(sample_min.min(sample_max));
            max = max.max(sample_min.max(sample_max));
        }

        let last_point = self.points.last().expect("A spline with no values?");
        if input_max > last_point.location {
            let (point_min, point_max) = last_point.value.calculate_min_and_max(component_stack);
            let sample_min = last_point.sample_outside_range(input_max, point_min);
            let sample_max = last_point.sample_outside_range(input_max, point_max);

            min = min.min(sample_min.min(sample_max));
            max = max.max(sample_min.max(sample_max));
        }

        for point in &self.points {
            let (point_min, point_max) = point.value.calculate_min_and_max(component_stack);
            min = min.min(point_min);
            max = max.max(point_max);
        }

        for window in self.points.windows(2) {
            let point_1 = &window[0];
            let point_2 = &window[1];

            if point_1.derivative != 0.0 || point_2.derivative != 0.0 {
                let location_delta = point_2.location - point_1.location;

                let (point_1_min, point_1_max) =
                    point_1.value.calculate_min_and_max(component_stack);
                let (point_2_min, point_2_max) =
                    point_2.value.calculate_min_and_max(component_stack);

                let point_1_partial = point_1.derivative * location_delta;
                let point_2_partial = point_2.derivative * location_delta;

                let points_min = point_1_min.min(point_2_min);
                let points_max = point_1_max.max(point_2_max);

                let z = point_1_partial - point_2_max + point_1_min;
                let aa = point_1_partial - point_2_min + point_1_max;
                let ab = -point_2_partial + point_2_min - point_1_max;
                let ac = -point_2_partial + point_2_max - point_1_max;

                let ad = z.min(ab);
                let ae = aa.max(ac);

                min = min.min(points_min + 0.25 * ad);
                max = max.max(points_max + 0.25 * ae);
            }
        }

        (min, max)
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
    spline: Spline,
    min_value: f64,
    max_value: f64,
}

impl SplineFunction {
    pub fn new(spline: Spline, component_stack: &[ProtoNoiseFunctionComponent]) -> Self {
        let (min_value, max_value) = spline.calculate_min_and_max(component_stack);
        Self {
            spline,
            min_value: min_value as f64,
            max_value: max_value as f64,
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
