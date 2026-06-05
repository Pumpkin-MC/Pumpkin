use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};

use pumpkin_data::biome::Biome;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::Sound;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Difficulty;
use pumpkin_util::math::get_section_cord;
use pumpkin_util::math::position::BlockPos;

use crate::entity::{
    Entity, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};
use crate::world::World;

// Vanilla slime spawn rules (Mojang 1.21+). See Slime.java in Mojang mappings
// (mirrored in Paper's patches/sources/net/minecraft/world/entity/monster/Slime.java.patch).
/// Below this Y (exclusive), slime chunks may spawn slimes.
const SLIME_SPAWN_MAX_Y_SLIME_CHUNK: i32 = 40;
/// Lower Y bound (exclusive) for swamp surface slime spawns.
const SLIME_SPAWN_MIN_Y_SWAMP: i32 = 50;
/// Upper Y bound (exclusive) for swamp surface slime spawns.
const SLIME_SPAWN_MAX_Y_SWAMP: i32 = 70;
/// Vanilla `random.nextFloat() < 0.5F` gate for swamp surface spawns.
const SWAMP_SURFACE_SPAWN_CHANCE: f64 = 0.5;

/// Vanilla slime-chunk check (Mojang mappings, `SpawnPlacements#isSlimeChunk`).
///
/// Computes the same polynomial Mojang seeds a `RandomSource` with, then
/// returns whether `result % 10 == 0`. Roughly 1/10 chunks are slime chunks.
#[must_use]
pub fn is_slime_chunk(world_seed: u64, chunk_x: i32, chunk_z: i32) -> bool {
    let cx = i64::from(chunk_x);
    let cz = i64::from(chunk_z);
    let k = (world_seed as i64).wrapping_add(
        cx.wrapping_mul(cx)
            .wrapping_mul(0x4c1906)
            .wrapping_add(cx.wrapping_mul(0x5ac0db))
            .wrapping_add(cz.wrapping_mul(cz).wrapping_mul(0x4307a7))
            .wrapping_add(cz.wrapping_mul(0x5f24f)),
    );
    // Mirror the Java LCG truncation to 48 bits before taking the modulo.
    ((k as u64) & 0x0000_FFFF_FFFF_FFFF).is_multiple_of(10)
}

fn is_swamp_biome(biome: &'static Biome) -> bool {
    biome.registry_id == "minecraft:swamp" || biome.registry_id == "minecraft:mangrove_swamp"
}

pub struct SlimeEntity {
    entity: Arc<MobEntity>,
}

impl SlimeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let slime = Self {
            entity: Arc::new(mob_entity),
        };
        let mob_arc = Arc::new(slime);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.entity.target_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.entity, &EntityType::PLAYER, true),
            );
 };

        mob_arc
    }

    pub(crate) const fn hurt_sound_for_size(size: i32) -> Sound {
        if size == 1 {
            Sound::EntitySlimeHurtSmall
        } else {
            Sound::EntitySlimeHurt
        }
    }

    /// Vanilla slime spawn rules (Mojang 1.21+).
    ///
    /// Slimes spawn in the overworld via two independent paths, each
    /// matching a separate `if` branch in Mojang's `Slime.checkSlimeSpawnRules`:
    ///
    /// - **Slime chunk**: position Y is strictly below
    ///   [`SLIME_SPAWN_MAX_Y_SLIME_CHUNK`], the chunk is a slime chunk
    ///   (~1/10 of chunks, derived from the world seed via the same
    ///   polynomial Mojang uses for `WorldgenRandom.seedSlimeChunk`), and
    ///   a `random.nextInt(10) == 0` roll must pass. No light, biome, or
    ///   difficulty-gating beyond the overworld check.
    /// - **Swamp surface**: biome is in
    ///   `BiomeTags.ALLOWS_SURFACE_SLIME_SPAWNS` (vanilla: `minecraft:swamp`
    ///   and `minecraft:mangrove_swamp`), Y is strictly between
    ///   [`SLIME_SPAWN_MIN_Y_SWAMP`] and [`SLIME_SPAWN_MAX_Y_SWAMP`], a
    ///   `nextFloat() < 0.5` roll must pass, and the Mojang light check
    ///   `getMaxLocalRawBrightness(pos) <= nextInt(8)` must hold.
    ///
    /// Slimes never spawn in the nether or the end via natural spawning,
    /// and the world difficulty must not be peaceful.
    pub fn check_spawn_rules(world: &World, pos: &BlockPos) -> bool {
        if world.level_info.load().difficulty == Difficulty::Peaceful {
            return false;
        }

        if world.dimension != Dimension::OVERWORLD {
            return false;
        }

        // Swamp surface spawn.
        if is_swamp_biome(world.get_biome(pos))
            && pos.0.y > SLIME_SPAWN_MIN_Y_SWAMP
            && pos.0.y < SLIME_SPAWN_MAX_Y_SWAMP
            && rand::random_bool(SWAMP_SURFACE_SPAWN_CHANCE)
            && world.get_max_local_raw_brightness(pos) <= rand::random_range(0..8)
        {
            return true;
        }

        // Slime chunk spawn.
        if pos.0.y < SLIME_SPAWN_MAX_Y_SLIME_CHUNK
            && rand::random_range(0..10) == 0
            && is_slime_chunk(
                world.level.seed.0,
                get_section_cord(pos.0.x),
                get_section_cord(pos.0.z),
            )
        {
            return true;
        }

        false
    }
}

