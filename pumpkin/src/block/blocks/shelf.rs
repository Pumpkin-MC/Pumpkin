use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockHitResult, GetComparatorOutputArgs, GetStateForNeighborUpdateArgs,
    NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs, OnStateReplacedArgs, PlacedArgs,
    UseWithItemArgs,
};
use crate::entity::player::Player;
use crate::world::World;
use async_trait::async_trait;
use futures::executor::block_on;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::SideChain;
use pumpkin_data::block_properties::{AcaciaShelfLikeProperties, HorizontalFacing};
use pumpkin_data::fluid::Fluid;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Block::MINECRAFT_WOODEN_SHELVES;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::{Taggable, get_tag_values};
use pumpkin_data::{Block, BlockState};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::shelf::ShelfBlockEntity;
use pumpkin_world::inventory::Inventory;
use pumpkin_world::item::ItemStack;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

#[pumpkin_block_from_tag("minecraft:wooden_shelves")]
pub struct Shelf;

#[async_trait]
impl BlockBehaviour for Shelf {
    async fn normal_use(&self, _args: NormalUseArgs<'_>) -> BlockActionResult {
        log::warn!("Shelf normal_use() called");
        BlockActionResult::Pass
    }

    async fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let state = AcaciaShelfLikeProperties::from_state_id(
            args.world.get_block_state(args.position).await.id,
            args.block,
        );
        if let Some(block_entity_any) = args.world.get_block_entity(args.position).await
            && let Some(block_entity) = block_entity_any.as_any().downcast_ref::<ShelfBlockEntity>()
        {
            if state.powered {
                todo!("Do the full hotbar swap")
            } else {
                if let Some(slot) = Self::get_slot_for_hit(args.hit, state.facing) {
                    let swaped = swap_single_stack(
                        &*args.item_stack.lock().await,
                        &args.player,
                        block_entity,
                        slot as usize,
                    );
                    if swaped {
                        args.world
                            .play_block_sound(
                                if args.item_stack.lock().await.is_empty() {
                                    Sound::BlockShelfTakeItem
                                } else {
                                    Sound::BlockShelfSingleSwap
                                },
                                SoundCategory::Blocks,
                                *args.position,
                            )
                            .await;
                    } else {
                        args.world
                            .play_block_sound(
                                Sound::BlockShelfPlaceItem,
                                SoundCategory::Blocks,
                                *args.position,
                            )
                            .await;
                        if args.item_stack.lock().await.is_empty() {
                            return BlockActionResult::Pass;
                        }
                    }
                    args.world.update_block_entity(&block_entity_any).await;
                    // TODO: I'm not happy with it and maybe better solution
                    args.world.update_neighbors(args.position, None).await;
                    return BlockActionResult::Consume;
                }
            }
        }
        BlockActionResult::Consume
    }

    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = AcaciaShelfLikeProperties::default(args.block);
        props.waterlogged = args.replacing.water_source();
        props.powered = block_receives_redstone_power(args.world, args.position).await;
        props.side_chain = SideChain::Unconnected;
        props.facing = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .opposite();
        props.to_state_id(args.block)
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        let block_entity = ShelfBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(block_entity)).await;
        let state = AcaciaShelfLikeProperties::from_state_id(
            args.world.get_block_state(args.position).await.id,
            args.block,
        );
        let old_state = AcaciaShelfLikeProperties::from_state_id(args.old_state_id, args.block);
        if state.powered {
            self.connect_neighbors(args.world, args.position, &state, &old_state)
        }
    }

    async fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let mut own_state = AcaciaShelfLikeProperties::from_state_id(
            args.world.get_block_state(args.position).await.id,
            args.block,
        );
        let powered = block_receives_redstone_power(args.world, args.position).await;
        if own_state.powered != powered {
            own_state.powered = powered;
            if !powered {
                own_state.side_chain = SideChain::Unconnected;
            }
            args.world
                .play_block_sound(
                    if powered {
                        Sound::BlockShelfActivate
                    } else {
                        Sound::BlockShelfDeactivate
                    },
                    SoundCategory::Blocks,
                    *args.position,
                )
                .await;
            args.world
                .set_block_state(
                    args.position,
                    own_state.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        }
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let state = args.world.get_block_state(args.position).await;
        let props = AcaciaShelfLikeProperties::from_state_id(state.id, args.block);
        if props.waterlogged {
            args.world
                .schedule_fluid_tick(
                    &Fluid::WATER,
                    *args.position,
                    Fluid::WATER.flow_speed as u8,
                    TickPriority::High,
                )
                .await;
        }
        props.to_state_id(args.block)
    }

    async fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        let props = AcaciaShelfLikeProperties::from_state_id(args.old_state_id, args.block);
        self.disconnect_neighbors(args.world, args.position, &props)
            .await;
    }

    async fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        // TODO: only allow the back comparator to get signal! (opposite facing comparator should get the signal)
        if let Some(entity) = args.world.get_block_entity(args.position).await {
            if let Some(shelf_entity) = entity.as_any().downcast_ref::<ShelfBlockEntity>() {
                let i = if shelf_entity.items[0].lock().await.is_empty() {
                    0
                } else {
                    1
                };
                let j = if shelf_entity.items[1].lock().await.is_empty() {
                    0
                } else {
                    1
                };
                let k = if shelf_entity.items[2].lock().await.is_empty() {
                    0
                } else {
                    1
                };
                return Some((i | j << 1 | k << 2) as u8);
            }
        }
        None
    }
}

