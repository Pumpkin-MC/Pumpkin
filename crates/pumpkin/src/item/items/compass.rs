use std::any::Any;
use std::future::Future;
use std::pin::Pin;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{DataComponentImpl, LodestoneTarget, LodestoneTrackerImpl};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub struct CompassItem;

impl ItemMetadata for CompassItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::COMPASS.id, Item::RECOVERY_COMPASS.id])
    }
}

impl ItemBehaviour for CompassItem {
    fn use_on_block<'a>(
        &'a self,
        item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if block.id == Block::LODESTONE.id && item.item.id == Item::COMPASS.id {
                let world = player.world();
                world.play_sound(
                    Sound::ItemLodestoneCompassLock,
                    SoundCategory::Players,
                    &location.to_f64(),
                );

                let mut lodestone_compass = ItemStack::new(1, &Item::COMPASS);
                let tracker = LodestoneTrackerImpl {
                    target: Some(LodestoneTarget {
                        dimension: world.dimension.minecraft_name.to_string(),
                        x: location.0.x,
                        y: location.0.y,
                        z: location.0.z,
                    }),
                    tracked: true,
                };
                lodestone_compass
                    .patch
                    .push((DataComponent::LodestoneTracker, Some(tracker.to_dyn())));

                item.decrement_unless_creative(player.gamemode.load(), 1);
                player
                    .inventory
                    .offer_or_drop_stack(lodestone_compass, player)
                    .await;
            }
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
