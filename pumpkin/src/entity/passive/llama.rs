use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering::Relaxed};
use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, breed::BreedGoal, escape_danger::EscapeDangerGoal,
        follow_parent::FollowParentGoal, llama_follow_caravan::LlamaFollowCaravanGoal,
        llama_hurt_by_target::LlamaHurtByTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, ranged_llama_spit_attack::RangedLlamaSpitAttackGoal,
        swim::SwimGoal, tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::{
        animal::Animal,
        equine::{AbstractChestedHorse, AbstractHorse, AbstractHorseData, ChestedHorseData},
    },
    player::Player,
};

/// Vanilla `TemptGoal(this, 1.25, i -> i.is(ItemTags.LLAMA_TEMPT_ITEMS), false)` -- hay bales
/// only, not the wider `LLAMA_FOOD` tag wheat also belongs to.
const TEMPT_ITEMS: &[&Item] = &[&Item::HAY_BLOCK];

const MAX_STRENGTH: u8 = 5;
const LLAMA_MAX_TEMPER: i32 = 30;
/// `AbstractHorse.MIN_HEALTH`/`MAX_HEALTH` bounds shared with Donkey/Mule.
const MIN_HEALTH: f64 = 15.0;
const MAX_HEALTH: f64 = 30.0;

/// `AbstractChestedHorse.randomizeAttributes`: only max-health is rolled, shared by
/// `LlamaEntity` and `TraderLlamaEntity`.
pub fn randomize_llama_max_health(mob: &dyn Mob, random: &mut impl RngExt) {
    let living = &mob.get_mob_entity().living_entity;
    let mut attrs = living.attributes.write().unwrap();
    if let Some(a) = attrs.get_mut(&pumpkin_data::attributes::Attributes::MAX_HEALTH.id) {
        a.base_value = crate::entity::passive::equine::generate_max_health(random);
        a.dirty.store(true, Relaxed);
    }
    drop(attrs);
    let max_health = living.get_max_health();
    living.health.store(max_health);
}

/// Re-sends `TrackedData::BABY_ID`.
///
/// This is the blanket `Mob` `EntityBase` impl's default `init_data_tracker` behavior
/// (`mob/mod.rs`), which a species-specific `mob_init_data_tracker` override otherwise silently
/// drops -- the same reason `CatEntity::mob_init_data_tracker` re-sends it manually.
pub fn send_baby_id_if_baby(entity: &Entity) {
    if entity.age.load(Relaxed) < 0 {
        entity.send_meta_data(
            &[Metadata::new(
                TrackedData::BABY_ID,
                MetaDataType::BOOLEAN,
                true,
            )],
            None,
        );
    }
}

/// Shared per-instance llama state (`Llama.java` fields not already covered by `AbstractHorse`).
///
/// `AbstractChestedHorse`'s state is separate; `TraderLlama extends Llama` in vanilla and inherits
/// all of it unchanged, so `TraderLlamaEntity` embeds this same struct rather than duplicating
/// the fields.
pub struct LlamaData {
    pub strength: AtomicU8,
    pub variant: AtomicU8,
    /// `Llama.didSpit` (`Llama.java:76`): set by `spit()`, consumed once by
    /// `LlamaHurtByTargetGoal::should_continue` to avoid double-targeting off a spit hit.
    pub did_spit: AtomicBool,
    /// Entity id of the llama this one is following (vanilla `caravanHead`); -1 = none.
    pub caravan_head_id: AtomicI32,
    /// Entity id of the llama following this one (vanilla `caravanTail`); -1 = none.
    pub caravan_tail_id: AtomicI32,
}

impl Default for LlamaData {
    fn default() -> Self {
        Self {
            strength: AtomicU8::new(1),
            variant: AtomicU8::new(0),
            did_spit: AtomicBool::new(false),
            caravan_head_id: AtomicI32::new(-1),
            caravan_tail_id: AtomicI32::new(-1),
        }
    }
}

impl LlamaData {
    #[must_use]
    pub fn in_caravan(&self) -> bool {
        self.caravan_head_id.load(Relaxed) != -1
    }

    #[must_use]
    pub fn has_caravan_tail(&self) -> bool {
        self.caravan_tail_id.load(Relaxed) != -1
    }
}

/// Reaches the shared `LlamaData` embedded in either `LlamaEntity` or `TraderLlamaEntity`.
///
/// Takes a generic `dyn EntityBase` handle -- the shape goal code (caravan-chain walking, the
/// spit attack goal) needs since the world only ever hands back type-erased entity references.
#[must_use]
pub fn llama_data_of(entity: &dyn EntityBase) -> Option<&LlamaData> {
    if let Some(llama) = entity.cast_any().downcast_ref::<LlamaEntity>() {
        return Some(&llama.llama_data);
    }
    if let Some(trader) = entity
        .cast_any()
        .downcast_ref::<crate::entity::passive::trader_llama::TraderLlamaEntity>()
    {
        return Some(&trader.llama_data);
    }
    None
}

