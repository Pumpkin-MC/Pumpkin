use std::sync::Arc;

use pumpkin_data::{Block, BlockState, damage::DamageType, entity::EntityType};
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use rustc_hash::FxHashMap;

use crate::{
    block::{ExplodeArgs, drop_loot},
    entity::{Entity, EntityBase},
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

    fn get_blocks_to_destroy(
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

                        if !world.is_in_build_limit(block_pos) {
                            continue 'block2;
                        }

                        // Skip blocks in unloaded chunks instead of force-loading them
                        let Some((block, state)) =
                            world.try_get_block_and_state(&block_pos)
                        else {
                            continue 'block2;
                        };
                        let fluid_state = world
                            .try_get_fluid_and_fluid_state(&block_pos)
                            .map(|(_, fs)| fs);

                        let is_empty_fluid =
                            fluid_state.is_none_or(|fs| fs.is_empty);

                        if !state.is_air() || !is_empty_fluid {
                            let fluid_resistance = fluid_state
                                .map_or(0.0, |fs| fs.blast_resistance);
                            let resistance =
                                fluid_resistance.max(block.blast_resistance);
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

    async fn damage_entities(&self, world: &Arc<World>) {
        // Explosion is too small
        if self.power < 1.0e-5 {
            return;
        }

        let radius = self.power as f64 * 2.0;
        let min_x = (self.pos.x - radius - 1.0).floor() as i32;
        let max_x = (self.pos.x + radius + 1.0).floor() as i32;
        let min_y = (self.pos.y - radius - 1.0).floor() as i32;
        let max_y = (self.pos.y + radius + 1.0).floor() as i32;
        let min_z = (self.pos.z - radius - 1.0).floor() as i32;
        let max_z = (self.pos.z + radius + 1.0).floor() as i32;

        let search_box = BoundingBox::new(
            Vector3::new(min_x as f64, min_y as f64, min_z as f64),
            Vector3::new(max_x as f64, max_y as f64, max_z as f64),
        );

        let entities = world.get_all_at_box(&search_box);

        for entity_base in entities {
            if entity_base.is_immune_to_explosion() {
                continue;
            }

            // Skip spectators (no damage, no knockback)
            if entity_base.is_spectator() {
                continue;
            }

            let entity = entity_base.get_entity();

            let distance = (entity.pos.load().squared_distance_to_vec(&self.pos)).sqrt() / radius;
            if distance > 1.0 {
                continue;
            }

            let exposure = Self::calculate_exposure(&self.pos, entity, world).await as f64;
            if exposure == 0.0 {
                continue;
            }

            let damage_multiplier = (1.0 - distance) * exposure;
            let damage = (f64::midpoint(damage_multiplier * damage_multiplier, damage_multiplier)
                * 7.0
                * self.power as f64
                + 1.0) as f32;

            // TODO: damage type
            entity
                .damage(entity_base.as_ref(), damage, DamageType::EXPLOSION)
                .await;

            // Calculate and apply knockback
            let dir_pos = if entity.entity_type == &EntityType::TNT {
                entity.pos.load()
            } else {
                entity.get_eye_pos()
            };
            let direction = (dir_pos - self.pos).normalize();
            // TODO
            let knockback_resistance = 0.0;

            let knockback_multiplier = (1.0 - distance) * exposure * (1.0 - knockback_resistance);
            let knockback = direction * knockback_multiplier;
            entity.add_velocity(knockback).await;
        }
    }

    async fn calculate_exposure(
        explosion_pos: &Vector3<f64>,
        entity: &Entity,
        world: &Arc<World>,
    ) -> f32 {
        let bbox = entity.bounding_box.load();

        let step_x = 1.0 / ((bbox.max.x - bbox.min.x) * 2.0 + 1.0);
        let step_y = 1.0 / ((bbox.max.y - bbox.min.y) * 2.0 + 1.0);
        let step_z = 1.0 / ((bbox.max.z - bbox.min.z) * 2.0 + 1.0);

        if step_x < 0.0 || step_y < 0.0 || step_z < 0.0 {
            return 0.0;
        }

        let offset_x = (1.0 - (1.0 / step_x).floor() * step_x) / 2.0;
        let offset_z = (1.0 - (1.0 / step_z).floor() * step_z) / 2.0;

        let mut visible_points = 0;
        let mut total_points = 0;

        let mut k = 0.0;
        while k <= 1.0 {
            let mut l = 0.0;
            while l <= 1.0 {
                let mut m = 0.0;
                while m <= 1.0 {
                    let n = bbox.min.x + (bbox.max.x - bbox.min.x) * k;
                    let o = bbox.min.y + (bbox.max.y - bbox.min.y) * l;
                    let p = bbox.min.z + (bbox.max.z - bbox.min.z) * m;

                    let vec3d = Vector3::new(n + offset_x, o, p + offset_z);

                    if world
                        .raycast(vec3d, *explosion_pos, async |pos, world_ref| {
                            // Use non-blocking lookup; treat unloaded chunks as transparent
                            let Some(id) = world_ref.try_get_block_state_id(pos) else {
                                return false;
                            };
                            let state = BlockState::from_id(id);
                            !state.is_air() && !state.collision_shapes.is_empty()
                        })
                        .await
                        .is_none()
                    {
                        visible_points += 1;
                    }

                    total_points += 1;
                    m += step_z;
                }
                l += step_y;
            }
            k += step_x;
        }

        if total_points == 0 {
            return 0.0;
        }

        visible_points as f32 / total_points as f32
    }

    /// Returns the removed block count
    pub async fn explode(&self, world: &Arc<World>) -> u32 {
        let blocks = self.get_blocks_to_destroy(world);
        self.damage_entities(world).await;
        for (pos, (block, state)) in &blocks {
            world.set_block_state(pos, 0, BlockFlags::NOTIFY_ALL).await;
            world.close_container_screens_at(pos).await;

            let pumpkin_block = world.block_registry.get_pumpkin_block(block.id);

            if pumpkin_block.is_none_or(|s| s.should_drop_items_on_explosion()) {
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
        // TODO: fire
        blocks.len() as u32
    }
}
