use pumpkin_data::BlockStateId;
use pumpkin_data::{
    Block, BlockState,
    block_properties::{
        BlockProperties, DoubleBlockHalf, GrassBlockLikeProperties, TallSeagrassLikeProperties,
    },
    tag::{self, Taggable},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::{RandomGenerator, RandomImpl, xoroshiro128::Xoroshiro};
use pumpkin_world::generation::feature::{
    configured_features::{BONE_MEAL_FEATURES, CONFIGURED_FEATURES, ConfiguredFeature},
    placed_features::{Feature, PLACED_FEATURES},
};
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

use crate::block::{BlockBehaviour, BlockFuture, GetStateForNeighborUpdateArgs};

#[pumpkin_block("minecraft:grass_block")]
pub struct GrassBlock;

impl BlockBehaviour for GrassBlock {
    fn is_valid_bonemeal_target(&self, args: crate::block::BonemealArgs<'_>) -> bool {
        let above = args.position.up();
        args.world.is_in_height_limit(above.0.y)
            && args.world.is_loaded(&above)
            && args.world.get_block_state(&above).is_air()
    }

    fn perform_bonemeal<'a>(&'a self, args: crate::block::BonemealArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            const SPREAD_ATTEMPTS: i32 = 128;
            const ATTEMPTS_PER_STEP: i32 = 16;
            const FLOWER_CHANCE: i32 = 8;

            let origin = args.position.up();
            for attempt in 0..SPREAD_ATTEMPTS {
                let mut target = origin;
                let mut valid = true;
                for _ in 0..attempt / ATTEMPTS_PER_STEP {
                    let offset_x = rand::rng().random_range(0..3) - 1;
                    let offset_y =
                        ((rand::rng().random_range(0..3) - 1) * rand::rng().random_range(0..3)) / 2;
                    let offset_z = rand::rng().random_range(0..3) - 1;
                    target = BlockPos::new(
                        target.0.x + offset_x,
                        target.0.y + offset_y,
                        target.0.z + offset_z,
                    );

                    if !args.world.is_loaded(&target)
                        || args.world.get_block(&target.down()) != args.block
                        || args.world.get_block_state(&target).is_full_cube()
                    {
                        valid = false;
                        break;
                    }
                }

                if !valid {
                    continue;
                }
                let target_state = args.world.get_block_state(&target);
                if Block::from_state_id(target_state.id) == &Block::SHORT_GRASS
                    && rand::rng().random_range(0..10) == 0
                {
                    let above = target.up();
                    if args.world.is_in_height_limit(above.0.y)
                        && args.world.is_loaded(&above)
                        && args.world.get_block_state(&above).is_air()
                    {
                        place_tall_grass(args.world, target).await;
                    }
                } else if target_state.is_air() && args.world.is_in_height_limit(target.0.y) {
                    let selected = if rand::rng().random_range(0..FLOWER_CHANCE) == 0 {
                        biome_bonemeal_state(args.world, target)
                    } else {
                        Some((Block::SHORT_GRASS.default_state, false))
                    };
                    let Some((state, schedule_tick)) = selected else {
                        continue;
                    };
                    let placed_block = Block::from_state_id(state.id);
                    if !args.world.block_registry.can_place_at(
                        None,
                        Some(args.world),
                        args.world.as_ref(),
                        None,
                        placed_block,
                        state,
                        &target,
                        None,
                        None,
                    ) {
                        continue;
                    }
                    if placed_block == &Block::TALL_GRASS
                        && (!args.world.is_loaded(&target.up())
                            || !args.world.get_block_state(&target.up()).is_air())
                    {
                        continue;
                    }
                    args.world
                        .set_block_state(&target, state.id, BlockFlags::NOTIFY_LISTENERS)
                        .await;
                    if schedule_tick {
                        args.world.schedule_block_tick(
                            placed_block,
                            target,
                            1,
                            TickPriority::Normal,
                        );
                    }
                    if placed_block == &Block::TALL_GRASS {
                        place_tall_grass_upper(args.world, target, state.id).await;
                    }
                }
            }
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let block_above = args.world.get_block(&args.position.up());
            let mut props =
                GrassBlockLikeProperties::from_state_id(args.state_id, &Block::GRASS_BLOCK);
            let should_be_snowy = block_above.has_tag(&tag::Block::MINECRAFT_SNOW);
            if props.snowy == should_be_snowy {
                return args.state_id;
            }
            props.snowy = should_be_snowy;

            props.to_state_id(&Block::GRASS_BLOCK)
        })
    }
}

async fn place_tall_grass(world: &std::sync::Arc<crate::world::World>, position: BlockPos) {
    let state = Block::TALL_GRASS.default_state.id;
    world
        .set_block_state(&position, state, BlockFlags::NOTIFY_LISTENERS)
        .await;
    place_tall_grass_upper(world, position, state).await;
}

async fn place_tall_grass_upper(
    world: &std::sync::Arc<crate::world::World>,
    position: BlockPos,
    lower_state: BlockStateId,
) {
    let mut props = TallSeagrassLikeProperties::from_state_id(lower_state, &Block::TALL_GRASS);
    props.half = DoubleBlockHalf::Upper;
    world
        .set_block_state(
            &position.up(),
            props.to_state_id(&Block::TALL_GRASS),
            BlockFlags::NOTIFY_LISTENERS,
        )
        .await;
}

fn biome_bonemeal_state(
    world: &crate::world::World,
    position: BlockPos,
) -> Option<(&'static BlockState, bool)> {
    let features: Vec<_> = world
        .get_biome(&position)
        .features
        .iter()
        .flat_map(|step| step.iter())
        .filter_map(|key| PLACED_FEATURES.get(key))
        .filter_map(|feature| match &feature.feature {
            Feature::Named(key) if BONE_MEAL_FEATURES.contains(key) => Some(*key),
            Feature::Named(_) | Feature::Inlined(_) => None,
        })
        .collect();
    if features.is_empty() {
        return None;
    }

    let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(rand::rng().random()));
    let key = features[random.next_bounded_i32(features.len() as i32) as usize];
    let ConfiguredFeature::SimpleBlock(feature) = CONFIGURED_FEATURES.get(&key)? else {
        return None;
    };
    feature
        .to_place
        .get_for_bonemeal(&mut random, position)
        .map(|state| (state, feature.schedule_tick.unwrap_or(false)))
}
