use pumpkin_util::math::vector3::Vector3;

use crate::generation::noise::router::{
    chunk_density_function::ChunkNoiseFunctionSampleOptions,
    chunk_noise_router::{ChunkNoiseFunctionComponent, StaticChunkNoiseFunctionComponentImpl},
    proto_noise_router::ProtoNoiseFunctionComponent,
};

use super::NoiseFunctionComponentRange;

pub enum SplineValue {
    Spline(Spline),
    Fixed(f32),
}

impl SplineValue {
    #[inline]
    fn sample(
        &self,
        pos: &Vector3<i32>,
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

pub struct SplinePoint {
    pub location: f32,
    pub value: SplineValue,
    pub derivative: f32,
}

impl SplinePoint {
    pub const fn new(location: f32, value: SplineValue, derivative: f32) -> Self {
        Self {
            location,
            value,
            derivative,
        }
    }

    const fn sample_outside_range(&self, sample_location: f32, last_known_sample: f32) -> f32 {
        if self.derivative == 0f32 {
            last_known_sample
        } else {
            self.derivative * (sample_location - self.location) + last_known_sample
        }
    }
}

pub enum Range {
    In(usize),
    Below,
}

pub struct Spline {
    pub input_index: usize,
    pub points: Box<[SplinePoint]>,
}

impl Spline {
    pub const fn new(input_index: usize, points: Box<[SplinePoint]>) -> Self {
        Self {
            input_index,
            points,
        }
    }

    fn calculate_min_and_max(&self, component_stack: &[ProtoNoiseFunctionComponent]) -> (f32, f32) {
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;

        let input_function = &component_stack[self.input_index];
        let input_max = input_function.max();
        let input_min = input_function.min();

        let Some(first_point) = self.points.first() else {
            return (0.0, 0.0);
        };
        if input_min < first_point.location {
            let (point_min, point_max) = first_point.value.calculate_min_and_max(component_stack);
            let sample_min = first_point.sample_outside_range(input_min, point_min);
            let sample_max = first_point.sample_outside_range(input_min, point_max);

            min = min.min(sample_min.min(sample_max));
            max = max.max(sample_min.max(sample_max));
        }

        let Some(last_point) = self.points.last() else {
            return (min, max);
        };
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

    fn sample(
        &self,
        pos: &Vector3<i32>,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f32 {
        let location = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.input_index],
            pos,
            sample_options,
        );

        let n = self.points.len();
        let index_greater_than_x = self.points.partition_point(|p| location >= p.location);

        if index_greater_than_x == 0 {
            let point = &self.points[0];
            let val = point.value.sample(pos, component_stack, sample_options);
            return val + point.derivative * (location - point.location);
        }

        if index_greater_than_x == n {
            let point = &self.points[n - 1];
            let val = point.value.sample(pos, component_stack, sample_options);
            return val + point.derivative * (location - point.location);
        }

        let previous = &self.points[index_greater_than_x - 1];
        let current = &self.points[index_greater_than_x];

        let start_x = previous.location;
        let end_x = current.location;

        let start_value = previous.value.sample(pos, component_stack, sample_options);
        let end_value = current.value.sample(pos, component_stack, sample_options);

        let start_derivative = previous.derivative;
        let end_derivative = current.derivative;

        let t = (location - start_x) / (end_x - start_x);

        let h00 = (1.0 + 2.0 * t) * (1.0 - t) * (1.0 - t);
        let h10 = t * (1.0 - t) * (1.0 - t);
        let h01 = t * t * (3.0 - 2.0 * t);
        let h11 = t * t * (t - 1.0);

        h00.mul_add(
            start_value,
            h10.mul_add(
                start_derivative * (end_x - start_x),
                h01.mul_add(end_value, h11 * (end_derivative * (end_x - start_x))),
            ),
        )
    }
}

pub struct SplineFunction {
    spline: Spline,
    min_value: f32,
    max_value: f32,
}

impl SplineFunction {
    pub fn new(spline: Spline, component_stack: &[ProtoNoiseFunctionComponent]) -> Self {
        let (min_value, max_value) = spline.calculate_min_and_max(component_stack);
        Self {
            spline,
            min_value,
            max_value,
        }
    }

    pub const fn spline(&self) -> &Spline {
        &self.spline
    }
}

impl StaticChunkNoiseFunctionComponentImpl for SplineFunction {
    #[inline]
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f32 {
        self.spline.sample(pos, component_stack, sample_options)
    }
}

impl NoiseFunctionComponentRange for SplineFunction {
    #[inline]
    fn min(&self) -> f32 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f32 {
        self.max_value
    }
}
