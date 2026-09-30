use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;

use crate::entity::experience_orb::ExperienceOrbEntity;
use crate::entity::item::ItemEntity;
use crate::entity::projectile::is_projectile;
use crate::world::World;
use crate::world::loot::LootContextParameters;
use crate::{
    entity::{Entity, EntityBase, living::LivingEntity, player::Player},
    server::Server,
};
use pumpkin_data::Block;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::Hand;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub struct FishingBobberEntity {
    pub entity: Entity,
    pub owner_id: i32,
    pub hooked_entity_id: AtomicI32,
    pub in_ground: AtomicBool,
    /// Vanilla `life`: ticks spent resting on the ground before the hook despawns.
    pub life: AtomicI32,
    pub bobbing: AtomicBool,
    pub wait_countdown: AtomicI32,
    pub bite_countdown: AtomicI32,
    pub hook_countdown: AtomicI32,
    pub fish_angle: AtomicCell<f32>,
    pub luck_bonus: i32,
    pub wait_time_reduction_ticks: i32,
}

impl FishingBobberEntity {
    const INERTIA: f64 = 0.92;
    const GRAVITY: f64 = 0.03;

    /// Creates a bobber owned by `owner`, thrown along the given `yaw` and `pitch`.
    pub fn new(
        entity: Entity,
        owner: &Player,
        yaw: f32,
        pitch: f32,
        luck_bonus: i32,
        wait_time_reduction_ticks: i32,
    ) -> Self {
        let owner_id = owner.living_entity.entity.entity_id;
        let owner_entity = &owner.living_entity.entity;

        let (origin, velocity) = throw_setup(
            yaw,
            pitch,
            // Vanilla `RandomSource.triangle(0.5, 0.0103365)`.
            (rand::random::<f64>() - rand::random::<f64>()) * 0.010_336_5 + 0.5,
        );

        let owner_pos = owner_entity.pos.load();
        entity.pos.store(Vector3::new(
            owner_pos.x + origin.x,
            owner_pos.y + owner_entity.get_eye_height(),
            owner_pos.z + origin.z,
        ));
        entity.yaw.store(yaw);
        entity.pitch.store(pitch);
        entity.head_yaw.store(yaw);
        // The client reads the owner id from the spawn packet's data field to render the line.
        entity.data.store(owner_id, Ordering::Relaxed);
        entity.velocity.store(velocity);

        let luck_bonus = luck_bonus.max(0);
        let wait_time_reduction_ticks = wait_time_reduction_ticks.max(0);
        // Vanilla `FishingBobberEntity.catchingFish` rolls `100..=600` minus the lure speed.
        let wait_countdown = rand::random_range(100..=600) - wait_time_reduction_ticks;

        Self {
            entity,
            owner_id,
            hooked_entity_id: AtomicI32::new(0),
            in_ground: AtomicBool::new(false),
            life: AtomicI32::new(0),
            bobbing: AtomicBool::new(false),
            wait_countdown: AtomicI32::new(wait_countdown),
            bite_countdown: AtomicI32::new(0),
            hook_countdown: AtomicI32::new(0),
            fish_angle: AtomicCell::new(0.0),
            luck_bonus,
            wait_time_reduction_ticks,
        }
    }

