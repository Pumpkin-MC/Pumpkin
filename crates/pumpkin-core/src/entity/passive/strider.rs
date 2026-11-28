use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::{EntityPose, EntityType};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockState};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use uuid::Uuid;

use crate::block::PathComputationType;
use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::attributes::{Modifier, ModifierOperation};
use crate::entity::mob::spawn::{EntitySpawnReason, SpawnGroupData, finalize_spawn};
use crate::entity::r#type::from_type;
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        Controls, Goal, ParentHandle,
        breed::BreedGoal,
        escape_danger::EscapeDangerGoal,
        follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal,
        move_to_target_pos::{MoveToTargetPos, MoveToTargetPosGoal},
        tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    dismount_helper,
    item_steerable::{ItemBasedSteering, ItemSteerable},
    mob::{Mob, MobEntity, equipment::equip_on_target, zombie},
    passive::animal::Animal,
    player::Player,
};
use crate::plugin::api::events::entity::strider_temperature_change::StriderTemperatureChangeEvent;
use crate::world::World;

pub const SUFFOCATING_MODIFIER_ID: &str = "minecraft:suffocating";
pub const SUFFOCATING_MODIFIER_AMOUNT: f64 = -0.340_000_003_576_278_7;
const STRIDER_LIQUID_COLLISION_SHAPE: BoundingBox =
    BoundingBox::new_array([0.0, 0.0, 0.0], [1.0, 0.5, 1.0]);

struct StriderGoToLavaTarget;

static STRIDER_GO_TO_LAVA_TARGET: StriderGoToLavaTarget = StriderGoToLavaTarget;

impl MoveToTargetPos for StriderGoToLavaTarget {
    fn is_target_pos(&self, world: &World, block_pos: BlockPos) -> bool {
        let (block, _) = world.get_block_and_state(&block_pos);
        if block != &Block::LAVA {
            return false;
        }
        let above = block_pos.up();
        let (above_block, above_state) = world.get_block_and_state(&above);
        world
            .block_registry
            .is_pathfindable(above_block, above_state, PathComputationType::Land)
    }

    fn get_move_to_target(&self, target_pos: BlockPos) -> BlockPos {
        target_pos
    }

    fn should_recalculate_path(&self, trying_time: i32) -> bool {
        trying_time % 20 == 0
    }
}

pub struct StriderGoToLavaGoal {
    move_to_target_pos_goal: MoveToTargetPosGoal<StriderGoToLavaTarget>,
}

impl StriderGoToLavaGoal {
    #[must_use]
    pub fn new(speed: f64) -> Self {
        Self {
            move_to_target_pos_goal: MoveToTargetPosGoal::new(
                ParentHandle::from_static(&STRIDER_GO_TO_LAVA_TARGET),
                speed,
                8,
                2,
            ),
        }
    }
}

impl Goal for StriderGoToLavaGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if mob.get_entity().touching_lava.load(Ordering::Relaxed) {
            return false;
        }
        self.move_to_target_pos_goal.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        if mob.get_entity().touching_lava.load(Ordering::Relaxed) {
            return false;
        }
        let world = mob.get_entity().world.load();
        STRIDER_GO_TO_LAVA_TARGET.is_target_pos(&world, self.move_to_target_pos_goal.target_pos)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.tick(mob);
    }

    fn should_run_every_tick(&self) -> bool {
        self.move_to_target_pos_goal.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.move_to_target_pos_goal.controls()
    }
}

pub struct StriderEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub steering: ItemBasedSteering,
    pub suffocating: AtomicBool,
}

