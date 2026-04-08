use pumpkin_data::noise_router::FindTopSurfaceData;
use pumpkin_util::math::vector3::Vector3;

use crate::generation::noise::router::{
    chunk_density_function::ChunkNoiseFunctionSampleOptions,
    chunk_noise_router::{ChunkNoiseFunctionComponent, StaticChunkNoiseFunctionComponentImpl},
    density_function::NoiseFunctionComponentRange,
};

#[derive(Clone)]
pub struct FindTopSurface {
    density_index: usize,
    upper_bound_index: usize,
    min_value: f64,
    max_value: f64,
    data: &'static FindTopSurfaceData,
}

impl FindTopSurface {
    pub const fn new(
        density_index: usize,
        upper_bound_index: usize,
        min_value: f64,
        max_value: f64,
        data: &'static FindTopSurfaceData,
    ) -> Self {
        Self {
            density_index,
            upper_bound_index,
            min_value,
            max_value,
            data,
        }
    }
}

impl NoiseFunctionComponentRange for FindTopSurface {
    #[inline]
    fn min(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f64 {
        self.max_value
    }
}

impl StaticChunkNoiseFunctionComponentImpl for FindTopSurface {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> f64 {
        let upper = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.upper_bound_index],
            pos,
            sample_options,
        );

        let cell_height = self.data.cell_height;
        let lower_bound = self.data.lower_bound;

        // Snap upper bound down to nearest cell boundary, matching Java:
        // int topY = Mth.floor(this.upperBound.compute(context) / this.cellHeight) * this.cellHeight
        let top_y = (upper / cell_height as f64).floor() as i32 * cell_height;

        if top_y <= lower_bound {
            return lower_bound as f64;
        }

        // Walk downward in cellHeight steps, return the first Y where density > 0.0
        let mut y = top_y;
        while y >= lower_bound {
            let sample_pos = Vector3::new(pos.x, y, pos.z);
            let density = ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=self.density_index],
                &sample_pos,
                sample_options,
            );
            if density > 0.0 {
                return y as f64;
            }
            y -= cell_height;
        }

        lower_bound as f64
    }
}
