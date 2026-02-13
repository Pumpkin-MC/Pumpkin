use std::sync::Arc;

use futures::future::join;
use pumpkin_data::block_properties::{
    BlockProperties, ChestLikeProperties, ChestType, HorizontalFacing,
};
use pumpkin_data::entity::EntityPose;
use pumpkin_data::{Block, BlockDirection, translation};
use pumpkin_inventory::double::DoubleInventory;
use pumpkin_inventory::generic_container_screen_handler::{create_generic_9x3, create_generic_9x6};
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::{pumpkin_block, pumpkin_block_from_tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::BlockEntity;
use pumpkin_world::block::entities::chest::ChestBlockEntity;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::world::BlockFlags;
use tokio::sync::Mutex;

use crate::block::{
    BlockFuture, BrokenArgs, EmitsRedstonePowerArgs, GetRedstonePowerArgs, NormalUseArgs,
    OnPlaceArgs, OnSyncedBlockEventArgs, PlacedArgs, RandomTickArgs,
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
            TextComponent::translate(translation::CONTAINER_CHESTDOUBLE, &[])
        } else {
            TextComponent::translate(translation::CONTAINER_CHEST, &[])
        }
    }
}

#[pumpkin_block_from_tag("c:chests/wooden")]
pub struct ChestBlock;

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

            if is_chest_blocked(args.world, args.position).await {
                return BlockActionResult::Success;
            }

            if let Some(direction) = connected_towards {
                let neighbor_pos = args.position.offset(direction.to_offset());
                if is_chest_blocked(args.world, &neighbor_pos).await {
                    return BlockActionResult::Success;
                }
            }

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

/// Copper chests have the same behavior as wooden chests but also oxidize over time.
#[pumpkin_block_from_tag("minecraft:copper_chests")]
pub struct CopperChestBlock;

impl BlockBehaviour for CopperChestBlock {
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
        Box::pin(async move { args.r#type == ChestBlock::LID_ANIMATION_EVENT_TYPE })
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

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let current_state_id = args.world.get_block_state_id(args.position).await;
            let chest_props = ChestLikeProperties::from_state_id(current_state_id, args.block);

            // Only oxidize LEFT or SINGLE chests (not RIGHT) to prevent double oxidation
            if chest_props.r#type == ChestType::Right {
                return;
            }

            // Only oxidize if no players are viewing the chest
            if let Some(block_entity) = args.world.get_block_entity(args.position).await
                && let Some(chest_entity) = block_entity.as_any().downcast_ref::<ChestBlockEntity>()
                && chest_entity.get_viewer_count() > 0
            {
                return;
            }

            // Try to oxidize the copper chest
            try_oxidize_copper_chest(args.world, args.position, args.block, chest_props).await;
        })
    }
}

/// Copper oxidation levels with their ordinal values
const COPPER_CHEST_OXIDATION: &[(&Block, &Block, u8)] = &[
    (&Block::COPPER_CHEST, &Block::EXPOSED_COPPER_CHEST, 0),
    (
        &Block::EXPOSED_COPPER_CHEST,
        &Block::WEATHERED_COPPER_CHEST,
        1,
    ),
    (
        &Block::WEATHERED_COPPER_CHEST,
        &Block::OXIDIZED_COPPER_CHEST,
        2,
    ),
];

/// Get the oxidation level ordinal for a block (None if not oxidizable copper chest)
fn get_oxidation_level(block: &Block) -> Option<u8> {
    // Check non-waxed variants
    if block == &Block::COPPER_CHEST {
        return Some(0);
    }
    if block == &Block::EXPOSED_COPPER_CHEST {
        return Some(1);
    }
    if block == &Block::WEATHERED_COPPER_CHEST {
        return Some(2);
    }
    if block == &Block::OXIDIZED_COPPER_CHEST {
        return Some(3);
    }
    // Waxed variants don't oxidize
    None
}

/// Try to oxidize a copper chest to its next oxidation level.
/// Uses vanilla's degradation algorithm with neighbor checking.
async fn try_oxidize_copper_chest(
    world: &Arc<World>,
    position: &BlockPos,
    current_block: &Block,
    chest_props: ChestLikeProperties,
) {
    use rand::RngExt;

    // Base chance per random tick: ~5.69%
    const BASE_DEGRADATION_CHANCE: f32 = 0.056_888_89;

    // First roll: only ~5.69% chance to even attempt oxidation
    if rand::rng().random::<f32>() >= BASE_DEGRADATION_CHANCE {
        return;
    }

    // Find the next oxidation level
    let (next_block, current_level) = match COPPER_CHEST_OXIDATION
        .iter()
        .find(|(from, _, _)| *from == current_block)
    {
        Some((_, to, level)) => (*to, *level),
        None => return, // Already fully oxidized or waxed
    };

    // Scan neighbors in 4-block Manhattan distance to calculate oxidation chance
    let (same_level_count, higher_level_count) =
        count_neighbor_oxidation_levels(world, position, current_level).await;

    // If we found any neighbors at a LOWER level, oxidation is blocked
    // (This is handled in count_neighbor_oxidation_levels by returning early)

    // Calculate weighted probability: ((higher + 1) / (higher + same + 1))^2 * multiplier
    let ratio =
        (higher_level_count + 1) as f32 / (higher_level_count + same_level_count + 1) as f32;
    // Multiplier is 0.75 for UNAFFECTED (level 0), 1.0 for others
    let multiplier = if current_level == 0 { 0.75 } else { 1.0 };
    let final_chance = ratio * ratio * multiplier;

    if rand::rng().random::<f32>() >= final_chance {
        return;
    }

    // Apply oxidation with same properties
    let new_state_id = chest_props.to_state_id(next_block);
    world
        .set_block_state(position, new_state_id, BlockFlags::NOTIFY_LISTENERS)
        .await;
}