impl StriderEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let strider = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            steering: ItemBasedSteering::new(pumpkin_data::tracked_data::strider::DATA_BOOST_TIME),
            suffocating: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(strider);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        let goal_selector = &mob_arc.mob_entity.goals_selector;

        goal_selector.add_goal(1, EscapeDangerGoal::new(1.65));
        goal_selector.add_goal(2, BreedGoal::new(1.0));
        goal_selector.add_goal(
            3,
            Box::new(TemptGoal::with_tag(
                1.4,
                &tag::Item::MINECRAFT_STRIDER_TEMPT_ITEMS,
                false,
            )),
        );
        goal_selector.add_goal(4, Box::new(StriderGoToLavaGoal::new(1.0)));
        goal_selector.add_goal(5, Box::new(FollowParentGoal::new(1.0)));
        let mut stroll = WanderAroundGoal::new(1.0);
        stroll.set_interval(60);
        goal_selector.add_goal(7, Box::new(stroll));
        goal_selector.add_goal(
            8,
            LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
        );
        goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));
        goal_selector.add_goal(
            9,
            LookAtEntityGoal::with_default(mob_weak, &EntityType::STRIDER, 8.0),
        );

        let mut navigator = mob_arc
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.set_pathfinding_malus(PathType::Water, -1.0);
        navigator.set_pathfinding_malus(PathType::Lava, 0.0);
        navigator.set_pathfinding_malus(PathType::DangerFire, 0.0);
        navigator.set_pathfinding_malus(PathType::DamageFire, 0.0);
        drop(navigator);

        mob_arc
    }

    #[must_use]
    pub fn is_suffocating(&self) -> bool {
        self.suffocating.load(Ordering::Relaxed)
    }

    pub fn set_suffocating(&self, suffocating: bool) {
        self.suffocating.store(suffocating, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::strider::DATA_SUFFOCATING,
            suffocating,
        );
        self.mob_entity
            .living_entity
            .update_attribute(&Attributes::MOVEMENT_SPEED, |inst| {
                if suffocating {
                    inst.add_or_replace_modifier(Modifier {
                        id: SUFFOCATING_MODIFIER_ID.to_string(),
                        amount: SUFFOCATING_MODIFIER_AMOUNT,
                        operation: ModifierOperation::MultiplyBase,
                    });
                } else {
                    inst.remove_modifier(SUFFOCATING_MODIFIER_ID);
                }
            });
    }

    fn update_suffocating(&self) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let block_pos = entity.block_pos.load();
        let (inside_block, _) = world.get_block_and_state(&block_pos);
        let on_pos = entity.get_pos_with_y_offset(0.2).0;
        let (on_block, _) = world.get_block_and_state(&on_pos);
        let in_warm_blocks = inside_block.has_tag(&tag::Block::MINECRAFT_STRIDER_WARM_BLOCKS)
            || on_block.has_tag(&tag::Block::MINECRAFT_STRIDER_WARM_BLOCKS)
            || entity.lava_height.load() > 0.0;
        let on_warm_strider = entity.get_vehicle().is_some_and(|v| {
            v.get_mob()
                .and_then(Mob::as_strider)
                .is_some_and(|s| !s.is_suffocating())
        });
        let should_shiver = !in_warm_blocks && !on_warm_strider;
        if should_shiver != self.is_suffocating() {
            let is_shivering = if let Some(server) = entity.world.load().server.upgrade() {
                let event = server.plugin_manager.fire_blocking(
                    &server,
                    StriderTemperatureChangeEvent::new(entity.entity_id, should_shiver),
                );
                if event.cancelled {
                    return;
                }
                event.is_shivering
            } else {
                should_shiver
            };
            if is_shivering != self.is_suffocating() {
                self.set_suffocating(is_shivering);
            }
        }
    }

    fn float_strider(&self) {
        let entity = self.get_entity();
        if entity.touching_lava.load(Ordering::Relaxed) {
            let pos = entity.block_pos.load();
            let world = entity.world.load();
            if !entity.is_above_shape(&STRIDER_LIQUID_COLLISION_SHAPE, &pos)
                || world.get_fluid(&pos.up()).matches_type(&Fluid::LAVA)
            {
                let velo = entity.velocity.load();
                entity.velocity.store(Vector3::new(
                    velo.x * 0.5,
                    velo.y.mul_add(0.5, 0.05),
                    velo.z * 0.5,
                ));
            } else {
                entity.on_ground.store(true, Ordering::Relaxed);
            }
        }
    }

    #[must_use]
    pub fn check_strider_spawn_rules(
        _entity_type: &'static EntityType,
        world: &World,
        pos: &BlockPos,
        _reason: EntitySpawnReason,
    ) -> bool {
        let mut check_pos = pos.up();
        while world
            .get_fluid(&check_pos)
            .has_tag(&tag::Fluid::MINECRAFT_LAVA)
        {
            check_pos = check_pos.up();
        }
        world.get_block(&check_pos).is_air()
    }

    #[must_use]
    pub fn is_being_tempted(&self) -> bool {
        self.mob_entity.goals_selector.is_running::<TemptGoal>()
    }
}

