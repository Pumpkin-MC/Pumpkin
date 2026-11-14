use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use pumpkin_data::BlockState;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use crate::entity::ai::goal::revenge::RevengeGoal;
use crate::entity::ai::goal::spear_use::SpearUseGoal;
use crate::entity::ai::goal::wander_around::WanderAroundGoal;
use crate::entity::ai::goal::zombie_attack::ZombieAttackGoal;
use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::ai::util::goal_utils;
use crate::entity::attributes::{Modifier, ModifierOperation};
use crate::entity::{
    Entity, EntityBase,
    ai::behavior::neutral::apply_targets,
    mob::{
        Mob, MobEntity,
        equipment::RegionalDifficulty,
        neutral::{NeutralData, NeutralMob},
        spawn::{EntitySpawnReason, SpawnGroupData},
        zombie::ZombieEntityBase,
    },
};
use crate::plugin::api::events::entity::entity_target::TargetReason;
use crate::plugin::api::events::entity::pig_zombie_anger::PigZombieAngerEvent;
use crate::world::World;

const SPEED_MODIFIER_ATTACKING_ID: &str = "minecraft:attacking";
const SPEED_MODIFIER_ATTACKING: f64 = 0.05;
const FIRST_ANGER_SOUND_DELAY: std::ops::RangeInclusive<i32> = 0..=20;
const ALERT_INTERVAL: std::ops::RangeInclusive<i32> = 80..=120;
const LAVA_MALUS: f32 = 8.0;
const GOLDEN_SPEAR_ODDS: i32 = 20;

pub struct ZombifiedPiglinEntity {
    entity: Arc<ZombieEntityBase>,
    neutral_data: NeutralData,
    play_first_anger_sound_in: AtomicI32,
    ticks_until_next_alert: AtomicI32,
}

impl ZombifiedPiglinEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = ZombieEntityBase::with_behaviour_goals(entity, add_behaviour_goals);
        entity
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_pathfinding_malus(PathType::Lava, LAVA_MALUS);
        Arc::new(Self {
            entity,
            neutral_data: NeutralData::default(),
            play_first_anger_sound_in: AtomicI32::new(0),
            ticks_until_next_alert: AtomicI32::new(0),
        })
    }

    fn maybe_play_first_anger_sound(&self) {
        let remaining = self.play_first_anger_sound_in.load(Ordering::Relaxed);
        if remaining > 0 {
            self.play_first_anger_sound_in
                .store(remaining - 1, Ordering::Relaxed);
            if remaining == 1 {
                let pitch = self.get_mob_entity().living_entity.get_pitch() * 1.8;
                self.play_entity_sound(Sound::EntityZombifiedPiglinAngry, 2.0, pitch);
            }
        }
    }

    /// Hands the current target to nearby piglins that have none of their own.
    fn maybe_alert_others(&self, target: &Arc<dyn EntityBase>) {
        let remaining = self.ticks_until_next_alert.load(Ordering::Relaxed);
        if remaining > 0 {
            self.ticks_until_next_alert
                .store(remaining - 1, Ordering::Relaxed);
            return;
        }

        if self.has_line_of_sight(target.get_entity()) {
            for other in goal_utils::nearby_same_type(self) {
                let Some(other_mob) = other.get_mob() else {
                    continue;
                };
                if other_mob.get_mob_entity().get_target().is_some()
                    || other.is_allied_to(target.as_ref())
                {
                    continue;
                }
                other_mob.set_mob_target(Some(target.clone()), TargetReason::PigZombieTarget);
            }
        }

        self.ticks_until_next_alert
            .store(rand::random_range(ALERT_INTERVAL), Ordering::Relaxed);
    }

    fn update_attacking_speed(&self) {
        let living = &self.get_mob_entity().living_entity;
        let angry = self.is_angry();
        living.update_attribute(&Attributes::MOVEMENT_SPEED, |speed| {
            let has_modifier = speed
                .modifiers
                .iter()
                .any(|modifier| modifier.id == SPEED_MODIFIER_ATTACKING_ID);
            if angry && !self.entity.is_baby() && !has_modifier {
                speed.add_or_replace_modifier(Modifier {
                    id: SPEED_MODIFIER_ATTACKING_ID.to_string(),
                    amount: SPEED_MODIFIER_ATTACKING,
                    operation: ModifierOperation::Add,
                });
            } else if !angry && has_modifier {
                speed.remove_modifier(SPEED_MODIFIER_ATTACKING_ID);
            }
        });
    }
}

