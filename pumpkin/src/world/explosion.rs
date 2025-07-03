use std::collections::HashSet;
use std::sync::Arc;

use pumpkin_data::block_properties::get_state_by_state_id;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::{
    block::{drop_loot, loot::LootContextParameters, pumpkin_block::ExplodeArgs},
    server::Server,
};

use super::{BlockFlags, World};

pub struct Explosion {
    power: f32,
    pos: Vector3<f64>,
}
impl Explosion {
    #[must_use]
    pub fn new(power: f32, pos: Vector3<f64>) -> Self {
        Self { power, pos }
    }
    async fn get_blocks_to_destroy(&self, world: &World) -> Vec<BlockPos> {
        let mut set = HashSet::new();
        for x in 0..16 {
            for y in 0..16 {
                'block2: for z in 0..16 {
                    if x != 0 && x != 15 && z != 0 && z != 15 && y != 0 && y != 15 {
                        continue;
                    }

                    let mut x = f64::from(x) / 15.0 * 2.0 - 1.0;
                    let mut y = f64::from(y) / 15.0 * 2.0 - 1.0;
                    let mut z = f64::from(z) / 15.0 * 2.0 - 1.0;

                    let sqrt = (x * x + y * y + z * z).sqrt();
                    x /= sqrt;
                    y /= sqrt;
                    z /= sqrt;

                    let mut pos_x = self.pos.x;
                    let mut pos_y = self.pos.y + 0.0625;
                    let mut pos_z = self.pos.z;

                    let mut h = self.power * (0.7 + rand::random::<f32>() * 0.6);
                    while h > 0.0 {
                        let block_pos = BlockPos::floored(pos_x, pos_y, pos_z);
                        let (block, state) = world.get_block_and_block_state(&block_pos).await;

                        // if !world.is_in_build_limit(&block_pos) {
                        //     // Pass by reference
                        //     continue 'block2;
                        // }

                        // TODO: This should only check air & fluid
                        if !state.is_air() {
                            h -= (block.blast_resistance + 0.3) * 0.3;
                        }
                        if h > 0.0 {
                            set.insert(block_pos);
                        }
                        pos_x += x * 0.3;
                        pos_y += y * 0.3;
                        pos_z += z * 0.3;
                        h -= 0.225_000_01f32;
                    }
                }
            }
        }

        set.into_iter().collect()
    }

    pub async fn explode(&self, server: &Server, world: &Arc<World>) {
        let blocks = self.get_blocks_to_destroy(world).await;
        // TODO: Entity damage, fire
        for pos in blocks {
            let block_state = world.get_block_state(&pos).await;

            if block_state.is_air() {
                continue;
            }

            let block = world.get_block(&pos).await;
            let pumpkin_block = server.block_registry.get_pumpkin_block(block);

            world.set_block_state(&pos, 0, BlockFlags::NOTIFY_ALL).await;

            if pumpkin_block.is_none_or(|s| s.should_drop_items_on_explosion()) {
                let params = LootContextParameters {
                    block_state: get_state_by_state_id(block_state.id),
                    explosion_radius: Some(self.power),
                };
                drop_loot(world, block, &pos, false, params).await;
            }
            if let Some(pumpkin_block) = pumpkin_block {
                pumpkin_block
                    .explode(ExplodeArgs {
                        world,
                        block,
                        location: &pos,
                    })
                    .await;
            }
        }
    }
}
