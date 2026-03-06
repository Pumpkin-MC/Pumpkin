use std::sync::Arc;

use super::{BlockFlags, World};
use crate::world::damage_source::DamageSource;
use crate::world::explosion_damage_calculator::ExplosionDamageCalculator;
use crate::{
    block::{ExplodeArgs, drop_loot},
    entity::{Entity, EntityBase},
    world::loot::LootContextParameters,
};
use pumpkin_data::attributes::Attributes;
use pumpkin_data::{Block, BlockState, entity::EntityType};
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use rustc_hash::FxHashMap;

pub struct Explosion {
    world: Arc<World>,
    power: f32,
    pos: Vector3<f64>,

    source_entity: Option<Arc<dyn EntityBase>>,
    damage_calculator: Box<dyn ExplosionDamageCalculator>,

    damage_source: DamageSource,

    block_interaction: BlockInteraction,

    // TODO
    fire: bool,
}

impl Explosion {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        world: &Arc<World>,
        source_entity: Option<Arc<dyn EntityBase>>,
        damage_source: Option<DamageSource>,
        damage_calculator: Option<Box<dyn ExplosionDamageCalculator>>,
        power: f32,
        pos: Vector3<f64>,
        fire: bool,
        block_interaction: BlockInteraction,
    ) -> Self {
        Self {
            world: world.clone(),
            power,
            pos,
            block_interaction,
            fire,
            source_entity: source_entity.clone(),
            damage_calculator: damage_calculator.unwrap_or_else(Self::create_damage_calculator),
            damage_source: damage_source.unwrap_or_else(|| {
                DamageSource::from_explosion(
                    Self::indirect_source_entity(source_entity.clone(), world.as_ref()),
                    source_entity,
                )
            }),
        }
    }

    /// Gets the indirect source entity from a direct one.
    fn indirect_source_entity(
        source_entity: Option<Arc<dyn EntityBase>>,
        world: &World,
    ) -> Option<Arc<dyn EntityBase>> {
        source_entity.and_then(|e| {
            if e.get_entity().entity_type == &EntityType::TNT {
                return None; // TODO: get owner of TNT
            }
            if e.get_living_entity().is_some() {
                return Some(e);
            }
            if let Some(thrown_entity) = e.get_thrown_item_entity()
                && let Some(i) = thrown_entity.owner_id
            {
                return world.get_player_by_id(i).map(|a| a as Arc<dyn EntityBase>);
            }
            None
        })
    }

    #[must_use]
    fn create_damage_calculator() -> Box<dyn ExplosionDamageCalculator> {
        todo!()
    }

    #[must_use]
    pub const fn power(&self) -> f32 {
        self.power
    }

    #[must_use]
    pub const fn pos(&self) -> Vector3<f64> {
        self.pos
    }

    async fn get_blocks_to_destroy(
        &self,
    ) -> FxHashMap<BlockPos, (&'static Block, &'static BlockState)> {
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
                        let (block, state) = self.world.get_block_and_state(&block_pos).await;
                        let (_, fluid_state) = self.world.get_fluid_and_fluid_state(&block_pos).await;

                        if !self.world.is_in_build_limit(block_pos) {
                            continue 'block2;
                        }

                        if let Some(resistance) = self.damage_calculator.block_blast_resistance(
                            self,
                            &self.world,
                            block_pos,
                            block,
                            state,
                            fluid_state,
                        ) {
                            h -= (resistance + 0.3) * 0.3;
                        }

                        if h > 0.0
                            && self
                                .damage_calculator
                                .block_should_explode(self, &self.world, block_pos, block, state, h)
                        {
                            map.insert(block_pos, (block, state));
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

    async fn damage_entities(&self) {
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

        let entities = self.source_entity.clone().map_or_else(
            || self.world.get_all_at_box(&search_box),
            |e| self.world.get_all_at_box_except(&search_box, e.as_ref()),
        );

        for entity_base in entities {
            if entity_base.is_immune_to_explosion() {
                continue;
            }

            // Skip spectators (no damage, no knockback)
            if entity_base.is_spectator() {
                continue;
            }

            let entity = entity_base.get_entity();

            let distance = entity.pos.load().squared_distance_to_vec(&self.pos).sqrt() / radius;
            if distance > 1.0 {
                continue;
            }

            // Calculate the dealt damage.
            let entity_takes_damage = self.damage_calculator.entity_takes_damage(self, entity);
            let knockback_multiplier = self.damage_calculator.knockback_multiplier(entity) as f64;
            let h = if !entity_takes_damage && knockback_multiplier == 0.0 {
                0.0
            } else {
                Self::calculate_exposure(&self.pos, entity, &self.world).await
            };

            if entity_takes_damage {
                entity
                    .get_entity()
                    .damage_with_context(
                        entity,
                        self.damage_calculator
                            .compute_entity_damage(self, entity, h),
                        self.damage_source.damage_type,
                        self.damage_source.damage_source_pos,
                        self.damage_source.direct_entity.as_deref(),
                        self.damage_source.causing_entity.as_deref(),
                    )
                    .await;
            }

            // Calculate the knockback velocity.
            let knockback_resistance = entity
                .get_living_entity()
                .map_or(0.0, |e| e.get_attribute_value(&Attributes::EXPLOSION_KNOCKBACK_RESISTANCE));
            let scale =
                (1.0 - distance) * (h as f64) * knockback_multiplier * (1.0 - knockback_resistance);

            let dir_pos = if entity.entity_type == &EntityType::TNT {
                entity.pos.load()
            } else {
                entity.get_eye_pos()
            };
            let knockback_velocity = (dir_pos - self.pos).normalize() * scale;

            entity.add_velocity(knockback_velocity).await;
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
                            let state = world_ref.get_block_state(pos).await;
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
    pub async fn explode(&self) -> u32 {
        let blocks = self.get_blocks_to_destroy().await;
        self.damage_entities().await;
        for (pos, (block, state)) in &blocks {
            self.world.set_block_state(pos, 0, BlockFlags::NOTIFY_ALL).await;
            self.world.close_container_screens_at(pos).await;

            let pumpkin_block = self.world.block_registry.get_pumpkin_block(block.id);

            if pumpkin_block.is_none_or(|s| s.should_drop_items_on_explosion()) {
                let params = LootContextParameters {
                    block_state: Some(state),
                    explosion_radius: Some(self.power),
                    ..Default::default()
                };
                drop_loot(&self.world, block, pos, false, params).await;
            }
            if let Some(pumpkin_block) = pumpkin_block {
                pumpkin_block
                    .explode(ExplodeArgs {
                        world: &self.world,
                        block,
                        position: pos,
                    })
                    .await;
            }
        }
        // TODO: fire
        blocks.len() as u32
    }

    pub fn affect_block_like_entities(&self, world: &World) -> bool {
        let b = self.source_entity.is_none()
            || self.source_entity.as_ref().is_some_and(|e| {
                let t = e.get_entity().entity_type;
                t == &EntityType::WIND_CHARGE || t == &EntityType::BREEZE_WIND_CHARGE
            });

        if world.level_info.load().game_rules.mob_griefing {
            b
        } else {
            self.block_interaction.affect_block_like_entities()
        }
    }
}

/// The interaction to a block by an explosion.
pub enum BlockInteraction {
    Keep,
    Destroy,
    DestroyWithDecay,
    TriggerBlock,
}

impl BlockInteraction {
    /// Whether this interaction represents affecting block-like entities (like levers).
    #[must_use]
    pub const fn affect_block_like_entities(&self) -> bool {
        matches!(self, Self::Destroy | Self::DestroyWithDecay)
    }
}
