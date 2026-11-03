use std::sync::{Arc, Weak, atomic::Ordering::Relaxed};

use pumpkin_data::{
    HorizontalFacingExt,
    data_component_impl::EquipmentSlot,
    effect::StatusEffect,
    entity::{EntityStatus, EntityType},
    item_stack::ItemStack,
    potion::Effect,
    sound::Sound,
    structures::{StructureKeys, StructureSet},
    tag::{self, Taggable},
};
use pumpkin_protocol::bedrock::server::ActorEventID;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3, wrap_degrees};
use pumpkin_world::generation::generator::structure_finder::find_nearest_structure_start_in_sets;
use rand::RngExt;

use super::DolphinEntity;
use crate::entity::{
    Entity, EntityBase,
    ai::{
        goal::{
            Controls, Goal, look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
            melee_attack::MeleeAttackGoal, random_swimming::RandomSwimmingGoal,
        },
        pathfinder::node::PathComputationType,
        target_match::{SearchArea, TargetMatch},
        target_predicate::TargetPredicate,
        util::default_random_pos,
    },
    item::ItemEntity,
    mob::Mob,
};

const ITEM_SEARCH_RANGE: f64 = 8.0;
const STEPS_TO_CHECK: [i32; 6] = [0, 1, 4, 5, 6, 7];
pub(super) const JUMP_HORIZONTAL_SPEED: f64 = 0.6;
pub(super) const JUMP_VERTICAL_SPEED: f64 = 0.7;
pub(super) const FOLLOW_ENTITY_MOVEMENT_SPEED: f32 = 0.015;
const FOLLOW_DIRECTION_MOVEMENT_SPEED: f32 = 0.01;

pub(super) fn register(dolphin: &Arc<DolphinEntity>) {
    let selector = &dolphin.mob_entity.goals_selector;
    selector.add_goal(
        1,
        Box::new(DolphinSwimToTreasureGoal {
            dolphin: Arc::downgrade(dolphin),
            treasure_pos: None,
            stuck: false,
        }),
    );
    selector.add_goal(
        2,
        Box::new(DolphinSwimWithPlayerGoal {
            player: None,
            retained_player: dolphin.swimming_player.clone(),
        }),
    );
    selector.add_goal(4, Box::new(RandomSwimmingGoal::new(1.0, 10)));
    selector.add_goal(4, Box::new(RandomLookAroundGoal::default()));
    let weak = Arc::downgrade(&(dolphin.clone() as Arc<dyn Mob>));
    selector.add_goal(
        5,
        LookAtEntityGoal::with_default(weak, &EntityType::PLAYER, 6.0),
    );
    selector.add_goal(5, Box::new(DolphinJumpGoal { breached: false }));
    selector.add_goal(6, Box::new(MeleeAttackGoal::new(1.2, true)));
    selector.add_goal(7, Box::new(MoveToItemGoal { cooldown: 0 }));
    selector.add_goal(8, Box::new(PlayWithItemsGoal));
    selector.add_goal(
        8,
        Box::new(FollowPlayerRiddenEntityGoal::new(
            |entity| {
                entity
                    .cast_any()
                    .is::<crate::entity::vehicle::boat::BoatEntity>()
            },
            dolphin.boat_player.clone(),
        )),
    );
    selector.add_goal(
        8,
        Box::new(FollowPlayerRiddenEntityGoal::new(
            |entity| {
                TargetMatch::AnyOf(&[&EntityType::NAUTILUS, &EntityType::ZOMBIE_NAUTILUS])
                    .matches(entity.get_entity().entity_type)
            },
            dolphin.nautilus_player.clone(),
        )),
    );
}

struct DolphinSwimToTreasureGoal {
    dolphin: Weak<DolphinEntity>,
    treasure_pos: Option<BlockPos>,
    stuck: bool,
}

impl DolphinSwimToTreasureGoal {
    fn reached_treasure(&self, mob: &dyn Mob) -> bool {
        self.treasure_pos.is_some_and(|treasure| {
            let pos = mob.get_entity().pos.load();
            let target = BlockPos::new(treasure.0.x, pos.y.floor() as i32, treasure.0.z);
            target
                .0
                .to_f64()
                .add_raw(0.5, 0.5, 0.5)
                .squared_distance_to_vec(&pos)
                < 4.0 * 4.0
        })
    }
}

impl Goal for DolphinSwimToTreasureGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.dolphin
            .upgrade()
            .is_some_and(|dolphin| dolphin.got_fish())
            && mob.get_entity().air_supply.load(Relaxed) >= 100
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.treasure_pos.is_some()
            && !self.reached_treasure(mob)
            && !self.stuck
            && mob.get_entity().air_supply.load(Relaxed) >= 100
    }

    fn can_stop(&self) -> bool {
        false
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.stuck = false;
        mob.get_mob_entity().steered().navigator().stop();
        let entity = mob.get_entity();
        let world = entity.world.load();
        let generator = world.level.world_gen();
        let origin = entity.block_pos.load();
        let wanted: Vec<_> = tag::WorldgenStructure::MINECRAFT_DOLPHIN_LOCATED
            .0
            .iter()
            .filter_map(|name| StructureKeys::from_name(name))
            .collect();
        let mut scans: Vec<(usize, Vec<StructureKeys>)> = Vec::new();
        for key in wanted {
            let Some(index) = StructureSet::ALL
                .iter()
                .position(|set| set.structures.iter().any(|entry| entry.structure == key))
            else {
                continue;
            };
            if let Some((_, keys)) = scans.iter_mut().find(|(set, _)| *set == index) {
                keys.push(key);
            } else {
                scans.push((index, vec![key]));
            }
        }
        let searches: Vec<_> = scans
            .iter()
            .map(|(index, keys)| (&StructureSet::ALL[*index], keys.as_slice()))
            .collect();
        if let Some((pos, _)) =
            find_nearest_structure_start_in_sets(origin, &searches, 50, &generator)
        {
            self.treasure_pos = Some(pos);
            world.send_entity_status(
                entity,
                EntityStatus::DolphinLookingForTreasure,
                ActorEventID::None,
            );
        } else {
            self.stuck = true;
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if (self.treasure_pos.is_none() || self.reached_treasure(mob) || self.stuck)
            && let Some(dolphin) = self.dolphin.upgrade()
        {
            dolphin.set_got_fish(false);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(treasure) = self.treasure_pos else {
            return;
        };
        let entity = mob.get_entity();
        let close_to_next = {
            let steered = mob.get_mob_entity().steered();
            let navigator = steered.navigator();
            navigator.is_done()
                || navigator.get_target_pos().is_some_and(|target| {
                    target
                        .0
                        .to_f64()
                        .add_raw(0.5, 0.5, 0.5)
                        .squared_distance_to_vec(&entity.pos.load())
                        < 12.0 * 12.0
                })
        };
        if !close_to_next {
            return;
        }
        let treasure = treasure.0.to_f64().add_raw(0.5, 0.5, 0.5);
        let mut next = default_random_pos::get_pos_towards(
            mob,
            16,
            1,
            treasure,
            f64::from(std::f32::consts::FRAC_PI_8),
        )
        .or_else(|| {
            default_random_pos::get_pos_towards(
                mob,
                8,
                4,
                treasure,
                f64::from(std::f32::consts::FRAC_PI_2),
            )
        });
        let world = entity.world.load();
        if let Some(pos) = next {
            let block_pos = pos.to_block_pos();
            let (block, state) = world.get_block_and_state(&block_pos);
            if !world
                .get_fluid(&block_pos)
                .has_tag(&tag::Fluid::MINECRAFT_WATER)
                || !world
                    .block_registry
                    .is_pathfindable(block, state, PathComputationType::Water)
            {
                next = default_random_pos::get_pos_towards(
                    mob,
                    8,
                    5,
                    treasure,
                    f64::from(std::f32::consts::FRAC_PI_2),
                );
            }
        }
        let Some(next) = next else {
            self.stuck = true;
            return;
        };
        let steered = mob.get_mob_entity().steered();
        mob.get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .look_at_with_range(
                next.x,
                next.y,
                next.z,
                mob.get_max_head_rotation() + 20.0,
                mob.get_max_look_pitch_change(),
            );
        steered.navigator().move_to_coords(
            next.x,
            next.y,
            next.z,
            1.3,
            &mob.get_mob_entity().living_entity,
        );
        if mob.get_random().random_range(0..self.get_tick_count(80)) == 0 {
            world.send_entity_status(
                entity,
                EntityStatus::DolphinLookingForTreasure,
                ActorEventID::None,
            );
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

struct DolphinSwimWithPlayerGoal {
    player: Option<Arc<dyn EntityBase>>,
    retained_player: Arc<crossbeam::atomic::AtomicCell<Option<i32>>>,
}

impl DolphinSwimWithPlayerGoal {
    fn give_grace(&self) {
        if let Some(player) = self.player.as_ref().and_then(|player| player.get_player()) {
            player.add_effect(Effect {
                effect_type: &StatusEffect::DOLPHINS_GRACE,
                duration: 100,
                amplifier: 0,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            });
        }
    }
}

impl Goal for DolphinSwimWithPlayerGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let conditions = TargetPredicate::create_non_attackable()
            .set_base_max_distance(10.0)
            .ignore_visibility();
        self.player = TargetMatch::Type(&EntityType::PLAYER).find_nearest(
            &world,
            &SearchArea::from_feet(entity, 10.0, 0.0).players_anywhere(),
            |player| conditions.test(&world, Some(mob), player),
        );
        let can_start = self.player.as_ref().is_some_and(|player| {
            player.get_entity().is_swimming()
                && mob.get_mob_entity().get_target().is_none_or(|target| {
                    target.get_entity().entity_id != player.get_entity().entity_id
                })
        });
        if !can_start {
            self.player = None;
        }
        self.retained_player.store(
            self.player
                .as_ref()
                .map(|player| player.get_entity().entity_id),
        );
        can_start
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.player.as_ref().is_some_and(|player| {
            player.get_entity().is_swimming()
                && player
                    .get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&mob.get_entity().pos.load())
                    < 256.0
        })
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.give_grace();
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.player = None;
        self.retained_player.store(None);
        mob.get_mob_entity().steered().navigator().stop();
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(player) = &self.player else {
            return;
        };
        let target = player.get_entity();
        let pos = target.get_eye_pos();
        let steered = mob.get_mob_entity().steered();
        mob.get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .look_at_with_range(
                pos.x,
                pos.y,
                pos.z,
                mob.get_max_head_rotation() + 20.0,
                mob.get_max_look_pitch_change(),
            );
        if target
            .pos
            .load()
            .squared_distance_to_vec(&mob.get_entity().pos.load())
            < 6.25
        {
            steered.navigator().stop();
        } else if let Some(living) = player.get_living_entity() {
            steered.navigator().move_to_entity(
                living,
                super::SWIM_WITH_PLAYER_SPEED,
                &mob.get_mob_entity().living_entity,
            );
        }
        if target.is_swimming() && mob.get_random().random_range(0..6) == 0 {
            self.give_grace();
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

struct DolphinJumpGoal {
    breached: bool,
}

impl Goal for DolphinJumpGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if mob.get_random().random_range(0..self.get_tick_count(10)) != 0 {
            return false;
        }
        let entity = mob.get_entity();
        let direction = entity
            .get_horizontal_facing()
            .to_block_direction()
            .to_offset();
        let origin = entity.block_pos.load();
        let world = entity.world.load();
        STEPS_TO_CHECK.iter().all(|step| {
            let pos = origin.offset(direction * *step);
            world.get_fluid(&pos).has_tag(&tag::Fluid::MINECRAFT_WATER)
                && !world
                    .get_block(&pos)
                    .has_tag(&tag::Block::MINECRAFT_BLOCKS_DOLPHIN_JUMP)
                && world.get_block(&pos.up()).is_air()
                && world.get_block(&pos.up_height(2)).is_air()
        })
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        let pitch = entity.pitch.load();
        (entity.velocity.load().y.powi(2) >= f64::from(0.03f32)
            || pitch == 0.0
            || pitch.abs() >= 10.0
            || !entity.is_in_water())
            && !entity.on_ground.load(Relaxed)
    }

    fn can_stop(&self) -> bool {
        false
    }

    fn start(&mut self, mob: &dyn Mob) {
        let entity = mob.get_entity();
        let direction = entity
            .get_horizontal_facing()
            .to_block_direction()
            .to_offset();
        entity.velocity.store(entity.velocity.load().add_raw(
            f64::from(direction.x) * JUMP_HORIZONTAL_SPEED,
            JUMP_VERTICAL_SPEED,
            f64::from(direction.z) * JUMP_HORIZONTAL_SPEED,
        ));
        mob.get_mob_entity().steered().navigator().stop();
    }

    fn stop(&mut self, mob: &dyn Mob) {
        mob.get_entity().set_pitch(0.0);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let entity = mob.get_entity();
        let already_breached = self.breached;
        if !already_breached {
            self.breached = entity
                .world
                .load()
                .get_fluid(&entity.block_pos.load())
                .has_tag(&tag::Fluid::MINECRAFT_WATER);
        }
        if self.breached && !already_breached {
            mob.play_entity_sound(Sound::EntityDolphinJump, 1.0, 1.0);
        }
        let movement = entity.velocity.load();
        let pitch = entity.pitch.load();
        if movement.y.powi(2) < f64::from(0.03f32) && pitch != 0.0 {
            entity.set_pitch(pitch + 0.2 * wrap_degrees(-pitch));
        } else if movement.length() > f64::from(1.0E-5f32) {
            entity.set_pitch(
                ((-movement.y).atan2(movement.horizontal_length()) * f64::from(180.0f32)
                    / f64::from(std::f32::consts::PI)) as f32,
            );
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::JUMP
    }
}

fn allowed_items(mob: &dyn Mob) -> Vec<Arc<dyn EntityBase>> {
    let entity = mob.get_entity();
    entity
        .world
        .load()
        .get_entities_at_box(&entity.bounding_box.load().expand(
            ITEM_SEARCH_RANGE,
            ITEM_SEARCH_RANGE,
            ITEM_SEARCH_RANGE,
        ))
        .into_iter()
        .filter(|other| {
            other.get_item_entity().is_some_and(|item| {
                item.get_pickup_delay() == 0
                    && item.get_entity().is_alive()
                    && item.get_entity().is_in_water()
            })
        })
        .collect()
}

fn drop_item(mob: &dyn Mob) -> bool {
    let stack = mob
        .get_mob_entity()
        .living_entity
        .entity_equipment
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&EquipmentSlot::MAIN_HAND);
    if stack.is_empty() {
        return false;
    }
    let entity = mob.get_entity();
    let world = entity.world.load();
    let mut event = crate::plugin::api::events::entity::entity_drop_item::EntityDropItemEvent::new(
        entity.entity_id,
        stack.item.registry_key.to_string(),
        stack.item_count,
    );
    if let Some(server) = world.server.upgrade() {
        server.plugin_manager.fire_blocking(&server, &mut event);
    }
    if event.cancelled {
        return false;
    }
    let mut rng = mob.get_random();
    let direction = rng.random::<f32>() * std::f32::consts::TAU;
    let spread = 0.02 * rng.random::<f32>();
    let yaw = entity.yaw.load().to_radians();
    let pitch = entity.pitch.load().to_radians();
    let velocity = Vector3::new(
        f64::from(
            -0.3 * pumpkin_util::math::sin(yaw) * pumpkin_util::math::cos(pitch)
                + pumpkin_util::math::cos(direction) * spread,
        ),
        f64::from(0.3 * pumpkin_util::math::sin(pitch) * 1.5),
        f64::from(
            0.3 * pumpkin_util::math::cos(yaw) * pumpkin_util::math::cos(pitch)
                + pumpkin_util::math::sin(direction) * spread,
        ),
    );
    let pos = entity.pos.load();
    let item = ItemEntity::new_with_velocity(
        Entity::new(
            world.clone(),
            Vector3::new(pos.x, entity.get_eye_y() - f64::from(0.3f32), pos.z),
            &EntityType::ITEM,
        ),
        stack,
        velocity,
        40,
    );
    item.set_thrower(entity.entity_uuid);
    if !world.spawn_entity(Arc::new(item)) {
        return false;
    }
    mob.get_mob_entity()
        .set_item_slot(&EquipmentSlot::MAIN_HAND, ItemStack::EMPTY.clone());
    true
}

struct MoveToItemGoal {
    cooldown: i32,
}

impl Goal for MoveToItemGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.cooldown <= mob.get_entity().tick_count.load(Relaxed) && !allowed_items(mob).is_empty()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(item) = allowed_items(mob).first() {
            let pos = item.get_entity().pos.load();
            mob.get_mob_entity().steered().navigator().move_to_coords(
                pos.x,
                pos.y,
                pos.z,
                f64::from(1.2f32),
                &mob.get_mob_entity().living_entity,
            );
            mob.play_entity_sound(Sound::EntityDolphinPlay, 1.0, 1.0);
        }
        self.cooldown = 0;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if drop_item(mob) {
            self.cooldown =
                mob.get_entity().tick_count.load(Relaxed) + mob.get_random().random_range(0..100);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        if let Some(item) = allowed_items(mob).first() {
            let pos = item.get_entity().pos.load();
            mob.get_mob_entity().steered().navigator().move_to_coords(
                pos.x,
                pos.y,
                pos.z,
                f64::from(1.2f32),
                &mob.get_mob_entity().living_entity,
            );
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

struct PlayWithItemsGoal;

impl Goal for PlayWithItemsGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        !mob.get_mob_entity()
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::MAIN_HAND)
            .is_empty()
    }

    fn stop(&mut self, mob: &dyn Mob) {
        drop_item(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        drop_item(mob);
    }
}

enum FollowEntityGoal {
    GoToEntity,
    GoInEntityDirection,
}

struct FollowPlayerRiddenEntityGoal {
    targets: fn(&dyn EntityBase) -> bool,
    player: Option<Arc<dyn EntityBase>>,
    retained_player: Arc<crossbeam::atomic::AtomicCell<Option<i32>>>,
    time_to_recalc_path: i32,
    current_goal: FollowEntityGoal,
}

impl FollowPlayerRiddenEntityGoal {
    fn new(
        targets: fn(&dyn EntityBase) -> bool,
        retained_player: Arc<crossbeam::atomic::AtomicCell<Option<i32>>>,
    ) -> Self {
        Self {
            targets,
            player: None,
            retained_player,
            time_to_recalc_path: 0,
            current_goal: FollowEntityGoal::GoToEntity,
        }
    }

    fn nearby_controllers(&self, mob: &dyn Mob) -> Vec<Arc<dyn EntityBase>> {
        let entity = mob.get_entity();
        entity
            .world
            .load()
            .get_entities_at_box(&entity.bounding_box.load().expand(5.0, 5.0, 5.0))
            .into_iter()
            .filter(|other| (self.targets)(other.as_ref()))
            .filter_map(|other| other.get_controlling_passenger())
            .filter(|controller| controller.get_player().is_some())
            .collect()
    }

    fn has_moved(player: &dyn EntityBase) -> bool {
        player.get_entity().movement.load().horizontal_length() > f64::from(1.0E-5f32)
    }
}

impl Goal for FollowPlayerRiddenEntityGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.player
            .as_ref()
            .is_some_and(|player| Self::has_moved(player.as_ref()))
            || self
                .nearby_controllers(mob)
                .iter()
                .any(|player| Self::has_moved(player.as_ref()))
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.player.as_ref().is_some_and(|player| {
            player.get_entity().has_vehicle() && Self::has_moved(player.as_ref())
        })
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(player) = self.nearby_controllers(mob).first() {
            self.player = Some(player.clone());
            self.retained_player
                .store(Some(player.get_entity().entity_id));
        }
        self.time_to_recalc_path = 0;
        self.current_goal = FollowEntityGoal::GoToEntity;
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.player = None;
        self.retained_player.store(None);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(player) = &self.player else {
            return;
        };
        let living = &mob.get_mob_entity().living_entity;
        let speed = match self.current_goal {
            FollowEntityGoal::GoToEntity => FOLLOW_ENTITY_MOVEMENT_SPEED,
            FollowEntityGoal::GoInEntityDirection => FOLLOW_DIRECTION_MOVEMENT_SPEED,
        };
        living
            .entity
            .update_velocity_from_input(living.movement_input.load(), f64::from(speed));
        living
            .entity
            .move_entity(mob, living.entity.velocity.load());
        self.time_to_recalc_path -= 1;
        if self.time_to_recalc_path > 0 {
            return;
        }
        self.time_to_recalc_path = self.get_tick_count(10);
        let target = player.get_entity();
        let direction = target
            .get_horizontal_facing()
            .to_block_direction()
            .to_offset();
        let distance = target
            .pos
            .load()
            .squared_distance_to_vec(&living.entity.pos.load());
        let (destination, switch) = match self.current_goal {
            FollowEntityGoal::GoToEntity => (
                target.block_pos.load().offset(direction * -1).down(),
                distance < 4.0 * 4.0,
            ),
            FollowEntityGoal::GoInEntityDirection => (
                target.block_pos.load().offset(direction * 10).down(),
                distance > 12.0 * 12.0,
            ),
        };
        mob.get_mob_entity()
            .steered()
            .navigator()
            .move_to_pos(destination, 1.0, living);
        if switch {
            self.time_to_recalc_path = 0;
            self.current_goal = match self.current_goal {
                FollowEntityGoal::GoToEntity => FollowEntityGoal::GoInEntityDirection,
                FollowEntityGoal::GoInEntityDirection => FollowEntityGoal::GoToEntity,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;
    use pumpkin_data::item::Item;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[tokio::test]
    async fn treasure_goal_keeps_fish_when_interrupted_for_air() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let dolphin = DolphinEntity::new(Entity::new(
            world.clone(),
            Vector3::new(4.5, 64.5, 0.5),
            &EntityType::DOLPHIN,
        ));
        let mut goal = DolphinSwimToTreasureGoal {
            dolphin: Arc::downgrade(&dolphin),
            treasure_pos: Some(BlockPos::new(0, 0, 0)),
            stuck: false,
        };
        dolphin.set_got_fish(true);
        assert!(goal.should_continue(dolphin.as_ref()));
        dolphin.get_entity().air_supply.store(99, Relaxed);
        assert!(!goal.should_continue(dolphin.as_ref()));
        goal.stop(dolphin.as_ref());
        assert!(dolphin.got_fish());
        dolphin.get_entity().set_pos(Vector3::new(4.49, 64.5, 0.5));
        goal.stop(dolphin.as_ref());
        assert!(!dolphin.got_fish());
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn item_play_picks_up_and_throws_the_whole_stack_with_a_delay() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let dolphin = DolphinEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::DOLPHIN,
        ));
        let item = ItemEntity::new_with_velocity(
            Entity::new(
                world.clone(),
                dolphin.get_entity().pos.load(),
                &EntityType::ITEM,
            ),
            ItemStack::new(3, &Item::DIAMOND),
            Vector3::default(),
            0,
        );
        world.add_entity_silent(Arc::new(item));
        crate::entity::mob::pick_up_nearby_items(dolphin.as_ref());
        assert!(drop_item(dolphin.as_ref()));
        let thrown = world
            .entities
            .load()
            .iter()
            .filter_map(|entity| entity.get_item_entity())
            .find(|item| !item.get_entity().is_removed())
            .map(|item| {
                (
                    item.get_item_stack()
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .clone(),
                    item.get_pickup_delay(),
                    item.get_thrower(),
                )
            });
        let (stack, delay, thrower) = thrown.ok_or("dolphin did not throw its item")?;
        assert_eq!(stack.item, &Item::DIAMOND);
        assert_eq!(stack.item_count, 3);
        assert_eq!(delay, 40);
        assert_eq!(thrower, Some(dolphin.get_entity().entity_uuid));
        assert!(!drop_item(dolphin.as_ref()));
        world.level.shutdown().await;
        Ok(())
    }
}
