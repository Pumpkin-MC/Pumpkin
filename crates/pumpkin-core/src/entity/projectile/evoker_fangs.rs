use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityStatus;
use pumpkin_data::sound::Sound;
use pumpkin_protocol::bedrock::server::ActorEventID;

use crate::{
    entity::{Entity, EntityBase, projectile::owner::EntityReference},
    server::Server,
};

pub struct EvokerFangsEntity {
    pub entity: Entity,
    pub warmup_delay_ticks: AtomicI32,
    pub life_ticks: AtomicI32,
    pub owner: EntityReference,
    pub sent_spike_event: AtomicBool,
}

impl EvokerFangsEntity {
    #[must_use]
    pub fn new(entity: Entity, warmup_ticks: u32, yaw: f32, owner: Option<&Entity>) -> Self {
        entity.set_rotation(yaw, 0.0);
        Self {
            entity,
            warmup_delay_ticks: AtomicI32::new(warmup_ticks as i32),
            life_ticks: AtomicI32::new(22),
            owner: EntityReference::new(owner),
            sent_spike_event: AtomicBool::new(false),
        }
    }
}

impl EntityBase for EvokerFangsEntity {
    fn write_custom_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        nbt.put_int("Warmup", self.warmup_delay_ticks.load(Ordering::Relaxed));
        self.owner.write_nbt(nbt, "Owner");
    }

    fn read_custom_nbt(&self, nbt: &pumpkin_nbt::compound::NbtCompound) {
        if let Some(warmup) = nbt.get_int("Warmup") {
            self.warmup_delay_ticks.store(warmup, Ordering::Relaxed);
        }
        self.owner.read_nbt(nbt, "Owner");
    }

    fn remove_custom_nbt(&self, removed: &[Box<str>]) {
        if removed.iter().any(|key| &**key == "Owner") {
            self.owner.clear();
        }
    }

    fn tick(&self, _caller: &dyn EntityBase, _server: &Server) {
        let entity = &self.entity;
        let world = entity.world.load();

        let warmup = self.warmup_delay_ticks.fetch_sub(1, Ordering::Relaxed) - 1;
        if warmup < 0 {
            if warmup == -8 {
                let bb = entity.bounding_box.load().expand(0.2, 0.0, 0.2);
                // Players count as targets, spectators don't.
                let candidates = world.get_all_at_box(&bb);
                let owner = self
                    .owner
                    .get(&world)
                    .filter(|owner| owner.get_living_entity().is_some());

                for cand in candidates {
                    let cand_ent = cand.get_entity();
                    if cand.get_living_entity().is_none()
                        || cand.is_spectator()
                        || !cand_ent.is_alive()
                        || cand_ent.invulnerable.load(Ordering::Relaxed)
                        || owner
                            .as_ref()
                            .is_some_and(|owner| owner.get_entity().entity_id == cand_ent.entity_id)
                    {
                        continue;
                    }
                    if let Some(owner) = &owner {
                        if owner.as_ref().is_allied_to(cand.as_ref()) {
                            continue;
                        }
                        cand.damage_with_context(
                            cand.as_ref(),
                            6.0,
                            DamageType::INDIRECT_MAGIC,
                            Some(entity.pos.load()),
                            Some(self),
                            Some(owner.as_ref()),
                        );
                    } else {
                        cand.damage(cand.as_ref(), 6.0, DamageType::MAGIC);
                    }
                }
            }

            if !self.sent_spike_event.swap(true, Ordering::SeqCst) {
                world.send_entity_status(
                    entity,
                    EntityStatus::StartAttacking,
                    ActorEventID::StartAttacking,
                );
                if !entity.silent.load(Ordering::Relaxed) {
                    entity.play_sound(Sound::EntityEvokerFangsAttack);
                }
            }

            let life = self.life_ticks.fetch_sub(1, Ordering::Relaxed) - 1;
            if life < 0 {
                entity.remove();
            }
        }
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