    /// Matches vanilla `FishingBobberEntity.use(ItemStack usedItem)`.
    pub fn reel_in(&self, player: &Player, used_item: &ItemStack) -> i32 {
        let world = self.entity.world.load();
        let mut damage = 0;
        let hooked_id = self.hooked_entity_id.load(Ordering::Relaxed);

        if hooked_id != 0
            && let Some(hooked) = world.get_entity_by_id(hooked_id)
        {
            let player_pos = player.get_entity().pos.load();
            let bobber_pos = self.entity.pos.load();
            let delta = player_pos - bobber_pos;
            let motion = delta.multiply(0.1, 0.1, 0.1);
            hooked.get_entity().add_velocity(motion);
            world.send_entity_status(&self.entity, EntityStatus::FishingRodReelIn, None);

            damage = if hooked.get_entity().entity_type == &EntityType::ITEM {
                3
            } else {
                5
            };
        } else if self.bite_countdown.load(Ordering::Relaxed) > 0 {
            let bobber_pos = self.entity.pos.load();
            let player_luck = player.living_entity.get_attribute_value(&Attributes::LUCK) as f32;
            let total_luck = self.luck_bonus as f32 + player_luck;

            let params = LootContextParameters {
                tool: Some(used_item.clone()),
                position: Some(bobber_pos),
                this_entity: Some(&EntityType::FISHING_BOBBER),
                luck: total_luck,
                ..Default::default()
            };

            let items = world
                .get_loot_table("gameplay/fishing")
                .map_or_else(Vec::new, |table| {
                    let seed = rand::random::<i64>();
                    table.generate_loot_with_context(seed, &params)
                });

            let loot = if items.is_empty() {
                vec![ItemStack::new(1, &Item::COD)]
            } else {
                items
            };

            for item_stack in loot {
                let player_pos = player.position();
                let delta = player_pos - bobber_pos;
                let dist_sq = delta.length_squared();
                let velocity = Vector3::new(
                    delta.x * 0.1,
                    delta.y * 0.1 + dist_sq.sqrt().sqrt() * 0.08,
                    delta.z * 0.1,
                );

                let entity = Entity::new(world.clone(), bobber_pos, &EntityType::ITEM);
                let mut item_event =
                    crate::plugin::api::events::entity::item_spawn::ItemSpawnEvent::new(
                        entity.entity_id,
                        bobber_pos,
                        item_stack.item.registry_key.to_string(),
                    );
                if let Some(server) = world.server.upgrade() {
                    server
                        .plugin_manager
                        .fire_blocking(&server, &mut item_event);
                }
                if !item_event.cancelled {
                    let item_entity = Arc::new(ItemEntity::new_with_velocity(
                        entity,
                        item_stack.clone(),
                        velocity,
                        0,
                    ));
                    world.spawn_entity(item_entity);
                }

                let xp = rand::random_range(1..=6);
                let mut xp_pos = player_pos;
                xp_pos.y += 0.5;
                xp_pos.z += 0.5;
                ExperienceOrbEntity::spawn(&world, xp_pos, xp);

                if item_stack.item.has_tag(&tag::Item::MINECRAFT_FISHES) {
                    player.increment_stat(
                        pumpkin_data::statistic::StatisticCategory::Custom,
                        pumpkin_data::statistic::CustomStatistic::FishCaught as i32,
                        1,
                    );
                }

                player.trigger_advancement(
                    crate::entity::player::advancement::trigger::AdvancementTrigger::FishedItem {
                        item_id: format!("minecraft:{}", item_stack.item.registry_key),
                    },
                );
            }

            damage = 1;
        }

        if self.in_ground.load(Ordering::Relaxed) {
            damage = 2;
        }

        damage
    }