fn add_behaviour_goals(mob_entity: &MobEntity) {
    let goal_selector = &mob_entity.goals_selector;
    goal_selector.add_goal(1, Box::new(SpearUseGoal::new(1.0, 1.0, 10.0, 2.0)));
    goal_selector.add_goal(2, ZombieAttackGoal::new(1.0, false));
    goal_selector.add_goal(7, Box::new(WanderAroundGoal::water_avoiding(1.0)));
    let target_selector = &mob_entity.target_selector;
    target_selector.add_goal(1, Box::new(RevengeGoal::new(true).alerting_others()));
    apply_targets(target_selector, mob_entity, 2, 3, true);
}

impl NeutralMob for ZombifiedPiglinEntity {
    fn get_neutral_data(&self) -> &NeutralData {
        &self.neutral_data
    }

    fn approve_grudge(&self, target: &dyn EntityBase, ticks: i32) -> Option<i32> {
        let entity = self.get_entity();
        let Some(server) = entity.world.load().server.upgrade() else {
            return Some(ticks);
        };
        let mut event =
            PigZombieAngerEvent::new(entity.entity_id, Some(target.get_entity().entity_id), ticks);
        server.plugin_manager.fire_blocking(&server, &mut event);
        (!event.cancelled).then_some(event.new_anger)
    }

    fn updates_anger_in_ai_step(&self) -> bool {
        true
    }
}

impl Mob for ZombifiedPiglinEntity {
    crate::zombie_mob_hooks!(entity);

    fn wants_to_pick_up(&self, stack: &ItemStack) -> bool {
        self.can_hold_item(stack)
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityZombieStep, 0.15, 1.0);
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(if self.is_angry() {
            Sound::EntityZombifiedPiglinAngry
        } else {
            Sound::EntityZombifiedPiglinAmbient
        })
    }

    fn as_neutral(&self) -> Option<&dyn NeutralMob> {
        Some(self)
    }

    fn is_preventing_player_rest(&self, player: &dyn EntityBase) -> bool {
        self.is_angry_at(player, &self.get_entity().world.load())
    }

    fn check_spawn_obstruction(&self, world: &World) -> bool {
        world.is_unobstructed(self.get_entity())
            && !world.contains_any_liquid(self.get_entity().bounding_box.load())
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        self.entity
            .finalize_zombie_spawn(self, world, reason, group_data)
    }

    fn do_hurt_target(&self, target: &dyn EntityBase) -> bool {
        self.entity.do_hurt_target(self, target)
    }

    fn populate_default_equipment_slots(
        &self,
        _world: &Arc<World>,
        _difficulty: &RegionalDifficulty,
    ) {
        let weapon = if rand::random_range(0..GOLDEN_SPEAR_ODDS) == 0 {
            &Item::GOLDEN_SPEAR
        } else {
            &Item::GOLDEN_SWORD
        };
        self.get_mob_entity()
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .put(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, weapon));
    }

    fn apply_mob_target(&self, target: Option<Arc<dyn EntityBase>>) {
        if self.get_mob_entity().get_target().is_none() && target.is_some() {
            self.play_first_anger_sound_in.store(
                rand::random_range(FIRST_ANGER_SOUND_DELAY),
                Ordering::Relaxed,
            );
            self.ticks_until_next_alert
                .store(rand::random_range(ALERT_INTERVAL), Ordering::Relaxed);
        }
        self.get_mob_entity().set_target(target);
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        self.update_attacking_speed();
        if self.is_angry() {
            self.maybe_play_first_anger_sound();
        }
        self.update_persistent_anger();
        if let Some(target) = self.get_mob_entity().get_target() {
            self.maybe_alert_others(&target);
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.entity.write_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.entity.read_nbt(self, nbt);
    }
}
