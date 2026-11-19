use std::sync::atomic::Ordering;

use pumpkin_data::villager::VillagerProfession;

use super::super::super::memory::{MemoryStatus, types};
use super::super::OneShot;
use crate::entity::passive::villager::{VillagerData, VillagerEntity};

#[must_use]
pub fn create() -> OneShot {
    OneShot::new(
        "ResetProfession",
        vec![(types::JOB_SITE.id(), MemoryStatus::ValueAbsent)],
        move |tick| {
            let Some(villager) = tick.mob.cast_any().downcast_ref::<VillagerEntity>() else {
                return false;
            };
            let data = *villager
                .villager_data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let profession = data.profession_enum();
            let can_be_fired =
                profession != VillagerProfession::None && profession != VillagerProfession::Nitwit;
            if can_be_fired && villager.xp.load(Ordering::Relaxed) == 0 && data.level.0 <= 1 {
                villager.set_villager_data(VillagerData::new(
                    data.type_enum(),
                    VillagerProfession::None,
                    data.level.0,
                ));
                tick.mob.get_mob_entity().request_refresh_brain();
                true
            } else {
                false
            }
        },
    )
}