    /// Advances the bobber a tick, moving it and running the fishing cycle.
    #[expect(clippy::too_many_lines)]
    pub fn process_tick(&self, caller: &dyn EntityBase) {
        let entity = self.get_entity();
        let world = entity.world.load();

        // Vanilla `FishingBobberEntity.tick`: drop the hook once the owner stops fishing.
        let Some(owner) = world.get_entity_by_id(self.owner_id) else {
            entity.remove();
            return;
        };
        let Some(player) = owner.cast_any().downcast_ref::<Player>() else {
            entity.remove();
            return;
        };
        if self.should_stop_fishing(player) {
            self.discard(player);
            return;
        }

        if self.in_ground.load(Ordering::Relaxed) {
            let life = self.life.fetch_add(1, Ordering::Relaxed) + 1;
            if life >= 1200 {
                self.discard(player);
            }
            return;
        }
        self.life.store(0, Ordering::Relaxed);

        let hooked_id = self.hooked_entity_id.load(Ordering::Relaxed);
        if hooked_id != 0 {
            if let Some(hooked) = world.get_entity_by_id(hooked_id) {
                if hooked.get_entity().removed.load(Ordering::Relaxed) {
                    self.hooked_entity_id.store(0, Ordering::Relaxed);
                } else {
                    let mut hooked_pos = hooked.get_entity().pos.load();
                    hooked_pos.y += f64::from(hooked.get_entity().height()) * 0.8;
                    entity.set_pos(hooked_pos);
                    return;
                }
            } else {
                self.hooked_entity_id.store(0, Ordering::Relaxed);
            }
        }

        let block_pos = entity.block_pos.load();
        let (fluid, state) = world.get_fluid_and_fluid_state(&block_pos);
        let water_height = if fluid.matches_type(&Fluid::WATER) {
            f64::from(world.get_fluid_height(&block_pos, fluid, &state))
        } else {
            0.0
        };
        let in_water = water_height > 0.0;

        let mut velocity = entity.velocity.load();
        let start_pos = entity.pos.load();

        // Vanilla `FishingBobberEntity.tick`: entering water turns the flying bobber
        // into a bobbing one and kills most of its momentum.
        if in_water && !self.bobbing.swap(true, Ordering::Relaxed) {
            entity.velocity.store(velocity.multiply(0.3, 0.2, 0.3));
            return;
        }

        if in_water {
            velocity = bob_velocity(
                velocity,
                start_pos.y,
                block_pos.0.y,
                water_height,
                rand::random::<f64>(),
            );

            // Vanilla `FishingBobberEntity.tick`: a biting bobber is pulled down every tick.
            if self.bite_countdown.load(Ordering::Relaxed) > 0 {
                velocity.y -= 0.1 * rand::random::<f64>() * rand::random::<f64>();
            }

            self.catching_fish(&world, &block_pos, &mut velocity);
        } else {
            self.bobbing.store(false, Ordering::Relaxed);
            velocity.y -= Self::GRAVITY;
        }

        velocity = velocity.multiply(Self::INERTIA, Self::INERTIA, Self::INERTIA);
        entity.velocity.store(velocity);

        let new_pos = start_pos.add(&velocity);

        let search_box = BoundingBox::new(
            Vector3::new(
                start_pos.x.min(new_pos.x),
                start_pos.y.min(new_pos.y),
                start_pos.z.min(new_pos.z),
            ),
            Vector3::new(
                start_pos.x.max(new_pos.x),
                start_pos.y.max(new_pos.y),
                start_pos.z.max(new_pos.z),
            ),
        )
        .expand(0.3, 0.3, 0.3);

        // Basic block collision to stop bobber
        let (block_cols, _) = world.get_block_collisions(search_box, caller);
        if !block_cols.is_empty() {
            self.in_ground.store(true, Ordering::Relaxed);
            entity.velocity.store(Vector3::new(0.0, 0.0, 0.0));
            return;
        }

        entity.set_pos(new_pos);

        let candidates = world.get_entities_at_box(&search_box);
        for cand in candidates {
            if cand.get_entity().entity_id == self.owner_id
                || cand.get_entity().entity_id == entity.entity_id
            {
                continue;
            }

            if is_projectile(cand.get_entity().entity_type) {
                continue;
            }

            let ebb = cand.get_entity().bounding_box.load().expand(0.3, 0.3, 0.3);
            if ebb.intersects(&search_box) {
                self.hooked_entity_id
                    .store(cand.get_entity().entity_id, Ordering::Relaxed);
                entity.set_synced_data(
                    pumpkin_data::tracked_data::fishing_bobber::HOOKED_ENTITY,
                    cand.get_entity().entity_id + 1,
                );
                return;
            }
        }
    }

