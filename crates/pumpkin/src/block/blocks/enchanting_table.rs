use std::sync::Arc;

use crate::block::entities::enchanting_table::EnchantingTableBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, BlockFuture, NormalUseArgs, PlacedArgs};
use pumpkin_data::{Block, BlockStateId, translation};
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
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let entity = EnchantingTableBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            // EnchantingTableBlock.BOOKSHELF_OFFSETS / isValidBookShelf (26.2 decompile): every
            // (x,y,z) with x,z in -2..=2, y in 0..=1 and (|x| == 2 or |z| == 2) is a candidate
            // shelf position, gated by a single-height air gap at (x/2, y, z/2) — integer
            // division truncating toward zero, same as Java's, so the gap sits exactly between
            // the table and the shelf.
            let mut bookshelf_count = 0;
            for off_x in -2i32..=2 {
                for off_z in -2i32..=2 {
                    if off_x.abs() != 2 && off_z.abs() != 2 {
                        continue;
                    }
                    for off_y in 0..=1 {
                        let gap = args.position.add(off_x / 2, off_y, off_z / 2);
                        if args.world.get_block_state(&gap).id != BlockStateId::AIR {
                            continue;
                        }
                        if Self::is_bookshelf(args.world, &args.position.add(off_x, off_y, off_z)) {
                            bookshelf_count += 1;
                        }
                    }
                }
            }
            // EnchantmentHelper.getEnchantmentCost (EnchantmentHelper.java:510-512) clamps the
            // raw count to 15 before using it; done here since Pumpkin's screen handler doesn't
            // re-clamp its own `bookshelf_count` field.
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
    fn is_bookshelf(world: &Arc<crate::world::World>, pos: &BlockPos) -> bool {
        let state = world.get_block_state(pos);
        let block = pumpkin_data::Block::from_state_id(state.id);
        block == &Block::BOOKSHELF
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
