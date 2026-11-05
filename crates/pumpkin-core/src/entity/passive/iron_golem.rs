use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering},
};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::{BlockDirection, BlockState};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::ActorEventID;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::enchantment::EnchantmentHelper;
use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::crackiness::{Crackiness, CrackinessLevel};
use crate::entity::{
    Entity, EntityBase,
    ai::behavior::neutral::apply_targets,
    ai::goal::{
        defend_village_target::DefendVillageTargetGoal,
        golem_random_stroll_in_village::golem_random_stroll_in_village,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, move_back_to_village::move_back_to_village,
        move_towards_target::MoveTowardsTargetGoal, offer_flower::OfferFlowerGoal,
        revenge::RevengeGoal,
    },
    mob::{
        Mob, MobEntity,
        neutral::{NeutralData, NeutralMob},
    },
    player::Player,
};
use crate::plugin::api::events::entity::entity_target::TargetReason;
use crate::world::{World, natural_spawner::is_valid_empty_spawn_block};

use super::water_animal;

const IRON_INGOT_HEAL_AMOUNT: f32 = 25.0;
const ATTACK_ANIMATION_TICKS: i32 = 10;
const OFFER_FLOWER_TICKS: i32 = 400;
const KNOCKBACK_UP_VELOCITY: f32 = 0.4;
const PUSH_TARGET_CHANCE: u32 = 20;
const STROLL_SPEED: f64 = 0.6;
const AMBIENT_SOUND_INTERVAL: i32 = 120;

/// Represents an Iron Golem, a powerful neutral mob that protects villagers and players.
///
/// Wiki: <https://minecraft.wiki/w/Iron_Golem>
const IRON_GOLEM_HUNTS: &[Hunt] = &[Hunt::new(3, TargetMatch::Enemy, false)
    .every(5)
    .matching(|_hunter, target, _world| target.entity.entity_type != &EntityType::CREEPER)];

pub struct IronGolemEntity {
    pub mob_entity: MobEntity,
    pub neutral_data: NeutralData,
    pub player_created: AtomicBool,
    pub attack_animation_tick: AtomicI32,
    pub offer_flower_tick: AtomicI32,
    crackiness_before_hurt: AtomicU8,
}

impl IronGolemEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let iron_golem = Self {
            mob_entity,
            neutral_data: NeutralData::default(),
            player_created: AtomicBool::new(false),
            attack_animation_tick: AtomicI32::new(0),
            offer_flower_tick: AtomicI32::new(0),
            crackiness_before_hurt: AtomicU8::new(CrackinessLevel::None as u8),
        };
        let mob_arc = Arc::new(iron_golem);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;
            let target_selector = &mob_arc.mob_entity.target_selector;

            goal_selector.add_goal(1, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(2, Box::new(MoveTowardsTargetGoal::new(0.9, 32.0)));
            goal_selector.add_goal(2, Box::new(move_back_to_village(STROLL_SPEED)));
            goal_selector.add_goal(4, Box::new(golem_random_stroll_in_village(STROLL_SPEED)));
            goal_selector.add_goal(5, Box::new(OfferFlowerGoal::new()));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, DefendVillageTargetGoal::new());
            target_selector.add_goal(2, Box::new(RevengeGoal::new(true)));
            apply_targets(target_selector, &mob_arc.mob_entity, 3, 4, false);
            hunting::apply(target_selector, &mob_arc.mob_entity, IRON_GOLEM_HUNTS);
        };

        mob_arc
    }

    #[must_use]
    pub fn is_player_created(&self) -> bool {
        self.player_created.load(Ordering::Relaxed)
    }

    pub fn set_player_created(&self, value: bool) {
        self.player_created.store(value, Ordering::Relaxed);
        let entity = self.get_entity();
        let flag: u8 = u8::from(value);
        entity.set_synced_data(pumpkin_data::tracked_data::iron_golem::FLAGS_ID, flag);
    }

    pub fn offer_flower(&self, offer: bool) {
        let entity = self.get_entity();
        let world = entity.world.load();
        if offer {
            self.offer_flower_tick
                .store(OFFER_FLOWER_TICKS, Ordering::Relaxed);
            world.send_entity_status(
                entity,
                EntityStatus::OfferFlower,
                ActorEventID::StartOfferFlower,
            );
        } else {
            self.offer_flower_tick.store(0, Ordering::Relaxed);
            world.send_entity_status(
                entity,
                EntityStatus::StopOfferFlower,
                ActorEventID::StopOfferFlower,
            );
        }
    }

    #[must_use]
    pub fn get_offer_flower_tick(&self) -> i32 {
        self.offer_flower_tick.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn get_crackiness(&self) -> CrackinessLevel {
        let living = &self.mob_entity.living_entity;
        Crackiness::GOLEM.by_fraction(living.health.load() / living.get_max_health())
    }
}

impl NeutralMob for IronGolemEntity {
    fn updates_anger_in_ai_step(&self) -> bool {
        true
    }

    fn get_neutral_data(&self) -> &NeutralData {
        &self.neutral_data
    }
}