/// `Llama.setRandomStrength` (`Llama.java:93-96`): normally 1-3, a 4% chance of 1-5.
pub fn set_random_strength(data: &LlamaData, random: &mut impl RngExt) {
    let max_strength = if random.random::<f32>() < 0.04 { 5 } else { 3 };
    let strength = 1 + random.random_range(0..max_strength);
    data.strength.store(strength, Relaxed);
}

/// Registers the goals `Llama.registerGoals` (`Llama.java:117-131`) adds.
///
/// On top of the base set every `AbstractHorse`-family species shares. Shared by `LlamaEntity`
/// and `TraderLlamaEntity` (`TraderLlama.registerGoals` calls `super.registerGoals()` first) so
/// the exact same priorities and goal instances back both.
pub fn register_llama_goals(mob_arc: &Arc<dyn Mob>, mob_weak: Weak<dyn Mob>) {
    let mut goal_selector = mob_arc.get_mob_entity().goals_selector.lock().unwrap();
    goal_selector.add_goal(0, Box::new(SwimGoal::default()));
    // Vanilla priority 1 is `RunAroundLikeCrazyGoal(this, 1.2)` (`Llama.java:119`) -- a
    // frenzied, undirected flee that's distinct from `EscapeDangerGoal`'s "path to a random
    // point" behavior and has no existing Pumpkin equivalent; left unported (documented gap)
    // rather than approximated with the wrong goal shape.
    goal_selector.add_goal(2, LlamaFollowCaravanGoal::new(2.1));
    goal_selector.add_goal(3, RangedLlamaSpitAttackGoal::new(1.25, 40, 20.0));
    goal_selector.add_goal(3, EscapeDangerGoal::new(1.2));
    goal_selector.add_goal(4, BreedGoal::new(1.0));
    goal_selector.add_goal(5, Box::new(TemptGoal::new(1.25, TEMPT_ITEMS, false)));
    goal_selector.add_goal(6, Box::new(FollowParentGoal::new(1.0)));
    goal_selector.add_goal(7, Box::new(WanderAroundGoal::new(0.7)));
    goal_selector.add_goal(
        8,
        LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
    );
    goal_selector.add_goal(9, Box::new(RandomLookAroundGoal::default()));
    drop(goal_selector);

    let mut target_selector = mob_arc.get_mob_entity().target_selector.lock().unwrap();
    target_selector.add_goal(1, LlamaHurtByTargetGoal::new());
    // `Llama.LlamaAttackWolfGoal` (`Llama.java:454-463`): a 16-random-interval, no-sight,
    // must-navigate `NearestAttackableTargetGoal<Wolf>` restricted to non-tame wolves. The
    // 25%-follow-distance override (`getFollowDistance`) isn't expressible through
    // `ActiveTargetGoal` (it always uses the mob's `FOLLOW_RANGE` attribute) and is dropped as
    // a documented gap.
    target_selector.add_goal(
        2,
        Box::new(ActiveTargetGoal::new(
            mob_arc.get_mob_entity(),
            &EntityType::WOLF,
            16,
            false,
            true,
            Some(
                |target: Arc<crate::entity::living::LivingEntity>,
                 world: Arc<crate::world::World>| async move {
                    let id = target.entity.entity_id;
                    world
                        .get_entity_by_id(id)
                        .is_none_or(|e| e.get_mob().is_none_or(|m| !m.get_mob_entity().is_tamed()))
                },
            ),
        )),
    );
}

/// Shared vanilla `Llama` behavior (`Llama.java`).
///
/// Implemented by both `LlamaEntity` and `TraderLlamaEntity` since vanilla `TraderLlama extends
/// Llama` and inherits all of it unchanged. Rust has no class inheritance, so this trait plays
/// the role of that shared superclass slice -- species implement it to get the food table, save
/// data and breeding logic for free, the same pattern `AbstractHorse`/`AbstractChestedHorse`
/// already use one level up.
pub trait LlamaMob: AbstractChestedHorse {
    fn llama_data(&self) -> &LlamaData;

    /// `Llama.addAdditionalSaveData` (`Llama.java:103-107`).
    fn write_llama_nbt(&self, nbt: &mut NbtCompound) {
        let data = self.llama_data();
        nbt.put_int("Variant", i32::from(data.variant.load(Relaxed)));
        nbt.put_int("Strength", i32::from(data.strength.load(Relaxed)));
    }

