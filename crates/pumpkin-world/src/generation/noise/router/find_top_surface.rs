use pumpkin_data::noise_router::FindTopSurfaceData;
use pumpkin_util::math::vector3::Vector3;

use crate::generation::noise::router::{
    chunk_noise_router::{ChunkNoiseFunctionComponent, StaticChunkNoiseFunctionComponentImpl},
    density_function::NoiseFunctionComponentRange,
    density_volume::{DensityBuffer, DensityVolume},
};

pub struct FindTopSurface {
    density_index: usize,
    upper_bound_index: usize,
    min_value: f32,
    max_value: f32,
    data: &'static FindTopSurfaceData,
}

impl FindTopSurface {
    #[must_use]
    pub const fn new(
        density_index: usize,
        upper_bound_index: usize,
        min_value: f32,
        max_value: f32,
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

    #[must_use]
    pub const fn density_index(&self) -> usize {
        self.density_index
    }

    #[must_use]
    pub const fn upper_bound_index(&self) -> usize {
        self.upper_bound_index
    }

    #[must_use]
    pub const fn cell_height(&self) -> i32 {
        self.data.cell_height
    }
}

impl NoiseFunctionComponentRange for FindTopSurface {
    #[inline]
    fn min(&self) -> f32 {
        self.min_value
    }

    #[inline]
    fn max(&self) -> f32 {
        self.max_value
    }
}

impl StaticChunkNoiseFunctionComponentImpl for FindTopSurface {
    fn sample(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        pos: &Vector3<i32>,
    ) -> f32 {
        let upper = ChunkNoiseFunctionComponent::sample_from_stack(
            &mut component_stack[..=self.upper_bound_index],
            &Vector3::new(pos.x, 0, pos.z),
        );
        self.find_surface_from(component_stack, pos.x, pos.z, upper)
    }

    fn sample_volume(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        buffer: &mut [f32],
        volume: &DensityVolume,
    ) {
        if volume.size_y != 1 || volume.min_block_y != 0 {
            let column_volume = DensityVolume::new(
                volume.size_x,
                1,
                volume.size_z,
                volume.min_block_x,
                0,
                volume.min_block_z,
                volume.step_block_x,
                volume.step_block_y,
                volume.step_block_z,
            );
            let mut columns = DensityBuffer::acquire(&column_volume);
            self.sample_volume(component_stack, &mut columns, &column_volume);
            for z in 0..volume.size_z {
                for x in 0..volume.size_x {
                    let value = columns[column_volume.index_unchecked(x, 0, z)];
                    let index = volume.index_unchecked(x, 0, z);
                    buffer[index..index + volume.size_y].fill(value);
                }
            }
            return;
        }
        ChunkNoiseFunctionComponent::sample_volume_from_stack(
            &mut component_stack[..=self.upper_bound_index],
            buffer,
            volume,
        );
        let mut index = 0;
        for z in 0..volume.size_z {
            let block_z = volume.block_z(z);
            for x in 0..volume.size_x {
                let block_x = volume.block_x(x);
                buffer[index] =
                    self.find_surface_from(component_stack, block_x, block_z, buffer[index]);
                index += 1;
            }
        }
    }
}

impl FindTopSurface {
    fn find_surface_from(
        &self,
        component_stack: &mut [ChunkNoiseFunctionComponent],
        x: i32,
        z: i32,
        upper: f32,
    ) -> f32 {
        let cell_height = self.data.cell_height;
        let lower_bound = self.data.lower_bound;

        let top_y = (upper / cell_height as f32).floor() as i32 * cell_height;

        if top_y <= lower_bound {
            return lower_bound as f32;
        }

        let mut y = top_y;
        while y >= lower_bound {
            let sample_pos = Vector3::new(x, y, z);
            let density = ChunkNoiseFunctionComponent::sample_from_stack(
                &mut component_stack[..=self.density_index],
                &sample_pos,
            );
            if density > 0.0 {
                return y as f32;
            }
            y -= cell_height;
        }

        lower_bound as f32
    }
}