impl Mob for IronGolemEntity {
    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityIronGolemStep, 1.0, 1.0);
    }

    fn can_attack(&self, target: &dyn EntityBase) -> bool {
        let target_type = target.get_entity().entity_type;
        if self.is_player_created() && target_type == &EntityType::PLAYER {
            return false;
        }
        // Vanilla's `super.canAttack` goes through `Mob` crepper and ghast
        target_type != &EntityType::CREEPER
            && target_type != &EntityType::GHAST
            && self.get_mob_entity().living_entity.can_attack(target)
    }

    fn as_neutral(&self) -> Option<&dyn NeutralMob> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("PlayerCreated", self.is_player_created());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(created) = nbt.get_bool("PlayerCreated") {
            self.set_player_created(created);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn as_iron_golem(&self) -> Option<&crate::entity::passive::iron_golem::IronGolemEntity> {
        Some(self)
    }

    fn remove_when_far_away(&self, _distance_sq: f64) -> bool {
        false
    }

    fn get_ambient_sound_interval(&self) -> i32 {
        AMBIENT_SOUND_INTERVAL
    }

    fn mob_decrease_air_supply(&self, current: i32) -> i32 {
        current
    }

    fn do_push(&self, entity: &Arc<dyn EntityBase>) {
        let entity_type = entity.get_entity().entity_type;
        if TargetMatch::Enemy.matches(entity_type)
            && entity_type != &EntityType::CREEPER
            && self.get_random().random_range(0..PUSH_TARGET_CHANCE) == 0
        {
            self.set_mob_target(Some(entity.clone()), TargetReason::Unknown);
        }
    }

    fn mob_ai_step(&self) {
        let attack_tick = self.attack_animation_tick.load(Ordering::Relaxed);
        if attack_tick > 0 {
            self.attack_animation_tick.fetch_sub(1, Ordering::Relaxed);
        }

        let flower_tick = self.offer_flower_tick.load(Ordering::Relaxed);
        if flower_tick > 0 {
            self.offer_flower_tick.fetch_sub(1, Ordering::Relaxed);
        }

        self.update_persistent_anger();
    }

    fn do_hurt_target(&self, target: &dyn EntityBase) -> bool {
        let entity = self.get_entity();
        self.attack_animation_tick
            .store(ATTACK_ANIMATION_TICKS, Ordering::Relaxed);
        entity.world.load().send_entity_status(
            entity,
            EntityStatus::StartAttacking,
            ActorEventID::StartAttacking,
        );
        let attack_damage = self
            .mob_entity
            .living_entity
            .get_attribute_value(&Attributes::ATTACK_DAMAGE) as f32;
        let damage = if attack_damage as i32 > 0 {
            attack_damage / 2.0 + self.get_random().random_range(0..attack_damage as i32) as f32
        } else {
            attack_damage
        };
        let hurt = target.damage_with_context(
            target,
            damage,
            DamageType::MOB_ATTACK,
            None,
            Some(entity),
            Some(entity),
        );
        if hurt {
            let knockback_resistance = target.get_living_entity().map_or(0.0, |living| {
                living.get_attribute_value(&Attributes::KNOCKBACK_RESISTANCE)
            });
            let scale = (1.0 - knockback_resistance).max(0.0);
            target.get_entity().add_velocity(Vector3::new(
                0.0,
                f64::from(KNOCKBACK_UP_VELOCITY) * scale,
                0.0,
            ));
            EnchantmentHelper::on_post_attack(
                target.get_entity(),
                &self.mob_entity.living_entity.held_item(self),
            );
        }
        self.play_entity_sound(Sound::EntityIronGolemAttack, 1.0, 1.0);
        hurt
    }

    fn pre_damage_with_context(
        &self,
        _damage_type: DamageType,
        _source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        self.crackiness_before_hurt
            .store(self.get_crackiness() as u8, Ordering::Relaxed);
        true
    }

    fn on_damage_with_context(
        &self,
        _damage_type: DamageType,
        _source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        if self.get_crackiness() as u8 != self.crackiness_before_hurt.load(Ordering::Relaxed) {
            self.play_entity_sound(Sound::EntityIronGolemDamage, 1.0, 1.0);
        }
        true
    }

    fn check_spawn_obstruction(&self, world: &World) -> bool {
        let pos = self.get_entity().block_pos.load();
        if !world
            .get_block_state(&pos.down())
            .is_side_solid(BlockDirection::Up)
        {
            return false;
        }
        let entity_type = &EntityType::IRON_GOLEM;
        [pos, pos.up(), pos.up().up()]
            .iter()
            .all(|pos| is_valid_empty_spawn_block(world.get_block_state(pos), entity_type))
            && water_animal::check_spawn_obstruction(self, world)
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let flag: u8 = u8::from(self.is_player_created());
        entity.set_synced_data(pumpkin_data::tracked_data::iron_golem::FLAGS_ID, flag);
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if item_stack.item != &Item::IRON_INGOT {
            return false;
        }
        let living = &self.mob_entity.living_entity;
        let health_before = living.health.load();
        living.heal(IRON_INGOT_HEAL_AMOUNT);
        if living.health.load() == health_before {
            return false;
        }
        let pitch = {
            let mut random = self.get_random();
            1.0 + (random.random::<f32>() - random.random::<f32>()) * 0.2
        };
        self.play_entity_sound(Sound::EntityIronGolemRepair, 1.0, pitch);
        item_stack.decrement_unless_creative(player.gamemode.load(), 1);
        true
    }
}