    /// Vanilla `FishingBobberEntity.shouldStopFishing(Player)`.
    fn should_stop_fishing(&self, player: &Player) -> bool {
        let owner = player.get_entity();
        // Vanilla `Entity.canInteractWithLevel` gates the checks below.
        let can_interact = player.living_entity.health.load() > 0.0
            && !owner.removed.load(Ordering::Relaxed)
            && !player.is_spectator();
        if can_interact {
            let inventory = player.inventory();
            let holding_rod = inventory.held_item().item.id == Item::FISHING_ROD.id
                || inventory.get_stack_in_hand(Hand::Left).item.id == Item::FISHING_ROD.id;
            if holding_rod {
                let owner_pos = owner.pos.load();
                let bobber_pos = self.entity.pos.load();
                return owner_pos.squared_distance_to_vec(&bobber_pos) > 1024.0;
            }
        }
        true
    }

    /// Vanilla `FishingBobberEntity.remove`: clears `owner.fishing` and drops the hook.
    fn discard(&self, player: &Player) {
        player.fishing_bobber.store(-1, Ordering::Relaxed);
        self.entity.remove();
    }

    /// Matches vanilla `FishingBobberEntity.catchingFish(BlockPos)`.
    #[expect(clippy::too_many_lines)]
    fn catching_fish(&self, world: &World, block_pos: &BlockPos, velocity: &mut Vector3<f64>) {
        let entity = self.get_entity();
        let pos = entity.pos.load();

        let mut fishing_speed = 1;
        let above = block_pos.up();
        if rand::random::<f32>() < 0.25 && world.is_raining_at(&above) {
            fishing_speed += 1;
        }
        if rand::random::<f32>() < 0.5 && !world.can_see_sky(&above) {
            fishing_speed -= 1;
        }

        let nibble = self.bite_countdown.load(Ordering::Relaxed);
        if nibble > 0 {
            let next = nibble - 1;
            self.bite_countdown.store(next, Ordering::Relaxed);
            if next <= 0 {
                self.wait_countdown.store(0, Ordering::Relaxed);
                self.hook_countdown.store(0, Ordering::Relaxed);
                entity.set_synced_data(
                    pumpkin_data::tracked_data::fishing_bobber::DATA_BITING,
                    false,
                );
            }
        } else if self.hook_countdown.load(Ordering::Relaxed) > 0 {
            let hooked = self.hook_countdown.load(Ordering::Relaxed) - fishing_speed;
            self.hook_countdown.store(hooked, Ordering::Relaxed);
            if hooked > 0 {
                // Vanilla `RandomSource.triangle(0.0, 9.188)`: the approaching fish wanders.
                let angle = self.fish_angle.load()
                    + ((rand::random::<f64>() - rand::random::<f64>()) * 9.188) as f32;
                self.fish_angle.store(angle);
                let radians = f64::from(angle).to_radians();
                let angle_sin = radians.sin();
                let angle_cos = radians.cos();
                let fish_x = pos.x + angle_sin * f64::from(hooked) * 0.1;
                let fish_y = f64::from(block_pos.0.y) + 1.0;
                let fish_z = pos.z + angle_cos * f64::from(hooked) * 0.1;
                let splash_pos = BlockPos::floored(fish_x, fish_y - 1.0, fish_z);
                if world.get_block(&splash_pos) == &Block::WATER {
                    if rand::random::<f32>() < 0.15 {
                        world.spawn_particle(
                            Vector3::new(fish_x, fish_y - 0.1, fish_z),
                            Vector3::new(angle_sin as f32, 0.1, angle_cos as f32),
                            0.0,
                            1,
                            Particle::Bubble,
                        );
                    }

                    let x_movement = angle_sin as f32 * 0.04;
                    let z_movement = angle_cos as f32 * 0.04;
                    world.spawn_particle(
                        Vector3::new(fish_x, fish_y, fish_z),
                        Vector3::new(z_movement, 0.01, -x_movement),
                        1.0,
                        0,
                        Particle::Fishing,
                    );
                    world.spawn_particle(
                        Vector3::new(fish_x, fish_y, fish_z),
                        Vector3::new(-z_movement, 0.01, x_movement),
                        1.0,
                        0,
                        Particle::Fishing,
                    );
                }
            } else {
                world.play_sound_fine(
                    Sound::EntityFishingBobberSplash,
                    SoundCategory::Neutral,
                    &pos,
                    0.25,
                    1.0 + (rand::random::<f32>() - rand::random::<f32>()) * 0.4,
                );

                let width = entity.width();
                let count = (1.0 + width * 20.0) as i32;
                let particle_pos = Vector3::new(pos.x, pos.y + 0.5, pos.z);
                let particle_offset = Vector3::new(width, 0.0, width);
                world.spawn_particle(particle_pos, particle_offset, 0.2, count, Particle::Bubble);
                world.spawn_particle(particle_pos, particle_offset, 0.2, count, Particle::Fishing);

                self.bite_countdown
                    .store(rand::random_range(20..=40), Ordering::Relaxed);
                entity.set_synced_data(
                    pumpkin_data::tracked_data::fishing_bobber::DATA_BITING,
                    true,
                );
                // Vanilla applies the bite's downward kick in `onSyncedDataUpdated`.
                velocity.y = -0.4 * (rand::random::<f64>() * 0.4 + 0.6);
            }
        } else if self.wait_countdown.load(Ordering::Relaxed) > 0 {
            let wait = self.wait_countdown.load(Ordering::Relaxed) - fishing_speed;
            self.wait_countdown.store(wait, Ordering::Relaxed);
            let mut tease_chance = 0.15;
            if wait < 20 {
                tease_chance += f64::from(20 - wait) * 0.05;
            } else if wait < 40 {
                tease_chance += f64::from(40 - wait) * 0.02;
            } else if wait < 60 {
                tease_chance += f64::from(60 - wait) * 0.01;
            }

            if rand::random::<f64>() < tease_chance {
                let angle = f64::from(rand::random::<f32>() * 360.0).to_radians();
                let distance = 25.0 + rand::random::<f64>() * 35.0;
                let fish_x = pos.x + angle.sin() * distance * 0.1;
                let fish_y = f64::from(block_pos.0.y) + 1.0;
                let fish_z = pos.z + angle.cos() * distance * 0.1;
                let splash_pos = BlockPos::floored(fish_x, fish_y - 1.0, fish_z);
                if world.get_block(&splash_pos) == &Block::WATER {
                    world.spawn_particle(
                        Vector3::new(fish_x, fish_y, fish_z),
                        Vector3::new(0.1, 0.0, 0.1),
                        0.0,
                        2 + rand::random_range(0..2),
                        Particle::Splash,
                    );
                }
            }

            if wait <= 0 {
                self.fish_angle.store(rand::random::<f32>() * 360.0);
                self.hook_countdown
                    .store(rand::random_range(20..=80), Ordering::Relaxed);
            }
        } else {
            let wait = rand::random_range(100..=600) - self.wait_time_reduction_ticks;
            self.wait_countdown.store(wait, Ordering::Relaxed);
        }
    }
}

