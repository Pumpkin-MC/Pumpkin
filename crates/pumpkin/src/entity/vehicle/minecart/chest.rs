use std::sync::Arc;

use pumpkin_data::translation;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::text::TextComponent;

use crate::entity::{Entity, player::Player};

use super::container::{self, MinecartInventory};

pub(super) struct ChestMinecart {
    inventory: Arc<MinecartInventory>,
}

impl ChestMinecart {
    pub(super) fn new() -> Self {
        Self {
            inventory: Arc::new(MinecartInventory::new(27)),
        }
    }

    pub(super) const fn inventory(&self) -> &Arc<MinecartInventory> {
        &self.inventory
    }

    pub(super) async fn interact(&self, entity: &Entity, player: &Arc<Player>) -> bool {
        container::open(
            entity,
            player,
            &self.inventory,
            TextComponent::translate_cross(
                translation::java::ENTITY_MINECRAFT_CHEST_MINECART,
                translation::bedrock::ENTITY_CHEST_MINECART_NAME,
                [],
            ),
            false,
        )
        .await
    }

    pub(super) async fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.inventory.write_nbt(nbt).await;
    }

    pub(super) async fn read_nbt(&self, nbt: &NbtCompound) {
        self.inventory.read_nbt(nbt).await;
    }
}