impl AgeableMob for StriderEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    fn default_baby_spawn_chance(&self) -> f32 {
        crate::entity::ageable::STRIDER_BABY_SPAWN_CHANCE
    }
}

impl Animal for StriderEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_STRIDER_FOOD)
    }

    fn animal_walk_target_value(&self, pos: &BlockPos) -> f32 {
        let world = self.get_mob_entity().living_entity.entity.world.load();
        let (block, _) = world.get_block_and_state(pos);
        if block == &Block::LAVA {
            10.0
        } else if self.get_entity().touching_lava.load(Ordering::Relaxed) {
            f32::NEG_INFINITY
        } else {
            0.0
        }
    }

    fn play_eating_sound(&self) {
        let mut rng = self.get_random();
        let pitch = 1.0 + (rng.random::<f32>() - rng.random::<f32>()) * 0.2;
        self.play_entity_sound(Sound::EntityStriderEat, 1.0, pitch);
    }
}

impl Mob for StriderEntity {
    fn can_dispenser_equip_into_slot(&self, slot: &EquipmentSlot) -> bool {
        *slot == EquipmentSlot::SADDLE || self.mob_entity.can_pick_up_loot()
    }

    fn mob_controlling_passenger(&self) -> Option<Arc<dyn EntityBase>> {
        self.saddled_controlling_passenger(Some(&Item::WARPED_FUNGUS_ON_A_STICK))
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        let sound = if self.get_entity().touching_lava.load(Ordering::Relaxed) {
            Sound::EntityStriderStepLava
        } else {
            Sound::EntityStriderStep
        };
        self.play_entity_sound(sound, 1.0, 1.0);
    }

    fn mob_next_step(&self) -> f32 {
        self.mob_entity.living_entity.entity.move_dist.load() + 0.6
    }

    fn mob_ai_step(&self) {
        if let Some(rider) = self.mob_controlling_passenger()
            && let Some(player) = rider.get_player()
        {
            let entity = player.get_entity();
            let yaw = entity.yaw.load();
            self.get_entity()
                .set_rotation(yaw, entity.pitch.load() * 0.5);
            self.get_entity().body_yaw.store(yaw);
            self.get_entity().head_yaw.store(yaw);
            self.steering.tick_boost();
        }
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let mut rng = self.get_random();
        if self.is_being_tempted() {
            if rng.random_range(0..140) == 0 {
                self.make_sound(Sound::EntityStriderHappy);
            }
        } else if self.is_panicking() && rng.random_range(0..60) == 0 {
            self.make_sound(Sound::EntityStriderRetreat);
        }

        if !self.mob_entity.is_no_ai() {
            self.update_suffocating();
        }
    }

    fn post_tick(&self) {
        self.float_strider();
    }

    fn can_stand_on_fluid(&self, fluid: &Fluid) -> bool {
        fluid.matches_type(&Fluid::LAVA)
    }

    fn liquid_collision_shape(&self) -> Option<BoundingBox> {
        Some(STRIDER_LIQUID_COLLISION_SHAPE)
    }

    fn mob_can_add_passenger(&self, _passenger: &dyn EntityBase) -> bool {
        !self.get_entity().has_passengers() && !self.get_entity().is_submerged_in(&Fluid::LAVA)
    }

