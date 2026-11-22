use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::{AxolotlVariantImpl, BucketEntityDataImpl};
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::entity_variant::AxolotlVariant;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use rand::{Rng, RngExt, seq::IndexedRandom};

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob, BABY_START_AGE},
    ai::{
        brain::{
            Brain, BrainTick,
            memory::{PackedMemories, types},
        },
        pathfinder::{Navigator, node::PathType},
    },
    bucketable::{Bucketable, bucket_mob_pickup},
    mob::{
        Mob, MobEntity,
        spawn::{EntitySpawnReason, SpawnGroupData},
    },
    passive::animal::Animal,
    player::Player,
};
use crate::world::World;

use super::water_animal;
use controls::{AxolotlLookControl, AxolotlMoveControl};

mod ai;
mod controls;

const MAX_HEAD_X_ROT: f32 = 1.0;
const MAX_HEAD_Y_ROT: f32 = 1.0;
const AXOLOTL_TOTAL_AIR_SUPPLY: i32 = 6000;
const REHYDRATE_AIR_SUPPLY: i32 = 1800;
const DRY_OUT_DAMAGE: f32 = 2.0;
const WATER_DRAG: f64 = 0.9;
const RARE_VARIANT_CHANCE: u32 = 1200;
const BABY_GROUP_SIZE: u32 = 2;

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

fn get_common_spawn_variant<R: Rng + ?Sized>(rng: &mut R) -> AxolotlVariant {
    get_spawn_variant(rng, true)
}

fn get_rare_spawn_variant<R: Rng + ?Sized>(rng: &mut R) -> AxolotlVariant {
    get_spawn_variant(rng, false)
}

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

fn fed_item_remainder(item: &Item) -> Option<&'static Item> {
    (item == &Item::TROPICAL_FISH_BUCKET).then_some(&Item::WATER_BUCKET)
}

fn read_variant(nbt: &NbtCompound) -> AxolotlVariant {
    nbt.get_int("Variant")
        .map_or(AxolotlVariant::DEFAULT, AxolotlVariant::by_id)
}

fn as_axolotl(mob: &dyn Mob) -> Option<&AxolotlEntity> {
    mob.cast_any().downcast_ref::<AxolotlEntity>()
}

const TOTAL_PLAYDEAD_TIME: i32 = 200;
const PLAYER_REGEN_DETECTION_RANGE: f64 = 20.0;
const REGEN_BUFF_MAX_DURATION: i32 = 2400;
const REGEN_BUFF_BASE_DURATION: i32 = 100;

struct HurtCheck {
    no_ai: bool,
    damage: f32,
    health: f32,
    max_health: f32,
    in_water: bool,
    has_attacker: bool,
    playing_dead: bool,
}

fn plays_dead_when_hurt<R: Rng + ?Sized>(rng: &mut R, hit: &HurtCheck) -> bool {
    !hit.no_ai
        && rng.random_range(0..3) == 0
        && (f32::from(rng.random_range(0..3i16)) < hit.damage || hit.health / hit.max_health < 0.5)
        && hit.damage < hit.health
        && hit.in_water
        && hit.has_attacker
        && !hit.playing_dead
}

// A stronger regeneration stays: adding an effect here replaces the active one, vanilla keeps the stronger.
fn supporting_regeneration_duration(current: Option<(i32, i32)>) -> Option<i32> {
    if current.is_some_and(|(_, amplifier)| amplifier > 0) {
        return None;
    }
    let left = current.map(|(left, _)| left);
    let renewed = left.is_none_or(|left| (0..REGEN_BUFF_MAX_DURATION).contains(&left));
    renewed.then_some((REGEN_BUFF_BASE_DURATION + left.unwrap_or(0)).min(REGEN_BUFF_MAX_DURATION))
}

fn apply_supporting_effects(player: &Player, source: &dyn crate::entity::EntityBase) {
    let current = player
        .get_effect(&StatusEffect::REGENERATION)
        .map(|effect| (effect.duration, i32::from(effect.amplifier)));
    if let Some(duration) = supporting_regeneration_duration(current) {
        player.add_effect_with_source(
            Effect {
                effect_type: &StatusEffect::REGENERATION,
                duration,
                amplifier: 0,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            },
            source,
        );
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
        {
            let navigator = mob_entity
                .navigator
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *navigator = Navigator::amphibious(false);
            navigator.set_pathfinding_malus(PathType::Water, 0.0);
        };
        let axolotl = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            variant: AtomicI32::new(AxolotlVariant::DEFAULT.id()),
            playing_dead: AtomicBool::new(false),
            from_bucket: AtomicBool::new(false),
        };
        Arc::new(axolotl)
    }

    #[must_use]
    pub fn get_variant(&self) -> AxolotlVariant {
        AxolotlVariant::by_id(self.variant.load(Ordering::Relaxed))
    }

    pub fn set_variant(&self, variant: AxolotlVariant) {
        self.variant.store(variant.id(), Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::axolotl::DATA_VARIANT,
            VarInt(variant.id()),
        );
    }

    fn handle_air_supply(&self, pre_tick_air_supply: i32) {
        water_animal::handle_air_supply(
            self,
            pre_tick_air_supply,
            self,
            self.get_entity().is_in_water_or_rain(),
            self.get_max_air_supply(),
            DRY_OUT_DAMAGE,
            DamageType::DRY_OUT,
        );
    }

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

    fn audible_ambient_sound(&self) -> Option<Sound> {
        if self.is_playing_dead() {
            None
        } else {
            self.get_ambient_sound()
        }
    }

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
            apply_supporting_effects(&player, tick.mob);
        }
    }
}

