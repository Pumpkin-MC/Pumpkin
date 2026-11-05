use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::entity::EntityType;
use pumpkin_data::entity_variant::AxolotlVariant;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use rand::{Rng, RngExt, seq::IndexedRandom};

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob, BABY_START_AGE},
    ai::goal::{
        active_target::ActiveTargetGoal, breed::BreedGoal, escape_danger::EscapeDangerGoal,
        follow_parent::FollowParentGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
        swim::SwimGoal, tempt::TemptGoal, try_find_water::TryFindWaterGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{
        Mob, MobEntity,
        spawn::{EntitySpawnReason, SpawnGroupData},
    },
    passive::animal::Animal,
    player::Player,
};
use crate::world::World;

const TEMPT_ITEMS: &[&Item] = &[&Item::TROPICAL_FISH_BUCKET];

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

/// Reads the `Variant` tag the way vanilla's legacy codec does: a missing tag is the default
/// variant and an unknown id falls back to it too.
fn read_variant(nbt: &NbtCompound) -> AxolotlVariant {
    nbt.get_int("Variant")
        .map_or(AxolotlVariant::DEFAULT, AxolotlVariant::by_id)
}

pub struct AxolotlEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub variant: AtomicI32,
    pub playing_dead: AtomicBool,
    pub from_bucket: AtomicBool,
    pub play_dead_ticks: AtomicI32,
}

impl AxolotlEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let axolotl = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            variant: AtomicI32::new(AxolotlVariant::DEFAULT.id()),
            playing_dead: AtomicBool::new(false),
            from_bucket: AtomicBool::new(false),
            play_dead_ticks: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(axolotl);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(TryFindWaterGoal::new()));
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.5));
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.25, TEMPT_ITEMS, false)));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.25)));
            goal_selector.add_goal(5, Box::new(MeleeAttackGoal::new(1.2, false)));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));
        };

        {
            let target_selector = &mob_arc.mob_entity.target_selector;

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::DROWNED, false),
            );
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::GUARDIAN, false),
            );
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(
                    &mob_arc.mob_entity,
                    &EntityType::ELDER_GUARDIAN,
                    false,
                ),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::SQUID, false),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::GLOW_SQUID, false),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::COD, false),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::SALMON, false),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(
                    &mob_arc.mob_entity,
                    &EntityType::TROPICAL_FISH,
                    false,
                ),
            );
        };

        mob_arc
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

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_AXOLOTL_FOOD)
            || item_stack.item == &Item::TROPICAL_FISH_BUCKET
            || item_stack.item == &Item::TROPICAL_FISH
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

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        let play_dead_ticks = self.play_dead_ticks.load(Ordering::Relaxed);
        if play_dead_ticks > 0 {
            let remaining = play_dead_ticks - 1;
            self.play_dead_ticks.store(remaining, Ordering::Relaxed);
            if remaining == 0 {
                self.set_playing_dead(false);
            }
        }
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

    use pumpkin_util::{math::vector3::Vector3, text::TextComponent};
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
}