    fn mob_dismount_location_for_passenger(
        &self,
        passenger: &dyn EntityBase,
    ) -> Option<Vector3<f64>> {
        let base = self.get_entity();
        let passenger_base = passenger.get_entity();
        let world = base.world.load();
        let pos = base.pos.load();
        let bounding_box = base.bounding_box.load();

        let strider_width = bounding_box.max.x - bounding_box.min.x;
        let passenger_box = passenger_base.bounding_box.load();
        let passenger_width = passenger_box.max.x - passenger_box.min.x;
        let passenger_yaw = passenger_base.yaw.load();

        let directions = [
            dismount_helper::get_collision_horizontal_escape_vector(
                strider_width,
                passenger_width,
                passenger_yaw,
            ),
            dismount_helper::get_collision_horizontal_escape_vector(
                strider_width,
                passenger_width,
                passenger_yaw - 22.5,
            ),
            dismount_helper::get_collision_horizontal_escape_vector(
                strider_width,
                passenger_width,
                passenger_yaw + 22.5,
            ),
            dismount_helper::get_collision_horizontal_escape_vector(
                strider_width,
                passenger_width,
                passenger_yaw - 45.0,
            ),
            dismount_helper::get_collision_horizontal_escape_vector(
                strider_width,
                passenger_width,
                passenger_yaw + 45.0,
            ),
        ];

        let collider_top = bounding_box.max.y;
        let collider_bottom = bounding_box.min.y - 0.5;

        let mut target_block_positions = Vec::new();
        for direction in directions {
            let x = (pos.x + direction.x).floor() as i32;
            let z = (pos.z + direction.z).floor() as i32;
            let mut y = collider_top;
            while y > collider_bottom {
                let bp = BlockPos(Vector3::new(x, y.floor() as i32, z));
                if !target_block_positions.contains(&bp) {
                    target_block_positions.push(bp);
                }
                y -= 1.0;
            }
        }

        let poses: &[EntityPose] = if passenger.get_player().is_some() {
            &[
                EntityPose::Standing,
                EntityPose::Crouching,
                EntityPose::Swimming,
            ]
        } else {
            &[EntityPose::Standing]
        };

        for target_pos in target_block_positions {
            let (fluid, _) = world.get_fluid_and_fluid_state(&target_pos);
            if fluid.matches_type(&Fluid::LAVA) {
                continue;
            }

            let block_floor_height = dismount_helper::block_floor_height(&world, &target_pos);
            if dismount_helper::is_block_floor_valid(block_floor_height) {
                let location = Vector3::new(
                    f64::from(target_pos.0.x) + 0.5,
                    f64::from(target_pos.0.y) + block_floor_height,
                    f64::from(target_pos.0.z) + 0.5,
                );

                for &pose in poses {
                    let dims = passenger_base.get_dimensions(pose);
                    let bbox = BoundingBox::new_from_pos(location.x, location.y, location.z, &dims);
                    if dismount_helper::can_dismount_to(&world, &bbox) {
                        passenger_base.set_pose(pose);
                        return Some(location);
                    }
                }
            }
        }

        Some(Vector3::new(pos.x, collider_top, pos.z))
    }

    fn should_check_fall_damage(&self) -> bool {
        if self.get_entity().touching_lava.load(Ordering::Relaxed) {
            self.mob_entity.living_entity.fall_distance.store(0.0);
            return false;
        }
        true
    }

    fn is_sensitive_to_water(&self) -> bool {
        true
    }

    fn check_spawn_obstruction(&self, world: &World) -> bool {
        world.is_unobstructed(self.get_entity())
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        if self.is_panicking() || self.is_being_tempted() {
            None
        } else {
            self.get_entity().entity_type.ambient_sound
        }
    }

