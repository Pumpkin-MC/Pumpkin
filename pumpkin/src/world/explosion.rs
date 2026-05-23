use std::sync::Arc;

use pumpkin_data::{
    Block, BlockState, attributes::Attributes, damage::DamageType, entity::EntityType,
};
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use rand::random_range;
use rustc_hash::FxHashMap;

use crate::{
    block::{ExplodeArgs, drop_loot},
    entity::{Entity, EntityBase},
    world::{
        explosion_behavior::{DefaultExplosion, ExplosionBehavior, ExplosionBlockInteraction},
        loot::LootContextParameters,
    },
};

use super::{BlockFlags, World};

pub struct Explosion {
    pub power: f32,
    pub pos: Vector3<f64>,
    pub behavior: Arc<dyn ExplosionBehavior>,
    pub source_id: Option<i32>,
    pub cause_id: Option<i32>,
    pub damage_type: DamageType,
    pub block_interaction: ExplosionBlockInteraction,
    pub fire: bool,
}

impl Explosion {
    #[must_use]
    pub fn new(power: f32, pos: Vector3<f64>) -> Self {
        Self {
            power,
            pos,
            behavior: Arc::new(DefaultExplosion),
            source_id: None,
            cause_id: None,
            damage_type: DamageType::EXPLOSION,
            block_interaction: ExplosionBlockInteraction::Destroy,
            fire: false,
        }
    }

    pub fn should_trigger_blocks(&self, world: &Arc<World>) -> bool {
        let mob_griefing = world.level_info.load().game_rules.mob_griefing;
        let is_not_wind_charge = !self
            .source_id
            .and_then(|id| world.get_entity_by_id(id))
            .is_some_and(|e| e.get_entity().entity_type == &EntityType::BREEZE_WIND_CHARGE);

        self.block_interaction == ExplosionBlockInteraction::TriggerBlock
            && (is_not_wind_charge || mob_griefing)
    }

