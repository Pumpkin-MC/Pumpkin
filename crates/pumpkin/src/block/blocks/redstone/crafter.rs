use std::sync::Arc;
use std::sync::Mutex;

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::entities::crafter::CrafterBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, GetComparatorOutputArgs, NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs, PlacedArgs,
};
use pumpkin_data::block_properties::{CrafterLikeProperties, HorizontalFacing, Orientation};
use pumpkin_data::translation;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{BlockDirection, BlockStateId};
use pumpkin_inventory::generic_container_screen_handler::create_crafter_3x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

struct CrafterScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for CrafterScreenFactory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        _player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        let handler = create_crafter_3x3(sync_id, player_inventory, self.0.clone());
        let screen_handler_arc = Arc::new(Mutex::new(handler));

        Some(screen_handler_arc as SharedScreenHandler)
    }

    fn get_display_name(&self) -> TextComponent {
        pumpkin_macros::translate_cross!(
            translation::java::CONTAINER_CRAFTER,
            translation::bedrock::CONTAINER_CRAFTER
        )
    }
}

#[pumpkin_block("minecraft:crafter")]
pub struct CrafterBlock;

impl BlockBehaviour for CrafterBlock {
    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(inventory) = block_entity.get_inventory()
        {
            args.player
                .open_handled_screen(&CrafterScreenFactory(inventory), Some(*args.position));
        }
        BlockActionResult::Success
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
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
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        let crafter_block_entity = CrafterBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(crafter_block_entity));
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let powered = block_receives_redstone_power(args.world, args.position);
        let mut props =
            CrafterLikeProperties::from_state_id(args.world.get_block_state(args.position).id);

        if powered && !props.triggered {
            props.triggered = true;
            args.world
                .schedule_block_tick(args.block, *args.position, 4, TickPriority::Normal);
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_LISTENERS,
            );
        } else if !powered && props.triggered {
            props.triggered = false;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_LISTENERS,
            );
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let mut props =
            CrafterLikeProperties::from_state_id(args.world.get_block_state(args.position).id);

        // Set to crafting state
        props.crafting = true;
        args.world.set_block_state(
            args.position,
            props.to_state_id(args.block),
            BlockFlags::NOTIFY_LISTENERS,
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

        props.crafting = false;
        args.world.set_block_state(
            args.position,
            props.to_state_id(args.block),
            BlockFlags::NOTIFY_LISTENERS,
        );
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        if let Some(block_entity) = args.world.get_block_entity(args.position) {
            let crafter = block_entity.as_any().downcast_ref::<CrafterBlockEntity>()?;

            let items = crafter
                .items
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let occupied = items.iter().filter(|s| !s.is_empty()).count() as u8;
            Some(occupied)
        } else {
            None
        }
    }
}
