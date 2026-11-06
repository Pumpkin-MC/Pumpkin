use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::entity_variant::AxolotlVariant;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::position::BlockPos;
use rand::{Rng, RngExt, seq::IndexedRandom};

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob, BABY_START_AGE},
    ai::{
        brain::{
            Brain, BrainTick,
            memory::{PackedMemories, types},
        },
        pathfinder::node::PathType,
    },
    breath::DROWNING_INTERVAL,
    mob::{
        Mob, MobEntity,
        spawn::{EntitySpawnReason, SpawnGroupData},
    },
    passive::animal::Animal,
    player::Player,
};
use crate::plugin::api::events::entity::entity_target::TargetReason;
use crate::world::World;

use controls::{AxolotlLookControl, AxolotlMoveControl};

mod ai;
mod controls;

/// `Axolotl.getMaxHeadXRot`: how far the head may turn up or down per tick.
const MAX_HEAD_X_ROT: f32 = 1.0;
/// `Axolotl.getMaxHeadYRot`: how far the head may turn away from the body.
const MAX_HEAD_Y_ROT: f32 = 1.0;
/// Vanilla `AXOLOTL_TOTAL_AIR_SUPPLY`: `getMaxAirSupply`, 5 minutes against the default 15 seconds.
const AXOLOTL_TOTAL_AIR_SUPPLY: i32 = 6000;
/// Vanilla `REHYDRATE_AIR_SUPPLY`: what `rehydrate` adds.
const REHYDRATE_AIR_SUPPLY: i32 = 1800;
/// The damage `Axolotl.handleAirSupply` deals with `DamageSources.dryOut`.
const DRY_OUT_DAMAGE: f32 = 2.0;
/// Per tick factor of `Axolotl.travelInWater` on the velocity after the move.
const WATER_DRAG: f64 = 0.9;
/// Vanilla `Axolotl.RARE_VARIANT_CHANCE`: one in this many babies takes the rare variant.
const RARE_VARIANT_CHANCE: u32 = 1200;
/// Members already in a spawn group from which `Axolotl.finalizeSpawn` makes babies.
const BABY_GROUP_SIZE: u32 = 2;

/// Vanilla `Axolotl.Variant.getSpawnVariant`: a uniform pick among the variants whose `common`
/// flag matches.
fn get_spawn_variant<R: Rng + ?Sized>(rng: &mut R, common: bool) -> AxolotlVariant {
    let valid: Vec<AxolotlVariant> = AxolotlVariant::ALL
        .iter()
        .copied()
        .filter(|variant| variant.common() == common)
        .collect();
    valid
        .choose(rng)
        .copied()
        .unwrap_or(AxolotlVariant::DEFAULT)
}

/// Vanilla `Axolotl.Variant.getCommonSpawnVariant`.
fn get_common_spawn_variant<R: Rng + ?Sized>(rng: &mut R) -> AxolotlVariant {
    get_spawn_variant(rng, true)
}

/// Vanilla `Axolotl.Variant.getRareSpawnVariant`.
fn get_rare_spawn_variant<R: Rng + ?Sized>(rng: &mut R) -> AxolotlVariant {
    get_spawn_variant(rng, false)
}

/// Vanilla `Axolotl.getBreedOffspring` variant rule: one baby in `RARE_VARIANT_CHANCE` gets a rare
/// variant, every other one takes this parent's variant or the partner's with equal odds.
fn get_breed_variant<R: Rng + ?Sized>(
    parent: AxolotlVariant,
    partner: AxolotlVariant,
    rng: &mut R,
) -> AxolotlVariant {
    if rng.random_range(0..RARE_VARIANT_CHANCE) == 0 {
        get_rare_spawn_variant(rng)
    } else if rng.random::<bool>() {
        parent
    } else {
        partner
    }
}

/// Vanilla `AxolotlGroupData` joined by one more axolotl: the first member of a group draws two
/// common variants and every member takes one of them; from the third member on the axolotl is a
/// baby. Returns the member's variant, whether it is a baby, and the updated group.
fn join_spawn_group<R: Rng + ?Sized>(
    group_data: Option<&SpawnGroupData>,
    rng: &mut R,
) -> (AxolotlVariant, bool, SpawnGroupData) {
    let (types, group_size) = match group_data {
        Some(&SpawnGroupData::Axolotl { types, group_size }) => (types, group_size),
        _ => (
            [get_common_spawn_variant(rng), get_common_spawn_variant(rng)],
            0,
        ),
    };
    let variant = types
        .choose(rng)
        .copied()
        .unwrap_or(AxolotlVariant::DEFAULT);
    (
        variant,
        group_size >= BABY_GROUP_SIZE,
        SpawnGroupData::Axolotl {
            types,
            group_size: group_size + 1,
        },
    )
}

/// The air of an axolotl after one tick and whether it dried out in it.
struct NextAirSupply {
    air_supply: i32,
    dries_out: bool,
}

/// Vanilla `Axolotl.handleAirSupply`: out of water and rain the air runs down by one from what
/// it was before this tick, and at the drowning threshold it is set to 0 and the axolotl dries
/// out; in water or rain (or when dead) it is the maximum at once.
const fn next_air_supply(
    alive: bool,
    in_water_or_rain: bool,
    pre_tick_air_supply: i32,
    max_air_supply: i32,
) -> NextAirSupply {
    if !alive || in_water_or_rain {
        return NextAirSupply {
            air_supply: max_air_supply,
            dries_out: false,
        };
    }
    let air_supply = pre_tick_air_supply.saturating_sub(1);
    if air_supply <= -DROWNING_INTERVAL {
        NextAirSupply {
            air_supply: 0,
            dries_out: true,
        }
    } else {
        NextAirSupply {
            air_supply,
            dries_out: false,
        }
    }
}

/// What `Axolotl.usePlayerItem` gives back for a fed item instead of just using it up: the
/// tropical fish bucket turns into a water bucket.
fn fed_item_remainder(item: &Item) -> Option<&'static Item> {
    (item == &Item::TROPICAL_FISH_BUCKET).then_some(&Item::WATER_BUCKET)
}

/// Reads the `Variant` tag the way vanilla's legacy codec does: a missing tag is the default
/// variant and an unknown id falls back to it too.
fn read_variant(nbt: &NbtCompound) -> AxolotlVariant {
    nbt.get_int("Variant")
        .map_or(AxolotlVariant::DEFAULT, AxolotlVariant::by_id)
}

/// The axolotl behind a control's mob.
fn as_axolotl(mob: &dyn Mob) -> Option<&AxolotlEntity> {
    mob.cast_any().downcast_ref::<AxolotlEntity>()
}

/// Vanilla `TOTAL_PLAYDEAD_TIME`: the ticks a hit sets the play dead memory to.
const TOTAL_PLAYDEAD_TIME: i32 = 200;
/// Vanilla `PLAYER_REGEN_DETECTION_RANGE`: how far from the axolotl a player has to be to be
/// thanked for a kill.
const PLAYER_REGEN_DETECTION_RANGE: f64 = 20.0;
/// Vanilla `REGEN_BUFF_MAX_DURATION`.
const REGEN_BUFF_MAX_DURATION: i32 = 2400;
/// What `applySupportingEffects` adds to the regeneration a player has.
const REGEN_BUFF_BASE_DURATION: i32 = 100;

/// What `Axolotl.hurtServer` looks at to decide that a hit makes the axolotl play dead.
struct HurtCheck {
    no_ai: bool,
    damage: f32,
    health: f32,
    max_health: f32,
    in_water: bool,
    has_attacker: bool,
    playing_dead: bool,
}

/// The condition of vanilla `hurtServer` in its own order, so that the random numbers drawn (and
/// how many) are the vanilla ones: without AI none, then one in three, then a second draw below
/// the damage or less than half of the health left, then the damage below the health, in the
/// water, with an entity behind the hit, and not already playing dead.
fn plays_dead_when_hurt<R: Rng + ?Sized>(rng: &mut R, hit: &HurtCheck) -> bool {
    !hit.no_ai
        && rng.random_range(0..3) == 0
        && (f32::from(rng.random_range(0..3i16)) < hit.damage || hit.health / hit.max_health < 0.5)
        && hit.damage < hit.health
        && hit.in_water
        && hit.has_attacker
        && !hit.playing_dead
}

/// The regeneration duration `Axolotl.applySupportingEffects` gives a player whose regeneration
/// is `current` (`(ticks left, amplifier)`, `None` when it has none), or `None` when it gives
/// nothing: only a missing effect or one that ends within 2399 ticks is renewed, by 100 more
/// ticks up to 2400. An infinite effect (a negative duration) never ends, so it is left alone.
/// A stronger effect (amplifier 1 or more) is left alone too: in vanilla the new level 0 effect
/// cannot replace it, while here adding an effect replaces the one that is active.
fn supporting_regeneration_duration(current: Option<(i32, i32)>) -> Option<i32> {
    if current.is_some_and(|(_, amplifier)| amplifier > 0) {
        return None;
    }
    let left = current.map(|(left, _)| left);
    let renewed = left.is_none_or(|left| (0..REGEN_BUFF_MAX_DURATION).contains(&left));
    renewed.then_some((REGEN_BUFF_BASE_DURATION + left.unwrap_or(0)).min(REGEN_BUFF_MAX_DURATION))
}

/// Vanilla `Axolotl.applySupportingEffects`: regeneration for the player (see
/// [`supporting_regeneration_duration`]) and the end of mining fatigue.
fn apply_supporting_effects(player: &Player) {
    let current = player
        .get_effect(&StatusEffect::REGENERATION)
        .map(|effect| (effect.duration, i32::from(effect.amplifier)));
    if let Some(duration) = supporting_regeneration_duration(current) {
        player.add_effect(Effect {
            effect_type: &StatusEffect::REGENERATION,
            duration,
            amplifier: 0,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        });
    }
    player.remove_effect(&StatusEffect::MINING_FATIGUE);
}

