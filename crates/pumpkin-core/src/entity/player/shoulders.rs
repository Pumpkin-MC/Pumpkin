use super::Player;
use crate::entity::{EntityBase, passive::parrot::ParrotEntity};
use pumpkin_data::{entity::EntityType, entity_variant::ParrotVariant, tracked_data::player};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use std::sync::atomic::Ordering::Relaxed;

const SHOULDER_AMBIENT_INTERVAL: i32 = 200;
const SHOULDER_RELEASE_Y_OFFSET: f32 = 0.7;
const SHOULDER_RELEASE_DELAY: i64 = 20;

#[derive(Default)]
pub(super) struct Shoulders {
    entities: [Option<NbtCompound>; 2],
    mounted_at: i64,
}

impl Shoulders {
    fn take_for_release(&mut self, now: i64) -> [Option<NbtCompound>; 2] {
        if self.mounted_at + SHOULDER_RELEASE_DELAY >= now {
            return [None, None];
        }
        // A cancelled spawn keeps its NBT and waits another release delay before retrying.
        self.mounted_at = now;
        std::mem::take(&mut self.entities)
    }
}

impl Player {
    fn sync_shoulder(&self, side: usize, nbt: Option<&NbtCompound>) {
        let variant = nbt
            .filter(|n| n.get_string("id") == Some("minecraft:parrot"))
            .map(|n| ParrotVariant::by_id(n.get_int("Variant").unwrap_or(0)));
        // OPTIONAL_UNSIGNED_INT uses zero for absent and id + 1 for present.
        self.get_entity().set_synced_data(
            if side == 0 {
                player::DATA_SHOULDER_PARROT_LEFT
            } else {
                player::DATA_SHOULDER_PARROT_RIGHT
            },
            VarInt(variant.map_or(0, |v| v.id() + 1)),
        );
    }

    pub(crate) fn accept_shoulder_parrot(&self, nbt: NbtCompound) -> bool {
        let e = self.get_entity();
        if e.has_vehicle() || !e.on_ground.load(Relaxed) || e.is_in_water() || e.is_in_powder_snow()
        {
            return false;
        }
        let mut shoulders = self
            .shoulders
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(side) = shoulders.entities.iter().position(Option::is_none) else {
            return false;
        };
        shoulders.entities[side] = Some(nbt);
        shoulders.mounted_at = self
            .world()
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .world_age;
        self.sync_shoulder(side, shoulders.entities[side].as_ref());
        true
    }

    pub(crate) fn release_shoulders(&self) {
        let mut shoulders = self
            .shoulders
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = self.world().get_world_age();
        let saved = shoulders.take_for_release(now);
        drop(shoulders);
        for (side, nbt) in saved.into_iter().enumerate() {
            if let Some(nbt) = nbt {
                let world = self.world();
                let pos = self.get_entity().pos.load()
                    + Vector3::new(0.0, f64::from(SHOULDER_RELEASE_Y_OFFSET), 0.0);
                let parrot = crate::entity::r#type::from_type(
                    &EntityType::PARROT,
                    pos,
                    &world,
                    nbt.get_uuid("UUID").unwrap_or_else(uuid::Uuid::new_v4),
                );
                parrot.read_nbt_non_mut(&nbt);
                parrot.get_entity().set_pos(pos);
                if let Some(tamable) = parrot.get_mob().and_then(|m| m.as_tamable()) {
                    tamable.set_owner(Some(self.gameprofile.id));
                }
                if world.spawn_entity(parrot) {
                    self.sync_shoulder(side, None);
                } else {
                    // A plugin cancelled the spawn: keep the parrot instead of losing it.
                    self.shoulders
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .entities[side] = Some(nbt);
                }
            }
        }
    }

    pub(super) fn tick_shoulders(&self) {
        let audible = {
            let shoulders = self
                .shoulders
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            shoulders.entities.each_ref().map(|nbt| {
                nbt.as_ref()
                    .is_some_and(|nbt| nbt.get_bool("Silent") != Some(true))
            })
        };
        for audible in audible {
            if audible
                && rand::rng().random_range(0..SHOULDER_AMBIENT_INTERVAL) == 0
                && !ParrotEntity::imitate_nearby(self)
            {
                // Vanilla `level.playSound(null, ...)`: the carrier hears it too.
                let world = self.world();
                world.play_sound_fine(
                    ParrotEntity::ambient(&world),
                    self.sound_source(),
                    &self.get_entity().pos.load(),
                    1.0,
                    ParrotEntity::pitch(),
                );
            }
        }
        let e = self.get_entity();
        let flying = self
            .abilities
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .flying;
        if self.living_entity.fall_distance.load() > 0.5
            || e.is_in_water()
            || flying
            || self.is_sleeping()
            || e.is_in_powder_snow()
            || self.living_entity.is_dead_or_dying()
            || self.is_spectator()
        {
            self.release_shoulders();
        }
    }

    pub(super) fn write_shoulders(&self, nbt: &mut NbtCompound) {
        let shoulders = self
            .shoulders
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (side, name) in ["ShoulderEntityLeft", "ShoulderEntityRight"]
            .iter()
            .enumerate()
        {
            if let Some(entity) = &shoulders.entities[side] {
                nbt.put_compound(name, entity.clone());
            }
        }
    }

    pub(super) fn read_shoulders(&self, nbt: &NbtCompound) {
        let mut shoulders = self
            .shoulders
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (side, name) in ["ShoulderEntityLeft", "ShoulderEntityRight"]
            .iter()
            .enumerate()
        {
            shoulders.entities[side] = nbt
                .get_compound(name)
                .filter(|n| n.get_string("id") == Some("minecraft:parrot"))
                .cloned();
            self.sync_shoulder(side, shoulders.entities[side].as_ref());
        }
        shoulders.mounted_at = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::Shoulders;
    use pumpkin_nbt::compound::NbtCompound;

    #[test]
    fn cancelled_shoulder_release_preserves_nbt_and_delays_retry() {
        let mut parrot = NbtCompound::new();
        parrot.put_int("Variant", 3);
        let mut shoulders = Shoulders {
            entities: [Some(parrot), None],
            mounted_at: 10,
        };
        assert!(shoulders.take_for_release(30).iter().all(Option::is_none));
        let saved = shoulders.take_for_release(31);
        assert_eq!(
            saved[0].as_ref().and_then(|nbt| nbt.get_int("Variant")),
            Some(3)
        );
        // Mirror the spawn cancellation path putting the entity back.
        shoulders.entities = saved;
        for now in 32..=51 {
            assert!(shoulders.take_for_release(now).iter().all(Option::is_none));
        }
        let retried = shoulders.take_for_release(52);
        assert_eq!(
            retried[0].as_ref().and_then(|nbt| nbt.get_int("Variant")),
            Some(3)
        );
        assert!(shoulders.entities.iter().all(Option::is_none));
    }
}
