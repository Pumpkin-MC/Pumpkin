use std::sync::Arc;

use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, BlockFuture, NormalUseArgs};
use pumpkin_data::translation;
use pumpkin_inventory::enchanting::enchanting_screen_handler::EnchantingTableScreenHandler;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::{Inventory, SimpleInventory};
use tokio::sync::Mutex;

#[pumpkin_block("minecraft:enchanting_table")]
pub struct EnchantingTableBlock;

impl BlockBehaviour for EnchantingTableBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let mut bookshelf_count = 0;

            for off_z in -1..=1 {
                for off_x in -1..=1 {
                    if (off_z != 0 || off_x != 0)
                        && args.world.get_block_state(&args.position.add(off_x, 0, off_z)).await.id == 0 // Air
                        && args.world.get_block_state(&args.position.add(off_x, 1, off_z)).await.id == 0
                    // Air
                    {
                        for off_y in 0..=1 {
                            if self
                                .is_bookshelf(
                                    args.world,
                                    &args.position.add(off_x * 2, off_y, off_z * 2),
                                )
                                .await
                            {
                                bookshelf_count += 1;
                            }
                            if off_x != 0 && off_z != 0 {
                                if self
                                    .is_bookshelf(
                                        args.world,
                                        &args.position.add(off_x * 2, off_y, off_z),
                                    )
                                    .await
                                {
                                    bookshelf_count += 1;
                                }
                                if self
                                    .is_bookshelf(
                                        args.world,
                                        &args.position.add(off_x, off_y, off_z * 2),
                                    )
                                    .await
                                {
                                    bookshelf_count += 1;
                                }
                            }
                        }
                    }
                }
            }
            let bookshelf_count = bookshelf_count.min(15);

            args.player
                .open_handled_screen(
                    &EnchantingTableScreenFactory {
                        bookshelf_count,
                        seed: args.player.enchantment_seed(),
                    },
                    Some(*args.position),
                )
                .await;
            BlockActionResult::Success
        })
    }
}

impl EnchantingTableBlock {
    async fn is_bookshelf(&self, world: &Arc<crate::world::World>, pos: &BlockPos) -> bool {
        let state = world.get_block_state(pos).await;
        let block = pumpkin_data::Block::from_state_id(state.id);
        block.name == "bookshelf"
    }
}

struct EnchantingTableScreenFactory {
    bookshelf_count: i32,
    seed: i32,
}

impl ScreenHandlerFactory for EnchantingTableScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let inventory: Arc<dyn Inventory> = Arc::new(SimpleInventory::new(2));
            let handler = EnchantingTableScreenHandler::new(
                sync_id,
                player_inventory,
                &inventory,
                self.seed,
                self.bookshelf_count,
            );
            let screen_handler_arc = Arc::new(Mutex::new(handler));
            Some(screen_handler_arc as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::translate_cross(
            translation::java::CONTAINER_ENCHANT,
            translation::bedrock::CONTAINER_ENCHANT,
            &[],
        )
    }
}