pub struct AxolotlEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub variant: AtomicI32,
    pub playing_dead: AtomicBool,
    pub from_bucket: AtomicBool,
}

impl AxolotlEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        entity
            .air_supply
            .store(AXOLOTL_TOTAL_AIR_SUPPLY, Ordering::Relaxed);
        let mut mob_entity = MobEntity::new(entity);
        *mob_entity
            .move_control
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(AxolotlMoveControl::default());
        *mob_entity
            .look_control
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(AxolotlLookControl::default());
        // `setPathfindingMalus(PathType.WATER, 0.0F)`: water costs nothing to path through.
        mob_entity
            .navigator
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_pathfinding_malus(PathType::Water, 0.0);
        let axolotl = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            variant: AtomicI32::new(AxolotlVariant::DEFAULT.id()),
            playing_dead: AtomicBool::new(false),
            from_bucket: AtomicBool::new(false),
        };
        Arc::new(axolotl)
    }

    /// Vanilla `Axolotl.getVariant`.
    #[must_use]
    pub fn get_variant(&self) -> AxolotlVariant {
        AxolotlVariant::by_id(self.variant.load(Ordering::Relaxed))
    }

    /// Vanilla `Axolotl.setVariant`: stores the variant and syncs it to clients.
    pub fn set_variant(&self, variant: AxolotlVariant) {
        self.variant.store(variant.id(), Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::axolotl::DATA_VARIANT,
            VarInt(variant.id()),
        );
    }

    /// Vanilla `Axolotl.handleAirSupply`: applies [`next_air_supply`] and hurts the axolotl when
    /// it dries out.
    fn handle_air_supply(&self, pre_tick_air_supply: i32) {
        let entity = self.get_entity();
        let air = next_air_supply(
            self.mob_entity.living_entity.is_alive(),
            entity.is_in_water_or_rain(),
            pre_tick_air_supply,
            self.get_max_air_supply(),
        );
        entity.set_air_supply(air.air_supply);
        if air.dries_out {
            self.damage(self, DRY_OUT_DAMAGE, DamageType::DRY_OUT);
        }
    }

    /// Vanilla `Axolotl.rehydrate`: adds `REHYDRATE_AIR_SUPPLY` air, up to the maximum.
    pub fn rehydrate(&self) {
        let entity = self.get_entity();
        let air_supply = entity
            .air_supply
            .load(Ordering::Relaxed)
            .saturating_add(REHYDRATE_AIR_SUPPLY);
        entity.set_air_supply(air_supply.min(self.get_max_air_supply()));
    }

    #[must_use]
    pub fn is_playing_dead(&self) -> bool {
        self.playing_dead.load(Ordering::Relaxed)
    }

    pub fn set_playing_dead(&self, playing_dead: bool) {
        self.playing_dead.store(playing_dead, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::axolotl::DATA_PLAYING_DEAD,
            playing_dead,
        );
    }

    /// The ambient sound `playAmbientSound` plays: none while playing dead.
    fn audible_ambient_sound(&self) -> Option<Sound> {
        if self.is_playing_dead() {
            None
        } else {
            self.get_ambient_sound()
        }
    }

    /// Vanilla `getTarget` is `getTargetFromBrain`, the attack target memory. The `target` of the
    /// mob is what the rest of the server reads (`execute on target`, plugins), so it follows the
    /// memory: a new target and a lost one both go through `EntityTargetEvent`, and the brain is
    /// then made to agree with the target the mob ended up with (a plugin may have refused the
    /// change or replaced the target).
    fn follow_attack_target(&self, attack_target: Option<Arc<dyn EntityBase>>) {
        self.follow_attack_target_with(attack_target, |target, reason| {
            self.set_mob_target(target, reason)
        });
    }

    /// `follow_attack_target` with the target change passed in: it returns whether the change
    /// was not refused, and may leave the mob with another target than the one asked for.
    fn follow_attack_target_with(
        &self,
        attack_target: Option<Arc<dyn EntityBase>>,
        set_target: impl FnOnce(Option<Arc<dyn EntityBase>>, TargetReason) -> bool,
    ) {
        let entity_id = |target: &Option<Arc<dyn EntityBase>>| {
            target.as_ref().map(|t| t.get_entity().entity_id)
        };
        let wanted = entity_id(&attack_target);
        if entity_id(&self.mob_entity.get_target()) == wanted {
            return;
        }
        let reason = if wanted.is_some() {
            TargetReason::ClosestEntity
        } else {
            TargetReason::ForgotTarget
        };
        set_target(attack_target, reason);
        let settled = self.mob_entity.get_target();
        if entity_id(&settled) == wanted {
            return;
        }
        self.mob_entity.with_brain(self, |brain| match settled {
            Some(target) => brain.set(types::ATTACK_TARGET, target),
            None => brain.erase(types::ATTACK_TARGET.id()),
        });
    }

    /// Vanilla `Axolotl.onStopAttacking`: when the target that ended the fight died to a player
    /// within 20 blocks of the axolotl, the axolotl thanks the player with [`apply_supporting_effects`].
    fn on_stop_attacking(tick: &BrainTick<'_>, target: &Arc<dyn EntityBase>) {
        let Some(living) = target.get_living_entity() else {
            return;
        };
        if !living.is_dead_or_dying() {
            return;
        }
        let Some((_, killer)) = living.get_last_damage_source() else {
            return;
        };
        let range = tick
            .mob
            .get_entity()
            .bounding_box
            .load()
            .expand_all(PLAYER_REGEN_DETECTION_RANGE);
        if let Some(player) = tick
            .world
            .get_players_at_box(&range)
            .into_iter()
            .find(|player| player.get_entity().entity_id == killer)
        {
            apply_supporting_effects(&player);
        }
    }

    #[must_use]
    pub fn is_from_bucket(&self) -> bool {
        self.from_bucket.load(Ordering::Relaxed)
    }

    pub fn set_from_bucket(&self, from_bucket: bool) {
        self.from_bucket.store(from_bucket, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::axolotl::FROM_BUCKET,
            from_bucket,
        );
    }
}

impl AgeableMob for AxolotlEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for AxolotlEntity {
    /// Vanilla `Axolotl.getBreedOffspring`: the baby gets a variant by [`get_breed_variant`] and
    /// is kept from despawning.
    fn get_breed_offspring(&self, partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        let baby = Self::new(Entity::new(
            entity.world.load_full(),
            entity.pos.load(),
            &EntityType::AXOLOTL,
        ));
        let partner_variant = partner
            .cast_any()
            .downcast_ref::<Self>()
            .map_or_else(|| self.get_variant(), Self::get_variant);
        baby.set_variant(get_breed_variant(
            self.get_variant(),
            partner_variant,
            &mut self.get_random(),
        ));
        baby.mob_entity
            .persistence_required
            .store(true, Ordering::Relaxed);
        baby
    }

    /// Vanilla `Axolotl.isFood`: only the `axolotl_food` item tag.
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_AXOLOTL_FOOD)
    }

    /// Vanilla `Axolotl.usePlayerItem`: the tropical fish bucket becomes a water bucket
    /// (`ItemUtils.createFilledResult`), anything else is used up as usual.
    fn use_player_item(&self, player: &Arc<Player>, item_stack: &mut ItemStack) {
        if let Some(remainder) = fed_item_remainder(item_stack.get_item()) {
            crate::item::items::bucket::create_filled_result(
                item_stack,
                player,
                ItemStack::new(1, remainder),
                true,
            );
        } else {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
        }
    }
}

impl Mob for AxolotlEntity {
    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    /// Vanilla `Axolotl.removeWhenFarAway`: only axolotls released from a bucket or named ones
    /// stay.
    fn remove_when_far_away(&self, _distance_sq: f64) -> bool {
        !self.is_from_bucket() && (**self.get_entity().custom_name.load()).is_none()
    }

    /// Vanilla `Axolotl.requiresCustomPersistence`: `Mob`'s rule (a passenger or a leashed mob
    /// stays) or a bucket axolotl.
    fn requires_custom_persistence(&self) -> bool {
        let entity = self.get_entity();
        entity.has_vehicle() || entity.is_leashed() || self.is_from_bucket()
    }

    /// Vanilla `Axolotl.checkSpawnObstruction`: only blocking entities obstruct, water does not.
    fn check_spawn_obstruction(&self, world: &World) -> bool {
        super::water_animal::check_spawn_obstruction(self, world)
    }

    /// Vanilla `Axolotl.finalizeSpawn`: bucket spawns keep the group untouched, everything else
    /// joins the spawn group (see [`join_spawn_group`]).
    fn finalize_spawn(
        &self,
        _world: &Arc<World>,
        reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        if reason == EntitySpawnReason::Bucket {
            return group_data;
        }
        let (variant, is_baby, group) =
            join_spawn_group(group_data.as_ref(), &mut self.get_random());
        self.set_variant(variant);
        if is_baby {
            AgeableMob::set_age(self, BABY_START_AGE);
        }
        self.mob_entity.finalize_spawn_base();
        Some(group)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("Variant", self.get_variant().id());
        nbt.put_bool("FromBucket", self.is_from_bucket());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.set_variant(read_variant(nbt));
        self.set_from_bucket(nbt.get_bool("FromBucket").unwrap_or(false));
    }

    /// Vanilla `Axolotl.getMaxAirSupply`.
    fn mob_max_air_supply(&self) -> i32 {
        AXOLOTL_TOTAL_AIR_SUPPLY
    }

