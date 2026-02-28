use std::sync::Arc;

use pumpkin_data::{
    block_properties::{BlockProperties, FurnaceLikeProperties},
    screen::WindowType,
    translation,
};
use pumpkin_inventory::{
    furnace_like::furnace_like_screen_handler::FurnaceLikeScreenHandler,
    player::player_inventory::PlayerInventory,
    screen_handler::{BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
use pumpkin_world::{
    BlockStateId,
    block::entities::{
        PropertyDelegate, furnace_like_block_entity::ExperienceContainer, smoker::SmokerBlockEntity,
    },
    inventory::Inventory,
};
use tokio::sync::Mutex;

use crate::{
    block::{
        BlockBehaviour, BlockFuture, BrokenArgs, NormalUseArgs, OnPlaceArgs, PlacedArgs,
        registry::BlockActionResult,
    },
    entity::experience_orb::ExperienceOrbEntity,
};

struct SmokerScreenFactory {
    inventory: Arc<dyn Inventory>,
    property_delegate: Arc<dyn PropertyDelegate>,
    experience_container: Arc<dyn ExperienceContainer>,
}

impl SmokerScreenFactory {
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

impl ScreenHandlerFactory for SmokerScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let concrete_handler = FurnaceLikeScreenHandler::new(
                sync_id,
                player_inventory,
                self.inventory.clone(),
                self.property_delegate.clone(),
                self.experience_container.clone(),
                WindowType::Smoker,
            )
            .await;

            let concrete_arc = Arc::new(Mutex::new(concrete_handler));

            Some(concrete_arc as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> pumpkin_util::text::TextComponent {
        TextComponent::translate(translation::CONTAINER_SMOKER, &[])
    }
}
#[pumpkin_block("minecraft:smoker")]
pub struct SmokerBlock;

impl BlockBehaviour for SmokerBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position).await
                && let Some(inventory) = block_entity.clone().get_inventory()
                && let Some(property_delegate) = block_entity.clone().to_property_delegate()
                && let Some(experience_container) = block_entity.to_experience_container()
            {
                let smoker_screen_factory =
                    SmokerScreenFactory::new(inventory, property_delegate, experience_container);
                args.player
                    .open_handled_screen(&smoker_screen_factory, Some(*args.position))
                    .await;
            }
            crate::block::registry::BlockActionResult::Consume
        })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = FurnaceLikeProperties::default(args.block);
            props.facing = args
                .player
                .living_entity
                .entity
                .get_horizontal_facing()
                .opposite();

            props.to_state_id(args.block)
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let smoker_block_entity = SmokerBlockEntity::new(*args.position);
            args.world
                .add_block_entity(Arc::new(smoker_block_entity))
                .await;
        })
    }

    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // Extract and drop accumulated XP as orbs before removing the block entity
            if let Some(block_entity) = args.world.get_block_entity(args.position).await
                && let Some(experience_container) = block_entity.to_experience_container()
            {
                let xp = experience_container.extract_experience();
                if xp > 0 {
                    let pos = args.position.to_f64();
                    ExperienceOrbEntity::spawn(args.world, pos, xp as u32).await;
                }
            }
            args.world.remove_block_entity(args.position).await;
        })
    }
}
