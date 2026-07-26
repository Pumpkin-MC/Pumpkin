use super::{Entity, EntityBase};
use crate::server::Server;
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_data::block_properties::blocks_movement;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::{EntityPose, EntityType};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::bedrock::client::set_actor_data::{
    EntityMetadata, MetadataValue, entity_data_key,
};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::Ordering::{self, Relaxed};

impl Entity {
    #[expect(dead_code)]
    fn tick_block_underneath(_caller: &Arc<dyn EntityBase>) {
        // let world = self.world.read();

        // let (pos, block, state) = self.get_block_with_y_offset(0.2);

        // world
        //     .block_registry
        //     .on_stepped_on(&world, caller.as_ref(), pos, block, state)
        //     ;

        // TODO: Add this to on_stepped_on

        /*


        if self.on_ground.load(Ordering::SeqCst) {


            let (_pos, block, state) = self.get_block_with_y_offset(0.2);


            if let Some(live) = living {


                if block == Block::CAMPFIRE


                    || block == Block::SOUL_CAMPFIRE


                        && CampfireLikeProperties::from_state_id(state.id, &block).r#signal_fire


                {


                    let _ = live.damage(1.0, DamageType::CAMPFIRE);


                }





                if block == Block::MAGMA_BLOCK {


                    let _ = live.damage(1.0, DamageType::HOT_FLOOR);


                }


            }


        }


        */
    }

    pub(super) async fn tick_block_collisions(
        &self,
        caller: &Arc<dyn EntityBase>,
        server: &Server,
    ) -> bool {
        let bounding_box = self.bounding_box.load();
        let aabb = bounding_box.expand(-1.0e-7, -1.0e-7, -1.0e-7);

        let min = aabb.min_block_pos();
        let max = aabb.max_block_pos();

        let eye_height = self.get_eye_height();
        let mut eye_level_box = aabb;
        eye_level_box.min.y += eye_height;
        eye_level_box.max.y = eye_level_box.min.y;

        let mut suffocating = false;
        let world = self.world.load();

        for pos in BlockPos::iterate(min, max) {
            let (block, state) = world.get_block_and_state(&pos);
            if state.is_air() {
                continue;
            }

            // TODO: this is default predicate, vanilla overwrites it for some blocks,
            // see .suffocates(...) in Blocks.java
            let check_suffocation =
                !suffocating && blocks_movement(state, block.id) && state.is_full_cube();

            World::check_collision(
                &bounding_box,
                pos,
                state,
                check_suffocation,
                |collision_shape: &BoundingBox| {
                    if collision_shape.intersects(&eye_level_box) {
                        suffocating = true;
                    }
                },
            );

            let collision_shape = if block == &Block::POWDER_SNOW {
                crate::block::blocks::powder_snow::inside_collision_shape_for_entity(
                    caller.as_ref(),
                    &pos,
                )
                .await
            } else {
                world
                    .block_registry
                    .get_inside_collision_shape(block, &world, state, &pos)
                    .await
            };

            if bounding_box.intersects(&collision_shape.at_pos(pos)) {
                if block == &Block::POWDER_SNOW {
                    self.is_in_powder_snow.store(true, Relaxed);
                }
                world
                    .block_registry
                    .on_entity_collision(block, &world, caller.as_ref(), &pos, state, server)
                    .await;
            }
        }

        suffocating
    }

    // updateWaterState() in yarn