/// Vanilla `FishHookState.BOBBING`: damp the bobber toward the water surface.
fn bob_velocity(
    velocity: Vector3<f64>,
    y: f64,
    block_y: i32,
    water_height: f64,
    random: f64,
) -> Vector3<f64> {
    let mut force = y + velocity.y - f64::from(block_y) - water_height;
    if force.abs() < 0.01 {
        force += force.signum() * 0.1;
    }
    Vector3::new(
        velocity.x * 0.9,
        velocity.y - force * random * 0.2,
        velocity.z * 0.9,
    )
}

/// Vanilla `FishingBobberEntity(Player, Level, int, int)`: the throw origin offset (X/Z,
/// relative to the owner) and launch velocity for the owner's yaw and pitch.
fn throw_setup(yaw: f32, pitch: f32, random_triangle: f64) -> (Vector3<f64>, Vector3<f64>) {
    let yaw_rad = f64::from(yaw).to_radians();
    let pitch_rad = f64::from(pitch).to_radians();
    let y_sin = (-yaw_rad - std::f64::consts::PI).sin();
    let y_cos = (-yaw_rad - std::f64::consts::PI).cos();
    let x_cos = -(-pitch_rad).cos();
    let x_sin = (-pitch_rad).sin();

    let origin = Vector3::new(-y_sin * 0.3, 0.0, -y_cos * 0.3);

    let mut movement = Vector3::new(-y_sin, (-(x_sin / x_cos)).clamp(-5.0, 5.0), -y_cos);
    let distance = movement.length();
    let scale = 0.6 / distance + random_triangle;
    movement = movement.multiply(scale, scale, scale);
    (origin, movement)
}