impl NBTStorage for SlimeEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.living_entity.write_nbt(nbt).await;
            nbt.put_int(
                "Size",
                self.entity
                    .living_entity
                    .entity
                    .data
                    .load(Ordering::Relaxed),
            );
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.living_entity.read_nbt_non_mut(nbt).await;
            self.entity
                .living_entity
                .entity
                .data
                .store(nbt.get_int("Size").unwrap_or(0), Ordering::Relaxed);
        })
    }
}

impl Mob for SlimeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_small_hurt_sound_only_for_smallest_slimes() {
        assert_eq!(
            SlimeEntity::hurt_sound_for_size(1),
            Sound::EntitySlimeHurtSmall
        );
        assert_eq!(SlimeEntity::hurt_sound_for_size(0), Sound::EntitySlimeHurt);
        assert_eq!(SlimeEntity::hurt_sound_for_size(2), Sound::EntitySlimeHurt);
    }

    #[test]
    fn slime_chunk_zero_zero_is_a_slime_chunk() {
        // seed=0, chunk (0,0): k = 0 + 0 + 0 + 0 + 0 = 0; 0 % 10 == 0
        assert!(is_slime_chunk(0, 0, 0));
    }

    #[test]
    fn slime_chunk_seed_one_at_origin_is_not_a_slime_chunk() {
        // seed=1, chunk (0,0): k = 1 + 0 + 0 + 0 + 0 = 1; 1 % 10 != 0
        assert!(!is_slime_chunk(1, 0, 0));
    }

    #[test]
    fn slime_chunk_is_deterministic() {
        // The function must be a pure function of (seed, chunk_x, chunk_z).
        let seeds = [0u64, 1, 42, 1_779_920_288_596_261_407];
        for seed in seeds {
            for cx in -2..=2 {
                for cz in -2..=2 {
                    assert_eq!(
                        is_slime_chunk(seed, cx, cz),
                        is_slime_chunk(seed, cx, cz),
                        "seed={seed}, chunk=({cx},{cz})"
                    );
                }
            }
        }
    }

    #[test]
    fn slime_chunk_produces_roughly_one_in_ten_chunks() {
        // Sanity-check the distribution over a 100x100 chunk window.
        // Vanilla gives exactly 10% so we allow a small tolerance for the
        // long-tail range, but the count should be very close to 1000.
        let seed = 1_779_920_288_596_261_407u64;
        let mut hits = 0u32;
        for cx in 0..100 {
            for cz in 0..100 {
                if is_slime_chunk(seed, cx, cz) {
                    hits += 1;
                }
            }
        }
        let total = 100 * 100;
        let ratio = f64::from(hits) / f64::from(total);
        assert!(
            (0.05..=0.15).contains(&ratio),
            "expected ~10% slime chunks, got {ratio} ({hits}/{total})"
        );
    }
}
