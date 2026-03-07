use super::World;
use crate::block::{OnExplosionHitArgs, on_explosion_hit_default};
use crate::entity::{Entity, EntityBase};
use crate::world::damage_source::DamageSource;
use crate::world::explosion_damage_calculator::ExplosionDamageCalculator;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::{Block, BlockState, entity::EntityType};
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use pumpkin_world::item::ItemStack;
use rand::seq::SliceRandom;
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// A struct representing an *explosion*, which can be triggered
/// using [`Explosion::explode`].
#[expect(dead_code)]
#[must_use]
pub struct Explosion {
    /// The world this explosion belongs to.
    world: Arc<World>,
    /// The explosion radius of this explosion.
    power: f32,
    /// The position of this explosion.
    pos: Vector3<f64>,
    /// The direct entity that caused this explosion. May not exist.
    source_entity: Option<Arc<dyn EntityBase>>,
    /// A calculator for various values found during the trigger of an `Explosion`.
    damage_calculator: ExplosionDamageCalculator,
    /// The damage source of this `Explosion`.
    damage_source: DamageSource,
    /// How this `Explosion` should interact with blocks it affects.
    pub block_interaction: BlockInteraction,

    // TODO
    fire: bool,
}

pub struct NewExplosionArgs<'a> {
    pub world: &'a Arc<World>,
    pub source_entity: Option<&'a Arc<dyn EntityBase>>,
    pub damage_source: Option<DamageSource>,
    pub damage_calculator: Option<ExplosionDamageCalculator>,
    pub power: f32,
    pub pos: Vector3<f64>,
    pub fire: bool,
    pub block_interaction: BlockInteraction,
}

impl Explosion {
    /// Creates a new `Explosion`.
    pub fn new(args: NewExplosionArgs) -> Self {
        Self {
            world: args.world.clone(),
            power: args.power,
            pos: args.pos,
            block_interaction: args.block_interaction,
            fire: args.fire,
            source_entity: args.source_entity.cloned(),
            damage_calculator: args
                .damage_calculator
                .unwrap_or_else(|| Self::create_damage_calculator(args.source_entity.cloned())),
            damage_source: args.damage_source.unwrap_or_else(|| {
                DamageSource::explosion_from_direct(
                    args.world.as_ref(),
                    args.source_entity.cloned(),
                )
            }),
        }
    }

    #[must_use]
    fn create_damage_calculator(entity: Option<Arc<dyn EntityBase>>) -> ExplosionDamageCalculator {
        entity.map_or_else(ExplosionDamageCalculator::default, |e| {
            ExplosionDamageCalculator::EntityBased { source_entity: e }
        })
    }

    #[must_use]
    pub const fn power(&self) -> f32 {
        self.power
    }

    #[must_use]
    pub const fn pos(&self) -> Vector3<f64> {
        self.pos
    }

    async fn get_blocks_to_destroy(&self) -> Vec<(BlockPos, &'static Block, &'static BlockState)> {
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
                        let (_, fluid_state) =
                            self.world.get_fluid_and_fluid_state(&block_pos).await;

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
                            && !state.is_air()
                            && self.damage_calculator.block_should_explode(
                                self,
                                &self.world,
                                block_pos,
                                block,
                                state,
                                h,
                            )
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
        map.into_iter().map(|(p, (b, s))| (p, b, s)).collect()
    }

