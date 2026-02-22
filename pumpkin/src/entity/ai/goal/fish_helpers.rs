use std::f64::consts::FRAC_PI_2;
use std::sync::atomic::Ordering::{Relaxed, SeqCst};

use pumpkin_data::{
    item::Item,
    sound::Sound,
    tag::{self, Taggable},
};
use pumpkin_util::{
    GameMode,
    math::{position::BlockPos, vector3::Vector3},
};
use pumpkin_world::item::ItemStack;
use rand::RngExt;

use crate::entity::{mob::Mob, player::Player};

pub const TARGET_REACHED_DISTANCE_SQ: f64 = 1.0;
const FLOP_CHANCE: f32 = 0.1;
const FLOP_XZ_PUSH: f64 = 0.05;
const FLOP_Y_PUSH: f64 = 0.4;

#[must_use]
pub fn has_reached_target(position: Vector3<f64>, target: Vector3<f64>) -> bool {
    position.squared_distance_to_vec(&target) <= TARGET_REACHED_DISTANCE_SQ
}

#[must_use]
pub fn is_in_water(mob: &dyn Mob) -> bool {
    mob.get_mob_entity()
        .living_entity
        .entity
        .touching_water
        .load(SeqCst)
}

/// Shared fish panic predicate: burning or recently damaged.
#[must_use]
pub fn is_in_danger(mob: &dyn Mob) -> bool {
    let mob_entity = mob.get_mob_entity();
    let entity = &mob_entity.living_entity.entity;
    let fire_ticks = entity.fire_ticks.load(Relaxed);
    if fire_ticks > 0 {
        return true;
    }

    let age = entity.age.load(Relaxed);
    let last_attacked_time = mob_entity.living_entity.last_attacked_time.load(Relaxed);
    last_attacked_time > 0 && (age - last_attacked_time) < 100
}

pub async fn set_move_target(mob: &dyn Mob, target: Vector3<f64>, speed: f64) {
    let mut move_control = mob.get_mob_entity().move_control.lock().await;
    move_control.set_wanted_position(target.x, target.y, target.z, speed);
}

pub async fn find_random_water_target(
    mob: &dyn Mob,
    horizontal_range: f64,
    vertical_range: i32,
    tries: usize,
) -> Option<Vector3<f64>> {
    let entity = &mob.get_mob_entity().living_entity.entity;
    let position = entity.pos.load();
    let world = entity.world.load();

    for _ in 0..tries {
        let (dx, dy, dz) = {
            let mut rng = mob.get_random();
            (
                rng.random_range(-horizontal_range..=horizontal_range),
                rng.random_range(-vertical_range..=vertical_range),
                rng.random_range(-horizontal_range..=horizontal_range),
            )
        };

        let candidate = BlockPos::new(
            (position.x + dx).floor() as i32,
            (position.y + dy as f64).floor() as i32,
            (position.z + dz).floor() as i32,
        );

        let fluid = world.get_fluid(&candidate).await;
        if !fluid.has_tag(&tag::Fluid::MINECRAFT_WATER) {
            continue;
        }

        let state = world.get_block_state(&candidate).await;
        if state.is_solid() {
            continue;
        }

        return Some(Vector3::new(
            f64::from(candidate.0.x) + 0.5,
            f64::from(candidate.0.y) + 0.5,
            f64::from(candidate.0.z) + 0.5,
        ));
    }

    None
}