/// Count copper blocks at same and higher oxidation levels within 4-block Manhattan distance.
/// Returns (same, higher) counts, or (0, 0) if a lower-level neighbor was found (blocking oxidation).
async fn count_neighbor_oxidation_levels(
    world: &Arc<World>,
    center: &BlockPos,
    current_level: u8,
) -> (i32, i32) {
    use std::cmp::Ordering;

    let mut same_level_count = 0i32;
    let mut higher_level_count = 0i32;

    // Iterate in a 4-block Manhattan distance (9x9x9 cube checked with distance filter)
    for dx in -4i32..=4 {
        for dy in -4i32..=4 {
            for dz in -4i32..=4 {
                let manhattan_dist = dx.abs() + dy.abs() + dz.abs();
                if manhattan_dist > 4 || manhattan_dist == 0 {
                    continue;
                }

                let neighbor_pos = BlockPos(pumpkin_util::math::vector3::Vector3::new(
                    center.0.x + dx,
                    center.0.y + dy,
                    center.0.z + dz,
                ));

                let (neighbor_block, _) = world.get_block_and_state_id(&neighbor_pos).await;

                if let Some(neighbor_level) = get_oxidation_level(neighbor_block) {
                    match neighbor_level.cmp(&current_level) {
                        Ordering::Less => {
                            // Found a neighbor at lower oxidation level - block oxidation entirely
                            return (0, 0);
                        }
                        Ordering::Greater => {
                            higher_level_count += 1;
                        }
                        Ordering::Equal => {
                            same_level_count += 1;
                        }
                    }
                }
            }
        }
    }

    (same_level_count, higher_level_count)
}

async fn compute_chest_props(
    world: &World,
    player: &Player,
    block: &Block,
    block_pos: &BlockPos,
    face: BlockDirection,
) -> (ChestType, HorizontalFacing) {
    let player_facing = player.get_entity().get_horizontal_facing();
    let chest_facing = player_facing.opposite();

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

async fn is_chest_blocked(world: &World, block_pos: &BlockPos) -> bool {
    // TODO: Block opening when a cat is sitting on top.
    has_block_on_top(world, block_pos).await
}
async fn has_block_on_top(world: &World, block_pos: &BlockPos) -> bool {
    let above_pos = block_pos.up();
    let above_state = world.get_block_state(&above_pos).await;
    above_state.is_solid_block()
}
/// Trapped chests have the same behavior as wooden chests but also emit redstone power based on viewer count.
#[pumpkin_block("minecraft:trapped_chest")]
pub struct TrappedChestBlock;

impl BlockBehaviour for TrappedChestBlock {
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
        Box::pin(async move { args.r#type == ChestBlock::LID_ANIMATION_EVENT_TYPE })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            use pumpkin_world::block::entities::trapped_chest::TrappedChestBlockEntity;

            let chest = TrappedChestBlockEntity::new(*args.position);
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

            if is_chest_blocked(args.world, args.position).await {
                return BlockActionResult::Success;
            }

            if let Some(direction) = connected_towards {
                let neighbor_pos = args.position.offset(direction.to_offset());
                if is_chest_blocked(args.world, &neighbor_pos).await {
                    return BlockActionResult::Success;
                }
            }

            let inventory = if let Some(direction) = connected_towards
                && let Some(second_inventory) = args
                    .world
                    .get_block_entity(&args.position.offset(direction.to_offset()))
                    .await
                    .and_then(BlockEntity::get_inventory)
            {
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

    fn emits_redstone_power<'a>(
        &'a self,
        _args: EmitsRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move { true })
    }

    fn get_weak_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            use pumpkin_world::block::entities::trapped_chest::TrappedChestBlockEntity;

            let viewer_count = if let Some(block_entity) =
                args.world.get_block_entity(args.position).await
                && let Some(trapped_chest) = block_entity
                    .as_any()
                    .downcast_ref::<TrappedChestBlockEntity>()
            {
                trapped_chest.get_viewer_count()
            } else {
                0
            };

            viewer_count.min(15) as u8
        })
    }

    fn get_strong_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            // Strong power only emitted upward (Direction::UP)
            if args.direction == BlockDirection::Down {
                // When checking from below (direction is Down), emit strong power
                self.get_weak_redstone_power(args).await
            } else {
                0
            }
        })
    }
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
