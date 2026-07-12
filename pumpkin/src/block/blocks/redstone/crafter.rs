use std::sync::Arc;
use tokio::sync::Mutex;

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::entities::crafter::CrafterBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockFuture, GetComparatorOutputArgs, NormalUseArgs, OnNeighborUpdateArgs,
    OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
};
use pumpkin_data::block_properties::{
    BlockProperties, CrafterLikeProperties, HorizontalFacing, Orientation,
};
use pumpkin_data::sound::Sound;
use pumpkin_data::translation;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{BlockDirection, BlockStateId};
use pumpkin_inventory::generic_container_screen_handler::create_crafter_3x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

struct CrafterScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for CrafterScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let handler = create_crafter_3x3(sync_id, player_inventory, self.0.clone()).await;
            let screen_handler_arc = Arc::new(Mutex::new(handler));

            Some(screen_handler_arc as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::translate_cross(
            translation::java::CONTAINER_CRAFTER,
            translation::bedrock::CONTAINER_CRAFTER,
            &[],
        )
    }
}

#[pumpkin_block("minecraft:crafter")]
pub struct CrafterBlock;

impl BlockBehaviour for CrafterBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position)
                && let Some(inventory) = block_entity.get_inventory()
            {
                args.player
                    .open_handled_screen(&CrafterScreenFactory(inventory), Some(*args.position))
                    .await;
            }
            BlockActionResult::Success
        })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = CrafterLikeProperties::default(args.block);
            let facing = args.direction;
            let horizontal = args.player.living_entity.entity.get_horizontal_facing();
            props.orientation = match facing {
                BlockDirection::Down => match horizontal {
                    HorizontalFacing::North => Orientation::DownNorth,
                    HorizontalFacing::South => Orientation::DownSouth,
                    HorizontalFacing::East => Orientation::DownEast,
                    HorizontalFacing::West => Orientation::DownWest,
                },
                BlockDirection::Up => match horizontal {
                    HorizontalFacing::North => Orientation::UpNorth,
                    HorizontalFacing::South => Orientation::UpSouth,
                    HorizontalFacing::East => Orientation::UpEast,
                    HorizontalFacing::West => Orientation::UpWest,
                },
                BlockDirection::North => Orientation::NorthUp,
                BlockDirection::South => Orientation::SouthUp,
                BlockDirection::East => Orientation::EastUp,
                BlockDirection::West => Orientation::WestUp,
            };
            props.to_state_id(args.block)
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let crafter_block_entity = CrafterBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(crafter_block_entity));
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let powered = block_receives_redstone_power(args.world, args.position).await;
            let mut props = CrafterLikeProperties::from_state_id(
                args.world.get_block_state(args.position).id,
                args.block,
            );

            if powered && !props.triggered {
                props.triggered = true;
                args.world
                    .schedule_block_tick(args.block, *args.position, 4, TickPriority::Normal);
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
            } else if !powered && props.triggered {
                props.triggered = false;
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
            }
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let mut props = CrafterLikeProperties::from_state_id(
                args.world.get_block_state(args.position).id,
                args.block,
            );

            // Set to crafting state
            props.crafting = true;
            args.world
                .set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;

            // Recipes/crafting logic not fully implemented yet - play fail effects
            args.world.play_sound(
                Sound::BlockCrafterFail,
                pumpkin_data::sound::SoundCategory::Blocks,
                &args.position.to_f64(),
            );

            // Spawn fail smoke particles
            args.world.sync_world_event(
                WorldEvent::ParticlesShootSmoke,
                *args.position,
                match props.orientation {
                    Orientation::DownEast
                    | Orientation::DownNorth
                    | Orientation::DownSouth
                    | Orientation::DownWest => 0,
                    Orientation::UpEast
                    | Orientation::UpNorth
                    | Orientation::UpSouth
                    | Orientation::UpWest => 1,
                    Orientation::NorthUp => 2,
                    Orientation::SouthUp => 3,
                    Orientation::WestUp => 4,
                    Orientation::EastUp => 5,
                },
            );

            // Set crafting state back to false
            props.crafting = false;
            args.world
                .set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
        })
    }

    fn get_comparator_output<'a>(
        &'a self,
        args: GetComparatorOutputArgs<'a>,
    ) -> BlockFuture<'a, Option<u8>> {
        Box::pin(async move {
            if let Some(block_entity) = args.world.get_block_entity(args.position) {
                let crafter = block_entity.as_any().downcast_ref::<CrafterBlockEntity>()?;

                let mut occupied = 0u8;
                for i in 0..9 {
                    let stack = crafter.get_stack(i).await;
                    if !stack.lock().await.is_empty() {
                        occupied += 1;
                    }
                }
                Some(occupied)
            } else {
                None
            }
        })
    }
}