impl Bucketable for AxolotlEntity {
    fn is_from_bucket(&self) -> bool {
        self.from_bucket.load(Ordering::Relaxed)
    }

    fn set_from_bucket(&self, from_bucket: bool) {
        self.from_bucket.store(from_bucket, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::axolotl::FROM_BUCKET,
            from_bucket,
        );
    }

    fn bucket_item(&self) -> &'static Item {
        &Item::AXOLOTL_BUCKET
    }

    fn pickup_sound(&self) -> Sound {
        Sound::ItemBucketFillAxolotl
    }

    fn save_to_bucket_tag(&self, bucket: &mut ItemStack) {
        self.save_default_data_to_bucket_tag(bucket);
        bucket.set_data_component(AxolotlVariantImpl {
            value: self.get_variant().name().into(),
        });
        let mut nbt = bucket
            .get_data_component::<BucketEntityDataImpl>()
            .and_then(|data| data.nbt.clone())
            .unwrap_or_default();
        nbt.put_int("Age", AgeableMob::get_age(self));
        nbt.put_bool(
            "AgeLocked",
            self.ageable_data.age_locked.load(Ordering::Relaxed),
        );
        if let Some(ticks) = self.mob_entity.with_brain(self, |brain| {
            brain.time_until_expiry(types::HAS_HUNTING_COOLDOWN.id())
        }) {
            nbt.put_long("HuntingCooldown", ticks);
        }
        bucket.set_data_component(BucketEntityDataImpl { nbt: Some(nbt) });
    }

    fn load_from_bucket_tag(&self, nbt: &NbtCompound) {
        self.load_default_data_from_bucket_tag(nbt);
        AgeableMob::set_age(self, nbt.get_int("Age").unwrap_or(0));
        self.ageable_data.age_locked.store(
            nbt.get_bool("AgeLocked").unwrap_or(false),
            Ordering::Relaxed,
        );
        self.mob_entity
            .with_brain(self, |brain| match nbt.get_long("HuntingCooldown") {
                Some(ticks) => brain.set_with_expiry(types::HAS_HUNTING_COOLDOWN, true, ticks),
                None => brain.erase(types::HAS_HUNTING_COOLDOWN.id()),
            });
    }
}

impl AgeableMob for AxolotlEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for AxolotlEntity {
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

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_AXOLOTL_FOOD)
    }

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

    fn as_bucketable(&self) -> Option<&dyn Bucketable> {
        Some(self)
    }

    fn mob_set_variant_name(&self, name: &str) {
        if let Some(variant) = AxolotlVariant::ALL.iter().find(|v| v.name() == name) {
            self.set_variant(*variant);
        }
    }

    fn remove_when_far_away(&self, _distance_sq: f64) -> bool {
        !self.is_from_bucket() && (**self.get_entity().custom_name.load()).is_none()
    }

    fn requires_custom_persistence(&self) -> bool {
        let entity = self.get_entity();
        entity.has_vehicle() || entity.is_leashed() || self.is_from_bucket()
    }

    fn check_spawn_obstruction(&self, world: &World) -> bool {
        super::water_animal::check_spawn_obstruction(self, world)
    }

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

    fn mob_max_air_supply(&self) -> i32 {
        AXOLOTL_TOTAL_AIR_SUPPLY
    }

    // Without AI the living air rules run instead, so the air of an axolotl on land refills.
    fn base_tick(&self) {
        if self.mob_entity.is_no_ai() {
            self.mob_entity.living_entity.tick_air_supply(self);
        } else {
            self.handle_air_supply(self.get_entity().air_supply.load(Ordering::Relaxed));
        }
    }

    fn get_walk_target_value(&self, _pos: &BlockPos) -> f32 {
        0.0
    }

    fn mob_is_pushed_by_fluids(&self) -> bool {
        false
    }

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

    fn mob_hurt_server(
        &self,
        amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> Option<bool> {
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
        None
    }

    fn mob_can_be_seen_as_enemy(&self) -> bool {
        !self.is_playing_dead()
    }

    fn play_ambient_sound(&self) {
        if let Some(sound) = self.audible_ambient_sound() {
            self.make_sound(sound);
        }
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(if self.get_entity().is_in_water() {
            Sound::EntityAxolotlIdleWater
        } else {
            Sound::EntityAxolotlIdleAir
        })
    }

    fn on_attack(&self, _target: &dyn EntityBase) {
        self.play_entity_sound(Sound::EntityAxolotlAttack, 1.0, 1.0);
    }

    fn get_max_look_pitch_change(&self) -> f32 {
        MAX_HEAD_X_ROT
    }

    fn get_max_head_rotation(&self) -> f32 {
        MAX_HEAD_Y_ROT
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        ai::make_brain(self, packed)
    }

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
        self.mob_entity.set_target(attack_target);
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

    fn mob_interact_with_hand(
        &self,
        player: &Arc<Player>,
        item_stack: &mut ItemStack,
        _hand: pumpkin_util::Hand,
    ) -> Option<bool> {
        bucket_mob_pickup(player, item_stack, self)
            .or_else(|| self.animal_interact(player, item_stack).then_some(true))
    }
}

