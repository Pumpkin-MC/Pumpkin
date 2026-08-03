use std::pin::Pin;

use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_protocol::java::client::play::CWorldEvent;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::{Vector2, to_chunk_pos};
use pumpkin_util::math::vector3::Vector3;

use crate::block::BonemealArgs;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;

/// Bone meal fertilizer. Dispatches to the clicked block's bone-meal hooks on
/// [`crate::block::BlockBehaviour`] (`is_bonemeal_target` / `is_bonemeal_success` /
/// `perform_bonemeal`).
///
/// Mirrors vanilla `BoneMealItem`: the stack is consumed and the growth effect plays whenever the
/// target is *valid*, regardless of whether the success roll passed. Only the land ("grow crop")
/// path is implemented here; the underwater and dispenser paths are follow-up work.
pub struct BoneMealItem;

impl ItemMetadata for BoneMealItem {
    fn ids() -> Box<[u16]> {
        [Item::BONE_MEAL.id].into()
    }
}

impl ItemBehaviour for BoneMealItem {
    fn use_on_block<'a>(
        &'a self,
        item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        _block: &'a Block,
        server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();
            // Read the current block/state from the world once and use it for both the behaviour
            // lookup and the dispatch args, so a change to the block after the interaction packet
            // was received can't leave them mismatched.
            let (block, state_id) = world.get_block_and_state_id(&location);
            let Some(pumpkin_block) = server.block_registry.get_pumpkin_block(block.id) else {
                return;
            };
            let args = BonemealArgs {
                world: &world,
                block,
                position: &location,
                state_id,
            };
            // Validity gates both consumption and the growth effect (vanilla puts the success
            // roll *inside* validity, so a valid-but-failed application still spends the item).
            if !pumpkin_block.is_bonemeal_target(args) {
                return;
            }
            if pumpkin_block.is_bonemeal_success(args) {
                pumpkin_block.perform_bonemeal(args).await;
            }
            item.decrement_unless_creative(player.gamemode.load(), 1);
            // World event 1505: `happy_villager` particles + the `item.bone_meal.use` sound in one
            // packet. `15` is the particle count, matching all four vanilla bone-meal call sites.
            world.broadcast_to_chunk(
                to_chunk_pos(&Vector2::new(location.0.x, location.0.z)),
                &CWorldEvent::new(
                    WorldEvent::ParticlesAndSoundPlantGrowth as i32,
                    location,
                    15,
                    false,
                ),
            );
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
