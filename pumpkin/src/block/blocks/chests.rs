use std::sync::Arc;

use futures::future::join;
use pumpkin_data::block_properties::{
    BlockProperties, ChestLikeProperties, ChestType, HorizontalFacing,
};
use pumpkin_data::entity::EntityPose;
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_inventory::double::DoubleInventory;
use pumpkin_inventory::generic_container_screen_handler::{create_generic_9x3, create_generic_9x6};
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::BlockEntity;
use pumpkin_world::block::entities::chest::ChestBlockEntity;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::world::BlockFlags;
use tokio::sync::Mutex;

use crate::block::{
    BlockFuture, BlockMetadata, BrokenArgs, NormalUseArgs, OnPlaceArgs, OnSyncedBlockEventArgs,
    PlacedArgs,
};
use crate::entity::EntityBase;
use crate::world::World;
use crate::{
    block::{BlockBehaviour, registry::BlockActionResult},
    entity::player::Player,
};

struct ChestScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for ChestScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let concrete_handler = if self.0.size() > 27 {
                create_generic_9x6(sync_id, player_inventory, self.0.clone()).await
            } else {
                create_generic_9x3(sync_id, player_inventory, self.0.clone()).await
            };

            let concrete_arc = Arc::new(Mutex::new(concrete_handler));

            Some(concrete_arc as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        if self.0.size() > 27 {
            TextComponent::translate("container.chestDouble", &[])
        } else {
            TextComponent::translate("container.chest", &[])
        }
    }
}

pub struct ChestBlock;

impl BlockMetadata for ChestBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "c:chests/wooden").unwrap()
    }
}

impl BlockBehaviour for ChestBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut chest_props = ChestLikeProperties::default(args.block);

            chest_props.waterlogged = args.replacing.water_source();

            let (r#type, facing) = compute_chest_props(
                args.world,
                args.player,
                args.block,
                args.position,
                args.direction,
            )
            .await;
            chest_props.facing = facing;
            chest_props.r#type = r#type;

            chest_props.to_state_id(args.block)
        })
    }

    fn on_synced_block_event<'a>(
        &'a self,
        args: OnSyncedBlockEventArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            // On the server, we don't need to do more because the client is responsible for that.
            args.r#type == Self::LID_ANIMATION_EVENT_TYPE
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let chest = ChestBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(chest)).await;

            let chest_props = ChestLikeProperties::from_state_id(args.state_id, args.block);
            let connected_towards = match chest_props.r#type {
                ChestType::Single => return,
                ChestType::Left => chest_props.facing.rotate_clockwise(),
                ChestType::Right => chest_props.facing.rotate_counter_clockwise(),
            };

            if let Some(mut neighbor_props) = get_chest_properties_if_can_connect(
                args.world,
                args.block,
                args.position,
                chest_props.facing,
                connected_towards,
                ChestType::Single,
            )
            .await
            {
                neighbor_props.r#type = chest_props.r#type.opposite();

                args.world
                    .set_block_state(
                        &args.position.offset(connected_towards.to_offset()),
                        neighbor_props.to_state_id(args.block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
            }
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let (state, first_chest) = join(
                args.world.get_block_state_id(args.position),
                args.world.get_block_entity(args.position),
            )
            .await;

            let Some(first_inventory) = first_chest.and_then(BlockEntity::get_inventory) else {
                return BlockActionResult::Fail;
            };

            let chest_props = ChestLikeProperties::from_state_id(state, args.block);
            let connected_towards = match chest_props.r#type {
                ChestType::Single => None,
                ChestType::Left => Some(chest_props.facing.rotate_clockwise()),
                ChestType::Right => Some(chest_props.facing.rotate_counter_clockwise()),
            };

            let inventory = if let Some(direction) = connected_towards
                && let Some(second_inventory) = args
                    .world
                    .get_block_entity(&args.position.offset(direction.to_offset()))
                    .await
                    .and_then(BlockEntity::get_inventory)
            {
                // Vanilla: chestType == ChestType.RIGHT ? DoubleBlockProperties.Type.FIRST : DoubleBlockProperties.Type.SECOND;
                if matches!(chest_props.r#type, ChestType::Right) {
                    DoubleInventory::new(first_inventory, second_inventory)
                } else {
                    DoubleInventory::new(second_inventory, first_inventory)
                }
            } else {
                first_inventory
            };

            args.player
                .open_handled_screen(&ChestScreenFactory(inventory))
                .await;

            BlockActionResult::Success
        })
    }

    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let chest_props = ChestLikeProperties::from_state_id(args.state.id, args.block);
            let connected_towards = match chest_props.r#type {
                ChestType::Single => return,
                ChestType::Left => chest_props.facing.rotate_clockwise(),
                ChestType::Right => chest_props.facing.rotate_counter_clockwise(),
            };

            if let Some(mut neighbor_props) = get_chest_properties_if_can_connect(
                args.world,
                args.block,
                args.position,
                chest_props.facing,
                connected_towards,
                chest_props.r#type.opposite(),
            )
            .await
            {
                neighbor_props.r#type = ChestType::Single;

                args.world
                    .set_block_state(
                        &args.position.offset(connected_towards.to_offset()),
                        neighbor_props.to_state_id(args.block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
            }
        })
    }
}

