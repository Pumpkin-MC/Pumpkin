use std::sync::Arc;
use std::sync::Mutex;

use crate::block::entities::enchanting_table::EnchantingTableBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, NormalUseArgs, PlacedArgs};
use pumpkin_data::{Block, BlockStateId, translation};
use pumpkin_inventory::enchanting::enchanting_screen_handler::EnchantingTableScreenHandler;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::{Inventory, SimpleInventory};

#[pumpkin_block("minecraft:enchanting_table")]
pub struct EnchantingTableBlock;

impl BlockBehaviour for EnchantingTableBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        let entity = EnchantingTableBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(entity));
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let mut bookshelf_count = 0;

        for off_z in -1..=1 {
            for off_x in -1..=1 {
                if (off_z != 0 || off_x != 0)
                    && args
                        .world
                        .get_block_state(&args.position.add(off_x, 0, off_z))
                        .id
                        == BlockStateId::AIR
                    && args
                        .world
                        .get_block_state(&args.position.add(off_x, 1, off_z))
                        .id
                        == BlockStateId::AIR
                // Air
                {
                    for off_y in 0..=1 {
                        if Self::is_bookshelf(
                            args.world,
                            &args.position.add(off_x * 2, off_y, off_z * 2),
                        ) {
                            bookshelf_count += 1;
                        }
                        if off_x != 0 && off_z != 0 {
                            if Self::is_bookshelf(
                                args.world,
                                &args.position.add(off_x * 2, off_y, off_z),
                            ) {
                                bookshelf_count += 1;
                            }
                            if Self::is_bookshelf(
                                args.world,
                                &args.position.add(off_x, off_y, off_z * 2),
                            ) {
                                bookshelf_count += 1;
                            }
                        }
                    }
                }
            }
        }
        let bookshelf_count = bookshelf_count.min(15);

        let seed = args.player.enchantment_seed();
        args.player.open_handled_screen(
            &EnchantingTableScreenFactory {
                bookshelf_count,
                seed,
            },
            Some(*args.position),
        );
        BlockActionResult::Success
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
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        _player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
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
    }

    fn get_display_name(&self) -> TextComponent {
        pumpkin_macros::translate_cross!(
            translation::java::CONTAINER_ENCHANT,
            translation::bedrock::CONTAINER_ENCHANT
        )
    }
}
