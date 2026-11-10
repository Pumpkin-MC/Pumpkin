use std::sync::atomic::{AtomicI32, Ordering::Relaxed};
use std::sync::{Arc, Weak};

use pumpkin_data::BlockState;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{DataComponentImpl, EquipmentSlot};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::ageable::AgeableMob;
use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::goal::escape_danger::EscapeDangerGoal;
use crate::entity::ai::goal::track_target::TrackTargetGoal;
use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::custom_sound::CustomSound;
use crate::entity::mob::spawn::SpawnGroupData;
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::passive::abstract_horse::AbstractHorse;
use crate::entity::passive::animal::Animal;
use crate::entity::passive::llama::LlamaEntity;
use crate::entity::passive::wandering_trader::WanderingTraderEntity;
use crate::entity::player::Player;
use crate::plugin::api::events::entity::entity_target::TargetReason;
use crate::world::World;

const TRADER_LLAMA_HUNTS: &[Hunt] = &[
    Hunt::new(2, TargetMatch::ZOMBIE_LIKE, true).matching(|_hunter, target, _world| {
        target.entity.entity_type != &EntityType::ZOMBIFIED_PIGLIN
    }),
    Hunt::new(2, TargetMatch::ILLAGER, true),
];

const DEFAULT_DESPAWN_DELAY: i32 = 47999;

pub struct TraderLlamaEntity {
    pub llama: Arc<LlamaEntity>,
    despawn_delay: AtomicI32,
}

impl TraderLlamaEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let llama = LlamaEntity::new(entity);
        llama
            .mob_entity
            .goals_selector
            .add_goal(1, EscapeDangerGoal::new(2.0));
        llama
            .mob_entity
            .target_selector
            .add_goal(1, Box::new(DefendWanderingTraderGoal::new()));
        hunting::apply(
            &llama.mob_entity.target_selector,
            &llama.mob_entity,
            TRADER_LLAMA_HUNTS,
        );
        Arc::new(Self {
            llama,
            despawn_delay: AtomicI32::new(DEFAULT_DESPAWN_DELAY),
        })
    }

    pub fn set_despawn_delay(&self, despawn_delay: i32) {
        self.despawn_delay.store(despawn_delay, Relaxed);
    }

    fn maybe_despawn(&self) {
        if !self.can_despawn() {
            return;
        }
        let entity = self.get_entity();
        let trader_delay = wandering_trader_holder(entity).and_then(|holder| {
            holder
                .cast_any()
                .downcast_ref::<WanderingTraderEntity>()
                .map(WanderingTraderEntity::get_despawn_delay)
        });
        let despawn_delay = trader_delay.unwrap_or_else(|| self.despawn_delay.load(Relaxed)) - 1;
        self.despawn_delay.store(despawn_delay, Relaxed);
        if despawn_delay <= 0 {
            entity.remove_leash();
            entity.remove();
        }
    }

    fn can_despawn(&self) -> bool {
        let entity = self.get_entity();
        let leashed_to_other = entity.is_leashed() && wandering_trader_holder(entity).is_none();
        let one_player_passenger = {
            let passengers = entity
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            passengers.len() == 1 && passengers[0].get_player().is_some()
        };
        !self.llama.is_tamed()
            && !leashed_to_other
            && !one_player_passenger
            && !self.llama.is_age_locked()
            && !self.llama.mob_entity.persistence_required.load(Relaxed)
    }
}

fn wandering_trader_holder(entity: &Entity) -> Option<Arc<dyn EntityBase>> {
    entity.get_leash_holder().filter(|holder| {
        holder
            .cast_any()
            .downcast_ref::<WanderingTraderEntity>()
            .is_some()
    })
}

/// Targets whatever last hurt the wandering trader holding this llama's leash.
struct DefendWanderingTraderGoal {
    track_target_goal: TrackTargetGoal,
    target: Option<Weak<dyn EntityBase>>,
    target_predicate: TargetPredicate,
    timestamp: i32,
}

impl DefendWanderingTraderGoal {
    fn new() -> Self {
        Self {
            track_target_goal: TrackTargetGoal::with_default(false),
            target: None,
            target_predicate: TargetPredicate::create_attackable(),
            timestamp: 0,
        }
    }
}

