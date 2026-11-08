use std::sync::Arc;

use pumpkin_nbt::NbtCompound;
use pumpkin_util::text::TextComponent;

use crate::entity::{
    player::Player,
    vehicle::boat::container::{BoatInventory, BoatScreenFactory},
};

#[derive(Clone)]
pub(super) struct ChestBoat {
    inventory: Arc<BoatInventory>,
}

impl ChestBoat {
    pub(super) fn new() -> Self {
        Self {
            inventory: Arc::new(BoatInventory::new(27)),
        }
    }

    pub(super) const fn inventory(&self) -> &Arc<BoatInventory> {
        &self.inventory
    }

    pub(super) fn interact(
        &self,
        custom_name: Option<TextComponent>,
        default_boat_name: TextComponent,
        player: &Arc<Player>,
    ) -> bool {
        player
            .open_handled_screen(
                &BoatScreenFactory {
                    inventory: self.inventory.clone(),
                    title: custom_name.unwrap_or(default_boat_name),
                },
                None,
            )
            .is_some()
    }

    pub(super) fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.inventory.write_nbt(nbt);
    }

    pub(super) fn read_nbt(&self, nbt: &NbtCompound) {
        self.inventory.read_nbt(nbt);
    }
}
