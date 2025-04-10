use std::{f32::consts::PI, f64::consts::PI};

use pumpkin_macros::block_state;
use pumpkin_util::{
    math::{float_provider::FloatProvider, vector2::Vector2, vector3::Vector3},
    random::RandomGenerator,
};
use serde::Deserialize;

use crate::{
    ProtoChunk,
    block::{ChunkBlockState, registry::get_block},
    generation::{
        aquifer_sampler::{AquiferSamplerImpl, WorldAquiferSampler},
        height_limit::HeightLimitView,
        height_provider::HeightProvider,
        positions::chunk_pos,
        section_coords,
        y_offset::YOffset,
    },
};

use super::mask::CarvingMask;

#[derive(Deserialize)]
pub struct CaveCraver {
    vertical_radius_multiplier: FloatProvider,
    horizontal_radius_multiplier: FloatProvider,
    floor_level: FloatProvider,
    y: HeightProvider,
    #[serde(rename = "yScale")]
    y_scale: FloatProvider,
    lava_level: YOffset,
    probability: f32,
}
const BRANCH_FACTOR: i32 = 4;
const MAX_CAVE_COUNT: i32 = 15;

impl CaveCraver {
    pub fn should_carve(&self, random: &mut RandomGenerator) -> bool {
        random.next_f32() <= self.probability
    }

    pub fn carve(
        &self,
        random: &mut RandomGenerator,
        chunk_pos: &Vector2<i32>,
        min_y: i8,
        height: u16,
    ) {
        let block_coord = section_coords::section_to_block(BRANCH_FACTOR * 2 - 1);
        let first_rnd = random.next_bounded_i32(MAX_CAVE_COUNT);
        let sec_rnd = random.next_bounded_i32(first_rnd + 1);
        let third_rnd = random.next_bounded_i32(sec_rnd + 1);
        let range = third_rnd;
        for _ in 0..range {
            let x = chunk_pos::start_block_x(chunk_pos) + random.next_bounded_i32(16); // offset
            let y = self.y.get(random, min_y, height);
            let z = chunk_pos::start_block_z(chunk_pos) + random.next_bounded_i32(16); // offset
            let vertical = self.vertical_radius_multiplier.get();
            let horizontal = self.horizontal_radius_multiplier.get();
            let floor = self.floor_level.get();
            let mut pitch;
            let mut tries = 0;
            if random.next_bounded_i32(4) == 0 {
                let scale = self.y_scale.get();
                pitch = 1.0 + random.next_f32() * 6.0;
                tries += random.next_bounded_i32(4);
            }
            for _ in 0..tries  {
                let yaw =   random.next_f32() * (PI * 2.0);
                let pitch = (random.next_f32() - 0.5) / 4.0;
                let width = Self::get_tunnel_width(random);
                let branch_count = block_coord - random.next_bounded_i32(block_coord / 4);
                Self::carve_tunnels(context, config, chunk, seed, aquifer_sampler, x, y, z, horizontal_scale, vertical_scale, width, &mut yaw, &mut pitch, branch_start_index, branch_count, yaw_pitch_ratio, mask, skip_predicate);
            }
        }
    }
    
    fn carve_tunnels(
        &self,
        chunk: &Chunk,
        seed: i64,
        aquifer_sampler: &AquiferSampler,
        x: &mut f64,
        y: &mut f64,
        z: &mut f64,
        horizontal_scale: f64,
        vertical_scale: f64,
        width: f32,
        yaw: &mut f32,
        pitch: &mut f32,
        branch_start_index: i32,
        branch_count: i32,
        yaw_pitch_ratio: f64,
        mask: &CarvingMask,
    ) {
        let mut random = Pcg64::seed_from_u64(seed as u64);
        let i = random.gen_range(branch_count / 4..branch_count / 2 + branch_count / 4);
        let bl = random.gen_range(0..6) == 0;
        let mut f = 0.0f32;
        let mut g = 0.0f32;
    
        for j in branch_start_index..branch_count {
            let d = 1.5 + (f32::sin(PI * j as f32 / branch_count as f32) * width) as f64;
            let e = d * yaw_pitch_ratio;
            let h = f32::cos(*pitch);
    
            *x += (*yaw).cos() as f64 * h as f64;
            *y += (*pitch).sin() as f64;
            *z += (*yaw).sin() as f64 * h as f64;
    
            *pitch *= if bl { 0.92f32 } else { 0.7f32 };
            *pitch += g * 0.1f32;
            *yaw += f * 0.1f32;
    
            g *= 0.9f32;
            f *= 0.75f32;
    
            g += (random.gen::<f32>() - random.gen::<f32>()) * random.gen::<f32>() * 2.0f32;
            f += (random.gen::<f32>() - random.gen::<f32>()) * random.gen::<f32>() * 4.0f32;
    
            if j == i && width > 1.0 {
                carve_tunnels(
                    context,
                    config,
                    chunk,
                    pos_to_biome,
                    random.gen::<i64>(),
                    aquifer_sampler,
                    x,
                    y,
                    z,
                    horizontal_scale,
                    vertical_scale,
                    random.gen::<f32>() * 0.5f32 + 0.5f32,
                    &mut (*yaw - PI / 2.0f32),
                    &mut (*pitch / 3.0f32),
                    j,
                    branch_count,
                    1.0,
                    mask,
                );
                carve_tunnels(
                    context,
                    config,
                    chunk,
                    pos_to_biome,
                    random.gen::<i64>(),
                    aquifer_sampler,
                    x,
                    y,
                    z,
                    horizontal_scale,
                    vertical_scale,
                    random.gen::<f32>() * 0.5f32 + 0.5f32,
                    &mut (*yaw + PI / 2.0f32),
                    &mut (*pitch / 3.0f32),
                    j,
                    branch_count,
                    1.0,
                    mask,
                );
                return;
            }
    
            if random.gen_range(0..4) == 0 {
                continue;
            }
    
            if !can_carve_branch(chunk_pos(chunk), *x, *z, j, branch_count, width) {
                return;
            }
    
            self.carve_region(
                context,
                config,
                chunk,
                aquifer_sampler,
                *x,
                *y,
                *z,
                d * horizontal_scale,
                e * vertical_scale,
                mask,
            );
        }
    }

