use pumpkin_data::noise_router::{SplinePoint, SplineRepr};
use pumpkin_util::math::vector3::Vector3;

use crate::generation::noise::router::{
    chunk_noise_router::{ChunkNoiseFunctionComponent, StaticChunkNoiseFunctionComponentImpl},
    density_volume::{DensityBuffer, DensityVolume},
    proto_noise_router::ProtoNoiseFunctionComponent,
};

use super::NoiseFunctionComponentRange;

const fn sample_outside_range(
    point: &SplinePoint,
    sample_location: f32,
    last_known_sample: f32,
) -> f32 {
    if point.derivative == 0f32 {
        last_known_sample
    } else {
        point.derivative * (sample_location - point.location) + last_known_sample
    }
}

fn calculate_min_and_max(
    spline: &SplineRepr,
    component_stack: &[ProtoNoiseFunctionComponent],
) -> (f32, f32) {
    match spline {
        SplineRepr::Fixed { value } => (*value, *value),
        SplineRepr::Standard {
            location_function_index,
            points,
        } => {
            let mut min = f32::INFINITY;
            let mut max = f32::NEG_INFINITY;

            let input_function = &component_stack[*location_function_index];
            let input_max = input_function.max();
            let input_min = input_function.min();

            let Some(first_point) = points.first() else {
                return (0.0, 0.0);
            };
            if input_min < first_point.location {
                let (point_min, point_max) =
                    calculate_min_and_max(first_point.value, component_stack);
                let sample_min = sample_outside_range(first_point, input_min, point_min);
                let sample_max = sample_outside_range(first_point, input_min, point_max);

                min = min.min(sample_min.min(sample_max));
                max = max.max(sample_min.max(sample_max));
            }

            let Some(last_point) = points.last() else {
                return (min, max);
            };
            if input_max > last_point.location {
                let (point_min, point_max) =
                    calculate_min_and_max(last_point.value, component_stack);
                let sample_min = sample_outside_range(last_point, input_max, point_min);
                let sample_max = sample_outside_range(last_point, input_max, point_max);

                min = min.min(sample_min.min(sample_max));
                max = max.max(sample_min.max(sample_max));
            }

            for point in *points {
                let (point_min, point_max) = calculate_min_and_max(point.value, component_stack);
                min = min.min(point_min);
                max = max.max(point_max);
            }

            for window in points.windows(2) {
                let point_1 = &window[0];
                let point_2 = &window[1];

                if point_1.derivative != 0.0 || point_2.derivative != 0.0 {
                    let location_delta = point_2.location - point_1.location;

                    let (point_1_min, point_1_max) =
                        calculate_min_and_max(point_1.value, component_stack);
                    let (point_2_min, point_2_max) =
                        calculate_min_and_max(point_2.value, component_stack);

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
    }
}

#[inline]
fn sample_spline_with(spline: &SplineRepr, location_of: &mut impl FnMut(usize) -> f32) -> f32 {
    match spline {
        SplineRepr::Fixed { value } => *value,
        SplineRepr::Standard {
            location_function_index,
            points,
        } => {
            let location = location_of(*location_function_index);

            let n = points.len();
            let index_greater_than_x = points.partition_point(|p| location >= p.location);

            if index_greater_than_x == 0 {
                let point = &points[0];
                let val = sample_spline_with(point.value, location_of);
                return val + point.derivative * (location - point.location);
            }

            if index_greater_than_x == n {
                let point = &points[n - 1];
                let val = sample_spline_with(point.value, location_of);
                return val + point.derivative * (location - point.location);
            }

            let previous = &points[index_greater_than_x - 1];
            let current = &points[index_greater_than_x];

            let start_x = previous.location;
            let end_x = current.location;

            let start_value = sample_spline_with(previous.value, location_of);
            let end_value = sample_spline_with(current.value, location_of);

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
}

#[derive(Clone, Copy)]
pub struct SplineFunction {
    spline: &'static SplineRepr,
    min_value: f32,
    max_value: f32,
}

impl SplineFunction {
    pub fn new(
        spline: &'static SplineRepr,
        component_stack: &[ProtoNoiseFunctionComponent],
    ) -> Self {
        let (min_value, max_value) = calculate_min_and_max(spline, component_stack);
        Self {
            spline,
            min_value,
            max_value,
        }
    }
}

impl StaticChunkNoiseFunctionComponentImpl for SplineFunction {
    #[inline]
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
    ) -> f32 {
        sample_spline_with(self.spline, &mut |index| {
            ChunkNoiseFunctionComponent::sample_from_stack(&mut component_stack[..=index], pos)
        })
    }

    fn sample_volume(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        buffer: &mut [f32],
        volume: &DensityVolume,
    ) {
        let mut coordinates: Vec<(usize, DensityBuffer)> = Vec::with_capacity(4);
        let mut last_value = 0.0f32;
        for (index, value) in buffer.iter_mut().enumerate() {
            if index > 0
                && coordinates
                    .iter()
                    .all(|(_, coord)| coord[index].to_bits() == coord[index - 1].to_bits())
            {
                *value = last_value;
                continue;
            }
            last_value = sample_spline_with(self.spline, &mut |location_index| {
                let position = coordinates
                    .iter()
                    .position(|(coordinate_index, _)| *coordinate_index == location_index)
                    .unwrap_or_else(|| {
                        let mut coordinate = DensityBuffer::acquire(volume);
                        ChunkNoiseFunctionComponent::sample_volume_from_stack(
                            &mut component_stack[..=location_index],
                            &mut coordinate,
                            volume,
                        );
                        coordinates.push((location_index, coordinate));
                        coordinates.len() - 1
                    });
                coordinates[position].1[index]
            });
            *value = last_value;
        }
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