#[cfg(test)]
mod tests {

    use pumpkin_data::environment_attribute::Activity;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::ai::test_support::world;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn axolotl(world: &Arc<World>) -> Arc<AxolotlEntity> {
        AxolotlEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::AXOLOTL,
        ))
    }

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

    fn air_supply(axolotl: &Arc<AxolotlEntity>) -> i32 {
        axolotl.get_entity().air_supply.load(Ordering::Relaxed)
    }

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

    #[tokio::test]
    async fn bucket_round_trip_preserves_variant_age_and_cooldown() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let source = axolotl(&world);
        source.set_variant(AxolotlVariant::Gold);
        source.get_entity().age.store(-24000, Ordering::Relaxed);
        source
            .ageable_data
            .age_locked
            .store(true, Ordering::Relaxed);
        source.mob_entity.with_brain(source.as_ref(), |brain| {
            brain.set_with_expiry(types::HAS_HUNTING_COOLDOWN, true, 1234);
        });

        let mut bucket = ItemStack::new(1, &Item::AXOLOTL_BUCKET);
        let bucketable = source
            .get_mob()
            .and_then(|mob| mob.as_bucketable())
            .expect("axolotl is Bucketable");
        bucketable.save_to_bucket_tag(&mut bucket);

        let variant_component = bucket
            .get_data_component::<AxolotlVariantImpl>()
            .expect("variant component saved");
        assert_eq!(variant_component.value.as_ref(), "gold");
        let data = bucket
            .get_data_component::<BucketEntityDataImpl>()
            .expect("bucket entity data saved");
        let tag = data.nbt.clone().expect("bucket NBT saved");
        assert_eq!(tag.get_int("Age"), Some(-24000));
        assert_eq!(tag.get_bool("AgeLocked"), Some(true));
        assert!(tag.get_long("HuntingCooldown").is_some());

        let restored = axolotl(&world);
        let restored_mob = restored.get_mob().expect("restored axolotl is a mob");
        restored_mob.set_variant_name(variant_component.value.as_ref());
        let restored_bucketable = restored_mob
            .as_bucketable()
            .expect("restored axolotl is Bucketable");
        restored_bucketable.load_from_bucket_tag(&tag);
        restored_bucketable.set_from_bucket(true);

        assert_eq!(restored.get_variant(), AxolotlVariant::Gold);
        assert_eq!(restored.get_entity().age.load(Ordering::Relaxed), -24000);
        assert!(restored.ageable_data.age_locked.load(Ordering::Relaxed));
        assert!(restored.from_bucket.load(Ordering::Relaxed));
        let cooldown = restored.mob_entity.with_brain(restored.as_ref(), |brain| {
            brain.time_until_expiry(types::HAS_HUNTING_COOLDOWN.id())
        });
        assert!(cooldown.is_some_and(|ticks| ticks > 0));

        let mut plain_tag = NbtCompound::new();
        plain_tag.put_int("Age", 100);
        restored_bucketable.load_from_bucket_tag(&plain_tag);
        assert_eq!(restored.get_entity().age.load(Ordering::Relaxed), 100);
        let has_cooldown = restored.mob_entity.with_brain(restored.as_ref(), |brain| {
            brain.has_memory_value(types::HAS_HUNTING_COOLDOWN.id())
        });
        assert!(!has_cooldown);

        world.level.shutdown().await;
        Ok(())
    }
}