impl Shelf {
    fn get_slot_for_hit(hit: &BlockHitResult<'_>, facing: HorizontalFacing) -> Option<i8> {
        Self::get_hit_pos(hit, facing).map(|position| Self::get_column(position.x))
    }

    fn get_hit_pos(hit: &BlockHitResult<'_>, facing: HorizontalFacing) -> Option<Vector2<f32>> {
        // If the direction is not horizontal, we cannot hit a slot
        let direction = hit.face.to_horizontal_facing()?;

        // If the facing direction does not match the block's facing, we cannot hit a slot
        if facing != direction {
            return None;
        }

        match direction {
            HorizontalFacing::North => Some(Vector2::new(1.0 - hit.cursor_pos.x, hit.cursor_pos.y)),
            HorizontalFacing::South => Some(Vector2::new(hit.cursor_pos.x, hit.cursor_pos.y)),
            HorizontalFacing::West => Some(Vector2::new(hit.cursor_pos.z, hit.cursor_pos.y)),
            HorizontalFacing::East => Some(Vector2::new(1.0 - hit.cursor_pos.z, hit.cursor_pos.y)),
        }
    }
    const OFFSET_SLOT_0: f32 = 0.375;
    const OFFSET_SLOT_1: f32 = 0.6875;
    fn get_column(x: f32) -> i8 {
        if x < Self::OFFSET_SLOT_0 {
            0
        } else if x < Self::OFFSET_SLOT_1 {
            1
        } else {
            2
        }
    }
    pub async fn disconnect_neighbors<'a>(
        &self,
        world: &Arc<World>,
        position: &BlockPos,
        props: &AcaciaShelfLikeProperties,
    ) {
        let new_pos = get_left_pos(position, props.facing);
        let block = world.get_block(&new_pos).await;
        if let Some(mut prop) = properties_if_shelf(&new_pos, world, block, props.facing).await {
            prop.side_chain = disconnect_from_right(prop.side_chain);
            world
                .set_block_state(&new_pos, prop.to_state_id(block), BlockFlags::NOTIFY_ALL)
                .await;
        }
        let new_pos = get_right_pos(position, props.facing);
        let block = world.get_block(&new_pos).await;
        if let Some(mut prop) = properties_if_shelf(&new_pos, world, block, props.facing).await {
            prop.side_chain = disconnect_from_left(prop.side_chain);
            world
                .set_block_state(&new_pos, prop.to_state_id(block), BlockFlags::NOTIFY_ALL)
                .await;
        }
    }

    fn connect_neighbors(
        &self,
        world: &Arc<World>,
        position: &BlockPos,
        props: &AcaciaShelfLikeProperties,
        old_props: &AcaciaShelfLikeProperties,
    ) {
    }
}

fn disconnect_from_right(side: SideChain) -> SideChain {
    match side {
        SideChain::Unconnected | SideChain::Left => SideChain::Unconnected,
        SideChain::Center | SideChain::Right => SideChain::Right,
    }
}

fn disconnect_from_left(side: SideChain) -> SideChain {
    match side {
        SideChain::Unconnected | SideChain::Right => SideChain::Unconnected,
        SideChain::Center | SideChain::Left => SideChain::Left,
    }
}

fn connect_from_right(side: SideChain) -> SideChain {
    match side {
        SideChain::Unconnected | SideChain::Left => SideChain::Left,
        SideChain::Center | SideChain::Right => SideChain::Center,
    }
}

fn connect_from_left(side: SideChain) -> SideChain {
    match side {
        SideChain::Unconnected | SideChain::Right => SideChain::Right,
        SideChain::Center | SideChain::Left => SideChain::Center,
    }
}

fn get_right_pos(cur_block_pos: &BlockPos, facing: HorizontalFacing) -> BlockPos {
    match facing {
        HorizontalFacing::South => cur_block_pos.east(),
        HorizontalFacing::North => cur_block_pos.west(),
        HorizontalFacing::West => cur_block_pos.south(),
        HorizontalFacing::East => cur_block_pos.north(),
    }
}

fn get_left_pos(cur_block_pos: &BlockPos, facing: HorizontalFacing) -> BlockPos {
    match facing {
        HorizontalFacing::South => cur_block_pos.west(),
        HorizontalFacing::North => cur_block_pos.east(),
        HorizontalFacing::West => cur_block_pos.north(),
        HorizontalFacing::East => cur_block_pos.south(),
    }
}

async fn properties_if_shelf(
    new_pos: &BlockPos,
    world: &World,
    block: &Block,
    facing: HorizontalFacing,
) -> Option<AcaciaShelfLikeProperties> {
    if block.has_tag(&MINECRAFT_WOODEN_SHELVES) {
        let state = AcaciaShelfLikeProperties::from_state_id(
            world.get_block_state(new_pos).await.id,
            block,
        );
        if state.facing == facing && state.powered {
            return Some(state);
        }
    }
    None
}

fn swap_single_stack(
    item_stack: &ItemStack,
    player: &Player,
    block_entity: &ShelfBlockEntity,
    hit_slot: usize,
) -> bool {
    let item = block_on(block_entity.remove_stack(hit_slot));
    block_on(block_entity.set_stack(hit_slot, item_stack.clone()));
    let item2 = if player.is_creative() && item.is_empty() {
        item_stack.clone()
    } else {
        item.clone()
    };
    // TODO race condition which I don't understand crashes because can't lock in slots.rs get_cloned_stack() line 54
    // block_on(
    //     player
    //         .inventory
    //         .set_stack(player.inventory.get_selected_slot() as usize, item2),
    // );
    player.inventory.mark_dirty();
    block_entity.mark_dirty();
    !item.is_empty()
}
