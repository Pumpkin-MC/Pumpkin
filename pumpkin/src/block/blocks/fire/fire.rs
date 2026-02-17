use pumpkin_data::biome::Biome;
use pumpkin_data::block_properties::{BlockProperties, EnumVariants, HorizontalAxis};
use pumpkin_data::dimension::Dimension;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::BlockStateId;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::RngExt;
use std::sync::Arc;

use crate::block::blocks::tnt::TNTBlock;
use crate::block::{
    BlockBehaviour, BlockFuture, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnEntityCollisionArgs, OnScheduledTickArgs, PlacedArgs,
};
use crate::world::World;
use crate::world::portal::nether::NetherPortal;

type FireProperties = pumpkin_data::block_properties::FireLikeProperties;

use super::FireBlockBase;

#[pumpkin_block("minecraft:fire")]
pub struct FireBlock;

impl FireBlock {
    #[must_use]
    pub fn get_fire_tick_delay() -> i32 {
        30 + rand::rng().random_range(0..10)
    }

    fn is_flammable(block_state: &BlockState) -> bool {
        if Block::from_state_id(block_state.id)
            .properties(block_state.id)
            .and_then(|props| {
                props
                    .to_props()
                    .into_iter()
                    .find(|p| p.0 == "waterlogged")
                    .map(|(_, v)| v == "true")
            })
            .unwrap_or(false)
        {
            return false;
        }
        Block::from_state_id(block_state.id)
            .flammable
            .as_ref()
            .is_some_and(|f| f.burn_chance > 0)
    }

    async fn are_blocks_around_flammable(
        &self,
        block_accessor: &dyn BlockAccessor,
        pos: &BlockPos,
    ) -> bool {
        for direction in BlockDirection::all() {
            let neighbor_pos = pos.offset(direction.to_offset());
            let block_state = block_accessor.get_block_state(&neighbor_pos).await;
            if Self::is_flammable(block_state) {
                return true;
            }
        }
        false
    }

    pub async fn get_state_for_position(
        &self,
        world: &World,
        block: &Block,
        pos: &BlockPos,
    ) -> BlockStateId {
        let down_pos = pos.down();
        let down_state = world.get_block_state(&down_pos).await;
        if Self::is_flammable(down_state) || down_state.is_side_solid(BlockDirection::Up) {
            return block.default_state.id;
        }
        let mut fire_props = FireProperties::from_state_id(block.default_state.id, block);
        for direction in BlockDirection::all() {
            let neighbor_pos = pos.offset(direction.to_offset());
            let neighbor_state = world.get_block_state(&neighbor_pos).await;
            if Self::is_flammable(neighbor_state) {
                match direction {
                    BlockDirection::North => fire_props.north = true,
                    BlockDirection::South => fire_props.south = true,
                    BlockDirection::East => fire_props.east = true,
                    BlockDirection::West => fire_props.west = true,
                    BlockDirection::Up => fire_props.up = true,
                    BlockDirection::Down => {}
                }
            }
        }
        fire_props.to_state_id(block)
    }

    // Used for spreading fire
    pub async fn get_burn_chance(&self, world: &Arc<World>, pos: &BlockPos) -> i32 {
        let block_state = world.get_block_state(pos).await;
        if !block_state.is_air() {
            return 0;
        }
        let mut total_burn_chance = 0;

        for dir in BlockDirection::all() {
            let neighbor_block = world.get_block(&pos.offset(dir.to_offset())).await;
            if world.get_fluid(&pos.offset(dir.to_offset())).await.name != Fluid::EMPTY.name {
                continue; // Skip if there is a fluid
            }
            if let Some(flammable) = &neighbor_block.flammable {
                total_burn_chance = total_burn_chance.max(i32::from(flammable.spread_chance));
            }
        }

        total_burn_chance
    }

    const fn is_near_rain(_world: &World, _pos: &BlockPos) -> bool {
        // TODO: Implement proper rain checking when weather is implemented
        // For now, return false to allow fire to work
        false
    }

    // Get burn odds for a block, used in try_spreading_fire
    fn get_burn_odds(block: &Block) -> i32 {
        block.flammable.as_ref().map_or(0, |f| f.burn_chance.into())
    }

    async fn is_increased_burnout_biome(world: &World, pos: &BlockPos) -> bool {
        // Fire burnout increases in the Nether
        if world.dimension == Dimension::THE_NETHER {
            return true;
        }

        // Fire burnout increases in specific biomes
        // TODO: Use proper tag or bool for this when available
        let biome_id = world.level.get_rough_biome(pos).await.id;
        matches!(
            biome_id,
            id if id == Biome::BAMBOO_JUNGLE.id
                || id == Biome::MUSHROOM_FIELDS.id
                || id == Biome::MANGROVE_SWAMP.id
                || id == Biome::SNOWY_SLOPES.id
                || id == Biome::FROZEN_PEAKS.id
                || id == Biome::JAGGED_PEAKS.id
                || id == Biome::SWAMP.id
                || id == Biome::JUNGLE.id
        )
    }

