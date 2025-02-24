use std::sync::Arc;

use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::server::Server;

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

                    let d = f64::from(x) / 15.0 * 2.0 - 1.0;
                    let e = f64::from(z) / 15.0 * 2.0 - 1.0;
                    let f = f64::from(y) / 15.0 * 2.0 - 1.0;

                    let g = (d * d + e * e + f * f).sqrt();
                    let d = d / g;
                    let e = e / g;
                    let f = f / g;

                    let mut m = self.pos.x;
                    let mut n = self.pos.y;
                    let mut o = self.pos.z;

                    let mut h = self.power * (0.7 + rand::random::<f32>() * 0.6);
                    while h > 0.0 {
                        let block_pos = BlockPos::floored(m, n, o);
                        let block = world.get_block(&block_pos).await.unwrap();

                        // if !world.is_in_build_limit(&block_pos) {
                        //     // Pass by reference
                        //     continue 'block2;
                        // }

                        h -= (block.blast_resistance + 0.3) * 0.3;
                        if h > 0.0 {
                            set.push(block_pos);
                        }

                        m += d * 0.3;
                        n += e * 0.3;
                        o += f * 0.3;
                        // h -= 0.22500001f32;
                    }
                }
            }
        }

        set
    }

    pub async fn explode(&self, server: &Server, world: &Arc<World>) {
        let blocks = self.get_blocks_to_destroy(world).await;
        // TODO: Entity damage, fire
        for block in blocks {
            let block_state = world.get_block_state(&block).await.unwrap();
            if block_state.air {
                continue;
            }
            world.break_block(server, &block, None, true).await;
        }
    }
}
