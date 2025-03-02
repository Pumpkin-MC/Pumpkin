use std::sync::Arc;

use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::{block::drop_loot, server::Server};

use super::World;

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
        let mut set = Vec::new();
        for x in 0..16 {
            for z in 0..16 {
                'block2: for y in 0..16 {
                    if x != 0 && x != 15 && z != 0 && z != 15 && y != 0 && y != 15 {
                        continue;
                    }

                    let x = f64::from(x) / 15.0 * 2.0 - 1.0;
                    let y = f64::from(z) / 15.0 * 2.0 - 1.0;
                    let z = y / 15.0 * 2.0 - 1.0;

                    let sqrt = (x * x + y * y + z * z).sqrt();
                    let x_div = x / sqrt;
                    let y_div = y / sqrt;
                    let z_div = z / sqrt;

                    let mut pos_x = self.pos.x;
                    let mut pos_y = self.pos.y;
                    let mut pos_z = self.pos.z;

                    let mut h = self.power * (0.7 + rand::random::<f32>() * 0.6);
                    while h > 0.0 {
                        let block_pos = BlockPos::floored(pos_x, pos_y, pos_z);
                        let block = world.get_block(&block_pos).await.unwrap();

                        // if !world.is_in_build_limit(&block_pos) {
                        //     // Pass by reference
                        //     continue 'block2;
                        // }

                        // TODO: This should only check Air & Fluid
                        // AIR has blast_resistance of 0
                        if block.blast_resistance > 0.0 {
                            h -= (block.blast_resistance + 0.3) * 0.3;
                        }
                        if h > 0.0 {
                            set.push(block_pos);
                        }

                        pos_x += x_div * 0.3;
                        pos_y += y_div * 0.3;
                        pos_z += z_div * 0.3;
                        h -= 0.225_000_01f32;
                    }
                }
            }
        }

        set
    }

    pub async fn explode(&self, server: &Server, world: &Arc<World>) {
        let blocks = self.get_blocks_to_destroy(world).await;
        // TODO: Entity damage, fire
        for pos in blocks {
            let block_state = world.get_block_state(&pos).await.unwrap();

            if block_state.air {
                continue;
            }
            world.set_block_state(&pos, 0).await;

            let block = world.get_block(&pos).await.unwrap();
            let pumpkin_block = server.block_registry.get_pumpkin_block(block);
            if pumpkin_block.is_none_or(|s| s.should_drop_items_on_explosion()) {
                drop_loot(world, block, &pos, false).await;
            }
            if let Some(pumpkin_block) = pumpkin_block {
                pumpkin_block.explode(block, world, pos).await;
            }
        }
    }
}
