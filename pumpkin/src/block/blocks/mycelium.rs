use pumpkin_data::BlockStateId;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::{
    Block, BlockState,
    block_properties::{BlockProperties, GrassBlockLikeProperties, SnowLikeProperties},
    tag::{self, Taggable},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::world::BlockFlags;

use crate::block::{BlockBehaviour, BlockFuture, GetStateForNeighborUpdateArgs, RandomTickArgs};
use crate::world::World;

/// `LevelReader#getMaxLightLevel` in vanilla. A covering block that dims light by at least this
/// much makes the mycelium below it die back to dirt.
const MAX_LIGHT_LEVEL: u8 = 15;

/// The `FluidState#getAmount` of a full fluid column (a source, a falling fluid or the water held
/// by a waterlogged block). Vanilla's `canStayAlive` rejects exactly this value.
const FULL_FLUID_AMOUNT: i16 = 8;

/// Minimum `getMaxLocalRawBrightness` above mycelium for it to spread.
const MIN_SPREAD_BRIGHTNESS: u8 = 9;

/// How many spread attempts vanilla makes per random tick.
const SPREAD_ATTEMPTS: u8 = 4;

/// `MyceliumBlock extends SpreadingSnowyBlock`.
///
/// Same die-back/spread logic as `GrassBlock` (`grass_block.rs`), but the spread target is
/// `Block::DIRT` and the produced block is mycelium itself rather than grass. See
/// `SpreadingSnowyBlock#randomTick` in `net/minecraft/world/level/block/SpreadingSnowyBlock.java`.
///
/// Unlike `GrassBlock`, `MyceliumBlock` does not implement `BonemealableBlock`, so there is no
/// bonemeal-driven spreading branch to port.
///
/// `MyceliumBlock#animateTick` additionally rolls `random.nextInt(10) == 0` to spawn a
/// `ParticleTypes.MYCELIUM` spore particle above the block. Pumpkin has no per-tick client-side
/// ambient block hook (`animateTick`'s equivalent) anywhere in `BlockBehaviour`; the closest hook,
/// `random_tick`, is vanilla's server-side `randomTick`/`scheduledTick`, not `animateTick`. This
/// purely cosmetic particle effect is deferred rather than bolted onto the wrong hook.
#[pumpkin_block("minecraft:mycelium")]
pub struct MyceliumBlock;

impl BlockBehaviour for MyceliumBlock {
    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let world = args.world;
            let position = args.position;

            if !can_stay_alive(world, position) {
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

                let mut props = GrassBlockLikeProperties::default(&Block::MYCELIUM);
                props.snowy = is_snowy_setting(world, &target);
                world
                    .set_block_state(
                        &target,
                        props.to_state_id(&Block::MYCELIUM),
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
                GrassBlockLikeProperties::from_state_id(args.state_id, &Block::MYCELIUM);
            let should_be_snowy = is_snowy_setting(args.world, args.position);
            if props.snowy == should_be_snowy {
                return args.state_id;
            }
            props.snowy = should_be_snowy;

            props.to_state_id(&Block::MYCELIUM)
        })
    }
}

/// `SnowyDirtBlock#isSnowySetting` applied to the block covering `position`.
fn is_snowy_setting(world: &World, position: &BlockPos) -> bool {
    world
        .get_block(&position.up())
        .has_tag(&tag::Block::MINECRAFT_SNOW)
}

/// `SpreadingSnowyBlock#canStayAlive`: mycelium survives as long as the block covering it neither
/// holds a full fluid column nor blocks all light. Die-back to `Block::DIRT` (the `randomTick`
/// caller above) uses this same check every random tick, identically to `GrassBlock` — there is
/// no separate decay timer or extra grace period for mycelium.
fn can_stay_alive(world: &World, position: &BlockPos) -> bool {
    let above = position.up();

    if is_covered_by_full_fluid(world, &above) {
        return false;
    }

    let (above_block, above_state) = world.get_block_and_state(&above);
    !ceiling_kills_mycelium(above_block, above_state)
}

/// `LightEngine#getLightDampeningInto` applied to the block covering the mycelium.
///
/// Pumpkin has no face-occlusion lookup, so the raw opacity of the covering block stands in
/// for the dampened-light calculation, same as `grass_block.rs::can_be_grass`. Known divergence
/// for low-opacity downward-occluding blocks (carpets, bottom slabs, thin snow); the common
/// fully-opaque-ceiling case still kills the mycelium correctly.
fn ceiling_kills_mycelium(above_block: &Block, above_state: &BlockState) -> bool {
    if above_block == &Block::SNOW {
        return SnowLikeProperties::from_state_id(above_state.id, above_block).layers > 1;
    }

    above_state.opacity >= MAX_LIGHT_LEVEL
}

/// `SpreadingSnowyBlock#canPropagate`: mycelium cannot spread into a spot that has water on top.
fn can_propagate(world: &World, position: &BlockPos) -> bool {
    can_stay_alive(world, position)
        && !world
            .get_fluid(&position.up())
            .has_tag(&tag::Fluid::MINECRAFT_WATER)
}

/// Whether the block at `position` reports a fluid state with an amount of 8, i.e. a fluid source,
/// a falling fluid or a waterlogged block.
fn is_covered_by_full_fluid(world: &World, position: &BlockPos) -> bool {
    let state_id = world.get_block_state_id(position);

    Fluid::from_state_id(state_id).map_or_else(
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

#[cfg(test)]
mod test {
    use super::{MIN_SPREAD_BRIGHTNESS, SPREAD_ATTEMPTS, ceiling_kills_mycelium};
    use pumpkin_data::{
        Block, BlockState,
        block_properties::{BlockProperties, SnowLikeProperties},
    };

    #[test]
    fn spread_attempts_and_brightness_threshold_match_vanilla() {
        assert_eq!(SPREAD_ATTEMPTS, 4);
        assert_eq!(MIN_SPREAD_BRIGHTNESS, 9);
    }

    #[test]
    fn opaque_ceiling_kills_mycelium() {
        assert!(ceiling_kills_mycelium(
            &Block::STONE,
            Block::STONE.default_state
        ));
    }

    #[test]
    fn transparent_ceiling_does_not_kill_mycelium() {
        assert!(!ceiling_kills_mycelium(
            &Block::AIR,
            Block::AIR.default_state
        ));
    }

    #[test]
    fn single_snow_layer_does_not_kill_mycelium() {
        let mut props = SnowLikeProperties::default(&Block::SNOW);
        props.layers = 1;
        let state = BlockState::from_id(props.to_state_id(&Block::SNOW));
        assert!(!ceiling_kills_mycelium(&Block::SNOW, state));
    }

    #[test]
    fn two_snow_layers_kill_mycelium() {
        let mut props = SnowLikeProperties::default(&Block::SNOW);
        props.layers = 2;
        let state = BlockState::from_id(props.to_state_id(&Block::SNOW));
        assert!(ceiling_kills_mycelium(&Block::SNOW, state));
    }
}