    pub fn should_affect_blocklike_entities(&self, world: &Arc<World>) -> bool {
        let mob_griefing = world.level_info.load().game_rules.mob_griefing;
        let is_not_wind_charge = !self
            .source_id
            .and_then(|id| world.get_entity_by_id(id))
            .is_some_and(|e| {
                let entity_type = e.get_entity().entity_type;
                entity_type == &EntityType::BREEZE_WIND_CHARGE
                    || entity_type == &EntityType::WIND_CHARGE
            });
        let should_destroy = matches!(
            self.block_interaction,
            ExplosionBlockInteraction::Destroy | ExplosionBlockInteraction::DestroyWithDecay
        );

        is_not_wind_charge && (mob_griefing || should_destroy)
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
                    let mut pos_y = self.pos.y;
                    let mut pos_z = self.pos.z;

                    let mut h = self.power * random_val.mul_add(0.6, 0.7);
                    while h > 0.0 {
                        let block_pos = BlockPos::floored(pos_x, pos_y, pos_z);
                        let (block, state) = world.get_block_and_state(&block_pos);
                        let (_, fluid_state) = world.get_fluid_and_fluid_state(&block_pos);

                        // if !world.is_in_build_limit(&block_pos) {
                        //     // Pass by reference
                        //     continue 'block2;
                        // }

                        if let Some(resistance) = self
                            .behavior
                            .get_block_explosion_resistance(block, fluid_state, &block_pos)
                            .await
                        {
                            h -= (resistance + 0.3) * 0.3;
                            if h > 0.0 && self.behavior.should_affect_block(block).await {
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

        // Exclude the source entity
        let entities: Vec<_> = world
            .get_all_at_box(&search_box)
            .into_iter()
            .filter(|e| {
                self.source_id
                    .is_none_or(|id| e.get_entity().entity_id != id)
            })
            .collect();

        let source = self.source_id.and_then(|id| world.get_entity_by_id(id));
        let cause = self.cause_id.and_then(|id| world.get_entity_by_id(id));

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

            let should_take_damage = self.behavior.should_damage_entity(entity).await;
            let knockback_multiplier = self.behavior.get_knockback_multiplier(entity).await;
            let exposure = if should_take_damage || knockback_multiplier != 0.0 {
                Self::calculate_exposure(&self.pos, entity, world).await as f64
            } else {
                0.0
            };

            if should_take_damage {
                let damage_multiplier = (1.0 - distance) * exposure;
                let damage =
                    (f64::midpoint(damage_multiplier * damage_multiplier, damage_multiplier)
                        * 7.0
                        * radius
                        + 1.0) as f32;

                entity_base
                    .damage_with_context(
                        entity,
                        damage,
                        self.damage_type,
                        Some(self.pos),
                        source.as_deref(),
                        cause.as_deref(),
                    )
                    .await;
            }

            // Calculate and apply knockback
            let dir_pos = if entity.entity_type == &EntityType::TNT {
                entity.pos.load()
            } else {
                entity.get_eye_pos()
            };
            let direction = (dir_pos - self.pos).normalize();
            let knockback_resistance = entity.get_living_entity().map_or(0.0, |living_entity| {
                living_entity.get_attribute_value(&Attributes::EXPLOSION_KNOCKBACK_RESISTANCE)
            });

            let knockback_power =
                (1.0 - distance) * exposure * knockback_multiplier * (1.0 - knockback_resistance);
            let knockback = direction * knockback_power;
            entity.add_velocity(knockback);

            // TODO: Transfer projectile owner to explosion source
            // TODO: Wind Charge fall damage immunity
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
                            let state = world_ref.get_block_state(pos);
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

    /// Returns the count of removed blocks
    /// The source entity must still exist when calling this.
    /// Call `entity.remove()` after this, not before.
    pub async fn explode(&self, world: &Arc<World>) -> u32 {
        let blocks = self.get_blocks_to_destroy(world).await;
        let source = self.source_id.and_then(|id| world.get_entity_by_id(id));
        let cause = self.cause_id.and_then(|id| world.get_entity_by_id(id));
        self.damage_entities(world).await;

        for (pos, (block, state)) in &blocks {
            let pumpkin_block = world.block_registry.get_pumpkin_block(block.id);

            match self.block_interaction {
                ExplosionBlockInteraction::Keep => {
                    continue;
                }
                ExplosionBlockInteraction::TriggerBlock => {}
                _ => {
                    world.set_block_state(pos, 0, BlockFlags::NOTIFY_ALL).await;
                    world.close_container_screens_at(pos).await;

                    if pumpkin_block.is_none_or(|s| s.should_drop_items_on_explosion()) {
                        let params = LootContextParameters {
                            block_state: Some(state),
                            explosion_radius: Some(self.power),
                            position: Some(pumpkin_util::math::vector3::Vector3::new(
                                pos.0.x as f64,
                                pos.0.y as f64,
                                pos.0.z as f64,
                            )),
                            world_time: world.level_info.load().day_time as u64,
                            ..Default::default()
                        };
                        drop_loot(world, block, pos, false, params).await;
                    }
                }
            }
            if let Some(pumpkin_block) = pumpkin_block {
                pumpkin_block
                    .explode(ExplodeArgs {
                        world,
                        block,
                        state,
                        position: pos,
                        source: source.as_deref(),
                        cause: cause.as_deref(),
                        block_interaction: &self.block_interaction,
                    })
                    .await;
            }
        }

        if self.fire {
            for pos in (blocks).keys() {
                if random_range(0..3) == 0
                    && world.get_block_state(pos).is_air()
                    && world.get_block_state(&pos.down()).is_solid()
                {
                    world
                        .set_block_state(pos, Block::FIRE.default_state.id, BlockFlags::NOTIFY_ALL)
                        .await;
                }
            }
        }
        blocks.len() as u32
    }
}
