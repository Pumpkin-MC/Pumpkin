// use pumpkin_macros::block_state;
// use pumpkin_util::{
//     math::{float_provider::FloatProvider, vector2::Vector2, vector3::Vector3},
//     random::RandomGenerator,
// };
// use serde::Deserialize;

// use crate::{
//     ProtoChunk,
//     block::{BlockState, registry::get_block},
//     generation::{
//         aquifer_sampler::{AquiferSamplerImpl, WorldAquiferSampler},
//         height_limit::HeightLimitView,
//         height_provider::HeightProvider,
//         positions::chunk_pos,
//         section_coords,
//         y_offset::YOffset,
//     },
// };

// use super::mask::CarvingMask;

// #[derive(Deserialize)]
// pub struct CaveCraver {
//     vertical_radius_multiplier: FloatProvider,
//     horizontal_radius_multiplier: FloatProvider,
//     floor_level: FloatProvider,
//     y: HeightProvider,
//     #[serde(rename = "yScale")]
//     y_scale: FloatProvider,
//     lava_level: YOffset,
//     probability: f32,
// }
// const BRANCH_FACTOR: i32 = 4;
// const MAX_CAVE_COUNT: i32 = 15;

// impl CaveCraver {
//     pub fn should_carve(&self, random: &mut RandomGenerator) -> bool {
//         random.next_f32() <= self.probability
//     }

//     pub fn carve(
//         &self,
//         random: &mut RandomGenerator,
//         chunk_pos: &Vector2<i32>,
//         min_y: i8,
//         height: u16,
//     ) {
//         todo!();
//         let block_coord = section_coords::section_to_block(BRANCH_FACTOR * 2 - 1);
//         let first_rnd = random.next_bounded_i32(MAX_CAVE_COUNT);
//         let sec_rnd = random.next_bounded_i32(first_rnd + 1);
//         let third_rnd = random.next_bounded_i32(sec_rnd + 1);
//         let range = third_rnd;
//         for _ in 0..range {
//             let x = chunk_pos::start_block_x(chunk_pos) + random.next_bounded_i32(16); // offset
//             let y = self.y.get(random, min_y, height);
//             let z = chunk_pos::start_block_z(chunk_pos) + random.next_bounded_i32(16); // offset
//             let vertical = self.vertical_radius_multiplier.get();
//             let horizontal = self.horizontal_radius_multiplier.get();
//             let floor = self.floor_level.get();
//             let mut pitch;
//             let mut tries = 0;
//             if random.next_bounded_i32(4) == 0 {
//                 let scale = self.y_scale.get();
//                 pitch = 1.0 + random.next_f32() * 6.0;
//                 tries += random.next_bounded_i32(4);
//             }
//         }
//     }

//     fn carve_cave(&self, chunk_pos: &Vector2<i32>, width: f64, height: f64) {
//         todo!();
//       //  let width = 1.5 + 1.5707964f64.sin() * width;
//      //   let height = width * height;
//      //   Self::carve_region(chunk_pos, width, height)
//     }

//     fn carve_region(
//         min_y: i8,
//         chunk_height: u16,
//         chunk_pos: &Vector2<i32>,
//         x: f64,
//         y: f64,
//         z: f64,
//         width: f64,
//         height: f64,
//         mask: &mut CarvingMask,
//         floor_level: f64,
//     ) {
//         let start_x = chunk_pos::start_block_x(chunk_pos);
//         let start_z = chunk_pos::start_block_z(chunk_pos);

//         let chunk_center_x = (start_x + 8) as f64;
//         let chunk_center_z = (start_z + 8) as f64;

//         let max_width = 16.0 + width * 2.0;
//         if (x - chunk_center_x).abs() > max_width || (z - chunk_center_z).abs() > max_width {
//             return;
//         }
//         let x_start = (x - width).floor() as i32 - start_x - 1.max(0);
//         let max_x = (x - width).floor() as i32 - start_x - 1.min(15);

//         let z_start = (z - width).floor() as i32 - start_z - 1.max(0);
//         let max_z = (z - width).floor() as i32 - start_z - 1.min(15);

//         let init_height =
//             (y + height).floor() as i32 + 1.min(min_y as i32 + chunk_height as i32 - 1 - 7);

//         let end_height = (y - height).floor() as i32 - 1.max(min_y as i32 + 1);

//         for current_x in max_x..x_start {
//             let x_offset = chunk_pos::start_block_x(chunk_pos) + current_x;
//             let x_offwidth = (x_offset as f64 + 0.5 - x) / width;
//             for current_z in max_z..z_start {
//                 let z_offset = chunk_pos::start_block_z(chunk_pos) + current_z;
//                 let z_offwidth = (z_offset as f64 + 0.5 - z) / width;

//                 if x_offwidth * x_offwidth + z_offwidth * z_offwidth >= 1.0 {
//                     continue;
//                 }
//                 for current_y in init_height..end_height {
//                     let y_offwidth = (current_y as f64 - 0.5 - y) / height;
//                     if Self::is_pos_excluded(
//                         Vector3::new(x_offwidth, y_offwidth, z_offwidth),
//                         floor_level,
//                     ) {
//                         continue;
//                     }
//                     mask.set(current_x, current_y, current_z);
//                 }
//             }
//         }
//     }

//     fn carve_at_point(&self, chunk: &ProtoChunk, pos: Vector3<i32>, min_y: i8, height: u16) {
//         let state = chunk.get_block_state(&pos);

//         // if state.block_id == block_state!("grass_block").block_id || state.block_id == block_state!("mycelium").block_id {
//         // TODO
//         // }
//         let state = self.get_state(pos, min_y, height);
//     }

//     fn get_state(&self, pos: Vector3<i32>, min_y: i8, height: u16) -> Option<BlockState> {
//         if pos.y <= self.lava_level.get_y(min_y, height) as i32 {
//             return Some(block_state!("lava"));
//         }
//         None
//     }

//     fn is_pos_excluded(scaled: Vector3<f64>, floor_y: f64) -> bool {
//         if scaled.y <= floor_y {
//             return true;
//         }
//         scaled.x * scaled.x + scaled.y * scaled.y + scaled.z * scaled.z >= 1.0
//     }

//     fn get_tunnel_width(random: &mut RandomGenerator) -> f32 {
//         let mut width = random.next_f32() * 2.0 + random.next_f32();
//         if random.next_bounded_i32(10) == 0 {
//             width *= random.next_f32() * random.next_f32() * 3.0 + 1.0;
//         }
//         width
//     }
// }