    fn is_stable_destination(&self, world: &World, pos: &BlockPos) -> bool {
        let (block, _) = world.get_block_and_state(pos);
        block == &Block::LAVA
            || self
                .get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_stable_destination(world, pos)
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn as_strider(&self) -> Option<&Self> {
        Some(self)
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        _reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        if self.is_baby() {
            return self.finalize_spawn_super(group_data);
        }

        let mut rng = self.get_random();
        let entity = self.get_entity();
        let pos = entity.pos.load();

        let group_data = if rng.random_range(0..30) == 0 && world.is_spawning_monsters() {
            let zp = from_type(&EntityType::ZOMBIFIED_PIGLIN, pos, world, Uuid::new_v4());
            zp.get_entity().set_rotation(entity.yaw.load(), 0.0);
            let is_baby = rng.random::<f32>() < zombie::BABY_CHANCE;
            finalize_spawn(
                &zp,
                world,
                EntitySpawnReason::Jockey,
                Some(SpawnGroupData::Zombie {
                    is_baby,
                    can_spawn_jockey: false,
                }),
            );
            if let Some(mob) = zp.get_mob() {
                mob.get_mob_entity().set_item_slot(
                    &EquipmentSlot::MAIN_HAND,
                    ItemStack::new(1, &Item::WARPED_FUNGUS_ON_A_STICK),
                );
            }
            self.mob_entity.set_item_slot_and_drop_when_killed(
                &EquipmentSlot::SADDLE,
                ItemStack::new(1, &Item::SADDLE),
            );
            self.mob_entity.add_pending_rider(zp);
            Some(SpawnGroupData::Ageable {
                group_size: 0,
                should_spawn_baby: true,
                baby_spawn_chance: 0.0,
            })
        } else if rng.random_range(0..10) == 0 {
            let baby = from_type(&EntityType::STRIDER, pos, world, Uuid::new_v4());
            baby.get_entity().set_rotation(entity.yaw.load(), 0.0);
            if let Some(ageable) = baby.get_mob().and_then(Mob::as_ageable) {
                ageable.set_baby(true);
            }
            finalize_spawn(&baby, world, EntitySpawnReason::Jockey, None);
            self.mob_entity.add_pending_rider(baby);
            Some(SpawnGroupData::Ageable {
                group_size: 0,
                should_spawn_baby: true,
                baby_spawn_chance: 0.0,
            })
        } else {
            Some(SpawnGroupData::Ageable {
                group_size: 0,
                should_spawn_baby: true,
                baby_spawn_chance: crate::entity::ageable::STRIDER_BABY_SPAWN_CHANCE,
            })
        };

        self.finalize_spawn_super(group_data)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_ageable_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_ageable_nbt(nbt);
        self.mob_entity
            .read_legacy_saddle(nbt.get_bool("Saddle") == Some(true));
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_item_steerable(&self) -> Option<&dyn ItemSteerable> {
        Some(self)
    }

    fn can_use_slot(&self, slot: &EquipmentSlot) -> bool {
        *slot != EquipmentSlot::SADDLE
            || (self.mob_entity.living_entity.entity.is_alive() && !self.is_baby())
    }

    fn get_equip_sound(&self, slot: &EquipmentSlot) -> Option<Sound> {
        (*slot == EquipmentSlot::SADDLE).then_some(Sound::EntityStriderSaddle)
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::strider::DATA_BABY_ID, true);
        }
        entity
            .synched_data
            .define(pumpkin_data::tracked_data::strider::DATA_SUFFOCATING, false);
        if self.is_suffocating() {
            entity.set_synced_data(pumpkin_data::tracked_data::strider::DATA_SUFFOCATING, true);
        }
        entity.synched_data.define(
            pumpkin_data::tracked_data::strider::DATA_BOOST_TIME,
            VarInt(0),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if !self.is_food(item_stack)
            && self.is_saddled()
            && !self.get_entity().has_passengers()
            && !player.get_entity().is_sneaking()
        {
            let world = player.world();
            let ent = &self.mob_entity.living_entity.entity;
            if let Some(vehicle) = world.get_entity_by_id(ent.entity_id)
                && let Some(passenger) = world.get_player_by_id(player.entity_id())
            {
                ent.try_add_passenger(vehicle, passenger as Arc<dyn EntityBase>);
            }
            return true;
        }

        if self.animal_interact(player, item_stack) {
            return true;
        }
        self.is_equippable_in_slot(item_stack, &EquipmentSlot::SADDLE)
            && equip_on_target(self, player, item_stack)
    }
}

impl ItemSteerable for StriderEntity {
    fn boost(&self) -> bool {
        let mut rng = self.get_random();
        self.steering.boost(self.get_entity(), &mut rng)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector2::Vector2;
    use pumpkin_util::math::vector3::Vector3;
    use pumpkin_world::chunk::ChunkData;
    use pumpkin_world::world::BlockFlags;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn create_strider(world: &Arc<World>) -> Arc<StriderEntity> {
        let strider = StriderEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::STRIDER,
        ));
        world.add_entity_silent(strider.clone());
        strider
    }

    #[tokio::test]
    async fn suffocating_modifier_updates_movement_speed() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let strider = create_strider(&world);

        let base_speed = strider
            .mob_entity
            .living_entity
            .get_attribute_value(&Attributes::MOVEMENT_SPEED);
        assert!(!strider.is_suffocating());

        strider.set_suffocating(true);
        assert!(strider.is_suffocating());
        let cold_speed = strider
            .mob_entity
            .living_entity
            .get_attribute_value(&Attributes::MOVEMENT_SPEED);
        assert!(cold_speed < base_speed);

        let expected = base_speed * (1.0 + SUFFOCATING_MODIFIER_AMOUNT);
        assert!((cold_speed - expected).abs() < 1e-6);