impl Goal for DefendWanderingTraderGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(owner) = wandering_trader_holder(mob.get_entity()) else {
            return false;
        };
        let Some(owner_living) = owner.get_living_entity() else {
            return false;
        };
        let timestamp = owner_living.get_last_hurt_by_mob_timestamp();
        let Some(attacker) = owner_living.get_last_hurt_by_mob() else {
            return false;
        };
        if timestamp == self.timestamp
            || !self.track_target_goal.can_track(
                mob,
                Some(attacker.as_ref()),
                &self.target_predicate,
            )
        {
            return false;
        }
        self.target = Some(Arc::downgrade(&attacker));
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.track_target_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.track_target_goal.start_targeting(
            mob,
            self.target.as_ref().and_then(Weak::upgrade),
            TargetReason::TargetAttackedOwner,
        );
        if let Some(owner) = wandering_trader_holder(mob.get_entity())
            && let Some(owner_living) = owner.get_living_entity()
        {
            self.timestamp = owner_living.get_last_hurt_by_mob_timestamp();
        }
        self.track_target_goal.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.target = None;
        self.track_target_goal.stop(mob);
    }

    fn controls(&self) -> Controls {
        self.track_target_goal.controls()
    }
}

impl Mob for TraderLlamaEntity {
    fn on_climbable(&self) -> bool {
        false
    }

    fn on_elastic_leash_pull(&self) {
        self.llama.on_elastic_leash_pull();
    }

    fn cause_fall_damage(
        &self,
        caller: &dyn EntityBase,
        fall_distance: f32,
        damage_modifier: f32,
    ) -> bool {
        self.llama
            .cause_fall_damage(caller, fall_distance, damage_modifier)
    }

    fn mob_is_pushable(&self) -> bool {
        self.llama.mob_is_pushable()
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.llama.mob_entity
    }

    fn is_tamed(&self) -> bool {
        self.llama.is_tamed()
    }

    fn get_owner_uuid(&self) -> Option<uuid::Uuid> {
        self.llama.get_owner_uuid()
    }

    fn follow_leash_speed(&self) -> f64 {
        self.llama.follow_leash_speed()
    }

    fn can_dispenser_equip_into_slot(&self, slot: &EquipmentSlot) -> bool {
        self.llama.can_dispenser_equip_into_slot(slot)
    }

    fn is_mob_immobile(&self) -> bool {
        self.llama.is_mob_immobile()
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        self.llama.get_ambient_sound()
    }

    fn get_ambient_sound_interval(&self) -> i32 {
        self.llama.get_ambient_sound_interval()
    }

    fn mob_play_step_sound(&self, pos: &BlockPos, state: &BlockState) {
        self.llama.mob_play_step_sound(pos, state);
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(&*self.llama)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(&*self.llama)
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(&*self.llama)
    }

    fn as_abstract_horse(&self) -> Option<&dyn AbstractHorse> {
        Some(&*self.llama)
    }

    fn drop_inventory(&self) {
        self.llama.drop_inventory();
    }

    fn tick_links(&self) -> Vec<i32> {
        self.llama.tick_links()
    }

    fn mob_ai_step(&self) {
        self.llama.mob_ai_step();
        self.maybe_despawn();
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        reason: crate::entity::mob::spawn::EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        if reason == crate::entity::mob::spawn::EntitySpawnReason::Event {
            self.llama.set_age(0);
        }
        self.llama.finalize_spawn(world, reason, group_data)
    }

    fn mob_get_component(&self, component: DataComponent) -> Option<Box<dyn DataComponentImpl>> {
        self.llama.mob_get_component(component)
    }

    fn mob_set_variant_name(&self, name: &str) {
        self.llama.mob_set_variant_name(name);
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.llama.mob_write_nbt(nbt);
        nbt.put_int("DespawnDelay", self.despawn_delay.load(Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.llama.mob_read_nbt(nbt);
        self.set_despawn_delay(nbt.get_int("DespawnDelay").unwrap_or(DEFAULT_DESPAWN_DELAY));
    }

    fn mob_init_data_tracker(&self) {
        self.llama.mob_init_data_tracker();
    }

    fn mob_bedrock_spawn_metadata(
        &self,
    ) -> Option<pumpkin_protocol::bedrock::client::set_actor_data::SyncedActorDataList> {
        self.llama.mob_bedrock_spawn_metadata()
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.llama.mob_interact(player, item_stack)
    }
}
