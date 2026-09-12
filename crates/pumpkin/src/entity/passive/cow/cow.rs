use crate::entity::ageable::AgeableMob;
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::passive::animal::Animal;
use crate::entity::passive::cow::CowEntityBase;
use crate::entity::{Entity, EntityBase};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};

pub struct CowEntity {
    entity: Arc<CowEntityBase>,
    pub variant: AtomicU8,
}

impl CowEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = CowEntityBase::new(entity);
        let cow = Self {
            entity,
            variant: AtomicU8::new(1),
        };
        Arc::new(cow)
    }

    fn set_variant_and_sync(&self, variant: u8) {
        let entity = self.get_entity();
        self.variant.store(variant, Ordering::Relaxed);
        entity.set_synced_data(
            pumpkin_data::tracked_data::chicken::VARIANT,
            VarInt(self.variant.load(Ordering::Relaxed) as i32),
        );
    }

    fn get_variant(&self) -> u8 {
        self.variant.load(Ordering::Relaxed)
    }
}

impl AgeableMob for CowEntity {
    fn get_ageable_data(&self) -> &crate::entity::ageable::AgeableData {
        &self.entity.ageable_data
    }
}

impl Animal for CowEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        self.entity.is_food(item_stack)
    }
}

impl Mob for CowEntity {
    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::cow::BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::cow::VARIANT,
            VarInt(self.get_variant() as i32),
        );
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.entity.mob_write_nbt(nbt);
        let variant_str = match self.get_variant() {
            0 => "minecraft:cold",
            2 => "minecraft:warm",
            _ => "minecraft:temperate",
        };
        nbt.put_string("variant", variant_str.to_string());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.entity.mob_read_nbt(nbt);
        if let Some(variant_str) = nbt.get_string("variant") {
            let variant = match variant_str.trim_start_matches("minecraft:") {
                "cold" => 0,
                "warm" => 2,
                _ => 1,
            };
            self.set_variant_and_sync(variant);
        }
    }

    fn mob_interact(
        &self,
        player: &Arc<crate::entity::player::Player>,
        item_stack: &mut ItemStack,
    ) -> bool {
        self.entity.mob_interact(player, item_stack)
    }
}