    pub(super) async fn update_fluid_state(&self, caller: &Arc<dyn EntityBase>) {
        let is_pushed = caller.is_pushed_by_fluids();
        let mut fluids = BTreeMap::new();

        let water_push = Vector3::default();

        let water_n = 0;

        let lava_push = Vector3::default();

        let lava_n = 0;

        let mut fluid_push = [water_push, lava_push];

        let mut fluid_n = [water_n, lava_n];

        let mut in_fluid = [false, false];

        // The maximum fluid height found

        let mut fluid_height: [f64; 2] = [0.0, 0.0];

        let bounding_box = self.bounding_box.load().expand(-0.001, -0.001, -0.001);

        let min = bounding_box.min_block_pos();

        let max = bounding_box.max_block_pos();

        let world = self.world.load();

        for x in min.0.x..=max.0.x {
            for y in min.0.y..=max.0.y {
                for z in min.0.z..=max.0.z {
                    let pos = BlockPos::new(x, y, z);

                    let (fluid, state) = world.get_fluid_and_fluid_state(&pos);

                    if fluid.id != Fluid::EMPTY.id {
                        let marginal_height =
                            f64::from(state.height) + f64::from(y) - bounding_box.min.y;

                        if marginal_height >= 0.0 {
                            let i = usize::from(
                                fluid.id == Fluid::FLOWING_LAVA.id || fluid.id == Fluid::LAVA.id,
                            );

                            fluid_height[i] = fluid_height[i].max(marginal_height);

                            in_fluid[i] = true;

                            if !is_pushed {
                                fluids.insert(fluid.id, fluid);

                                continue;
                            }

                            let mut fluid_velo = world.get_fluid_velocity(pos, fluid, state);

                            if fluid_height[i] < 0.4 {
                                fluid_velo = fluid_velo * fluid_height[i];
                            }

                            fluid_push[i] += fluid_velo;

                            fluid_n[i] += 1;

                            fluids.insert(fluid.id, fluid);
                        }
                    }
                }
            }
        }

        // BTreeMap auto-sorts water before lava as in vanilla

        for (_, fluid) in fluids {
            world
                .block_registry
                .on_entity_collision_fluid(fluid, caller.as_ref())
                .await;
        }

        let lava_speed = if world.dimension == Dimension::THE_NETHER {
            0.007
        } else {
            0.002_333_333
        };

        self.push_by_fluid(0.014, fluid_push[0], fluid_n[0]);

        self.push_by_fluid(lava_speed, fluid_push[1], fluid_n[1]);

        let water_height = fluid_height[0];

        let in_water = in_fluid[0];

        if in_water {
            if let Some(living) = caller.get_living_entity() {
                living.fall_distance.store(0.0);
            }

            if !self.touching_water.load(Ordering::SeqCst) {

                // TODO: Spawn splash particles
            }
        }

        self.water_height.store(water_height);

        self.touching_water.store(in_water, Ordering::SeqCst);

        let lava_height = fluid_height[1];

        let in_lava = in_fluid[1];

        if in_lava && let Some(living) = caller.get_living_entity() {
            let halved_fall = living.fall_distance.load() / 2.0;

            if halved_fall != 0.0 {
                living.fall_distance.store(halved_fall);
            }
        }

        self.lava_height.store(lava_height);

        self.touching_lava.store(in_lava, Ordering::SeqCst);
    }

    fn push_by_fluid(&self, speed: f64, mut push: Vector3<f64>, n: usize) {
        if push.length_squared() != 0.0 {
            if n > 0 {
                push = push * (1.0 / (n as f64));
            }

            if self.entity_type != &EntityType::PLAYER {
                push = push.normalize();
            }

            push = push * speed;

            let velo = self.velocity.load();

            if velo.x.abs() < 0.003 && velo.z.abs() < 0.003 && velo.length_squared() < 0.000_020_25
            {
                push = push.normalize() * 0.0045;
            }

            self.velocity.store(velo + push);
        }
    }

    /// Extinguishes this entity.
    pub fn extinguish(&self) {
        self.fire_ticks.store(0, Ordering::Relaxed);
    }

    /// Maximum freeze ticks (7 seconds at 20 tps)
    pub const MAX_FROZEN_TICKS: i32 = 140;

    /// Freeze damage is dealt every 40 ticks when fully frozen
    const FREEZE_DAMAGE_INTERVAL: i32 = 40;

    /// Check if the entity is currently in powder snow.
    ///
    /// The flag is reset at the start of each tick and set while processing
    /// block collisions for the current tick.
    pub fn is_in_powder_snow(&self) -> bool {
        self.is_in_powder_snow.load(Ordering::Relaxed)
    }

    /// Check if this entity type is immune to freezing
    pub fn is_freeze_immune(&self) -> bool {
        self.entity_type
            .has_tag(&tag::EntityType::MINECRAFT_FREEZE_IMMUNE_ENTITY_TYPES)
    }

    /// Mirrors vanilla `LivingEntity#canFreeze`: spectators and entities wearing
    /// freeze-immune wearables (e.g. leather armor) cannot freeze.
    async fn can_freeze(&self, caller: &dyn EntityBase) -> bool {
        if caller.is_spectator() || self.is_freeze_immune() {
            return false;
        }

        let Some(living) = caller.get_living_entity() else {
            return true;
        };

        let armor = {
            let equipment = living.entity_equipment.lock().await;
            [
                equipment.get(&EquipmentSlot::HEAD),
                equipment.get(&EquipmentSlot::CHEST),
                equipment.get(&EquipmentSlot::LEGS),
                equipment.get(&EquipmentSlot::FEET),
            ]
        };

        for stack in armor {
            let stack = stack.lock().await;
            if stack
                .get_item()
                .has_tag(&tag::Item::MINECRAFT_FREEZE_IMMUNE_WEARABLES)
            {
                return false;
            }
        }

        true
    }

