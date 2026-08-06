use pumpkin_data::BlockStateId;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, GrassBlockLikeProperties, SnowLikeProperties},
    tag::{self, Taggable},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::world::BlockFlags;

use crate::block::{BlockBehaviour, BlockFuture, GetStateForNeighborUpdateArgs, RandomTickArgs};
use crate::world::World;

/// `LevelReader#getMaxLightLevel` in vanilla. A covering block that dims light by at least this
/// much makes the grass below it die.
const MAX_LIGHT_LEVEL: u8 = 15;

/// The `FluidState#getAmount` of a full fluid column (a source, a falling fluid or the water held
/// by a waterlogged block). Vanilla's `canBeGrass` rejects exactly this value.
const FULL_FLUID_AMOUNT: i16 = 8;

/// Minimum `getMaxLocalRawBrightness` above a grass block for it to spread.
const MIN_SPREAD_BRIGHTNESS: u8 = 9;

/// How many spread attempts vanilla makes per random tick.
const SPREAD_ATTEMPTS: u8 = 4;

#[pumpkin_block("minecraft:grass_block")]
pub struct GrassBlock;

impl BlockBehaviour for GrassBlock {
    /// `SpreadingSnowyDirtBlock#randomTick`: grass that is covered turns back into dirt, otherwise
    /// it makes four attempts to spread onto nearby dirt.
    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let world = args.world;
            let position = args.position;

            if !can_be_grass(world, position) {
                world
                    .set_block_state(
                        position,
                        Block::DIRT.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                return;
            }

            if world.get_max_local_raw_brightness(&position.up()) < MIN_SPREAD_BRIGHTNESS {
                return;
            }

            for _ in 0..SPREAD_ATTEMPTS {
                // Vanilla: `pos.offset(random.nextInt(3) - 1, random.nextInt(5) - 3, random.nextInt(3) - 1)`
                let target = position.offset(Vector3::new(
                    rand::random_range(-1..=1),
                    rand::random_range(-3..=1),
                    rand::random_range(-1..=1),
                ));

                if world.get_block(&target) != &Block::DIRT || !can_propagate(world, &target) {
                    continue;
                }

                let mut props = GrassBlockLikeProperties::default(&Block::GRASS_BLOCK);
                props.snowy = is_snowy_setting(world, &target);
                world
                    .set_block_state(
                        &target,
                        props.to_state_id(&Block::GRASS_BLOCK),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props =
                GrassBlockLikeProperties::from_state_id(args.state_id, &Block::GRASS_BLOCK);
            let should_be_snowy = is_snowy_setting(args.world, args.position);
            if props.snowy == should_be_snowy {
                return args.state_id;
            }
            props.snowy = should_be_snowy;

            props.to_state_id(&Block::GRASS_BLOCK)
        })
    }
}

/// `SnowyDirtBlock#isSnowySetting` applied to the block covering `position`.
fn is_snowy_setting(world: &World, position: &BlockPos) -> bool {
    world
        .get_block(&position.up())
        .has_tag(&tag::Block::MINECRAFT_SNOW)
}

/// `SpreadingSnowyDirtBlock#canBeGrass`: grass survives as long as the block covering it neither
/// holds a full fluid column nor blocks all light.
fn can_be_grass(world: &World, position: &BlockPos) -> bool {
    let above = position.up();

    if is_covered_by_full_fluid(world, &above) {
        return false;
    }

    // Vanilla short-circuits on a single snow layer, then runs the opacity
    // through `LightEngine#getLightBlockInto`, which reports 16 when the two
    // touching faces fully occlude each other.
    //
    // Pumpkin has no face-occlusion lookup, so only the raw opacity is
    // available here. That is a known divergence rather than an equivalence:
    // blocks that occlude downwards but carry a low opacity, such as carpets,
    // bottom slabs and snow layers of two or more, let the grass survive where
    // vanilla would kill it. Grass under a full opaque block still dies, which
    // is the common case. Closing the gap needs a face-occlusion flag from the
    // data generator.
    let (above_block, above_state) = world.get_block_and_state(&above);

    // Handle the snow layers explicitly, since that is the one case the raw
    // opacity gets wrong that vanilla calls out by name.
    if above_block == &Block::SNOW {
        return SnowLikeProperties::from_state_id(above_state.id, above_block).layers <= 1;
    }

    above_state.opacity < MAX_LIGHT_LEVEL
}

/// `SpreadingSnowyDirtBlock#canPropagate`: grass cannot spread into a spot that has water on top.
fn can_propagate(world: &World, position: &BlockPos) -> bool {
    can_be_grass(world, position)
        && !world
            .get_fluid(&position.up())
            .has_tag(&tag::Fluid::MINECRAFT_WATER)
}

/// Whether the block at `position` reports a fluid state with an amount of 8, i.e. a fluid source,
/// a falling fluid or a waterlogged block.
fn is_covered_by_full_fluid(world: &World, position: &BlockPos) -> bool {
    let state_id = world.get_block_state_id(position);

    Fluid::from_state_id(state_id).map_or_else(
        // Not a fluid block itself; a waterlogged block still carries a water source.
        || {
            world
                .get_fluid(position)
                .has_tag(&tag::Fluid::MINECRAFT_WATER)
        },
        |fluid| {
            fluid
                .states
                .iter()
                .any(|state| state.block_state_id == state_id && state.level == FULL_FLUID_AMOUNT)
        },
    )
}