    async fn damage_entities(&self) -> FxHashMap<i32, Vector3<f64>> {
        let mut map = FxHashMap::default();

        // Explosion is too small
        if self.power < 1.0e-5 {
            return map;
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
            let knockback_multiplier =
                self.damage_calculator.knockback_multiplier(entity).await as f64;
            let h = if !entity_takes_damage && knockback_multiplier == 0.0 {
                0.0
            } else {
                Self::calculate_exposure(&self.pos, entity, &self.world).await
            };

            if entity_takes_damage {
                let damage = self
                    .damage_calculator
                    .compute_entity_damage(self, entity, h);
                entity
                    .damage_with_context(
                        entity_base.as_ref(),
                        damage,
                        self.damage_source.damage_type,
                        self.damage_source.damage_source_pos,
                        self.damage_source.direct_entity.as_deref(),
                        self.damage_source.causing_entity.as_deref(),
                    )
                    .await;
            }

            // Calculate the knockback velocity.
            let knockback_resistance = entity.get_living_entity().map_or(0.0, |e| {
                e.get_attribute_value(&Attributes::EXPLOSION_KNOCKBACK_RESISTANCE)
            });
            let scale =
                (1.0 - distance) * (h as f64) * knockback_multiplier * (1.0 - knockback_resistance);

            let dir_pos = if entity.entity_type == &EntityType::TNT {
                entity.pos.load()
            } else {
                entity.get_eye_pos()
            };
            let knockback_velocity = (dir_pos - self.pos).normalize() * scale;

            if let Some(player) = entity.get_player() {
                map.insert(player.entity_id(), knockback_velocity);
            }
            entity.add_velocity(knockback_velocity).await;
        }

        map
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

    fn interacts_with_blocks(&self) -> bool {
        self.block_interaction != BlockInteraction::Keep
    }

    async fn interact_with_blocks(
        &self,
        blocks: &mut [(BlockPos, &'static Block, &'static BlockState)],
    ) {
        let mut collectors: Vec<StackCollector> = Vec::new();
        blocks.shuffle(&mut rand::rng());

        for (pos, block, state) in blocks {
            let args = OnExplosionHitArgs {
                world: &self.world,
                block,
                state,
                position: pos,
                explosion: self,
            };
            let stacks = if let Some(pumpkin_block) =
                self.world.block_registry.get_pumpkin_block(block.id)
            {
                pumpkin_block.on_explosion_hit(args).await
            } else {
                on_explosion_hit_default(args).await
            };
            if let Some(stacks) = stacks {
                for mut stack in stacks {
                    for collector in &mut collectors {
                        collector.try_merge(&mut stack);
                        if stack.is_empty() {
                            break;
                        }
                    }
                    if !stack.is_empty() {
                        collectors.push(StackCollector { pos: *pos, stack });
                    }
                }
            }
        }

        for collector in collectors {
            self.world.drop_stack(&collector.pos, collector.stack).await;
        }
    }

    /// Calls this [`Explosion`] to explode.
    ///
    /// # Returns
    /// A tuple containing:
    /// - the removed block count.
    /// - a map of the affected **players** and their knockback velocities.
    pub async fn explode(self) -> (u32, FxHashMap<i32, Vector3<f64>>) {
        let mut blocks = self.get_blocks_to_destroy().await;
        let knockback_map = self.damage_entities().await;
        if self.interacts_with_blocks() {
            self.interact_with_blocks(&mut blocks).await;
        }
        /*
        for (pos, (block, state)) in &blocks {
            self.world
                .set_block_state(pos, 0, BlockFlags::NOTIFY_ALL)
                .await;
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
        */

        (blocks.len() as u32, knockback_map)
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

    /// Returns whether this `Explosion` triggers blocks.
    /// For example, if an explosion does trigger a block,
    /// a button can be pressed temporarily by the explosion.
    #[must_use]
    pub fn triggers_blocks(&self) -> bool {
        self.block_interaction == BlockInteraction::TriggerBlock
            && self.source_entity.as_ref().is_some_and(|e| {
                if e.get_entity().entity_type == &EntityType::BREEZE_WIND_CHARGE {
                    self.world.level_info.load().game_rules.mob_griefing
                } else {
                    true
                }
            })
    }

    /// Whether an explosion is considered to be "small".
    #[must_use]
    pub fn is_small(&self) -> bool {
        self.power < 2.0 || !self.interacts_with_blocks()
    }
}

struct StackCollector {
    pos: BlockPos,
    stack: ItemStack,
}

impl StackCollector {
    fn try_merge(&mut self, stack: &mut ItemStack) {
        if self.stack.item_count + stack.item_count <= stack.get_max_stack_size()
            && self.stack.are_items_and_components_equal(stack)
        {
            self.stack.merge(stack, 16);
        }
    }
}

/// The interaction to a block by an explosion.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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

    #[must_use]
    pub const fn from_game_rule(game_rule: bool) -> Self {
        if game_rule {
            Self::DestroyWithDecay
        } else {
            Self::Destroy
        }
    }
}

/// The type of interaction of an explosion.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ExplosionInteraction {
    None,
    Block,
    Mob,
    Tnt,
    Trigger,
}
