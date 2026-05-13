use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crate::entity::projectile::{ProjectileHit, is_projectile};
use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture, NBTStorage, living::LivingEntity, player::Player,
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
            wait_countdown: AtomicI32::new(rand::random::<i32>().abs() % 600 + 100),
            bite_countdown: AtomicI32::new(0),
        }
    }

    pub async fn reel_in(&self, player: &Player) -> i32 {
        let world = self.entity.world.load();
        let hooked_id = self.hooked_entity_id.load(Ordering::Relaxed);

        if hooked_id != 0
            && let Some(hooked) = world.get_entity_by_id(hooked_id)
        {
            let player_pos = player.living_entity.entity.pos.load();
            let hooked_pos = hooked.get_entity().pos.load();
            let delta = player_pos - hooked_pos;
            let motion =
                delta
                    .multiply(0.1, 0.1, 0.1)
                    .add_raw(0.0, delta.length().sqrt() * 0.08, 0.0);
            hooked.get_entity().add_velocity(motion).await;
            return 1;
        }

        if self.bite_countdown.load(Ordering::Relaxed) > 0 {
            // Caught something!
            // TODO: Use actual loot tables. For now, just give a raw cod.
            use pumpkin_data::item::Item;
            let _item_stack = ItemStack::new(1, &Item::COD);
            // player.inventory().add_item(item_stack).await; // Need public add_item

            world
                .play_sound(
                    Sound::EntityExperienceOrbPickup,
                    SoundCategory::Neutral,
                    &player.position(),
                )
                .await;
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

            let bite = self.bite_countdown.load(Ordering::Relaxed);
            if bite > 0 {
                self.bite_countdown.store(bite - 1, Ordering::Relaxed);
                if bite % 5 == 0 {
                    world
                        .spawn_particle(
                            entity.pos.load(),
                            Vector3::new(0.1f32, 0.1f32, 0.1f32),
                            0.0,
                            5,
                            pumpkin_data::particle::Particle::Bubble,
                        )
                        .await;
                }
            } else {
                let wait = self.wait_countdown.load(Ordering::Relaxed);
                if wait > 0 {
                    self.wait_countdown.store(wait - 1, Ordering::Relaxed);
                } else {
                    // Start bite
                    self.bite_countdown.store(40, Ordering::Relaxed);
                    self.wait_countdown
                        .store(rand::random::<i32>().abs() % 600 + 100, Ordering::Relaxed);

                    world
                        .play_sound(
                            Sound::EntityFishingBobberSplash,
                            SoundCategory::Neutral,
                            &entity.pos.load(),
                        )
                        .await;
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
                entity
                    .send_meta_data(&[Metadata::new(
                        TrackedData::HOOKED_ENTITY,
                        MetaDataType::INT,
                        cand.get_entity().entity_id + 1,
                    )])
                    .await;
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