impl EntityBase for FishingBobberEntity {
    fn get_owner_id(&self) -> Option<i32> {
        Some(self.owner_id)
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        self.process_tick(caller);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bob_settles_at_the_water_surface() {
        let block_y = 62;
        let water_height = 0.9;
        let surface = f64::from(block_y) + water_height;

        // From above and below the surface the bobber must converge instead of rising
        // out of the water and falling back in.
        for start in [surface + 2.0, surface - 1.0] {
            let mut y = start;
            let mut velocity = Vector3::new(0.0, 0.0, 0.0);
            for _ in 0..200 {
                velocity = bob_velocity(velocity, y, block_y, water_height, 0.5);
                velocity = velocity.multiply(
                    FishingBobberEntity::INERTIA,
                    FishingBobberEntity::INERTIA,
                    FishingBobberEntity::INERTIA,
                );
                y += velocity.y;
                assert!(
                    y - surface < 2.1,
                    "bobber rose {:.3} above the surface from {start}",
                    y - surface
                );
            }
            assert!(
                (y - surface).abs() < 0.2,
                "bobber settled at {y}, expected the surface at {surface}"
            );
        }
    }

    #[test]
    fn throw_uses_the_look_direction() {
        // Due +Z at the default rotation, spawned 0.3 blocks in front of the eye.
        let (origin, forward) = throw_setup(0.0, 0.0, 0.5);
        assert!((origin.z - 0.3).abs() < 1e-9 && origin.x.abs() < 1e-9);
        assert!(
            forward.x.abs() < 1e-9 && (forward.z - 1.1).abs() < 1e-9 && forward.y.abs() < 1e-9,
            "expected due +Z at 1.1, got {forward:?}"
        );

        // Looking east throws east and offsets the origin to -X.
        let (origin, east) = throw_setup(90.0, 0.0, 0.5);
        assert!((origin.x + 0.3).abs() < 1e-9 && origin.z.abs() < 1e-9);
        assert!(
            (east.x + 1.1).abs() < 1e-9 && east.z.abs() < 1e-9,
            "expected due -X at 1.1, got {east:?}"
        );
    }

    #[test]
    fn throw_clamps_steep_pitch() {
        let (_, down) = throw_setup(0.0, 90.0, 0.0);
        assert!(down.y < 0.0 && down.z > 0.0);
        assert!(
            (down.y / down.z + 5.0).abs() < 1e-9,
            "expected the -Y clamp, got {down:?}"
        );

        let (_, up) = throw_setup(0.0, -90.0, 0.0);
        assert!(up.y > 0.0 && up.z > 0.0);
        assert!(
            (up.y / up.z - 5.0).abs() < 1e-9,
            "expected the +Y clamp, got {up:?}"
        );
    }
}
