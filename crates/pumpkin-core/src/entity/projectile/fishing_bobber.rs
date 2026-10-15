use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crate::entity::experience_orb::ExperienceOrbEntity;
use crate::entity::item::ItemEntity;
use crate::entity::projectile::{ProjectileHit, is_projectile};
use crate::{
    entity::{Entity, EntityBase, living::LivingEntity, player::Player},
    server::Server,
};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;

pub struct FishingBobberEntity {
    pub entity: Entity,
    pub owner_id: i32,
    pub hooked_entity_id: AtomicI32,
    pub in_ground: AtomicBool,
    pub has_hit: AtomicBool,
    pub wait_countdown: AtomicI32,
    pub bite_countdown: AtomicI32,
    pub luck: i32,
    pub lure_speed: i32,
}

impl FishingBobberEntity {
    const WATER_INERTIA: f64 = 0.8;
    const AIR_INERTIA: f64 = 0.92;
    const GRAVITY: f64 = 0.03;

    pub fn new(entity: Entity, owner: &Player, luck: i32, lure_speed: i32) -> Self {
        let yaw = owner.living_entity.entity.yaw.load();
        let pitch = owner.living_entity.entity.pitch.load();
        let yaw_rad = -yaw * (std::f32::consts::PI / 180.0) - std::f32::consts::PI;
        let pitch_rad = -pitch * (std::f32::consts::PI / 180.0);
        let y_cos = yaw_rad.cos();
        let y_sin = yaw_rad.sin();
        let x_cos = -pitch_rad.cos();
        let x_sin = pitch_rad.sin();

        let owner_pos = owner.living_entity.entity.pos.load();
        let spawn_pos = Vector3::new(
            owner_pos.x - f64::from(y_sin) * 0.3,
            owner_pos.y + f64::from(owner.living_entity.entity.get_eye_height()),
            owner_pos.z - f64::from(y_cos) * 0.3,
        );
        entity.pos.store(spawn_pos);
        entity.yaw.store(yaw);
        entity.pitch.store(pitch);

        let tan_pitch = if x_cos.abs() < 1e-6 {
            0.0
        } else {
            -(x_sin / x_cos)
        };
        let new_movement = Vector3::new(
            -f64::from(y_sin),
            f64::from(tan_pitch.clamp(-5.0, 5.0)),
            -f64::from(y_cos),
        );
        let dist = new_movement.length();
        let spread = 0.5 + (rand::random::<f64>() - rand::random::<f64>()) * 0.010_336_5;
        let factor = 0.6 / dist + spread;
        let vel = new_movement.multiply(factor, factor, factor);
        entity.velocity.store(vel);

        let luck = luck.max(0);
        let lure_speed = lure_speed.max(0);
        let initial_wait = (rand::random::<i32>().abs() % 501 + 100 - lure_speed).max(1);

        Self {
            entity,
            owner_id: owner.living_entity.entity.entity_id,
            hooked_entity_id: AtomicI32::new(0),
            in_ground: AtomicBool::new(false),
            has_hit: AtomicBool::new(false),
            wait_countdown: AtomicI32::new(initial_wait),
            bite_countdown: AtomicI32::new(0),
            luck,
            lure_speed,
        }
    }

    #[must_use]
    pub fn new_unowned(entity: Entity) -> Self {
        Self {
            entity,
            owner_id: 0,
            hooked_entity_id: AtomicI32::new(0),
            in_ground: AtomicBool::new(false),
            has_hit: AtomicBool::new(false),
            wait_countdown: AtomicI32::new(rand::random::<i32>().abs() % 501 + 100),
            bite_countdown: AtomicI32::new(0),
            luck: 0,
            lure_speed: 0,
        }
    }