impl ChestBlock {
    pub const LID_ANIMATION_EVENT_TYPE: u8 = 1;
}

async fn compute_chest_props(
    world: &World,
    player: &Player,
    block: &Block,
    block_pos: &BlockPos,
    face: BlockDirection,
) -> (ChestType, HorizontalFacing) {
    let chest_facing = player.get_entity().get_horizontal_facing().opposite();

    if player.get_entity().pose.load() == EntityPose::Crouching {
        let Some(face) = face.to_horizontal_facing() else {
            return (ChestType::Single, chest_facing);
        };

        let (clicked_block, clicked_block_state) = world
            .get_block_and_state_id(&block_pos.offset(face.to_offset()))
            .await;

        if clicked_block == block {
            let clicked_props =
                ChestLikeProperties::from_state_id(clicked_block_state, clicked_block);

            if clicked_props.r#type != ChestType::Single {
                return (ChestType::Single, chest_facing);
            }

            if clicked_props.facing.rotate_clockwise() == face {
                return (ChestType::Left, clicked_props.facing);
            } else if clicked_props.facing.rotate_counter_clockwise() == face {
                return (ChestType::Right, clicked_props.facing);
            }
        }

        return (ChestType::Single, chest_facing);
    }

    if get_chest_properties_if_can_connect(
        world,
        block,
        block_pos,
        chest_facing,
        chest_facing.rotate_clockwise(),
        ChestType::Single,
    )
    .await
    .is_some()
    {
        (ChestType::Left, chest_facing)
    } else if get_chest_properties_if_can_connect(
        world,
        block,
        block_pos,
        chest_facing,
        chest_facing.rotate_counter_clockwise(),
        ChestType::Single,
    )
    .await
    .is_some()
    {
        (ChestType::Right, chest_facing)
    } else {
        (ChestType::Single, chest_facing)
    }
}

async fn get_chest_properties_if_can_connect(
    world: &World,
    block: &Block,
    block_pos: &BlockPos,
    facing: HorizontalFacing,
    direction: HorizontalFacing,
    wanted_type: ChestType,
) -> Option<ChestLikeProperties> {
    let (neighbor_block, neighbor_block_state) = world
        .get_block_and_state_id(&block_pos.offset(direction.to_offset()))
        .await;

    if neighbor_block != block {
        return None;
    }

    let neighbor_props = ChestLikeProperties::from_state_id(neighbor_block_state, neighbor_block);
    if neighbor_props.facing == facing && neighbor_props.r#type == wanted_type {
        return Some(neighbor_props);
    }

    None
}

trait ChestTypeExt {
    fn opposite(&self) -> ChestType;
}

impl ChestTypeExt for ChestType {
    fn opposite(&self) -> Self {
        match self {
            Self::Single => Self::Single,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}
