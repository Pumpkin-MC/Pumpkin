use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering, Ordering::Relaxed};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        escape_danger::EscapeDangerGoal,
        trader_llama_defend_wandering_trader::TraderLlamaDefendWanderingTraderGoal,
    },
    mob::{Mob, MobEntity},
    passive::{
        animal::Animal,
        equine::{AbstractChestedHorse, AbstractHorse, AbstractHorseData, ChestedHorseData},
        llama::{LlamaData, LlamaMob, register_llama_goals, set_random_strength},
        wandering_trader::WanderingTraderEntity,
    },
    player::Player,
};

/// Vanilla `TraderLlama::despawnDelay` default (`TraderLlama.java:28-29`).
const DEFAULT_DESPAWN_DELAY: i32 = 47999;

/// `TraderLlama.java`, layered on the same `LlamaMob`/`AbstractChestedHorse`/`AbstractHorse`
/// stack `LlamaEntity` uses (vanilla `TraderLlama extends Llama`).
///
/// `doPlayerRide`'s mount-block override (`TraderLlama.java:75-81`) is not implemented: Pumpkin
/// has no generic animal-mounting/riding system to attach it to. `finalizeSpawn`'s spawn-reason
/// age-forcing (`TraderLlama.java:117-130`) is also not ported (no `EntitySpawnReason` concept
/// confirmed in this pass).
pub struct TraderLlamaEntity {
    pub mob_entity: MobEntity,
    pub horse_data: AbstractHorseData,
    pub chested_data: ChestedHorseData,
    pub llama_data: LlamaData,
    /// Vanilla `despawnDelay` (`TraderLlama.java:28-29, 71-73`).
    pub despawn_delay: AtomicI32,
}

impl TraderLlamaEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let llama = Self {
            mob_entity,
            horse_data: AbstractHorseData::default(),
            chested_data: ChestedHorseData::default(),
            llama_data: LlamaData::default(),
            despawn_delay: AtomicI32::new(DEFAULT_DESPAWN_DELAY),
        };
        let mob_arc = Arc::new(llama);
        AbstractHorse::randomize_attributes(mob_arc.as_ref(), &mut rand::rng());
        set_random_strength(&mob_arc.llama_data, &mut rand::rng());
        mob_arc
            .llama_data
            .variant
            .store(rand::random_range(0..4), Relaxed);

        let dyn_mob: Arc<dyn Mob> = mob_arc.clone();
        let mob_weak = Arc::downgrade(&dyn_mob);
        register_llama_goals(&dyn_mob, mob_weak);

        // `TraderLlama.registerGoals` (`TraderLlama.java:63-68`): an extra, faster
        // `PanicGoal` copy on top of the one `Llama.registerGoals` already adds, plus the
        // wandering-trader-defense target goal. The zombie/`AbstractIllager`
        // `NearestAttackableTargetGoal`s from the same method target a multi-type class
        // predicate that doesn't fit `ActiveTargetGoal`'s single-`EntityType` shape and are
        // out of scope here.
        mob_arc
            .mob_entity
            .goals_selector
            .lock()
            .unwrap()
            .add_goal(1, EscapeDangerGoal::new(2.0));
        mob_arc
            .mob_entity
            .target_selector
            .lock()
            .unwrap()
            .add_goal(1, TraderLlamaDefendWanderingTraderGoal::new());

        mob_arc
    }
}

impl NBTStorage for TraderLlamaEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            self.write_animal_nbt(nbt);
            self.write_horse_nbt(nbt);
            self.write_chested_horse_nbt(nbt);
            self.write_llama_nbt(nbt);
            nbt.put_int("DespawnDelay", self.despawn_delay.load(Ordering::Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.read_animal_nbt(nbt);
            self.read_llama_strength_variant(nbt);
            self.read_horse_nbt(nbt);
            self.read_chested_horse_nbt(nbt).await;
            if let Some(delay) = nbt.get_int("DespawnDelay") {
                self.despawn_delay.store(delay, Ordering::Relaxed);
            }
        })
    }
}