    pub fn reel_in(&self, player: &Player) -> i32 {
        let world = self.entity.world.load();
        let hooked_id = self.hooked_entity_id.load(Ordering::Relaxed);

        if hooked_id != 0
            && let Some(hooked) = world.get_entity_by_id(hooked_id)
        {
            let player_pos = player.get_entity().pos.load();
            let hooked_pos = hooked.get_entity().pos.load();
            let delta = player_pos - hooked_pos;
            let motion =
                delta
                    .multiply(0.1, 0.1, 0.1)
                    .add_raw(0.0, delta.length().sqrt() * 0.08, 0.0);
            hooked.get_entity().add_velocity(motion);
            let is_item = hooked.get_entity().entity_type == &EntityType::ITEM;
            return if is_item { 3 } else { 5 };
        }

        if self.bite_countdown.load(Ordering::Relaxed) > 0 {
            self.bite_countdown.store(0, Ordering::Relaxed);
            self.entity.set_synced_data(
                pumpkin_data::tracked_data::fishing_bobber::BITING,
                false,
            );

            // Caught something!
            player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Custom,
                pumpkin_data::statistic::CustomStatistic::FishCaught as i32,
                1,
            );

            // Sample loot table: gameplay/fishing
            let params = crate::world::loot::LootContextParameters {
                luck: self.luck as f32,
                position: Some(self.entity.pos.load()),
                ..Default::default()
            };
            let items = if let Some(handle) = world.get_loot_table("gameplay/fishing") {
                crate::world::loot::generate_loot_from_handle(
                    &handle,
                    rand::random(),
                    &params,
                )
            } else {
                Vec::new()
            };

            let items = if items.is_empty() {
                vec![ItemStack::new(1, &pumpkin_data::item::Item::COD)]
            } else {
                items
            };

            let bobber_pos = self.entity.pos.load();
            let player_pos = player.get_entity().pos.load();
            let xa = player_pos.x - bobber_pos.x;
            let ya = player_pos.y - bobber_pos.y;
            let za = player_pos.z - bobber_pos.z;
            let dist_sq = xa * xa + ya * ya + za * za;
            let vel = Vector3::new(
                xa * 0.1,
                ya * 0.1 + dist_sq.sqrt().sqrt() * 0.08,
                za * 0.1,
            );

            for item_stack in items {
                player.trigger_advancement(
                    crate::entity::player::advancement::trigger::AdvancementTrigger::FishedItem {
                        item_id: format!("minecraft:{}", item_stack.item.registry_key),
                    },
                );

                let item_entity = Entity::new(world.clone(), bobber_pos, &EntityType::ITEM);
                let item = Arc::new(ItemEntity::new_with_velocity(
                    item_entity,
                    item_stack,
                    vel,
                    10,
                ));
                world.spawn_entity(item);
            }

            // Experience orb: random(1..=6)
            let xp = rand::random_range(1..=6);
            let orb_pos = Vector3::new(player_pos.x, player_pos.y + 0.5, player_pos.z);
            ExperienceOrbEntity::spawn(&world, orb_pos, xp);

            world.play_sound(
                Sound::EntityExperienceOrbPickup,
                SoundCategory::Neutral,
                &player.position(),
            );
            return 1;
        }

        if self.in_ground.load(Ordering::Relaxed) {
            return 2;
        }

        0
    }

    #[expect(clippy::too_many_lines)]
    pub fn process_tick(&self, caller: &dyn EntityBase) {
        let entity = self.get_entity();
        let world = entity.world.load();

        if self.in_ground.load(Ordering::Relaxed) {
            return;
        }

        let hooked_id = self.hooked_entity_id.load(Ordering::Relaxed);
        if hooked_id != 0 {
            if let Some(hooked) = world.get_entity_by_id(hooked_id) {
                if hooked.get_entity().removed.load(Ordering::Relaxed) {
                    self.hooked_entity_id.store(0, Ordering::Relaxed);
                    entity.set_synced_data(
                        pumpkin_data::tracked_data::fishing_bobber::HOOKED_ENTITY,
                        0,
                    );
                } else {
                    let mut hooked_pos = hooked.get_entity().pos.load();
                    hooked_pos.y += hooked.get_entity().get_eye_height() * 0.8;
                    entity.set_pos(hooked_pos);
                    return;
                }
            } else {
                self.hooked_entity_id.store(0, Ordering::Relaxed);
                entity.set_synced_data(
                    pumpkin_data::tracked_data::fishing_bobber::HOOKED_ENTITY,
                    0,
                );
            }
        }

        let mut velocity = entity.velocity.load();
        let start_pos = entity.pos.load();

        if entity.touching_water.load(Ordering::Relaxed) {
            velocity.y += 0.02; // Buoyancy

            let bite = self.bite_countdown.load(Ordering::Relaxed);
            if bite > 0 {
                let new_bite = bite - 1;
                self.bite_countdown.store(new_bite, Ordering::Relaxed);
                if new_bite == 0 {
                    self.entity.set_synced_data(
                        pumpkin_data::tracked_data::fishing_bobber::BITING,
                        false,
                    );
                    let new_wait =
                        (rand::random::<i32>().abs() % 501 + 100 - self.lure_speed).max(1);
                    self.wait_countdown.store(new_wait, Ordering::Relaxed);
                } else if new_bite % 5 == 0 {
                    world.spawn_particle(
                        entity.pos.load(),
                        Vector3::new(0.1f32, 0.1f32, 0.1f32),
                        0.0,
                        5,
                        pumpkin_data::particle::Particle::Bubble,
                    );
                }
            } else {
                let wait = self.wait_countdown.load(Ordering::Relaxed);
                if wait > 0 {
                    self.wait_countdown.store(wait - 1, Ordering::Relaxed);
                } else {
                    // Start bite
                    let bite_ticks = rand::random_range(20..=40);
                    self.bite_countdown.store(bite_ticks, Ordering::Relaxed);
                    self.entity.set_synced_data(
                        pumpkin_data::tracked_data::fishing_bobber::BITING,
                        true,
                    );

                    world.play_sound(
                        Sound::EntityFishingBobberSplash,
                        SoundCategory::Neutral,
                        &entity.pos.load(),
                    );
                }
            }
        } else {
            velocity.y -= Self::GRAVITY;
        }

        let inertia = if entity.touching_water.load(Ordering::Relaxed) {
            Self::WATER_INERTIA
        } else {
            Self::AIR_INERTIA
        };
        velocity = velocity.multiply(inertia, inertia, inertia);
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
    fn on_hit(&self, _hit: ProjectileHit) {
        self.has_hit.store(true, Ordering::Relaxed);
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        self.process_tick(caller);
    }
}