    fn carve_cave(
        &self,
        chunk: &mut ProtoChunk,
        min_y: i8,
        chunk_height: u16,
        chunk_pos: &Vector2<i32>,
        x: f64,
        y: f64,
        z: f64,
        width: f64,
        height: f64,
        mask: &mut CarvingMask,
        floor_level: f64,
    ) {
        let width = 1.5 + 1.5707964f64.sin() * width;
        let height = width * height;
        self.carve_region(
            chunk,
            min_y,
            chunk_height,
            chunk_pos,
            x,
            y,
            z,
            width,
            height,
            mask,
            floor_level,
        );
    }

    fn carve_region(
        &self,
        chunk: &mut ProtoChunk,
        min_y: i8,
        chunk_height: u16,
        chunk_pos: &Vector2<i32>,
        x: f64,
        y: f64,
        z: f64,
        width: f64,
        height: f64,
        mask: &mut CarvingMask,
        floor_level: f64,
    ) {
        let start_x = chunk_pos::start_block_x(chunk_pos);
        let start_z = chunk_pos::start_block_z(chunk_pos);

        let chunk_center_x = (start_x + 8) as f64;
        let chunk_center_z = (start_z + 8) as f64;

        let max_width = 16.0 + width * 2.0;
        if (x - chunk_center_x).abs() > max_width || (z - chunk_center_z).abs() > max_width {
            return;
        }
        let x_start = (x - width).floor() as i32 - start_x - 1.max(0);
        let max_x = (x - width).floor() as i32 - start_x - 1.min(15);

        let z_start = (z - width).floor() as i32 - start_z - 1.max(0);
        let max_z = (z - width).floor() as i32 - start_z - 1.min(15);

        let init_height =
            (y + height).floor() as i32 + 1.min(min_y as i32 + chunk_height as i32 - 1 - 7);

        let end_height = (y - height).floor() as i32 - 1.max(min_y as i32 + 1);

        for current_x in max_x..x_start {
            let x_offset = chunk_pos::start_block_x(chunk_pos) + current_x;
            let x_offwidth = (x_offset as f64 + 0.5 - x) / width;
            for current_z in max_z..z_start {
                let z_offset = chunk_pos::start_block_z(chunk_pos) + current_z;
                let z_offwidth = (z_offset as f64 + 0.5 - z) / width;

                if x_offwidth * x_offwidth + z_offwidth * z_offwidth >= 1.0 {
                    continue;
                }
                let grass = false;
                for current_y in init_height..end_height {
                    let y_offwidth = (current_y as f64 - 0.5 - y) / height;
                    if Self::is_pos_excluded(
                        Vector3::new(x_offwidth, y_offwidth, z_offwidth),
                        floor_level,
                    ) {
                        continue;
                    }
                    mask.set(current_x, current_y, current_z);
                    self.carve_at_point(
                        chunk,
                        Vector3::new(x_offset, current_y, z_offset),
                        min_y,
                        chunk_height,
                        grass,
                    );
                }
            }
        }
    }

    fn carve_at_point(
        &self,
        chunk: &mut ProtoChunk,
        pos: Vector3<i32>,
        min_y: i8,
        height: u16,
        mut grass: bool,
    ) {
        let state = chunk.get_block_state(&pos);

        if state.block_id == block_state!("grass_block").block_id
            || state.block_id == block_state!("mycelium").block_id
        {
            grass = true;
        }
        let state = self.get_state(pos, min_y, height);
        if let Some(state) = state {
            chunk.set_block_state(&pos, state);
        }
        // if grass {

        // }
    }

    fn get_state(&self, pos: Vector3<i32>, min_y: i8, height: u16) -> Option<ChunkBlockState> {
        if pos.y <= self.lava_level.get_y(min_y, height) as i32 {
            return Some(block_state!("lava"));
        }
        None
    }

    fn is_pos_excluded(scaled: Vector3<f64>, floor_y: f64) -> bool {
        if scaled.y <= floor_y {
            return true;
        }
        scaled.x * scaled.x + scaled.y * scaled.y + scaled.z * scaled.z >= 1.0
    }

    fn get_tunnel_width(random: &mut RandomGenerator) -> f32 {
        let mut width = random.next_f32() * 2.0 + random.next_f32();
        if random.next_bounded_i32(10) == 0 {
            width *= random.next_f32() * random.next_f32() * 3.0 + 1.0;
        }
        width
    }
}