/// Finds a random swimmable position in a cone pointing away from the threat.
pub async fn find_water_target_away_from(
    mob: &dyn Mob,
    threat_pos: Vector3<f64>,
    horizontal_range: f64,
    vertical_range: i32,
    tries: usize,
) -> Option<Vector3<f64>> {
    let entity = &mob.get_mob_entity().living_entity.entity;
    let mob_pos = entity.pos.load();
    let world = entity.world.load();
    let threat_to_mob_sq = threat_pos.squared_distance_to_vec(&mob_pos);

    let candidates = {
        let mut rng = mob.get_random();
        let dir_x = mob_pos.x - threat_pos.x;
        let dir_z = mob_pos.z - threat_pos.z;
        let (dir_x, dir_z) = if dir_x == 0.0 && dir_z == 0.0 {
            (rng.random_range(-1.0..1.0), rng.random_range(-1.0..1.0))
        } else {
            (dir_x, dir_z)
        };
        let base_angle = dir_z.atan2(dir_x) - FRAC_PI_2;

        let mut candidates = Vec::with_capacity(tries);
        for _ in 0..tries {
            let angle =
                base_angle + (2.0 * rng.random_range(0.0..1.0) - 1.0) * std::f64::consts::FRAC_PI_2;
            let t = rng.random_range(0.0..1.0f64).sqrt();
            let dist = t * horizontal_range * std::f64::consts::SQRT_2;
            let dx = -dist * angle.sin();
            let dz = dist * angle.cos();
            let dy = rng.random_range(-vertical_range..=vertical_range);
            candidates.push((dx, dy, dz));
        }
        candidates
    };

    for (dx, dy, dz) in candidates {
        if dx.abs() > horizontal_range || dz.abs() > horizontal_range {
            continue;
        }

        let candidate = BlockPos::new(
            (mob_pos.x + dx) as i32,
            (mob_pos.y + dy as f64) as i32,
            (mob_pos.z + dz) as i32,
        );

        let fluid = world.get_fluid(&candidate).await;
        if !fluid.has_tag(&tag::Fluid::MINECRAFT_WATER) {
            continue;
        }

        let state = world.get_block_state(&candidate).await;
        if state.is_solid() {
            continue;
        }

        let flee_vec = Vector3::new(
            f64::from(candidate.0.x) + 0.5,
            f64::from(candidate.0.y) + 0.5,
            f64::from(candidate.0.z) + 0.5,
        );

        if threat_pos.squared_distance_to_vec(&flee_vec) < threat_to_mob_sq {
            continue;
        }

        return Some(flee_vec);
    }

    None
}

pub async fn maybe_flop(mob: &dyn Mob, flop_sound: Sound) {
    let mob_entity = mob.get_mob_entity();
    let living = &mob_entity.living_entity;
    let entity = &living.entity;

    if entity.touching_water.load(SeqCst)
        || !entity.on_ground.load(SeqCst)
        || mob.get_random().random::<f32>() >= FLOP_CHANCE
    {
        return;
    }

    let (push_x, push_z) = {
        let mut rng = mob.get_random();
        (
            rng.random_range(-FLOP_XZ_PUSH..=FLOP_XZ_PUSH),
            rng.random_range(-FLOP_XZ_PUSH..=FLOP_XZ_PUSH),
        )
    };

    let mut velocity = entity.velocity.load();
    velocity.x += push_x;
    velocity.y = FLOP_Y_PUSH;
    velocity.z += push_z;
    entity.velocity.store(velocity);
    entity.on_ground.store(false, SeqCst);
    living.movement_input.store(Vector3::default());
    entity.play_sound(flop_sound).await;
}

pub async fn try_bucket_mob_pickup(
    mob: &dyn Mob,
    player: &Player,
    item_stack: &mut ItemStack,
    fish_bucket_item: &'static Item,
) -> bool {
    if item_stack.item.id != Item::WATER_BUCKET.id {
        return false;
    }

    let entity = &mob.get_mob_entity().living_entity.entity;
    entity.play_sound(Sound::ItemBucketFillFish).await;

    if player.gamemode.load() == GameMode::Creative {
        item_stack.item = fish_bucket_item;
        item_stack.item_count = 1;
        item_stack.patch.clear();
    } else {
        *item_stack = ItemStack::new(1, fish_bucket_item);
    }

    entity.remove().await;
    true
}