    /// Ticks the frozen state of the entity.
    /// In powder snow and freezeable: `frozen_ticks` increases by 1 (up to `MAX_FROZEN_TICKS`)
    /// Otherwise: `frozen_ticks` decreases by 2 (down to 0)
    /// When fully frozen, deals 1 damage every 40 ticks
    pub async fn tick_frozen(&self, caller: &dyn EntityBase) {
        let can_freeze = self.can_freeze(caller).await;
        let in_powder_snow = self.is_in_powder_snow();
        let old_frozen_ticks = self.frozen_ticks.load(Ordering::Relaxed);

        let new_frozen_ticks = if in_powder_snow && can_freeze {
            // Increase frozen ticks when in powder snow
            (old_frozen_ticks + 1).min(Self::MAX_FROZEN_TICKS)
        } else {
            // Vanilla: thaw whenever not in powder snow OR when freezing is prevented
            (old_frozen_ticks - 2).max(0)
        };

        // Only update and send metadata if the value changed
        if new_frozen_ticks != old_frozen_ticks {
            self.frozen_ticks.store(new_frozen_ticks, Ordering::Relaxed);
            let mut bedrock_meta = EntityMetadata::new();
            bedrock_meta.set(
                entity_data_key::FREEZING_EFFECT_STRENGTH,
                MetadataValue::Float(new_frozen_ticks as f32),
            );
            self.send_meta_data(
                &[Metadata::new(
                    TrackedData::TICKS_FROZEN,
                    MetaDataType::INTEGER,
                    VarInt(new_frozen_ticks),
                )],
                Some(&bedrock_meta),
            );
        }

        // Vanilla parity: full-freeze damage is tick-phase based.
        if can_freeze
            && new_frozen_ticks >= Self::MAX_FROZEN_TICKS
            && self.age.load(Ordering::Relaxed) % Self::FREEZE_DAMAGE_INTERVAL == 0
        {
            caller.damage(caller, 1.0, DamageType::FREEZE).await;
        }
    }

    pub async fn check_block_collision(entity: &dyn EntityBase, server: &Server) {
        let aabb = entity.get_entity().bounding_box.load();
        let blockpos = BlockPos::new(
            (aabb.min.x + 0.001).floor() as i32,
            (aabb.min.y + 0.001).floor() as i32,
            (aabb.min.z + 0.001).floor() as i32,
        );
        let blockpos1 = BlockPos::new(
            (aabb.max.x - 0.001).floor() as i32,
            (aabb.max.y - 0.001).floor() as i32,
            (aabb.max.z - 0.001).floor() as i32,
        );
        let world = entity.get_entity().world.load();

        for x in blockpos.0.x..=blockpos1.0.x {
            for y in blockpos.0.y..=blockpos1.0.y {
                for z in blockpos.0.z..=blockpos1.0.z {
                    let pos = BlockPos::new(x, y, z);
                    let (block, state) = world.get_block_and_state(&pos);
                    let block_outlines = state.get_block_outline_shapes();

                    if state.outline_shapes.is_empty() {
                        world
                            .block_registry
                            .on_entity_collision(block, &world, entity, &pos, state, server)
                            .await;
                        let fluid = world.get_fluid(&pos);
                        world
                            .block_registry
                            .on_entity_collision_fluid(fluid, entity)
                            .await;
                        continue;
                    }
                    for outline in block_outlines {
                        let outline_aabb = outline.at_pos(pos);
                        if outline_aabb.intersects(&aabb) {
                            world
                                .block_registry
                                .on_entity_collision(block, &world, entity, &pos, state, server)
                                .await;
                            let fluid = world.get_fluid(&pos);
                            world
                                .block_registry
                                .on_entity_collision_fluid(fluid, entity)
                                .await;
                            break;
                        }
                    }
                }
            }
        }
    }

    pub async fn check_out_of_world(&self, dyn_self: &dyn EntityBase) {
        if self.pos.load().y < f64::from(self.world.load().dimension.min_y) - 64.0 {
            dyn_self.tick_in_void(dyn_self).await;
        }
    }

    pub async fn reset_state(&self) {
        self.pose.store(EntityPose::Standing);
        self.fall_flying.store(false, Relaxed);
        self.extinguish();
        self.set_on_fire(false).await;
    }
}