    /// `Llama.readAdditionalSaveData`'s `Strength`/`Variant` reads (`Llama.java:110-114`), split
    /// out so it can run before the super-chain's chest sizing (see the module doc on
    /// `LlamaEntity::write_nbt`/`read_nbt_non_mut` for why the read order matters).
    fn read_llama_strength_variant(&self, nbt: &NbtCompound) {
        let data = self.llama_data();
        let strength = nbt
            .get_int("Strength")
            .unwrap_or(0)
            .clamp(1, i32::from(MAX_STRENGTH));
        data.strength.store(strength as u8, Relaxed);
        let variant = nbt.get_int("Variant").unwrap_or(0).clamp(0, 3);
        data.variant.store(variant as u8, Relaxed);
    }

    /// Sends the strength/variant synced data (`Llama.defineSynchedData`, `Llama.java:138-142`).
    fn send_llama_metadata(&self) {
        let data = self.llama_data();
        self.get_entity().send_meta_data(
            &[
                Metadata::new(
                    TrackedData::STRENGTH_ID,
                    MetaDataType::INT,
                    VarInt(i32::from(data.strength.load(Relaxed))),
                ),
                Metadata::new(
                    TrackedData::VARIANT_ID,
                    MetaDataType::INT,
                    VarInt(i32::from(data.variant.load(Relaxed))),
                ),
            ],
            None,
        );
    }

    /// `Llama.handleEating` (`Llama.java:178-234`): llama has its own wheat/hay-block food table
    /// instead of `AbstractHorse`'s default.
    fn handle_llama_eating<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a ItemStack,
    ) -> EntityBaseFuture<'a, bool>
    where
        Self: Sized,
    {
        Box::pin(async move {
            let id = item_stack.item.id;
            let (age_up_seconds, temper, heal): (i32, i32, f32) = if id == Item::WHEAT.id {
                (10, 3, 2.0)
            } else if id == Item::HAY_BLOCK.id {
                (90, 6, 10.0)
            } else {
                return false;
            };

            let mob_entity = self.get_mob_entity();
            let mut item_used = false;

            if id == Item::HAY_BLOCK.id
                && self.is_tamed()
                && !self.is_baby()
                && self.can_fall_in_love()
                && !mob_entity.is_in_love()
            {
                item_used = true;
                mob_entity.set_love_ticks(600, Some(player.gameprofile.id));
            }

            let living = &mob_entity.living_entity;
            if living.health.load() < living.get_max_health() && heal > 0.0 {
                living.heal(heal);
                item_used = true;
            }

            if self.is_baby() && age_up_seconds > 0 {
                let entity = &living.entity;
                let world = entity.world.load();
                let pos = entity.pos.load();
                world.spawn_particle(
                    pos + Vector3::new(0.0, f64::from(entity.height()) * 0.5, 0.0),
                    Vector3::new(0.5, 0.5, 0.5),
                    1.0,
                    7,
                    Particle::HappyVillager,
                );
                let new_age = (entity.age.load(Relaxed) + age_up_seconds * 20).min(0);
                entity.age.store(new_age, Relaxed);
                item_used = true;
            }

            if temper > 0
                && (item_used || !self.is_tamed())
                && self.horse_data().temper.load(Relaxed) < self.max_temper()
            {
                let new_temper =
                    (self.horse_data().temper.load(Relaxed) + temper).clamp(0, self.max_temper());
                self.horse_data().temper.store(new_temper, Relaxed);
                item_used = true;
            }

            item_used
        })
    }

    /// `Llama.getBreedOffspring` (`Llama.java:319-334`): strength rolls `rnd(max(a,b)) + 1`, a 3%
    /// chance of one extra point, and variant is a coin-flip between the parents. `makeNewLlama`
    /// (which `TraderLlama` overrides to spawn a `TraderLlama` and mark it persistence-required)
    /// is folded into `create_offspring`'s generic `from_type` species lookup, which already
    /// spawns the same concrete entity type as `self`.
    fn create_llama_offspring<'a>(
        &'a self,
        mate: &'a dyn EntityBase,
        world: &'a Arc<crate::world::World>,
    ) -> EntityBaseFuture<'a, Option<Arc<dyn EntityBase>>>
    where
        Self: Sized,
    {
        Box::pin(async move {
            let entity = self.get_entity();
            let baby = crate::entity::r#type::from_type(
                entity.entity_type,
                entity.pos.load(),
                world,
                Uuid::new_v4(),
            );

            let (Some(mate_data), Some(baby_data)) = (
                llama_data_of(mate),
                baby.get_mob()
                    .and_then(|m| llama_data_of(m as &dyn EntityBase)),
            ) else {
                return Some(baby);
            };

            let mut random = rand::rng();

            // `AbstractHorse.setOffspringAttributes` (`Llama.java:322`, called before the
            // strength/variant rolls).
            if let Some(baby_mob) = baby.get_mob() {
                let mate_max_health = mate.get_mob().map_or(MIN_HEALTH, |m| {
                    m.get_mob_entity()
                        .living_entity
                        .get_attribute_base(&pumpkin_data::attributes::Attributes::MAX_HEALTH)
                });
                crate::entity::passive::equine::apply_offspring_attribute(
                    baby_mob,
                    &pumpkin_data::attributes::Attributes::MAX_HEALTH,
                    self.get_mob_entity()
                        .living_entity
                        .get_attribute_base(&pumpkin_data::attributes::Attributes::MAX_HEALTH),
                    mate_max_health,
                    MIN_HEALTH,
                    MAX_HEALTH,
                    &mut random,
                );
            }

            let a = self.llama_data().strength.load(Relaxed);
            let b = mate_data.strength.load(Relaxed);
            let mut strength = 1 + random.random_range(0..a.max(b));
            if random.random::<f32>() < 0.03 {
                strength += 1;
            }
            baby_data
                .strength
                .store(strength.min(MAX_STRENGTH), Relaxed);

            let picked_variant = if random.random::<bool>() {
                self.llama_data().variant.load(Relaxed)
            } else {
                mate_data.variant.load(Relaxed)
            };
            baby_data.variant.store(picked_variant, Relaxed);

            Some(baby)
        })
    }
}

