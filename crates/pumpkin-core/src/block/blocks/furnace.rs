use std::sync::Arc;
use std::sync::Mutex;

use crate::block::entities::{PropertyDelegate, furnace_like_block_entity::ExperienceContainer};
use pumpkin_data::{
    BlockStateId, block_properties::FurnaceLikeProperties, screen::WindowType, translation,
};
use pumpkin_inventory::Inventory;
use pumpkin_inventory::{
    furnace_like::furnace_like_screen_handler::FurnaceLikeScreenHandler,
    player::player_inventory::PlayerInventory,
    screen_handler::{InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler},
};
use pumpkin_macros::pumpkin_block;

use crate::{
    block::{
        BlockBehaviour, BrokenArgs, GetComparatorOutputArgs, GetScreenHandlerFactoryArgs,
        NormalUseArgs, OnPlaceArgs, registry::BlockActionResult,
    },
    entity::experience_orb::ExperienceOrbEntity,
};

struct FurnaceScreenFactory {
    inventory: Arc<dyn Inventory>,
    property_delegate: Arc<dyn PropertyDelegate>,
    experience_container: Arc<dyn ExperienceContainer>,
}

impl FurnaceScreenFactory {
    fn new(
        inventory: Arc<dyn Inventory>,
        property_delegate: Arc<dyn PropertyDelegate>,
        experience_container: Arc<dyn ExperienceContainer>,
    ) -> Self {
        Self {
            inventory,
            property_delegate,
            experience_container,
        }
    }
}

impl ScreenHandlerFactory for FurnaceScreenFactory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        _player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        let concrete_handler = FurnaceLikeScreenHandler::new(
            sync_id,
            player_inventory,
            self.inventory.clone(),
            &self.property_delegate,
            self.experience_container.clone(),
            WindowType::Furnace,
        );

        let concrete_arc = Arc::new(Mutex::new(concrete_handler));

        Some(concrete_arc as SharedScreenHandler)
    }

    fn get_display_name(&self) -> pumpkin_util::text::TextComponent {
        pumpkin_macros::translate_cross!(
            translation::java::CONTAINER_FURNACE,
            translation::bedrock::CONTAINER_FURNACE
        )
    }
}

#[pumpkin_block("minecraft:furnace")]
pub struct FurnaceBlock;

impl BlockBehaviour for FurnaceBlock {
    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if let Some(factory) = self.get_screen_handler_factory(GetScreenHandlerFactoryArgs {
            server: args.server,
            world: args.world,
            block: args.block,
            position: args.position,
            player: args.player,
        }) {
            args.player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Custom,
                pumpkin_data::statistic::CustomStatistic::InteractWithFurnace as i32,
                1,
            );
            args.player
                .open_handled_screen(factory.as_ref(), Some(*args.position));
        }
        crate::block::registry::BlockActionResult::Success
    }

    fn get_screen_handler_factory(
        &self,
        args: GetScreenHandlerFactoryArgs<'_>,
    ) -> Option<Box<dyn ScreenHandlerFactory>> {
        let block_entity = args.world.get_block_entity(args.position)?;
        let inventory = block_entity.clone().get_inventory()?;
        let property_delegate = block_entity.clone().to_property_delegate()?;
        let experience_container = block_entity.to_experience_container()?;
        Some(Box::new(FurnaceScreenFactory::new(
            inventory,
            property_delegate,
            experience_container,
        )))
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = FurnaceLikeProperties::default(args.block);
        props.facing = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .opposite();

        props.to_state_id(args.block)
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        // Extract and drop accumulated XP as orbs before removing the block entity
        if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(experience_container) = block_entity.to_experience_container()
        {
            let xp = experience_container.extract_experience();
            if xp > 0 {
                let pos = args.position.to_f64();
                ExperienceOrbEntity::spawn(args.world, pos, xp as u32);
            }
        }
        args.world.remove_block_entity(args.position);
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        crate::block::container_comparator_output(&args)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn furnace_callback_preserves_the_registered_inventory()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::block::{
            BlockBehaviour, BlockMetadata, PlacedArgs, blocks::furnace::FurnaceBlock,
            registry::BlockRegistry,
        };
        use pumpkin_data::{Block, BlockId, item::Item, item_stack::ItemStack};
        use pumpkin_util::math::position::BlockPos;
        use pumpkin_world::chunk::ChunkData;
        use std::sync::Arc;

        struct ObservedFurnace;
        impl BlockMetadata for ObservedFurnace {
            fn ids() -> Box<[BlockId]> {
                FurnaceBlock::ids()
            }
        }
        impl BlockBehaviour for ObservedFurnace {
            fn placed(&self, args: PlacedArgs<'_>) {
                let world = args.world;
                let position = args.position;
                let before = world.get_block_entity(position);
                assert!(before.as_ref().is_some_and(|entity| {
                    entity.clone().get_inventory().is_some_and(|inventory| {
                        inventory.set_stack(0, ItemStack::new(3, &Item::IRON_ORE));
                        true
                    })
                }));
                FurnaceBlock.placed(args);
                let after = world.get_block_entity(position);
                let same_instance = before
                    .zip(after.clone())
                    .is_some_and(|(registered, placed)| Arc::ptr_eq(&registered, &placed));
                let retained_count = after
                    .and_then(crate::block::entities::BlockEntity::get_inventory)
                    .map(|inventory| inventory.get_stack(0))
                    .filter(|stack| stack.get_item() == &Item::IRON_ORE)
                    .map_or(0, |stack| stack.item_count);
                assert_eq!(
                    (same_instance, retained_count),
                    (true, 3),
                    "furnace placement must preserve the registered instance and its inventory"
                );
            }
        }

        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        let position = BlockPos::new(8, 64, 8);
        let chunk_pos = position.chunk_position();
        let chunk = ChunkData::empty_sync(chunk_pos.x, chunk_pos.y);
        let state = Block::FURNACE.default_state.id;
        chunk.set_block_absolute_y(8, 64, 8, state);
        world.level.loaded_chunks.insert(chunk_pos, chunk);
        assert!(world.get_block_entity(&position).is_none());

        let mut registry = BlockRegistry::default();
        registry.register(ObservedFurnace);
        registry.on_placed(
            &world,
            &Block::FURNACE,
            state,
            &position,
            Block::AIR.default_state.id,
            false,
        );
        let placed = world
            .get_block_entity(&position)
            .ok_or("missing placed furnace")?;
        let inventory = placed.get_inventory().ok_or("missing placed inventory")?;
        let stack = inventory.get_stack(0);
        assert_eq!(
            (stack.get_item(), stack.item_count),
            (&Item::IRON_ORE, 3),
            "the placement callback must run and retain the inserted stack"
        );
        world.level.shutdown().await;
        Ok(())
    }
}
