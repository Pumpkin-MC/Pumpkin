use super::MobEntity;
use crate::world::World;
use pumpkin_util::Difficulty;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_util::random::{RandomGenerator, get_seed};

impl MobEntity {
    pub fn is_dark_enough_to_spawn(world: &World, pos: &BlockPos, is_thundering: bool) -> bool {
        // Vanilla: raw SKY light vs random(0..32) — no ambient darken on this check.
        let sky_light = world.get_sky_light_level(pos);
        if sky_light > rand::random_range(0..32) {
            return false;
        }

        let dimension = &world.dimension;
        let block_light_limit = dimension.monster_spawn_block_light_limit;

        let block_light = world.get_block_light_level(pos).unwrap_or(0);
        if block_light_limit < 15 && block_light > block_light_limit {
            return false;
        }

        // Vanilla: thundering uses fixed ambient 10; otherwise world ambientDarkness.
        // Without sky darken, open sky stays at 15 all night and surface mobs never spawn.
        let ambient = if is_thundering {
            10
        } else {
            world.sky_darken.load(std::sync::atomic::Ordering::Relaxed)
        };
        let current_brightness = world.get_light_level_with_darken(pos, ambient);

        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));
        current_brightness <= dimension.monster_spawn_light_level.get(&mut random) as u8
    }

    pub fn check_monster_spawn_rules(world: &World, pos: &BlockPos, is_thundering: bool) -> bool {
        if world.level_info.load().difficulty == Difficulty::Peaceful {
            return false;
        }

        if !Self::is_dark_enough_to_spawn(world, pos, is_thundering) {
            return false;
        }

        //TODO:check_mob_spawn_rules(entity_type, world, spawn_reason, pos).await
        true
    }

    /// Vanilla `Monster.checkAnyLightMonsterSpawnRules` — peaceful only, no light gate.
    /// Used by husk (partial), silverfish, endermite, blaze in nether, etc.
    pub fn check_any_light_monster_spawn_rules(world: &World, _pos: &BlockPos) -> bool {
        world.level_info.load().difficulty != Difficulty::Peaceful
    }

    /// Vanilla `Monster.checkSurfaceMonstersSpawnRules` — dark + open sky.
    /// Used by stray / parched natural spawns.
    pub fn check_surface_monster_spawn_rules(
        world: &World,
        pos: &BlockPos,
        is_thundering: bool,
    ) -> bool {
        if !Self::check_monster_spawn_rules(world, pos, is_thundering) {
            return false;
        }
        // Approximate canSeeSky: full sky light at feet.
        world.get_sky_light_level(pos) >= 15
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile-time assertions that the public spawn-rule entry points survived
    // the module split (still reachable at `crate::entity::mob::MobEntity`).
    const _: fn(&World, &BlockPos, bool) -> bool =
        crate::entity::mob::MobEntity::is_dark_enough_to_spawn;
    const _: fn(&World, &BlockPos, bool) -> bool =
        crate::entity::mob::MobEntity::check_monster_spawn_rules;
    const _: fn(&World, &BlockPos) -> bool =
        crate::entity::mob::MobEntity::check_any_light_monster_spawn_rules;
    const _: fn(&World, &BlockPos, bool) -> bool =
        crate::entity::mob::MobEntity::check_surface_monster_spawn_rules;
}