    /// Vanilla `Axolotl.baseTick`: `handleAirSupply` replaces the air rules of `LivingEntity`
    /// (water breathers never drown, the rest of the air logic), except without AI, where only
    /// those run and the air of an axolotl on land refills instead of running out.
    fn base_tick(&self) {
        if self.mob_entity.is_no_ai() {
            self.mob_entity.living_entity.tick_air_supply(self);
        } else {
            self.handle_air_supply(self.get_entity().air_supply.load(Ordering::Relaxed));
        }
    }

    /// Vanilla `Axolotl.getWalkTargetValue`: no spot is better than another, unlike the grass and
    /// light preference of other animals.
    fn get_walk_target_value(&self, _pos: &BlockPos) -> f32 {
        0.0
    }

    /// Vanilla `Axolotl.isPushedByFluid`.
    fn mob_is_pushed_by_fluids(&self) -> bool {
        false
    }

    /// Vanilla `Axolotl.travelInWater`: it swims by its own `getSpeed()` with neither gravity nor
    /// the jump out of the water of other mobs, and the water slows it by a tenth each tick.
    fn travel_in_water(&self, caller: &dyn EntityBase, _gravity: f64, _falling: bool) {
        let living = &self.mob_entity.living_entity;
        let entity = &living.entity;
        entity.update_velocity_from_input(
            living.movement_input.load(),
            f64::from(living.get_speed()),
        );
        living.make_move(caller);
        entity.velocity.store(entity.velocity.load() * WATER_DRAG);
    }

    /// Vanilla `Axolotl.hurtServer`, the part before `super`: a hit may make the axolotl play dead
    /// (see [`plays_dead_when_hurt`]), which sets the `PLAY_DEAD_TICKS` memory to 200.
    fn before_hurt(
        &self,
        amount: f32,
        _damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) {
        let living = &self.mob_entity.living_entity;
        let hit = HurtCheck {
            no_ai: self.mob_entity.is_no_ai(),
            damage: amount,
            health: living.health.load(),
            max_health: living.get_max_health(),
            in_water: self.get_entity().is_in_water(),
            has_attacker: source.is_some() || cause.is_some(),
            playing_dead: self.is_playing_dead(),
        };
        if plays_dead_when_hurt(&mut self.get_random(), &hit) {
            self.mob_entity.with_brain(self, |brain| {
                brain.set(types::PLAY_DEAD_TICKS, TOTAL_PLAYDEAD_TIME);
            });
        }
    }

    /// Vanilla `Axolotl.canBeSeenAsEnemy`: not while playing dead.
    fn mob_can_be_seen_as_enemy(&self) -> bool {
        !self.is_playing_dead()
    }

    /// Vanilla `Axolotl.playAmbientSound`: silent while playing dead.
    fn play_ambient_sound(&self) {
        if let Some(sound) = self.audible_ambient_sound() {
            self.make_sound(sound);
        }
    }

    /// Vanilla `Axolotl.getAmbientSound`: the water sound in water, the air sound outside. The
    /// animal default is silent, so axolotls did not make idle sounds at all before.
    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(if self.get_entity().is_in_water() {
            Sound::EntityAxolotlIdleWater
        } else {
            Sound::EntityAxolotlIdleAir
        })
    }

    /// Vanilla `Axolotl.playAttackSound`, played when the axolotl hurts its target.
    fn on_attack(&self, _target: &dyn EntityBase) {
        self.play_entity_sound(Sound::EntityAxolotlAttack, 1.0, 1.0);
    }

    /// Vanilla `Axolotl.getMaxHeadXRot`.
    fn get_max_look_pitch_change(&self) -> f32 {
        MAX_HEAD_X_ROT
    }

    /// Vanilla `Axolotl.getMaxHeadYRot`.
    fn get_max_head_rotation(&self) -> f32 {
        MAX_HEAD_Y_ROT
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        ai::make_brain(self, packed)
    }

    /// Vanilla `Axolotl.customServerAiStep`: the brain ticks, the activity is chosen, and the
    /// playing dead flag follows the `PLAY_DEAD_TICKS` memory unless the axolotl has no AI.
    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        self.mob_entity.tick_brain(self);
        let (play_dead_ticks, attack_target) = self.mob_entity.with_brain(self, |brain| {
            ai::update_activity(brain);
            (
                brain.get(types::PLAY_DEAD_TICKS).copied(),
                brain.get(types::ATTACK_TARGET).cloned(),
            )
        });
        if !self.mob_entity.is_no_ai() {
            self.set_playing_dead(play_dead_ticks.is_some_and(|ticks| ticks > 0));
        }
        self.follow_attack_target(attack_target);
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::axolotl::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::axolotl::DATA_VARIANT,
            VarInt(self.get_variant().id()),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::axolotl::DATA_PLAYING_DEAD,
            self.is_playing_dead(),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::axolotl::FROM_BUCKET,
            self.is_from_bucket(),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let item = item_stack.get_item();

        if item == &Item::WATER_BUCKET {
            let entity = self.get_entity();
            let world = entity.world.load();
            if let Some(server) = world.server.upgrade() {
                let mut event = crate::plugin::api::events::player::player_bucket_entity::PlayerBucketEntityEvent {
                    player: player.clone(),
                    entity_id: entity.entity_id,
                    bucket_item: "axolotl_bucket".to_string(),
                    cancelled: false,
                };
                server.plugin_manager.fire_blocking(&server, &mut event);
                if event.cancelled {
                    return false;
                }
            }
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            let pos = entity.pos.load();
            world.play_sound(Sound::ItemBucketFillAxolotl, SoundCategory::Neutral, &pos);
            entity.remove();
            return true;
        }

        self.animal_interact(player, item_stack)
    }
}

#[cfg(test)]
mod tests {
    use core::convert::Infallible;
    use std::collections::VecDeque;

    use pumpkin_data::environment_attribute::Activity;
    use pumpkin_util::{
        math::{vector3::Vector3, wrap_degrees},
        text::TextComponent,
    };
    use rand::TryRng;

    use super::*;
    use crate::entity::ai::test_support::world;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    /// A random source that hands out prepared draws, so a test fixes the value of every call
    /// and how many calls the code under test makes. A draw is a fraction in `[0, 1)`; a request
    /// for a range of `bound` values gets the index `floor(fraction * bound)`, and a `bool` is true
    /// from one half up.
    struct ScriptedRng(VecDeque<f64>);

    impl ScriptedRng {
        fn new(draws: &[f64]) -> Self {
            Self(draws.iter().copied().collect())
        }

        /// The draw that makes a request for `bound` values pick `index`.
        fn pick(index: u32, bound: u32) -> f64 {
            (f64::from(index) + 0.5) / f64::from(bound)
        }

        fn remaining(&self) -> usize {
            self.0.len()
        }

        fn next_fraction(&mut self) -> f64 {
            self.0
                .pop_front()
                .expect("the code under test drew more random numbers than scripted")
        }
    }

    impl TryRng for ScriptedRng {
        type Error = Infallible;

        fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
            Ok((self.next_fraction() * 2f64.powi(32)) as u32)
        }

        fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
            Ok((self.next_fraction() * 2f64.powi(64)) as u64)
        }

        fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
            for chunk in dst.chunks_mut(8) {
                let bytes = self.try_next_u64()?.to_le_bytes();
                chunk.copy_from_slice(&bytes[..chunk.len()]);
            }
            Ok(())
        }
    }

    fn axolotl(world: &Arc<World>) -> Arc<AxolotlEntity> {
        AxolotlEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::AXOLOTL,
        ))
    }

    /// The variants and member count of an axolotl group.
    fn group_parts(group: &SpawnGroupData) -> ([AxolotlVariant; 2], u32) {
        let SpawnGroupData::Axolotl { types, group_size } = group else {
            panic!("not an axolotl group");
        };
        (*types, *group_size)
    }

    fn natural_group(types: [AxolotlVariant; 2], group_size: u32) -> SpawnGroupData {
        SpawnGroupData::Axolotl { types, group_size }
    }

    /// The four common variants are the whole pool, in id order, and blue is not among them: the
    /// pick that would reach a fifth entry of an unfiltered list lands on cyan instead.
    #[test]
    fn common_spawn_variant_is_one_of_the_four_common_ones() {
        let expected = [
            AxolotlVariant::Lucy,
            AxolotlVariant::Wild,
            AxolotlVariant::Gold,
            AxolotlVariant::Cyan,
        ];
        for (index, variant) in (0..).zip(expected) {
            let mut rng = ScriptedRng::new(&[ScriptedRng::pick(index, 4)]);
            assert_eq!(get_common_spawn_variant(&mut rng), variant);
            assert_eq!(rng.remaining(), 0);
        }
        let mut rng = ScriptedRng::new(&[0.99]);
        assert_eq!(get_common_spawn_variant(&mut rng), AxolotlVariant::Cyan);
    }

    /// The first axolotl draws the two group variants and then its own pick; later members only
    /// pick, and only the third and later members are babies.
    #[test]
    fn spawn_group_draws_its_variants_once_and_makes_babies_from_the_third_member() {
        let mut rng = ScriptedRng::new(&[
            // first member: group variants gold and cyan, then the member takes the second
            ScriptedRng::pick(2, 4),
            ScriptedRng::pick(3, 4),
            ScriptedRng::pick(1, 2),
            // second member takes the first, third the second, fourth the first
            ScriptedRng::pick(0, 2),
            ScriptedRng::pick(1, 2),
            ScriptedRng::pick(0, 2),
        ]);
        let expected = [
            (AxolotlVariant::Cyan, false),
            (AxolotlVariant::Gold, false),
            (AxolotlVariant::Cyan, true),
            (AxolotlVariant::Gold, true),
        ];
        let mut group = None;
        for (member, (variant, baby)) in (1..).zip(expected) {
            let (picked, is_baby, next) = join_spawn_group(group.as_ref(), &mut rng);
            assert_eq!((picked, is_baby), (variant, baby), "member {member}");
            let (types, size) = group_parts(&next);
            assert_eq!(types, [AxolotlVariant::Gold, AxolotlVariant::Cyan]);
            assert_eq!(size, member);
            group = Some(next);
        }
        assert_eq!(rng.remaining(), 0);
    }

    /// Group data made for another species is not an axolotl group: the axolotl starts a new one.
    #[test]
    fn foreign_group_data_starts_a_new_group() {
        let foreign = SpawnGroupData::Ageable {
            group_size: 9,
            should_spawn_baby: true,
            baby_spawn_chance: 1.0,
        };
        let mut rng = ScriptedRng::new(&[
            ScriptedRng::pick(0, 4),
            ScriptedRng::pick(1, 4),
            ScriptedRng::pick(0, 2),
        ]);
        let (variant, is_baby, group) = join_spawn_group(Some(&foreign), &mut rng);
        assert_eq!((variant, is_baby), (AxolotlVariant::Lucy, false));
        assert_eq!(
            group_parts(&group),
            ([AxolotlVariant::Lucy, AxolotlVariant::Wild], 1)
        );
        assert_eq!(rng.remaining(), 0);
    }

    /// `Variant` is the vanilla id and a missing, negative or unknown id is Lucy, not what the
    /// entity had before. `FromBucket` is read the same way.
    #[tokio::test]
    async fn variant_and_from_bucket_round_trip_through_nbt() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let saved = axolotl(&world);
        saved.set_variant(AxolotlVariant::Gold);
        saved.set_from_bucket(true);
        let mut nbt = NbtCompound::new();
        EntityBase::write_nbt(saved.as_ref(), &mut nbt);
        assert_eq!(nbt.get_int("Variant"), Some(2));
        assert_eq!(nbt.get_bool("FromBucket"), Some(true));

        let loaded = axolotl(&world);
        loaded.set_variant(AxolotlVariant::Blue);
        EntityBase::read_nbt_non_mut(loaded.as_ref(), &nbt);
        assert_eq!(loaded.get_variant(), AxolotlVariant::Gold);
        assert!(loaded.is_from_bucket());

        let mut empty = NbtCompound::new();
        empty.put_string("id", "minecraft:axolotl".to_string());
        EntityBase::read_nbt_non_mut(loaded.as_ref(), &empty);
        assert_eq!(loaded.get_variant(), AxolotlVariant::Lucy);
        assert!(!loaded.is_from_bucket());

        for (id, variant) in [
            (4, AxolotlVariant::Blue),
            (5, AxolotlVariant::Lucy),
            (-1, AxolotlVariant::Lucy),
            (i32::MAX, AxolotlVariant::Lucy),
        ] {
            empty.put_int("Variant", id);
            EntityBase::read_nbt_non_mut(loaded.as_ref(), &empty);
            assert_eq!(loaded.get_variant(), variant, "Variant {id}");
        }
        Ok(())
    }

    /// `finalize_spawn` takes the group's variant and ages the third and later members; a spawn
    /// from a bucket leaves both the axolotl and the group alone.
    #[tokio::test]
    async fn finalize_spawn_joins_the_group_except_for_buckets() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);

        let first = axolotl(&world);
        let group = first
            .finalize_spawn(&world, EntitySpawnReason::Natural, None)
            .expect("a spawn group");
        let (types, size) = group_parts(&group);
        assert_eq!(size, 1);
        assert!(types.iter().all(AxolotlVariant::common));
        assert!(types.contains(&first.get_variant()));
        assert!(!first.is_baby());

        let shared = [AxolotlVariant::Wild, AxolotlVariant::Wild];
        let second = axolotl(&world);
        let group = second
            .finalize_spawn(
                &world,
                EntitySpawnReason::Natural,
                Some(natural_group(shared, 1)),
            )
            .expect("a spawn group");
        assert_eq!(second.get_variant(), AxolotlVariant::Wild);
        assert!(!second.is_baby());
        assert_eq!(group_parts(&group), (shared, 2));

        let third = axolotl(&world);
        let group = third
            .finalize_spawn(&world, EntitySpawnReason::Natural, Some(group))
            .expect("a spawn group");
        assert_eq!(third.get_variant(), AxolotlVariant::Wild);
        assert!(third.is_baby());
        assert_eq!(group_parts(&group), (shared, 3));

        let bucket = axolotl(&world);
        let group = bucket
            .finalize_spawn(
                &world,
                EntitySpawnReason::Bucket,
                Some(natural_group([AxolotlVariant::Gold; 2], 5)),
            )
            .expect("the group is handed back");
        assert_eq!(bucket.get_variant(), AxolotlVariant::Lucy);
        assert!(!bucket.is_baby());
        assert_eq!(group_parts(&group), ([AxolotlVariant::Gold; 2], 5));
        Ok(())
    }

    /// One baby in 1200 is rare whatever the parents are: a draw just inside 1/1200 gives the rare
    /// variant, one just outside it falls through to the parent choice, which uses exactly one more
    /// draw (and a rare baby uses one more for the rare variant, which has no alternative).
    #[test]
    fn breeding_rolls_the_rare_variant_one_in_1200() {
        let mut rng = ScriptedRng::new(&[1.0 / 1300.0, 0.5]);
        assert_eq!(
            get_breed_variant(AxolotlVariant::Lucy, AxolotlVariant::Gold, &mut rng),
            AxolotlVariant::Blue
        );
        assert_eq!(rng.remaining(), 0);

        for draw in [1.0 / 1100.0, ScriptedRng::pick(1, 1200)] {
            let mut rng = ScriptedRng::new(&[draw, 0.75]);
            assert_eq!(
                get_breed_variant(AxolotlVariant::Lucy, AxolotlVariant::Gold, &mut rng),
                AxolotlVariant::Lucy,
                "draw {draw}"
            );
            assert_eq!(rng.remaining(), 0);
        }
    }

    /// Without the rare roll the baby takes this parent's variant or the partner's, chosen by the
    /// draw after the rare roll; a blue parent passes blue on like any other variant.
    #[test]
    fn breeding_takes_this_parent_or_the_partner() {
        let miss = ScriptedRng::pick(600, 1200);
        for (draw, expected) in [(0.75, AxolotlVariant::Blue), (0.25, AxolotlVariant::Gold)] {
            let mut rng = ScriptedRng::new(&[miss, draw]);
            assert_eq!(
                get_breed_variant(AxolotlVariant::Blue, AxolotlVariant::Gold, &mut rng),
                expected,
                "draw {draw}"
            );
            assert_eq!(rng.remaining(), 0);
        }
    }

    /// The baby is an axolotl that takes the variant of either parent (blue being the rare
    /// exception) and never despawns. With 100 babies both parents are picked unless the odds
    /// are about 1 in 2^99 against it.
    #[tokio::test]
    async fn bred_baby_takes_either_parents_variant_and_persists() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let parent = axolotl(&world);
        parent.set_variant(AxolotlVariant::Cyan);
        let partner = axolotl(&world);
        partner.set_variant(AxolotlVariant::Gold);
        let mut seen = Vec::new();
        for _ in 0..100 {
            let offspring = parent.get_breed_offspring(partner.as_ref());
            let baby = offspring
                .cast_any()
                .downcast_ref::<AxolotlEntity>()
                .expect("an axolotl baby");
            assert!(baby.mob_entity.persistence_required.load(Ordering::Relaxed));
            seen.push(baby.get_variant());
        }
        assert!(seen.iter().all(|variant| {
            matches!(
                variant,
                AxolotlVariant::Cyan | AxolotlVariant::Gold | AxolotlVariant::Blue
            )
        }));
        assert!(seen.contains(&AxolotlVariant::Cyan));
        assert!(seen.contains(&AxolotlVariant::Gold));
        Ok(())
    }

    /// Only a bucket axolotl or a named one is kept from despawning by `remove_when_far_away`;
    /// riding or being leashed (vanilla `Mob.requiresCustomPersistence`) and a bucket also keep
    /// it through `requires_custom_persistence`.
    #[tokio::test]
    async fn axolotls_despawn_unless_bucketed_named_leashed_or_riding() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let other: Arc<dyn EntityBase> = axolotl(&world);

        let plain = axolotl(&world);
        assert!(plain.remove_when_far_away(1.0e6));
        assert!(!plain.requires_custom_persistence());

        let named = axolotl(&world);
        named
            .get_entity()
            .set_custom_name(TextComponent::text("Bob"));
        assert!(!named.remove_when_far_away(1.0e6));

        let bucket = axolotl(&world);
        bucket.set_from_bucket(true);
        assert!(!bucket.remove_when_far_away(1.0e6));
        assert!(bucket.requires_custom_persistence());

        let leashed = axolotl(&world);
        *leashed.get_entity().leashed_to.lock().unwrap() = Some(other.clone());
        assert!(leashed.requires_custom_persistence());

        let rider = axolotl(&world);
        *rider.get_entity().vehicle.lock().unwrap() = Some(other);
        assert!(rider.requires_custom_persistence());
        Ok(())
    }

    /// Only the tropical fish bucket is axolotl food: not the fish, not another bucket.
    #[tokio::test]
    async fn only_the_tropical_fish_bucket_is_food() -> TestResult {
        let folder = tempfile::tempdir()?;
        let axolotl = axolotl(&world(&folder));
        for (item, is_food) in [
            (&Item::TROPICAL_FISH_BUCKET, true),
            (&Item::TROPICAL_FISH, false),
            (&Item::COD_BUCKET, false),
            (&Item::WATER_BUCKET, false),
        ] {
            assert_eq!(
                axolotl.is_food(&ItemStack::new(1, item)),
                is_food,
                "{}",
                item.registry_key
            );
        }
        Ok(())
    }

    fn tick_move_control(axolotl: &Arc<AxolotlEntity>) {
        axolotl
            .mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .tick(axolotl.as_ref());
    }

    /// Points the move control at `(x, y, z)` and gives the navigator a one node path there, so
    /// the control has something to follow: it only steers while the navigation is not done.
    fn move_towards(axolotl: &Arc<AxolotlEntity>, x: f64, y: f64, z: f64) {
        use crate::entity::ai::pathfinder::{node::Node, path::Path};
        let target = pumpkin_util::math::position::BlockPos::floored(x, y, z);
        axolotl
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .move_to_path(
                Some(Path::new(vec![Node::new(target)], target, true)),
                1.0,
                &axolotl.mob_entity.living_entity,
            );
        axolotl
            .mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_wanted_position(x, y, z, 1.0);
    }

    fn walking_speed(axolotl: &Arc<AxolotlEntity>) -> f32 {
        axolotl.mob_entity.living_entity.get_speed()
    }

    /// The axolotl swims at a tenth of its speed attribute in water and walks at half of it on
    /// land, and it tilts towards a target below or above it, never by more than the 5 degrees
    /// of one tick (and with the right sign for a target above it).
    #[tokio::test]
    async fn move_control_uses_the_axolotl_swim_and_walk_speeds() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        let entity = axolotl.get_entity();
        let attribute_speed = axolotl
            .mob_entity
            .living_entity
            .get_attribute_value(&pumpkin_data::attributes::Attributes::MOVEMENT_SPEED)
            as f32;

        // On land the wanted position is straight ahead (yaw 0 faces +z), so no turning is left.
        move_towards(&axolotl, 0.0, 64.0, 8.0);
        tick_move_control(&axolotl);
        assert!((walking_speed(&axolotl) - attribute_speed * 0.5).abs() < 1e-6);

        entity.touching_water.store(true, Ordering::Relaxed);
        tick_move_control(&axolotl);
        assert!((walking_speed(&axolotl) - attribute_speed * 0.1).abs() < 1e-6);

        move_towards(&axolotl, 0.0, 62.0, 8.0);
        tick_move_control(&axolotl);
        assert!(
            (entity.pitch.load() - 5.0).abs() < 1e-4,
            "{}",
            entity.pitch.load()
        );
        let input = axolotl.mob_entity.living_entity.movement_input.load();
        assert!(input.y < 0.0 && input.z > 0.0);

        entity.pitch.store(0.0);
        move_towards(&axolotl, 0.0, 66.0, 8.0);
        tick_move_control(&axolotl);
        assert!(
            (entity.pitch.load() + 5.0).abs() < 1e-4,
            "{}",
            entity.pitch.load()
        );
        assert!(axolotl.mob_entity.living_entity.movement_input.load().y > 0.0);
        world.level.shutdown().await;
        Ok(())
    }

    /// A target to the side turns the body by at most 10 degrees a tick (`maxTurnY`), and a
    /// target straight below tilts the nose down by 5 degrees a tick until it is 85 degrees
    /// down (`maxTurnX`), not the 90 degrees that point at the target.
    #[tokio::test]
    async fn move_control_limits_how_far_it_turns() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        let entity = axolotl.get_entity();
        entity.touching_water.store(true, Ordering::Relaxed);

        // Yaw 0 faces +z, so a target along +x is 90 degrees to the left (yaw -90).
        move_towards(&axolotl, 8.0, 64.0, 0.0);
        tick_move_control(&axolotl);
        assert!(
            wrap_degrees(entity.yaw.load() + 10.0).abs() < 1e-3,
            "{}",
            entity.yaw.load()
        );
        tick_move_control(&axolotl);
        assert!(
            wrap_degrees(entity.yaw.load() + 20.0).abs() < 1e-3,
            "{}",
            entity.yaw.load()
        );

        move_towards(&axolotl, 0.0, 40.0, 0.0);
        for _ in 0..30 {
            tick_move_control(&axolotl);
        }
        assert!(
            (entity.pitch.load() - 85.0).abs() < 1e-3,
            "{}",
            entity.pitch.load()
        );
        world.level.shutdown().await;
        Ok(())
    }

    /// A playing dead axolotl keeps its wanted position but neither turns nor moves, and swims
    /// on once it gets up.
    #[tokio::test]
    async fn move_control_waits_while_the_axolotl_plays_dead() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        move_towards(&axolotl, 0.0, 64.0, 8.0);

        axolotl.set_playing_dead(true);
        tick_move_control(&axolotl);
        assert!(walking_speed(&axolotl).abs() < 1e-9, "a dead axolotl moved");
        assert!(
            axolotl
                .mob_entity
                .move_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .has_wanted(),
            "the wanted position was dropped while playing dead"
        );

        axolotl.set_playing_dead(false);
        tick_move_control(&axolotl);
        assert!(walking_speed(&axolotl) > 0.0);
        world.level.shutdown().await;
        Ok(())
    }

    /// Looks at a point 10 blocks to the side and 10 blocks above the eyes, then runs the look
    /// control for one tick.
    fn look_up_and_aside(axolotl: &Arc<AxolotlEntity>) {
        let entity = axolotl.get_entity();
        let pos = entity.pos.load();
        let mut look = axolotl
            .mob_entity
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        look.look_at(
            axolotl.as_ref(),
            pos.x + 10.0,
            entity.get_eye_y() + 10.0,
            pos.z,
        );
        look.tick(axolotl.as_ref());
    }

    /// The head turns 10 degrees per tick towards a target 20 degrees past the target (the
    /// smooth swimming offset) and tilts by one degree per tick (`getMaxHeadXRot`), so a target
    /// above makes a small negative pitch. The body stays until the head is more than 20 degrees
    /// (`maxYRotFromCenter`) off it, then it follows 4 degrees per tick.
    #[tokio::test]
    async fn look_control_turns_the_head_slowly() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        let entity = axolotl.get_entity();

        look_up_and_aside(&axolotl);
        assert!((entity.head_yaw.load() + 10.0).abs() < 1e-4);
        assert!((entity.pitch.load() + 1.0).abs() < 1e-4);
        look_up_and_aside(&axolotl);
        assert!((entity.head_yaw.load() + 20.0).abs() < 1e-4);
        assert!(
            entity.body_yaw.load().abs() < 1e-4,
            "the body turned at 20 degrees"
        );
        look_up_and_aside(&axolotl);
        assert!((entity.head_yaw.load() + 30.0).abs() < 1e-4);
        assert!((entity.body_yaw.load() + 4.0).abs() < 1e-4);
        assert!((entity.pitch.load() + 3.0).abs() < 1e-4);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn look_control_waits_while_the_axolotl_plays_dead() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        let entity = axolotl.get_entity();

        axolotl.set_playing_dead(true);
        look_up_and_aside(&axolotl);
        assert!(entity.head_yaw.load().abs() < 1e-9);
        assert!(entity.pitch.load().abs() < 1e-9);

        axolotl.set_playing_dead(false);
        look_up_and_aside(&axolotl);
        assert!(entity.head_yaw.load() < 0.0 && entity.pitch.load() < 0.0);
        world.level.shutdown().await;
        Ok(())
    }

    /// The body follows a turned head to within one degree (`getMaxHeadYRot` is 1), where a
    /// default mob lets the head turn 75 degrees away from a body that stays put.
    #[tokio::test]
    async fn body_follows_the_head_to_within_one_degree() -> TestResult {
        use crate::entity::ai::control::body_rotation_control::BodyRotationControl;
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        let entity = axolotl.get_entity();
        entity.head_yaw.store(30.0);

        BodyRotationControl::new().client_tick(axolotl.as_ref());
        assert!((entity.body_yaw.load() - 29.0).abs() < 1e-4);
        world.level.shutdown().await;
        Ok(())
    }

    /// Water costs nothing to path through (a normal mob pays for it) and no spot is a favourite
    /// walk target, not even grass, which makes the animal default rate 10.
    #[tokio::test]
    async fn axolotl_paths_through_water_and_has_no_favourite_spot() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let chunk = pumpkin_world::chunk::ChunkData::empty_sync(0, 0);
        chunk.set_block_absolute_y(0, 63, 0, pumpkin_data::Block::GRASS_BLOCK.default_state.id);
        world
            .level
            .loaded_chunks
            .insert(pumpkin_util::math::vector2::Vector2::new(0, 0), chunk);
        let axolotl = axolotl(&world);

        assert!(PathType::Water.get_malus() > 0.0);
        let water_malus = axolotl
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_pathfinding_malus(PathType::Water);
        assert!(water_malus.abs() < 1e-6, "{water_malus}");

        let on_grass = BlockPos::new(0, 64, 0);
        assert!((axolotl.animal_walk_target_value(&on_grass) - 10.0).abs() < 1e-6);
        assert!(axolotl.get_walk_target_value(&on_grass).abs() < 1e-6);
        world.level.shutdown().await;
        Ok(())
    }

    /// Currents and bubble columns do not push an axolotl.
    #[tokio::test]
    async fn fluids_do_not_push_the_axolotl() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        assert!(!EntityBase::is_pushed_by_fluids(axolotl(&world).as_ref()));
        world.level.shutdown().await;
        Ok(())
    }

    /// In water the axolotl accelerates by its own speed (`getSpeed()`, here 0.5) along the way it
    /// faces, loses a tenth of the velocity each tick and does not sink, where the generic water
    /// travel would add gravity and the jump out of the water.
    #[tokio::test]
    async fn travel_in_water_swims_with_its_speed_and_no_gravity() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        let living = &axolotl.mob_entity.living_entity;
        let entity = axolotl.get_entity();
        entity.touching_water.store(true, Ordering::Relaxed);
        axolotl.mob_entity.set_speed(0.5);
        living.movement_input.store(Vector3::new(0.0, 0.0, 1.0));

        axolotl.travel_in_water(axolotl.as_ref(), 0.08, false);
        let velocity = entity.velocity.load();
        assert!((velocity.z - 0.45).abs() < 1e-6, "{velocity:?}");
        assert!(
            velocity.x.abs() < 1e-9 && velocity.y.abs() < 1e-9,
            "{velocity:?}"
        );
        assert!(entity.pos.load().z > 0.4);

        axolotl.travel_in_water(axolotl.as_ref(), 0.08, false);
        assert!((entity.velocity.load().z - 0.855).abs() < 1e-6);
        world.level.shutdown().await;
        Ok(())
    }

    /// The idle sound is the water one in water and the air one outside, and it exists at all
    /// (the animal default has none).
    #[tokio::test]
    async fn idle_sound_depends_on_the_water() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        let entity = axolotl.get_entity();

        assert_eq!(
            axolotl.get_ambient_sound(),
            Some(Sound::EntityAxolotlIdleAir)
        );
        entity.touching_water.store(true, Ordering::Relaxed);
        assert_eq!(
            axolotl.get_ambient_sound(),
            Some(Sound::EntityAxolotlIdleWater)
        );
        entity.touching_water.store(false, Ordering::Relaxed);
        assert_eq!(
            axolotl.get_ambient_sound(),
            Some(Sound::EntityAxolotlIdleAir)
        );
        world.level.shutdown().await;
        Ok(())
    }

    /// The sounds the axolotl takes from its entity data: hurt, death, swimming and the splash
    /// when it enters water are the axolotl ones, not the generic ones.
    #[tokio::test]
    async fn hurt_death_swim_and_splash_sounds_are_the_axolotls() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        assert_eq!(
            EntityType::AXOLOTL.hurt_sound,
            Some(Sound::EntityAxolotlHurt)
        );
        assert_eq!(
            EntityType::AXOLOTL.death_sound,
            Some(Sound::EntityAxolotlDeath)
        );
        assert_eq!(
            EntityBase::get_swim_sound(axolotl.as_ref()),
            Some(Sound::EntityAxolotlSwim)
        );
        assert_eq!(
            EntityBase::get_swim_splash_sound(axolotl.as_ref()),
            Some(Sound::EntityAxolotlSplash)
        );
        world.level.shutdown().await;
        Ok(())
    }

    fn air_supply(axolotl: &Arc<AxolotlEntity>) -> i32 {
        axolotl.get_entity().air_supply.load(Ordering::Relaxed)
    }

    /// An axolotl holds its breath for 6000 ticks, starts with all of it, and an `Air` tag that is
    /// missing means a full supply (not the 300 of other mobs) while a saved one is kept.
    #[tokio::test]
    async fn axolotl_air_supply_is_6000() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        assert_eq!(axolotl.get_max_air_supply(), 6000);
        assert_eq!(air_supply(&axolotl), 6000);

        let mut nbt = NbtCompound::new();
        nbt.put_string("id", "minecraft:axolotl".to_string());
        axolotl.get_entity().air_supply.store(10, Ordering::Relaxed);
        EntityBase::read_nbt_non_mut(axolotl.as_ref(), &nbt);
        assert_eq!(air_supply(&axolotl), 6000);
        nbt.put_short("Air", 4321);
        EntityBase::read_nbt_non_mut(axolotl.as_ref(), &nbt);
        assert_eq!(air_supply(&axolotl), 4321);
        world.level.shutdown().await;
        Ok(())
    }

    /// Out of the water the air drops by one a tick. In water it is full at once, whatever it was.
    #[tokio::test]
    async fn air_runs_down_on_land_and_refills_in_water() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);

        axolotl.base_tick();
        axolotl.base_tick();
        axolotl.base_tick();
        assert_eq!(air_supply(&axolotl), 5997);
        assert!(
            axolotl
                .mob_entity
                .living_entity
                .get_last_damage_source()
                .is_none()
        );

        axolotl
            .get_entity()
            .touching_water
            .store(true, Ordering::Relaxed);
        axolotl.base_tick();
        assert_eq!(air_supply(&axolotl), 6000);
        axolotl
            .get_entity()
            .air_supply
            .store(-15, Ordering::Relaxed);
        axolotl.base_tick();
        assert_eq!(air_supply(&axolotl), 6000);
        world.level.shutdown().await;
        Ok(())
    }

    /// The pure air rule: one less a tick on land, dry out at -20 with the air back at 0, the maximum
    /// in water or rain and when dead, and the drop always starts from the air before the tick.
    #[test]
    fn next_air_supply_follows_the_vanilla_rule() {
        let cases = [
            // (alive, wet, before, air, dries out)
            (true, false, 6000, 5999, false),
            (true, false, 1, 0, false),
            (true, false, -18, -19, false),
            (true, false, -19, 0, true),
            (true, false, -20, 0, true),
            (true, false, i32::MIN, 0, true),
            (true, true, 100, 6000, false),
            (true, true, -50, 6000, false),
            (false, false, 100, 6000, false),
            (false, true, -50, 6000, false),
        ];
        for (alive, wet, before, air, dries_out) in cases {
            let next = next_air_supply(alive, wet, before, 6000);
            assert_eq!(
                (next.air_supply, next.dries_out),
                (air, dries_out),
                "alive {alive}, wet {wet}, before {before}"
            );
        }
    }

    /// At -20 air the axolotl dries out: the air is set to 0 and it is hurt by `DRY_OUT` for 2, then
    /// again each time 20 more ticks have gone by. (The test world has no server, so the damage
    /// stops after it has been recorded: the health stays, the record shows the hit.)
    #[tokio::test]
    async fn axolotl_dries_out_once_the_air_is_below_minus_20() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        let living = &axolotl.mob_entity.living_entity;
        assert!(living.get_last_damage_source().is_none());

        axolotl.get_entity().air_supply.store(1, Ordering::Relaxed);
        for tick in 1..=20 {
            axolotl.base_tick();
            assert_eq!(air_supply(&axolotl), 1 - tick, "tick {tick}");
            assert!(
                living.get_last_damage_source().is_none(),
                "hurt too early at tick {tick}"
            );
        }
        axolotl.base_tick();
        assert_eq!(air_supply(&axolotl), 0);
        assert_eq!(
            living.get_last_damage_source(),
            Some((DamageType::DRY_OUT, 0))
        );
        assert!((living.last_damage_taken.load() - 2.0).abs() < 1e-6);

        living.last_damage_taken.store(0.0);
        living.hurt_cooldown.store(0, Ordering::Relaxed);
        for _ in 0..19 {
            axolotl.base_tick();
        }
        assert!(
            living.last_damage_taken.load().abs() < 1e-6,
            "hurt again too early"
        );
        axolotl.base_tick();
        assert_eq!(air_supply(&axolotl), 0);
        assert!((living.last_damage_taken.load() - 2.0).abs() < 1e-6);
        world.level.shutdown().await;
        Ok(())
    }

    /// Without AI the dry out rule does not run: the air of an axolotl on land refills by 4 a tick
    /// (the `LivingEntity` rule) up to the maximum instead.
    #[tokio::test]
    async fn axolotl_without_ai_does_not_dry_out() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        axolotl.mob_entity.set_no_ai(true);
        axolotl
            .get_entity()
            .air_supply
            .store(100, Ordering::Relaxed);

        axolotl.base_tick();
        assert_eq!(air_supply(&axolotl), 104);
        axolotl
            .get_entity()
            .air_supply
            .store(5999, Ordering::Relaxed);
        axolotl.base_tick();
        assert_eq!(air_supply(&axolotl), 6000);
        axolotl
            .get_entity()
            .air_supply
            .store(-100, Ordering::Relaxed);
        axolotl.base_tick();
        assert_eq!(air_supply(&axolotl), -96);
        assert!(
            axolotl
                .mob_entity
                .living_entity
                .get_last_damage_source()
                .is_none()
        );
        world.level.shutdown().await;
        Ok(())
    }

    /// `rehydrate` adds 1800 air and stops at the maximum.
    #[tokio::test]
    async fn rehydrate_adds_1800_air_up_to_the_maximum() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        for (before, after) in [
            (1000, 2800),
            (-50, 1750),
            (4200, 6000),
            (5000, 6000),
            (6000, 6000),
        ] {
            axolotl
                .get_entity()
                .air_supply
                .store(before, Ordering::Relaxed);
            axolotl.rehydrate();
            assert_eq!(air_supply(&axolotl), after, "from {before}");
        }
        world.level.shutdown().await;
        Ok(())
    }

    /// Feeding a tropical fish bucket gives back a water bucket; nothing else leaves anything
    /// behind (the raw fish, other buckets and the water bucket itself).
    #[test]
    fn only_the_tropical_fish_bucket_leaves_a_water_bucket_when_fed() {
        assert_eq!(
            fed_item_remainder(&Item::TROPICAL_FISH_BUCKET),
            Some(&Item::WATER_BUCKET)
        );
        for item in [
            &Item::TROPICAL_FISH,
            &Item::COD_BUCKET,
            &Item::WATER_BUCKET,
            &Item::BUCKET,
        ] {
            assert_eq!(fed_item_remainder(item), None, "{}", item.registry_key);
        }
    }

    fn cod(world: &Arc<World>, x: f64) -> Arc<crate::entity::passive::cod::CodEntity> {
        let entity = Entity::new(world.clone(), Vector3::new(x, 64.0, 0.0), &EntityType::COD);
        entity.touching_water.store(true, Ordering::Relaxed);
        crate::entity::passive::cod::CodEntity::new(entity)
    }

    fn brain_value<R>(
        axolotl: &Arc<AxolotlEntity>,
        read: impl FnOnce(&crate::entity::ai::brain::Brain) -> R,
    ) -> R {
        axolotl
            .mob_entity
            .with_brain(axolotl.as_ref(), |brain| read(brain))
    }

    /// The playing dead flag follows the `PLAY_DEAD_TICKS` memory on every AI step, after the
    /// brain has counted it down by one: on while the memory is still above 0, so 2 ticks left
    /// are on and 1 tick left is off, and off at 0 and without the memory. An axolotl without AI
    /// keeps whatever flag it had.
    #[tokio::test]
    async fn the_playing_dead_flag_follows_the_memory_unless_there_is_no_ai() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        let flag_after = |ticks: Option<i32>| {
            axolotl.mob_entity.with_brain(axolotl.as_ref(), |brain| {
                brain.set_optional(types::PLAY_DEAD_TICKS, ticks);
            });
            axolotl.custom_server_ai_step(axolotl.as_ref());
            axolotl.is_playing_dead()
        };

        let results = [
            flag_after(None),
            flag_after(Some(200)),
            flag_after(Some(2)),
            flag_after(Some(1)),
            flag_after(Some(0)),
            flag_after(Some(5)),
            flag_after(None),
        ];
        axolotl.mob_entity.set_no_ai(true);
        axolotl.set_playing_dead(true);
        let no_ai_stays_on = flag_after(None);
        axolotl.set_playing_dead(false);
        let no_ai_stays_off = flag_after(Some(100));
        world.level.shutdown().await;

        assert_eq!(results, [false, true, true, false, false, true, false]);
        assert!(no_ai_stays_on);
        assert!(!no_ai_stays_off);
        Ok(())
    }

    /// The target the mob reports (what `execute on target` and plugins read) is the attack
    /// target memory, set when the brain picks a target and cleared when it forgets it.
    #[tokio::test]
    async fn the_mob_target_follows_the_attack_target_memory() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        let prey = cod(&world, 2.0);
        let prey_id = prey.get_entity().entity_id;
        let target_id = |axolotl: &Arc<AxolotlEntity>| {
            axolotl
                .mob_entity
                .get_target()
                .map(|target| target.get_entity().entity_id)
        };

        axolotl.mob_entity.with_brain(axolotl.as_ref(), |brain| {
            brain.set(types::ATTACK_TARGET, prey as Arc<dyn EntityBase>);
        });
        axolotl.custom_server_ai_step(axolotl.as_ref());
        let after_set = target_id(&axolotl);
        axolotl.mob_entity.with_brain(axolotl.as_ref(), |brain| {
            brain.erase(types::ATTACK_TARGET.id());
        });
        axolotl.custom_server_ai_step(axolotl.as_ref());
        let after_forgetting = target_id(&axolotl);
        world.level.shutdown().await;

        assert_eq!(after_set, Some(prey_id));
        assert_eq!(after_forgetting, None);
        Ok(())
    }

    /// A plugin that redirects the new target (or clears it) leaves the mob with another target
    /// than the brain asked for: the brain follows. A refused change leaves the old target, and
    /// a lost target is announced as `ForgotTarget` and cannot be kept by the brain alone.
    #[tokio::test]
    async fn the_brain_follows_the_target_the_event_settled_on() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        let wanted = cod(&world, 2.0);
        let redirected = cod(&world, 3.0);
        let old = cod(&world, 4.0);
        let id =
            |entity: &Arc<crate::entity::passive::cod::CodEntity>| entity.get_entity().entity_id;
        let brain_target = |axolotl: &Arc<AxolotlEntity>| {
            axolotl.mob_entity.with_brain(axolotl.as_ref(), |brain| {
                brain
                    .get(types::ATTACK_TARGET)
                    .map(|target| target.get_entity().entity_id)
            })
        };
        let mob_target = |axolotl: &Arc<AxolotlEntity>| {
            axolotl
                .mob_entity
                .get_target()
                .map(|target| target.get_entity().entity_id)
        };
        let mut reasons = Vec::new();

        // Redirected: the mob ends up with another cod, the brain takes that one.
        let other: Arc<dyn EntityBase> = redirected.clone();
        axolotl.follow_attack_target_with(Some(wanted.clone()), |_, reason| {
            reasons.push(reason);
            axolotl.mob_entity.set_target(Some(other));
            true
        });
        let after_redirect = (mob_target(&axolotl), brain_target(&axolotl));

        // Refused while the mob has another target: the brain gets that one back.
        axolotl.mob_entity.set_target(Some(old.clone()));
        axolotl.follow_attack_target_with(Some(wanted.clone()), |_, reason| {
            reasons.push(reason);
            false
        });
        let after_refusal = (mob_target(&axolotl), brain_target(&axolotl));

        // Refused with no target at all: the brain forgets it.
        axolotl.mob_entity.set_target(None);
        axolotl.follow_attack_target_with(Some(wanted.clone()), |_, _| false);
        let after_refusal_without_target = (mob_target(&axolotl), brain_target(&axolotl));

        // A lost target: asked through the event, and a refusal keeps it in the brain too.
        axolotl.mob_entity.set_target(Some(old.clone()));
        axolotl.follow_attack_target_with(None, |target, reason| {
            assert!(target.is_none());
            reasons.push(reason);
            false
        });
        let after_refused_loss = (mob_target(&axolotl), brain_target(&axolotl));
        axolotl.mob_entity.with_brain(axolotl.as_ref(), |brain| {
            brain.erase(types::ATTACK_TARGET.id());
        });
        axolotl.follow_attack_target_with(None, |_, reason| {
            reasons.push(reason);
            axolotl.mob_entity.set_target(None);
            true
        });
        let after_loss = (mob_target(&axolotl), brain_target(&axolotl));
        world.level.shutdown().await;

        assert_eq!(
            after_redirect,
            (Some(id(&redirected)), Some(id(&redirected)))
        );
        assert_eq!(after_refusal, (Some(id(&old)), Some(id(&old))));
        assert_eq!(after_refusal_without_target, (None, None));
        assert_eq!(after_refused_loss, (Some(id(&old)), Some(id(&old))));
        assert_eq!(after_loss, (None, None));
        assert_eq!(
            reasons,
            [
                TargetReason::ClosestEntity,
                TargetReason::ClosestEntity,
                TargetReason::ForgotTarget,
                TargetReason::ForgotTarget
            ]
        );
        Ok(())
    }

    /// The whole hunt through the real brain: a cod in the water within 8 blocks becomes the
    /// nearest attackable, the axolotl starts attacking and fights, and bites it with
    /// `MOB_ATTACK` once it is in reach. When the cod dies the fight ends and the hunting
    /// cooldown starts.
    #[tokio::test]
    async fn the_brain_hunts_a_cod_in_the_water_and_rests_after_the_kill() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);
        axolotl
            .get_entity()
            .touching_water
            .store(true, Ordering::Relaxed);
        let prey = cod(&world, 1.0);
        let everyone: Vec<Arc<dyn EntityBase>> = vec![axolotl.clone(), prey.clone()];
        world.entities.store(Arc::new(everyone));
        let step = |time: i64| {
            axolotl.mob_entity.with_brain(axolotl.as_ref(), |brain| {
                brain.tick(&world, axolotl.as_ref(), time);
                ai::update_activity(brain);
            });
        };

        for time in 0..60 {
            step(time);
        }
        let hunting = brain_value(&axolotl, |brain| {
            (
                brain.get_active_non_core_activity(),
                brain
                    .get(types::ATTACK_TARGET)
                    .map(|target| target.get_entity().entity_id),
            )
        });
        let bite = prey.mob_entity.living_entity.get_last_damage_source();
        prey.mob_entity.living_entity.health.store(0.0);
        for time in 60..64 {
            step(time);
        }
        let after_the_kill = brain_value(&axolotl, |brain| {
            (
                brain.get_active_non_core_activity(),
                brain.has_memory_value(types::ATTACK_TARGET.id()),
                brain.is_value(types::HAS_HUNTING_COOLDOWN, &true),
            )
        });
        world.level.shutdown().await;

        assert_eq!(
            hunting,
            (Some(Activity::Fight), Some(prey.get_entity().entity_id))
        );
        assert_eq!(
            bite,
            Some((DamageType::MOB_ATTACK, axolotl.get_entity().entity_id))
        );
        assert_eq!(after_the_kill, (Some(Activity::Idle), false, true));
        Ok(())
    }

    /// The memories vanilla saves survive a save and load: the play dead ticks (and the playing
    /// dead flag that follows them) and the hunting cooldown with its ticks left. The attack
    /// target is not saved.
    #[tokio::test]
    async fn the_saved_memories_come_back_from_nbt() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let original = axolotl(&world);
        let prey = cod(&world, 2.0);
        original.mob_entity.with_brain(original.as_ref(), |brain| {
            brain.set(types::PLAY_DEAD_TICKS, 120);
            brain.set_with_expiry(types::HAS_HUNTING_COOLDOWN, true, 1500);
            brain.set(types::ATTACK_TARGET, prey as Arc<dyn EntityBase>);
        });
        let mut nbt = NbtCompound::new();
        nbt.put_string("id", "minecraft:axolotl".to_owned());
        original.write_nbt(&mut nbt);

        let loaded = axolotl(&world);
        loaded.read_nbt_non_mut(&nbt);
        let flag_before = loaded.is_playing_dead();
        loaded.custom_server_ai_step(loaded.as_ref());
        let restored = brain_value(&loaded, |brain| {
            (
                brain.get(types::PLAY_DEAD_TICKS).copied(),
                brain.time_until_expiry(types::HAS_HUNTING_COOLDOWN.id()),
                brain.has_memory_value(types::ATTACK_TARGET.id()),
                brain.get_active_non_core_activity(),
            )
        });
        world.level.shutdown().await;

        assert!(!flag_before);
        assert!(loaded.is_playing_dead());
        let (play_dead, cooldown, target, activity) = restored;
        assert!(matches!(play_dead, Some(119..=120)), "{play_dead:?}");
        assert!(matches!(cooldown, Some(1495..=1500)), "{cooldown:?}");
        assert!(!target);
        assert_eq!(
            activity,
            Some(pumpkin_data::environment_attribute::Activity::PlayDead)
        );
        Ok(())
    }

    /// The regeneration a player gets for a kill: 100 ticks when there is none, 100 more on top
    /// of what is left up to 2400, nothing when 2400 or more are left (or it never ends).
    #[test]
    fn supporting_regeneration_adds_100_ticks_up_to_2400() {
        let given: Vec<Option<i32>> = [
            None,
            Some(0),
            Some(50),
            Some(2299),
            Some(2300),
            Some(2399),
            Some(2400),
            Some(5000),
            Some(-1),
        ]
        .into_iter()
        .map(|left| supporting_regeneration_duration(left.map(|left| (left, 0))))
        .collect();

        assert_eq!(
            given,
            vec![
                Some(100),
                Some(100),
                Some(150),
                Some(2399),
                Some(2400),
                Some(2400),
                None,
                None,
                None
            ]
        );

        // A stronger level is never replaced by the level 0 reward, however little is left of it.
        assert_eq!(supporting_regeneration_duration(Some((100, 1))), None);
        assert_eq!(supporting_regeneration_duration(Some((0, 2))), None);
    }

    /// A hit in the water by an entity on an unhurt-looking axolotl; the tests change one thing at
    /// a time from it.
    fn hit() -> HurtCheck {
        HurtCheck {
            no_ai: false,
            damage: 1.0,
            health: 10.0,
            max_health: 14.0,
            in_water: true,
            has_attacker: true,
            playing_dead: false,
        }
    }

    /// The outcome of the play dead rule for the scripted draws, and how many draws are left.
    fn plays_dead(hit: &HurtCheck, draws: &[u32]) -> (bool, usize) {
        let mut rng = ScriptedRng::new(
            &draws
                .iter()
                .map(|index| ScriptedRng::pick(*index, 3))
                .collect::<Vec<_>>(),
        );
        let result = plays_dead_when_hurt(&mut rng, hit);
        (result, rng.remaining())
    }

    /// The first draw has to be a 0 of 3 and the second one below the damage (here 1: only a 0),
    /// and the second draw is only made after the first one passed.
    #[test]
    fn a_hit_plays_dead_one_time_in_three_and_then_by_the_damage() {
        let one = hit();
        let two = HurtCheck {
            damage: 2.0,
            ..hit()
        };

        assert_eq!(plays_dead(&one, &[1, 0]), (false, 1));
        assert_eq!(plays_dead(&one, &[2, 0]), (false, 1));
        assert_eq!(plays_dead(&one, &[0, 0]), (true, 0));
        assert_eq!(plays_dead(&one, &[0, 1]), (false, 0));
        assert_eq!(plays_dead(&two, &[0, 1]), (true, 0));
        assert_eq!(plays_dead(&two, &[0, 2]), (false, 0));
    }

    /// Under half of the health the second draw does not matter (it is still made), exactly half
    /// does not count.
    #[test]
    fn under_half_of_the_health_a_second_draw_is_not_needed() {
        let low = HurtCheck {
            health: 6.9,
            ..hit()
        };
        let half = HurtCheck {
            health: 7.0,
            ..hit()
        };

        assert_eq!(plays_dead(&low, &[0, 2]), (true, 0));
        assert_eq!(plays_dead(&half, &[0, 2]), (false, 0));
        assert_eq!(plays_dead(&half, &[0, 0]), (true, 0));
    }

    /// A hit that would kill it, or only just not, and the other conditions: all of them are
    /// needed, and without AI not even the first random number is drawn.
    #[test]
    fn a_deadly_hit_or_a_missing_condition_keeps_it_standing() {
        let deadly = HurtCheck {
            damage: 10.0,
            ..hit()
        };
        let nearly = HurtCheck {
            damage: 9.99,
            ..hit()
        };
        let dry = HurtCheck {
            in_water: false,
            ..hit()
        };
        let unseen = HurtCheck {
            has_attacker: false,
            ..hit()
        };
        let already = HurtCheck {
            playing_dead: true,
            ..hit()
        };
        let no_ai = HurtCheck {
            no_ai: true,
            ..hit()
        };

        assert_eq!(plays_dead(&deadly, &[0, 0]), (false, 0));
        assert_eq!(plays_dead(&nearly, &[0, 0]), (true, 0));
        assert_eq!(plays_dead(&dry, &[0, 0]), (false, 0));
        assert_eq!(plays_dead(&unseen, &[0, 0]), (false, 0));
        assert_eq!(plays_dead(&already, &[0, 0]), (false, 0));
        assert_eq!(plays_dead(&no_ai, &[0, 0]), (false, 2));
    }

    /// A weak hit from an entity in the water sets the memory to 200 (a source or a cause alone is
    /// enough), whatever the dice say if it is repeated often enough (the chance that 300 hits all miss is below one in 10^50). Hits
    /// without an entity behind them (the dry out), out of the water, without AI, while already
    /// playing dead and killing hits leave the memory alone.
    #[tokio::test]
    async fn hits_set_the_play_dead_memory_only_when_the_conditions_hold() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let attacker = cod(&world, 2.0);
        let play_dead_ticks = |axolotl: &Arc<AxolotlEntity>| {
            axolotl.mob_entity.with_brain(axolotl.as_ref(), |brain| {
                brain.get(types::PLAY_DEAD_TICKS).copied()
            })
        };
        let hurt = |axolotl: &Arc<AxolotlEntity>, amount: f32, by_source: bool, by_cause: bool| {
            let source: Option<&dyn EntityBase> = by_source.then_some(attacker.as_ref());
            let cause: Option<&dyn EntityBase> = by_cause.then_some(attacker.as_ref());
            for _ in 0..300 {
                EntityBase::damage_with_context(
                    axolotl.as_ref(),
                    axolotl.as_ref(),
                    amount,
                    DamageType::MOB_ATTACK,
                    None,
                    source,
                    cause,
                );
            }
        };
        let wet = |health: f32| {
            let axolotl = axolotl(&world);
            axolotl
                .get_entity()
                .touching_water
                .store(true, Ordering::Relaxed);
            axolotl.mob_entity.living_entity.health.store(health);
            axolotl
        };

        let hit = wet(4.0);
        hurt(&hit, 0.1, true, true);
        let only_source = wet(4.0);
        hurt(&only_source, 0.1, true, false);
        let only_cause = wet(4.0);
        hurt(&only_cause, 0.1, false, true);
        let dry_out = wet(4.0);
        hurt(&dry_out, 0.1, false, false);
        let out_of_water = wet(4.0);
        out_of_water
            .get_entity()
            .touching_water
            .store(false, Ordering::Relaxed);
        hurt(&out_of_water, 0.1, true, true);
        let no_ai = wet(4.0);
        no_ai.mob_entity.set_no_ai(true);
        hurt(&no_ai, 0.1, true, true);
        let killing = wet(4.0);
        hurt(&killing, 4.0, true, true);
        let resting = wet(4.0);
        resting.set_playing_dead(true);
        resting.mob_entity.with_brain(resting.as_ref(), |brain| {
            brain.set(types::PLAY_DEAD_TICKS, 77);
        });
        hurt(&resting, 0.1, true, true);
        world.level.shutdown().await;

        assert_eq!(play_dead_ticks(&hit), Some(200));
        assert_eq!(play_dead_ticks(&only_source), Some(200));
        assert_eq!(play_dead_ticks(&only_cause), Some(200));
        assert_eq!(play_dead_ticks(&dry_out), None);
        assert_eq!(play_dead_ticks(&out_of_water), None);
        assert_eq!(play_dead_ticks(&no_ai), None);
        assert_eq!(play_dead_ticks(&killing), None);
        assert_eq!(play_dead_ticks(&resting), Some(77));
        Ok(())
    }

    /// Playing dead takes the axolotl out of what monsters see as an enemy, and its idle sound
    /// is not played.
    #[tokio::test]
    async fn playing_dead_hides_it_from_enemies_and_silences_it() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let axolotl = axolotl(&world);

        let awake = (
            axolotl.can_be_seen_as_enemy(),
            axolotl.audible_ambient_sound(),
        );
        axolotl.set_playing_dead(true);
        let asleep = (
            axolotl.can_be_seen_as_enemy(),
            axolotl.audible_ambient_sound(),
        );
        axolotl.set_playing_dead(false);
        world.level.shutdown().await;

        assert_eq!(awake, (true, Some(Sound::EntityAxolotlIdleAir)));
        assert_eq!(asleep, (false, None));
        Ok(())
    }
}