        strider.set_suffocating(false);
        assert!(!strider.is_suffocating());
        let restored_speed = strider
            .mob_entity
            .living_entity
            .get_attribute_value(&Attributes::MOVEMENT_SPEED);
        assert!((restored_speed - base_speed).abs() < 1e-6);

        Ok(())
    }

    #[tokio::test]
    async fn passenger_mounting_and_lava_check() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
        let strider = create_strider(&world);

        let rider = crate::entity::passive::cow::CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::COW,
        ));

        assert!(strider.can_add_passenger(rider.as_ref()));

        strider
            .get_entity()
            .add_passenger(strider.clone(), rider.clone());
        assert!(!strider.can_add_passenger(rider.as_ref()));

        let strider2 = create_strider(&world);
        let eye_y = strider2.get_entity().pos.load().y + strider2.get_entity().get_eye_height();
        let eye_pos = BlockPos::floored(0.5, eye_y, 0.5);
        world.set_block_state(
            &eye_pos,
            Block::LAVA.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
        assert!(!strider2.can_add_passenger(rider.as_ref()));

        Ok(())
    }

    #[tokio::test]
    async fn dismount_on_lava_lake_falls_back_to_strider_top() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
        let strider = create_strider(&world);
        strider.get_entity().set_pos(Vector3::new(0.5, 64.5, 0.5));

        for x in -2..=2 {
            for z in -2..=2 {
                world.set_block_state(
                    &BlockPos::new(x, 64, z),
                    Block::LAVA.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                );
            }
        }

        let rider = crate::entity::passive::cow::CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.5, 0.5),
            &EntityType::COW,
        ));
        world.add_entity_silent(rider.clone());
        strider
            .get_entity()
            .add_passenger(strider.clone(), rider.clone());

        strider
            .get_entity()
            .remove_passenger(rider.get_entity().entity_id);

        let dismount_y = rider.get_entity().pos.load().y;
        let strider_top_y = strider.get_entity().bounding_box.load().max.y;
        assert!((dismount_y - strider_top_y).abs() < 1e-4);

        Ok(())
    }

    #[tokio::test]
    async fn dismount_lands_on_safe_ground_next_to_lava() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
        let strider = create_strider(&world);
        strider.get_entity().set_pos(Vector3::new(0.5, 64.5, 0.5));

        for x in -2..=2 {
            for z in -2..=2 {
                let block = if x == 0 && z == 1 {
                    Block::STONE.default_state.id
                } else {
                    Block::LAVA.default_state.id
                };
                world.set_block_state(&BlockPos::new(x, 64, z), block, BlockFlags::NOTIFY_ALL);
            }
        }

        let rider = crate::entity::passive::cow::CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.5, 0.5),
            &EntityType::COW,
        ));
        world.add_entity_silent(rider.clone());
        strider
            .get_entity()
            .add_passenger(strider.clone(), rider.clone());

        strider
            .get_entity()
            .remove_passenger(rider.get_entity().entity_id);

        let dismount_pos = rider.get_entity().pos.load();
        assert!((dismount_pos.x - 0.5).abs() < 1e-4);
        assert!((dismount_pos.y - 65.0).abs() < 1e-4);
        assert!((dismount_pos.z - 1.5).abs() < 1e-4);

        Ok(())
    }

    #[tokio::test]
    async fn strider_spawn_rules() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));

        let lava_pos = BlockPos::new(0, 64, 0);
        let above_pos = BlockPos::new(0, 65, 0);
        let solid_pos = BlockPos::new(1, 64, 0);

        world.set_block_state(
            &lava_pos,
            Block::LAVA.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
        world.set_block_state(
            &above_pos,
            Block::AIR.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
        world.set_block_state(
            &solid_pos,
            Block::STONE.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );

        assert!(StriderEntity::check_strider_spawn_rules(
            &EntityType::STRIDER,
            &world,
            &lava_pos,
            EntitySpawnReason::Natural,
        ));

        assert!(StriderEntity::check_strider_spawn_rules(
            &EntityType::STRIDER,
            &world,
            &solid_pos,
            EntitySpawnReason::Natural,
        ));

        world.set_block_state(
            &above_pos,
            Block::STONE.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
        assert!(!StriderEntity::check_strider_spawn_rules(
            &EntityType::STRIDER,
            &world,
            &lava_pos,
            EntitySpawnReason::Natural,
        ));

        Ok(())
    }
}