/// Represents a Llama, a neutral mob that can carry a chest, form caravans on a lead, and spits at
/// enemies. `Llama.java`.
///
/// Wiki: <https://minecraft.wiki/w/Llama>
pub struct LlamaEntity {
    pub mob_entity: MobEntity,
    pub horse_data: AbstractHorseData,
    pub chested_data: ChestedHorseData,
    pub llama_data: LlamaData,
}

impl LlamaEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let llama = Self {
            mob_entity,
            horse_data: AbstractHorseData::default(),
            chested_data: ChestedHorseData::default(),
            llama_data: LlamaData::default(),
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

        mob_arc
    }
}

impl NBTStorage for LlamaEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            self.write_animal_nbt(nbt);
            self.write_horse_nbt(nbt);
            self.write_chested_horse_nbt(nbt);
            self.write_llama_nbt(nbt);
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.read_animal_nbt(nbt);
            // Vanilla reads `Strength` before `super.readAdditionalSaveData` on purpose: the
            // super chain's chest handling sizes the inventory off `getInventoryColumns()`, which
            // for a llama is strength-dependent (`Llama.java:110-114`).
            self.read_llama_strength_variant(nbt);
            self.read_horse_nbt(nbt);
            self.read_chested_horse_nbt(nbt).await;
        })
    }
}

impl Animal for LlamaEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_LLAMA_FOOD)
    }
}

impl AbstractHorse for LlamaEntity {
    fn horse_data(&self) -> &AbstractHorseData {
        &self.horse_data
    }

    /// `AbstractChestedHorse.randomizeAttributes`: only max-health is rolled.
    fn randomize_attributes(&self, random: &mut impl RngExt)
    where
        Self: Sized,
    {
        randomize_llama_max_health(self, random);
    }

    fn max_temper(&self) -> i32 {
        LLAMA_MAX_TEMPER
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

impl AbstractChestedHorse for LlamaEntity {
    fn chested_data(&self) -> &ChestedHorseData {
        &self.chested_data
    }

    /// `Llama.getInventoryColumns`: strength-based, not the flat 5 columns Donkey/Mule use.
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
            SoundCategory::Neutral,
            &entity.pos.load(),
        );
    }
}

impl LlamaMob for LlamaEntity {
    fn llama_data(&self) -> &LlamaData {
        &self.llama_data
    }
}

impl Mob for LlamaEntity {
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
            send_baby_id_if_baby(self.get_entity());
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
}

#[cfg(test)]
mod tests {
    use super::{LlamaData, set_random_strength};
    use rand::rng;
    use std::sync::atomic::Ordering::Relaxed;

    #[test]
    fn set_random_strength_stays_within_vanilla_bounds() {
        let mut random = rng();
        let data = LlamaData::default();
        let mut saw_five = false;
        for _ in 0..5000 {
            set_random_strength(&data, &mut random);
            let strength = data.strength.load(Relaxed);
            assert!((1..=5).contains(&strength), "{strength} out of range");
            if strength == 5 {
                saw_five = true;
            }
        }
        // With a 4% chance per roll of the 1-5 branch, 5000 rolls should hit it at least once.
        assert!(saw_five, "never rolled the rare 1-5 branch in 5000 tries");
    }
}
