use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crate::entity::projectile::{ProjectileHit, is_projectile};
use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture, NBTStorage, item::ItemEntity, living::LivingEntity,
        player::Player,
    },
    server::Server,
};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::java::client::play::Metadata;
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
    wait_adjusted: AtomicBool,
}

const fn hooked_reel_damage(is_item: bool) -> i32 {
    if is_item { 3 } else { 5 }
}

const fn fishing_experience_reward(random_value: u8) -> i32 {
    (random_value % 6) as i32 + 1
}

const fn fishing_wait_countdown(random_value: u32) -> i32 {
    (random_value % 100 + 100) as i32
}

const fn fishing_catch_item(random_value: u8) -> &'static pumpkin_data::item::Item {
    match random_value % 100 {
        0..60 => &pumpkin_data::item::Item::COD,
        60..85 => &pumpkin_data::item::Item::SALMON,
        85..98 => &pumpkin_data::item::Item::PUFFERFISH,
        _ => &pumpkin_data::item::Item::TROPICAL_FISH,
    }
}

impl FishingBobberEntity {
    const WATER_INERTIA: f64 = 0.8;
    const AIR_INERTIA: f64 = 0.92;
    const GRAVITY: f64 = 0.03;

    pub fn new(entity: Entity, owner: &Player) -> Self {
        let mut owner_pos = owner.living_entity.entity.pos.load();
        owner_pos.y += owner.living_entity.entity.get_eye_height() - 0.1;
        entity.pos.store(owner_pos);

        Self {
            entity,
            owner_id: owner.living_entity.entity.entity_id,
            hooked_entity_id: AtomicI32::new(0),
            in_ground: AtomicBool::new(false),
            has_hit: AtomicBool::new(false),
            wait_countdown: AtomicI32::new(fishing_wait_countdown(rand::random())),
            bite_countdown: AtomicI32::new(0),
            wait_adjusted: AtomicBool::new(false),
        }
    }

    pub async fn reel_in(&self, player: &Player) -> i32 {
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
            return hooked_reel_damage(
                hooked.get_entity().entity_type == &pumpkin_data::entity::EntityType::ITEM,
            );
        }

        if self.bite_countdown.load(Ordering::Relaxed) > 0 {
            // Caught something!
            player
                .increment_stat(
                    pumpkin_data::statistic::StatisticCategory::Custom,
                    pumpkin_data::statistic::CustomStatistic::FishCaught as i32,
                    1,
                )
                .await;

            let mut item_stack = ItemStack::new(1, fishing_catch_item(rand::random()));
            if !player
                .inventory
                .insert_stack_anywhere(&mut item_stack)
                .await
                && !item_stack.is_empty()
            {
                let item_entity = Entity::new(
                    world.clone(),
                    self.entity.pos.load(),
                    &pumpkin_data::entity::EntityType::ITEM,
                );
                world
                    .spawn_entity(Arc::new(ItemEntity::new(item_entity, item_stack)))
                    .await;
            }

            if let Some(owner) = world.get_player_by_id(self.owner_id) {
                owner
                    .add_experience_points(fishing_experience_reward(rand::random()))
                    .await;
            }

            world.play_sound(
                Sound::EntityExperienceOrbPickup,
                SoundCategory::Neutral,
                &player.position(),
            );
            return 1;
        }

        0
    }

    #[expect(clippy::too_many_lines)]
    pub async fn process_tick<'a>(&'a self, caller: &'a Arc<dyn EntityBase>, _server: &'a Server) {
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
                } else {
                    let mut hooked_pos = hooked.get_entity().pos.load();
                    hooked_pos.y += hooked.get_entity().get_eye_height() * 0.8;
                    entity.set_pos(hooked_pos);
                    return;
                }
            } else {
                self.hooked_entity_id.store(0, Ordering::Relaxed);
            }
        }

        let mut velocity = entity.velocity.load();
        let start_pos = entity.pos.load();

        if entity.touching_water.load(Ordering::Relaxed) {
            velocity.y += 0.02; // Buoyancy

            if !self.wait_adjusted.swap(true, Ordering::Relaxed) {
                let mut reduction = if world.is_raining_at(&entity.block_pos.load()).await {
                    20
                } else {
                    0
                };
                if let Some(owner) = world.get_player_by_id(self.owner_id) {
                    let held = owner.inventory.held_item();
                    reduction += held
                        .lock()
                        .await
                        .get_enchantment_level(&pumpkin_data::Enchantment::LURE)
                        .max(0)
                        * 20;
                }
                let wait = self.wait_countdown.load(Ordering::Relaxed);
                self.wait_countdown
                    .store((wait - reduction).max(0), Ordering::Relaxed);
            }

            let bite = self.bite_countdown.load(Ordering::Relaxed);
            if bite > 0 {
                self.bite_countdown.store(bite - 1, Ordering::Relaxed);
                if bite % 5 == 0 {
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
                    self.bite_countdown.store(40, Ordering::Relaxed);
                    self.wait_countdown
                        .store(fishing_wait_countdown(rand::random()), Ordering::Relaxed);

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
        let (block_cols, _) = world
            .get_block_collisions(search_box, caller.as_ref())
            .await;
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
                entity.send_meta_data(
                    &[Metadata::new(
                        TrackedData::HOOKED_ENTITY,
                        MetaDataType::INT,
                        cand.get_entity().entity_id + 1,
                    )],
                    None,
                );
                return;
            }
        }
    }
}

impl NBTStorage for FishingBobberEntity {}

impl EntityBase for FishingBobberEntity {
    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn on_hit(&self, _hit: ProjectileHit) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.has_hit.store(true, Ordering::Relaxed);
        })
    }

    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.process_tick(caller, server).await;
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        fishing_catch_item, fishing_experience_reward, fishing_wait_countdown, hooked_reel_damage,
    };

    #[test]
    fn fishing_wait_matches_vanilla_base_window() {
        assert_eq!(fishing_wait_countdown(0), 100);
        assert_eq!(fishing_wait_countdown(99), 199);
        assert_eq!(fishing_wait_countdown(100), 100);
        for value in [0, 1, 42, 99, 100, u32::MAX] {
            assert!((100..=199).contains(&fishing_wait_countdown(value)));
        }
    }

    #[test]
    fn hooked_retrieval_damage_matches_vanilla_categories() {
        assert_eq!(hooked_reel_damage(true), 3);
        assert_eq!(hooked_reel_damage(false), 5);
    }

    #[test]
    fn fishing_experience_reward_stays_within_vanilla_range() {
        for value in 0..=u8::MAX {
            assert!((1..=6).contains(&fishing_experience_reward(value)));
        }
    }

    #[test]
    fn fishing_catch_weights_match_vanilla_fish_distribution() {
        assert_eq!(fishing_catch_item(0).id, pumpkin_data::item::Item::COD.id);
        assert_eq!(fishing_catch_item(59).id, pumpkin_data::item::Item::COD.id);
        assert_eq!(
            fishing_catch_item(60).id,
            pumpkin_data::item::Item::SALMON.id
        );
        assert_eq!(
            fishing_catch_item(84).id,
            pumpkin_data::item::Item::SALMON.id
        );
        assert_eq!(
            fishing_catch_item(85).id,
            pumpkin_data::item::Item::PUFFERFISH.id
        );
        assert_eq!(
            fishing_catch_item(97).id,
            pumpkin_data::item::Item::PUFFERFISH.id
        );
        assert_eq!(
            fishing_catch_item(98).id,
            pumpkin_data::item::Item::TROPICAL_FISH.id
        );
    }
}
