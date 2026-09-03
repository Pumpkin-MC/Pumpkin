use std::sync::Arc;
use std::sync::Mutex;

use crate::block::entities::{
    PropertyDelegate, blasting_furnace::BlastingFurnaceBlockEntity,
    furnace_like_block_entity::ExperienceContainer,
};
use pumpkin_data::{
    BlockStateId, block_properties::FurnaceLikeProperties, screen::WindowType, translation,
};
use pumpkin_inventory::{
    furnace_like::furnace_like_screen_handler::FurnaceLikeScreenHandler,
    player::player_inventory::PlayerInventory,
    screen_handler::{InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::inventory::Inventory;

use crate::{
    block::{
        BlockBehaviour, BrokenArgs, GetComparatorOutputArgs, GetScreenHandlerFactoryArgs,
        NormalUseArgs, OnPlaceArgs, PlacedArgs, registry::BlockActionResult,
    },
    entity::experience_orb::ExperienceOrbEntity,
};

struct BlastingFurnaceScreenFactory {
    inventory: Arc<dyn Inventory>,
    property_delegate: Arc<dyn PropertyDelegate>,
    experience_container: Arc<dyn ExperienceContainer>,
}

impl BlastingFurnaceScreenFactory {
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

impl ScreenHandlerFactory for BlastingFurnaceScreenFactory {
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
            WindowType::BlastFurnace,
        );

        let concrete_arc = Arc::new(Mutex::new(concrete_handler));

        Some(concrete_arc as SharedScreenHandler)
    }

    fn get_display_name(&self) -> pumpkin_util::text::TextComponent {
        pumpkin_macros::translate_cross!(
            translation::java::CONTAINER_BLAST_FURNACE,
            translation::bedrock::TILE_BLAST_FURNACE_NAME
        )
    }
}

#[pumpkin_block("minecraft:blast_furnace")]
pub struct BlastFurnaceBlock;

impl BlockBehaviour for BlastFurnaceBlock {
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
                pumpkin_data::statistic::CustomStatistic::InteractWithBlastFurnace as i32,
                1,
            );
            args.player
                .open_handled_screen(factory.as_ref(), Some(*args.position));
        }
        crate::block::registry::BlockActionResult::Consume
    }

    fn get_screen_handler_factory(
        &self,
        args: GetScreenHandlerFactoryArgs<'_>,
    ) -> Option<Box<dyn ScreenHandlerFactory>> {
        let block_entity = args.world.get_block_entity(args.position)?;
        let inventory = block_entity.clone().get_inventory()?;
        let property_delegate = block_entity.clone().to_property_delegate()?;
        let experience_container = block_entity.to_experience_container()?;
        Some(Box::new(BlastingFurnaceScreenFactory::new(
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

    fn placed(&self, args: PlacedArgs<'_>) {
        let blasting_furnace_block_entity = BlastingFurnaceBlockEntity::new(*args.position);
        args.world
            .add_block_entity(Arc::new(blasting_furnace_block_entity));
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
        if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(inventory) = block_entity.get_inventory()
        {
            Some(crate::block::calculate_comparator_output(
                inventory.as_ref(),
            ))
        } else {
            None
        }
    }
}
