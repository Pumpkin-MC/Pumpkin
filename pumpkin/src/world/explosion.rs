use std::sync::Arc;

use pumpkin_data::{Block, BlockState};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use rustc_hash::FxHashMap;

use crate::{
    block::{ExplodeArgs, drop_loot},
    world::loot::LootContextParameters,
};

use super::{BlockFlags, World};

pub struct Explosion {
    power: f32,
    pos: Vector3<f64>,
}
impl Explosion {
    #[must_use]
    pub const fn new(power: f32, pos: Vector3<f64>) -> Self {
        Self { power, pos }
    }
    async fn get_blocks_to_destroy(
        &self,
        world: &World,
    ) -> FxHashMap<BlockPos, (&'static Block, &'static BlockState)> {
        // Somethings are not vanilla here but make it way faster
        let mut map = FxHashMap::default();
        let random_val = rand::random::<f32>();
        for x in 0..16 {
            for y in 0..16 {
                'block2: for z in 0..16 {
                    if x > 0 && x < 15 && y > 0 && y < 15 && z > 0 && z < 15 {
                        continue;
                    }

                    let mut x = f64::from(x) / 7.5 - 1.0;
                    let mut y = f64::from(y) / 7.5 - 1.0;
                    let mut z = f64::from(z) / 7.5 - 1.0;

                    let sqrt = 1.0 / (x * x + y * y + z * z).sqrt();
                    x *= sqrt;
                    y *= sqrt;
                    z *= sqrt;

                    let mut pos_x = self.pos.x;
                    let mut pos_y = self.pos.y + 0.0625;
                    let mut pos_z = self.pos.z;

                    let mut h = self.power * random_val.mul_add(0.6, 0.7);
                    while h > 0.0 {
                        let block_pos = BlockPos::floored(pos_x, pos_y, pos_z);
                        let (block, state) = world.get_block_and_state(&block_pos).await;
                        let (_, fluid_state) = world.get_fluid_and_fluid_state(&block_pos).await;

                        // if !world.is_in_build_limit(&block_pos) {
                        //     // Pass by reference
                        //     continue 'block2;
                        // }

                        if !state.is_air() || !fluid_state.is_empty {
                            let resistance =
                                fluid_state.blast_resistance.max(block.blast_resistance);
                            h -= resistance * 0.3;
                            if h > 0.0 {
                                map.insert(block_pos, (block, state));
                            }
                        }
                        pos_x += x * 0.3;
                        pos_y += y * 0.3;
                        pos_z += z * 0.3;
                        h -= 0.225_000_01;
                    }
                }
            }
        }
        map
    }

    /// Returns the removed block count
    pub async fn explode(&self, world: &Arc<World>) -> u32 {
        let blocks = self.get_blocks_to_destroy(world).await;
        let source_pos = BlockPos::floored(self.pos.x, self.pos.y, self.pos.z);
        let (source_block, _source_state) = world.get_block_and_state(&source_pos).await;
        let mut block_positions: Vec<BlockPos> = blocks.keys().copied().collect();
        let mut yield_rate = 1.0f32;

        if let Some(server) = world.server.upgrade() {
            let event = crate::plugin::block::block_explode::BlockExplodeEvent::new(
                source_block,
                source_pos,
                world.uuid,
                block_positions,
                yield_rate,
            );
            let event = server.plugin_manager.fire(event).await;
            if event.cancelled {
                return 0;
            }
            block_positions = event.blocks;
            yield_rate = event.yield_rate;
        }
        // TODO: Entity damage, fire
        for pos in &block_positions {
            let Some((block, state)) = blocks.get(pos) else {
                continue;
            };
            world.set_block_state(pos, 0, BlockFlags::NOTIFY_ALL).await;

            let pumpkin_block = world.block_registry.get_pumpkin_block(block.id);

            if pumpkin_block.is_none_or(|s| s.should_drop_items_on_explosion()) {
                if yield_rate <= 0.0 || (yield_rate < 1.0 && rand::random::<f32>() > yield_rate) {
                    continue;
                }
                let params = LootContextParameters {
                    block_state: Some(state),
                    explosion_radius: Some(self.power),
                    ..Default::default()
                };
                drop_loot(world, block, pos, false, params).await;
            }
            if let Some(pumpkin_block) = pumpkin_block {
                pumpkin_block
                    .explode(ExplodeArgs {
                        world,
                        block,
                        position: pos,
                    })
                    .await;
            }
        }
        block_positions.len() as u32
    }
}