impl Animal for TraderLlamaEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_LLAMA_FOOD)
    }
}

impl AbstractHorse for TraderLlamaEntity {
    fn horse_data(&self) -> &AbstractHorseData {
        &self.horse_data
    }

    /// `AbstractChestedHorse.randomizeAttributes`: only max-health is rolled.
    fn randomize_attributes(&self, random: &mut impl RngExt)
    where
        Self: Sized,
    {
        crate::entity::passive::llama::randomize_llama_max_health(self, random);
    }

    fn max_temper(&self) -> i32 {
        30
    }

    fn can_perform_rearing(&self) -> bool {
        false
    }

    fn angry_sound(&self) -> Option<Sound> {
        Some(Sound::EntityLlamaAngry)
    }

    fn eating_sound(&self) -> Option<Sound> {
        Some(Sound::EntityLlamaEat)
    }

    fn handle_eating<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        self.handle_llama_eating(player, item_stack)
    }
}

impl AbstractChestedHorse for TraderLlamaEntity {
    fn chested_data(&self) -> &ChestedHorseData {
        &self.chested_data
    }

    fn get_inventory_columns(&self) -> u8 {
        if self.has_chest() {
            self.llama_data.strength.load(Relaxed)
        } else {
            0
        }
    }

    fn play_chest_equips_sound(&self) {
        let entity = self.get_entity();
        let world = entity.world.load();
        world.play_sound(
            Sound::EntityLlamaChest,
            pumpkin_data::sound::SoundCategory::Neutral,
            &entity.pos.load(),
        );
    }
}

impl LlamaMob for TraderLlamaEntity {
    fn llama_data(&self) -> &LlamaData {
        &self.llama_data
    }
}

impl Mob for TraderLlamaEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        self.chested_mob_interact(player, item_stack)
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            crate::entity::passive::llama::send_baby_id_if_baby(self.get_entity());
            self.send_llama_metadata();
        })
    }

    fn create_offspring<'a>(
        &'a self,
        mate: &'a dyn EntityBase,
        world: &'a Arc<crate::world::World>,
    ) -> EntityBaseFuture<'a, Option<Arc<dyn EntityBase>>> {
        self.create_llama_offspring(mate, world)
    }

    /// Vanilla `TraderLlama::maybeDespawn` (`TraderLlama.java:91-99`). `canDespawn`
    /// (`TraderLlama.java:101-107`) now checks `is_tamed` (available since `Llama` gained real
    /// `AbstractHorse` taming) but still can't check `hasExactlyOnePlayerPassenger`, `isAgeLocked`
    /// or `isPersistenceRequired` -- none of those concepts exist on any Pumpkin entity yet, a
    /// pre-existing gap in the despawn system generally, not introduced here. While leashed to a
    /// `WanderingTrader`, the countdown slaves itself to the trader's own `despawn_delay` minus
    /// one every tick (`TraderLlama.java:93-95`) so it automatically tracks any reset/extension of
    /// the trader's timer; otherwise it decrements independently.
    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.mob_entity.living_entity.entity;
            let holder = entity.leashed_to.lock().await.clone();

            let leashed_to_other = holder.is_some()
                && holder
                    .as_ref()
                    .and_then(|h| h.cast_any().downcast_ref::<WanderingTraderEntity>())
                    .is_none();

            let can_despawn = !self.is_tamed() && !leashed_to_other;
            if !can_despawn {
                return;
            }

            let trader = holder
                .as_ref()
                .and_then(|h| h.cast_any().downcast_ref::<WanderingTraderEntity>());

            let new_delay = trader.map_or_else(
                || self.despawn_delay.load(Ordering::Relaxed) - 1,
                |trader| trader.despawn_delay.load(Ordering::Relaxed) - 1,
            );
            self.despawn_delay.store(new_delay, Ordering::Relaxed);

            if new_delay <= 0 {
                entity.unleash().await;
                let world = entity.world.load();
                world.remove_entity(self).await;
            }
        })
    }
}