    async fn try_spreading_fire(&self, world: &Arc<World>, pos: &BlockPos, chance: i32, age: u16) {
        let block = world.get_block(pos).await;
        let odds = Self::get_burn_odds(block);
        if rand::rng().random_range(0..chance) < odds {
            let old_block = block;
            if rand::rng().random_range(0..(age + 10) as i32) < 5
                && !Self::is_near_rain(world.as_ref(), pos)
            {
                let new_age = (age + (rand::rng().random_range(0..5) / 4)).min(15);
                let state_id = self
                    .get_state_for_position(world.as_ref(), &Block::FIRE, pos)
                    .await;
                let mut fire_props = FireProperties::from_state_id(state_id, &Block::FIRE);
                fire_props.age = EnumVariants::from_index(new_age);
                let new_state_id = fire_props.to_state_id(&Block::FIRE);
                world
                    .set_block_state(pos, new_state_id, BlockFlags::NOTIFY_NEIGHBORS)
                    .await;
            } else {
                world
                    .set_block_state(
                        pos,
                        Block::AIR.default_state.id,
                        BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
            }

            if old_block == &Block::TNT {
                TNTBlock::prime(world, pos).await;
            }
        }
    }
}

impl BlockBehaviour for FireBlock {
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if args.old_state_id == args.state_id {
                // Already a fire
                return;
            }

            let dimension = args.world.dimension;
            // First lets check if we are in OverWorld or Nether, its not possible to place an Nether portal in other dimensions in Vanilla
            if (dimension == Dimension::OVERWORLD || dimension == Dimension::THE_NETHER)
                && let Some(portal) =
                    NetherPortal::get_new_portal(args.world, args.position, HorizontalAxis::X).await
            {
                portal.create(args.world).await;
                return;
            }

