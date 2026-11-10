use super::ParrotEntity;
use crate::entity::{
    EntityBase,
    ai::goal::{Controls, Goal},
    mob::Mob,
    passive::tamable::TamableAnimal,
};
use pumpkin_nbt::compound::NbtCompound;
use std::sync::{Weak, atomic::Ordering::Relaxed};

const RIDE_COOLDOWN: i32 = 100;

pub(super) struct LandOnShoulderGoal {
    parrot: Weak<ParrotEntity>,
    perched: bool,
}

impl LandOnShoulderGoal {
    pub(super) const fn new(parrot: Weak<ParrotEntity>) -> Self {
        Self {
            parrot,
            perched: false,
        }
    }
}

impl Goal for LandOnShoulderGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(parrot) = self.parrot.upgrade() else {
            return false;
        };
        let e = parrot.get_entity();
        let world = e.world.load();
        let Some(owner) = parrot
            .get_owner()
            .and_then(|id| world.get_player_by_uuid(id))
        else {
            return false;
        };
        !parrot.is_ordered_to_sit()
            && e.tick_count.load(Relaxed) > RIDE_COOLDOWN
            && !owner.is_spectator()
            && !owner.get_entity().is_in_water()
            && !owner.get_entity().is_in_powder_snow()
            && !owner
                .abilities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .flying
    }
    fn start(&mut self, _mob: &dyn Mob) {
        self.perched = false;
    }
    fn tick(&mut self, _mob: &dyn Mob) {
        let Some(parrot) = self.parrot.upgrade() else {
            return;
        };
        let e = parrot.get_entity();
        if self.perched || parrot.is_in_sitting_pose() || e.is_leashed() {
            return;
        }
        let world = e.world.load();
        let Some(owner) = parrot
            .get_owner()
            .and_then(|id| world.get_player_by_uuid(id))
        else {
            return;
        };
        if e.bounding_box
            .load()
            .intersects(&owner.get_entity().bounding_box.load())
        {
            let mut nbt = NbtCompound::new();
            EntityBase::write_nbt(parrot.as_ref(), &mut nbt);
            nbt.put_string("id", "minecraft:parrot".to_string());
            if owner.accept_shoulder_parrot(nbt) {
                self.perched = true;
                e.remove();
            }
        }
    }
    fn can_stop(&self) -> bool {
        !self.perched
    }
    fn controls(&self) -> Controls {
        Controls::empty()
    }
}
