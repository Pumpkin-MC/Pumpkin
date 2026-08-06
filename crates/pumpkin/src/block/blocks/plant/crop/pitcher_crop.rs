use std::sync::Arc;

use pumpkin_data::block_properties::{BlockProperties, DoubleBlockHalf, PitcherCropLikeProperties};
use pumpkin_data::{Block, BlockStateId, tag, tag::Taggable};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::RngExt;

use crate::block::blocks::plant::crop::{MIN_GROWTH_LIGHT, get_available_moisture};
use crate::block::{
    BlockBehaviour, BlockFuture, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnPlaceArgs, RandomTickArgs,
};
use crate::world::World;

type PitcherCropProperties = PitcherCropLikeProperties;

/// `pitcher_crop`: a two-tall crop grown from a pitcher pod. Vanilla:
/// `net.minecraft.world.level.block.PitcherCropBlock`.
///
/// Bonemeal support (`BonemealableBlock`) is not ported: Pumpkin has no bonemeal-application
/// hook on `BlockBehaviour` yet (checked `wheat.rs`/`beetroot.rs`, neither implements it either),
/// so this only covers natural growth via `random_tick`.
#[pumpkin_block("minecraft:pitcher_crop")]
pub struct PitcherCropBlock;

/// `isDouble`: age >= 3 means the crop occupies two blocks.
const fn is_double(age: u8) -> bool {
    age >= 3
}

/// `isMaxAge`.
const fn is_max_age(age: u8) -> bool {
    age >= 4
}

fn is_lower(block: &Block, half: DoubleBlockHalf) -> bool {
    block == &Block::PITCHER_CROP && half == DoubleBlockHalf::Lower
}

/// `PitcherCropBlock.mayPlaceOn`: `minecraft:supports_crops` (farmland) below.
fn may_place_on(accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
    let below = accessor.get_block(&pos.down());
    below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_CROPS)
}

/// `PitcherCropBlock.sufficientLight` / `CropBlock.hasSufficientLight`.
fn sufficient_light(world: &World, pos: &BlockPos) -> bool {
    world.get_raw_brightness(pos, 0) >= MIN_GROWTH_LIGHT
}

/// `PitcherCropBlock.canGrowInto`: the space above the lower half must be air, or already the
/// (upper half of the) same crop.
fn can_grow_into(world: &World, pos: &BlockPos) -> bool {
    let (block, state) = world.get_block_and_state(pos);
    state.is_air() || block == &Block::PITCHER_CROP
}

/// `PitcherCropBlock.canGrow`.
fn can_grow(world: &World, lower_pos: &BlockPos, current_age: u8, new_age: u8) -> bool {
    if is_max_age(current_age) || !sufficient_light(world, lower_pos) {
        return false;
    }
    let above_pos = lower_pos.up();
    if above_pos.0.y > world.get_top_y() {
        return false;
    }
    !is_double(new_age) || can_grow_into(world, &above_pos)
}

/// `PitcherCropBlock.grow`.
async fn grow(
    world: &Arc<World>,
    lower_pos: &BlockPos,
    lower_state_id: BlockStateId,
    increase: u8,
) {
    let mut props = PitcherCropProperties::from_state_id(lower_state_id, &Block::PITCHER_CROP);
    let new_age = (props.age + increase).min(4);
    if !can_grow(world, lower_pos, props.age, new_age) {
        return;
    }
    props.age = new_age;
    world
        .set_block_state(
            lower_pos,
            props.to_state_id(&Block::PITCHER_CROP),
            BlockFlags::NOTIFY_LISTENERS,
        )
        .await;
    if is_double(new_age) {
        let mut upper_props = props;
        upper_props.half = DoubleBlockHalf::Upper;
        world
            .set_block_state(
                &lower_pos.up(),
                upper_props.to_state_id(&Block::PITCHER_CROP),
                BlockFlags::NOTIFY_ALL,
            )
            .await;
    }
}

impl BlockBehaviour for PitcherCropBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        may_place_on(args.block_accessor, args.position)
    }

    /// `PitcherCropBlock.getStateForPlacement`: always starts as the lower half, age 0.
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move { args.block.default_state.id })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let props = PitcherCropProperties::from_state_id(args.state_id, args.block);
            if is_double(props.age) {
                // Double-plant half consistency, same shape as `TallPlantBlock`.
                let other_pos = match props.half {
                    DoubleBlockHalf::Upper => args.position.down(),
                    DoubleBlockHalf::Lower => args.position.up(),
                };
                let (other_block, other_state_id) = args.world.get_block_and_state_id(&other_pos);
                if other_block == &Block::PITCHER_CROP {
                    let other_props =
                        PitcherCropProperties::from_state_id(other_state_id, other_block);
                    let opposite_half = match props.half {
                        DoubleBlockHalf::Upper => DoubleBlockHalf::Lower,
                        DoubleBlockHalf::Lower => DoubleBlockHalf::Upper,
                    };
                    if other_props.half == opposite_half {
                        return args.state_id;
                    }
                }
                return Block::AIR.default_state.id;
            }

            if !may_place_on(args.world, args.position) {
                return Block::AIR.default_state.id;
            }
            args.state_id
        })
    }

    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // Breaking either half removes the other, matching `DoublePlantBlock` behaviour
            // (already established for the generic tall-plant blocks in `tall_plant.rs`).
            let props = PitcherCropProperties::from_state_id(args.state.id, args.block);
            if !is_double(props.age) {
                return;
            }
            let other_pos = match props.half {
                DoubleBlockHalf::Upper => args.position.down(),
                DoubleBlockHalf::Lower => args.position.up(),
            };
            let (other_block, _) = args.world.get_block_and_state_id(&other_pos);
            if other_block == &Block::PITCHER_CROP {
                args.world
                    .break_block(
                        &other_pos,
                        None,
                        BlockFlags::SKIP_DROPS | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
                    )
                    .await;
            }
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let (block, state_id) = args.world.get_block_and_state_id(args.position);
            let props = PitcherCropProperties::from_state_id(state_id, block);
            // Only the lower half is randomly ticking, and only while still growing.
            if !is_lower(block, props.half) || is_max_age(props.age) {
                return;
            }
            let growth_speed = get_available_moisture(args.world, args.position, block).await;
            let roll_max = (25.0 / growth_speed).floor() as i64;
            if rand::rng().random_range(0..=roll_max.max(0)) == 0 {
                grow(args.world, args.position, state_id, 1).await;
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn double_threshold_matches_vanilla() {
        assert!(!is_double(2));
        assert!(is_double(3));
        assert!(is_double(4));
    }

    #[test]
    fn max_age_is_four() {
        assert!(!is_max_age(3));
        assert!(is_max_age(4));
    }

    #[test]
    fn only_lower_half_of_pitcher_crop_is_lower() {
        assert!(is_lower(&Block::PITCHER_CROP, DoubleBlockHalf::Lower));
        assert!(!is_lower(&Block::PITCHER_CROP, DoubleBlockHalf::Upper));
        assert!(!is_lower(&Block::PITCHER_PLANT, DoubleBlockHalf::Lower));
    }
}