            args.world
                .schedule_block_tick(
                    args.block,
                    *args.position,
                    Self::get_fire_tick_delay() as u8,
                    TickPriority::Normal,
                )
                .await;
        })
    }

    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        FireBlockBase::apply_fire_collision(args, false)
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if self
                .can_place_at(CanPlaceAtArgs {
                    server: None,
                    world: Some(args.world),
                    block_accessor: args.world,
                    block: &Block::FIRE,
                    state: Block::FIRE.default_state,
                    position: args.position,
                    direction: None,
                    player: None,
                    use_item_on: None,
                })
                .await
            {
                let old_fire_props = FireProperties::from_state_id(args.state_id, &Block::FIRE);
                let fire_state_id = self
                    .get_state_for_position(args.world, &Block::FIRE, args.position)
                    .await;
                let mut fire_props = FireProperties::from_state_id(fire_state_id, &Block::FIRE);
                fire_props.age = EnumVariants::from_index(old_fire_props.age.to_index());
                return fire_props.to_state_id(&Block::FIRE);
            }
            Block::AIR.default_state.id
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let state = args
                .block_accessor
                .get_block_state(&args.position.down())
                .await;
            if state.is_side_solid(BlockDirection::Up) {
                return true;
            }
            self.are_blocks_around_flammable(args.block_accessor, args.position)
                .await
        })
    }

    #[expect(clippy::too_many_lines)]
    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let (world, block, pos) = (args.world, args.block, args.position);

            // Schedule next tick first
            world
                .schedule_block_tick(
                    block,
                    *pos,
                    Self::get_fire_tick_delay() as u8,
                    TickPriority::Normal,
                )
                .await;

            // Check if fire can survive
            if !Self
                .can_place_at(CanPlaceAtArgs {
                    server: None,
                    world: Some(world),
                    block_accessor: world.as_ref(),
                    block,
                    state: block.default_state,
                    position: pos,
                    direction: None,
                    player: None,
                    use_item_on: None,
                })
                .await
            {
                world
                    .set_block_state(
                        pos,
                        Block::AIR.default_state.id,
                        BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
                return;
            }

            let block_state = world.get_block_state(pos).await;
            let block_below = world.get_block(&pos.down()).await;

            // Check for infiniburn blocks (depending on dimension)
            let infiniburn = match world.dimension.id {
                id if id == Dimension::OVERWORLD.id => {
                    block_below.has_tag(&tag::Block::MINECRAFT_INFINIBURN_OVERWORLD)
                }
                id if id == Dimension::THE_NETHER.id => {
                    block_below.has_tag(&tag::Block::MINECRAFT_INFINIBURN_NETHER)
                }
                id if id == Dimension::THE_END.id => {
                    block_below.has_tag(&tag::Block::MINECRAFT_INFINIBURN_END)
                }
                _ => false,
            };

            let mut fire_props = FireProperties::from_state_id(block_state.id, &Block::FIRE);
            let age = fire_props.age.to_index();

            // Check if rain should extinguish the fire
            if !infiniburn && Self::is_near_rain(world.as_ref(), pos) {
                let rain_chance = 0.2 + (age as f32) * 0.03;
                if rand::random::<f32>() < rain_chance {
                    world
                        .set_block_state(
                            pos,
                            Block::AIR.default_state.id,
                            BlockFlags::NOTIFY_NEIGHBORS,
                        )
                        .await;
                    return;
                }
            }

            // Increment age
            let random = rand::rng().random_range(0..3) / 2;
            let new_age = (age + random).min(15);
            if new_age != age {
                fire_props.age = EnumVariants::from_index(new_age);
                let new_state_id = fire_props.to_state_id(&Block::FIRE);
                world
                    .set_block_state(pos, new_state_id, BlockFlags::NOTIFY_NEIGHBORS)
                    .await;
            }

            if !infiniburn {
                // Check if fire should extinguish due to lack of fuel
                if !self.are_blocks_around_flammable(world.as_ref(), pos).await {
                    let block_below_state = world.get_block_state(&pos.down()).await;
                    if !block_below_state.is_side_solid(BlockDirection::Up) || new_age > 3 {
                        world
                            .set_block_state(
                                pos,
                                Block::AIR.default_state.id,
                                BlockFlags::NOTIFY_NEIGHBORS,
                            )
                            .await;
                        return;
                    }
                }

                // At max age, fire has a chance to extinguish if not on flammable block
                if new_age == 15
                    && rand::rng().random_range(0..4) == 0
                    && !Self::is_flammable(world.get_block_state(&pos.down()).await)
                {
                    world
                        .set_block_state(
                            pos,
                            Block::AIR.default_state.id,
                            BlockFlags::NOTIFY_NEIGHBORS,
                        )
                        .await;
                    return;
                }
            }

            // Burn adjacent blocks
            let extra = if Self::is_increased_burnout_biome(world, pos).await {
                -50 // Increases chance of block being destroyed
            } else {
                0
            };

            self.try_spreading_fire(
                world,
                &pos.offset(BlockDirection::East.to_offset()),
                300 + extra,
                new_age,
            )
            .await;
            self.try_spreading_fire(
                world,
                &pos.offset(BlockDirection::West.to_offset()),
                300 + extra,
                new_age,
            )
            .await;
            self.try_spreading_fire(
                world,
                &pos.offset(BlockDirection::Down.to_offset()),
                250 + extra,
                new_age,
            )
            .await;
            self.try_spreading_fire(
                world,
                &pos.offset(BlockDirection::Up.to_offset()),
                250 + extra,
                new_age,
            )
            .await;
            self.try_spreading_fire(
                world,
                &pos.offset(BlockDirection::North.to_offset()),
                300 + extra,
                new_age,
            )
            .await;
            self.try_spreading_fire(
                world,
                &pos.offset(BlockDirection::South.to_offset()),
                300 + extra,
                new_age,
            )
            .await;

            // Respect the `fire_spread_radius_around_player` gamerule.
            // -1 = disabled (allow unlimited spread), 0 = disabled (no spread), >0 = radius in blocks
            let spread_radius = world
                .level_info
                .load()
                .game_rules
                .fire_spread_radius_around_player;

            // Try to spread fire to nearby air blocks
            let difficulty = world.level_info.load().difficulty as i32;
            for xx in -1..=1 {
                for zz in -1..=1 {
                    for yy in -1..=4 {
                        if xx != 0 || yy != 0 || zz != 0 {
                            let offset_pos = pos.offset(Vector3::new(xx, yy, zz));
                            let ignite_odds = self.get_burn_chance(world, &offset_pos).await;

                            if ignite_odds > 0 {
                                // Skip if spreding is disabled or if there are no players nearby
                                if spread_radius == 0 {
                                    continue;
                                }
                                if spread_radius != -1 {
                                    let center = offset_pos.to_centered_f64();
                                    if world
                                        .get_closest_player(center, spread_radius as f64)
                                        .is_none()
                                    {
                                        continue;
                                    }
                                }

                                // Calculate spread rate based on height
                                let rate = if yy > 1 { 100 + (yy - 1) * 100 } else { 100 };

                                // Calculate odds of spreading
                                let mut odds =
                                    (ignite_odds + 40 + difficulty * 7) / (new_age as i32 + 30);

                                // Reduce spread odds in certain biomes
                                if Self::is_increased_burnout_biome(world, &offset_pos).await {
                                    odds /= 2; // Fire spreads 50% slower
                                }

                                if odds > 0
                                    && rand::rng().random_range(0..rate) <= odds
                                    && !Self::is_near_rain(world.as_ref(), &offset_pos)
                                {
                                    let spread_age =
                                        (new_age + rand::rng().random_range(0..5) / 4).min(15);
                                    let fire_state_id = self
                                        .get_state_for_position(world.as_ref(), block, &offset_pos)
                                        .await;
                                    let mut new_fire_props =
                                        FireProperties::from_state_id(fire_state_id, &Block::FIRE);
                                    new_fire_props.age = EnumVariants::from_index(spread_age);

                                    world
                                        .set_block_state(
                                            &offset_pos,
                                            new_fire_props.to_state_id(&Block::FIRE),
                                            BlockFlags::NOTIFY_NEIGHBORS,
                                        )
                                        .await;
                                }
                            }
                        }
                    }
                }
            }
        })
    }

    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            FireBlockBase::broken(args.world, *args.position).await;
        })
    }
}
