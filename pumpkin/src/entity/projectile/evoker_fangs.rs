use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};

use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage};
use crate::server::Server;
use crate::world::World;

/// Vanilla: `EvokerFangs`. A short-lived hazard that telegraphs for `warmup_delay_ticks`, then
/// bites once and lingers briefly before despawning.
pub struct EvokerFangsEntity {
    pub entity: Entity,
    warmup_delay_ticks: AtomicI32,
    life_ticks: AtomicI32,
    sent_spike_event: AtomicBool,
    owner_id: Option<i32>,
}

impl EvokerFangsEntity {
    /// Used when restoring from disk or `/summon`, where there is no casting owner.
    pub const fn orphan(entity: Entity) -> Self {
        Self {
            entity,
            warmup_delay_ticks: AtomicI32::new(0),
            life_ticks: AtomicI32::new(22),
            sent_spike_event: AtomicBool::new(false),
            owner_id: None,
        }
    }

    /// Vanilla: `EvokerFangs(Level, double, double, double, float, int, LivingEntity)`.
    /// `yaw_radians` matches the angle vanilla passes in (it stores yaw in degrees internally).
    pub fn new_spell(
        world: &Arc<World>,
        pos: Vector3<f64>,
        yaw_radians: f32,
        warmup_delay_ticks: i32,
        owner: &Entity,
    ) -> Self {
        let entity = Entity::new(world.clone(), pos, &EntityType::EVOKER_FANGS);
        entity.set_rotation(yaw_radians.to_degrees(), 0.0);
        Self {
            entity,
            warmup_delay_ticks: AtomicI32::new(warmup_delay_ticks),
            life_ticks: AtomicI32::new(22),
            sent_spike_event: AtomicBool::new(false),
            owner_id: Some(owner.entity_id),
        }
    }

    async fn bite_nearby(&self, caller: &Arc<dyn EntityBase>) {
        let world = self.entity.world.load();
        let bb = self.entity.bounding_box.load().expand(0.2, 0.0, 0.2);
        let owner = self.owner_id.and_then(|id| world.get_entity_by_id(id));

        for candidate in world.get_entities_at_box(&bb) {
            let target_entity = candidate.get_entity();
            if candidate.get_living_entity().is_none()
                || !target_entity.is_alive()
                || target_entity.invulnerable.load(Relaxed)
                || Some(target_entity.entity_id) == self.owner_id
            {
                continue;
            }

            // Scope reduction: vanilla also skips the target when `currentOwner.isAlliedTo`
            // (shared raid/illager team), which Pumpkin has no team/alliance system to check
            // here; only the direct owner is exempted.
            let damage_type = if owner.is_some() {
                DamageType::INDIRECT_MAGIC
            } else {
                DamageType::MAGIC
            };
            candidate
                .damage_with_context(
                    candidate.as_ref(),
                    6.0,
                    damage_type,
                    None,
                    owner.as_deref(),
                    Some(caller.as_ref()),
                )
                .await;
        }
    }
}

impl NBTStorage for EvokerFangsEntity {}

impl EntityBase for EvokerFangsEntity {
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        _server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let warmup = self.warmup_delay_ticks.fetch_sub(1, Relaxed) - 1;
            if warmup < 0 {
                // Vanilla: `ATTACK_TRIGGER_TICKS = 14` client-side counts down from `lifeTicks`
                // after the spike starts; server damage instead fires 8 ticks after warmup ends.
                if warmup == -8 {
                    self.bite_nearby(caller).await;
                }

                if !self.sent_spike_event.swap(true, Relaxed) {
                    self.entity
                        .world
                        .load()
                        .send_entity_status(&self.entity, EntityStatus::StartAttacking);
                }

                if self.life_ticks.fetch_sub(1, Relaxed) - 1 < 0 {
                    self.entity.remove().await;
                }
            }
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
