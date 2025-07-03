use pumpkin_data::block_properties::{BlockProperties, EnumVariants, HorizontalAxis};
use pumpkin_data::entity::EntityType;
use pumpkin_data::fluid::Fluid;
use pumpkin_registry::VanillaDimensionType;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::Rng;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use async_trait::async_trait;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::chunk::TickPriority;

use crate::block::blocks::tnt::TNTBlock;
use crate::block::pumpkin_block::{
    BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnEntityCollisionArgs,
    OnScheduledTickArgs, PlacedArgs, PumpkinBlock,
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
        if block_state
            .block()
            .properties(block_state.id)
            .and_then(|props| {
                props
                    .to_props()
                    .into_iter()
                    .find(|p| p.0 == "waterlogged")
                    .map(|(_, v)| v == true.to_string())
            })
            .unwrap_or(false)
        {
            return false;
        }
        block_state
            .block()
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
        _block: &Block,
        pos: &BlockPos,
    ) -> BlockStateId {
        let down_pos = pos.down();
        let down_state = world.get_block_state(&down_pos).await;
        if Self::is_flammable(down_state) || down_state.is_side_solid(BlockDirection::Up) {
            return Block::FIRE.default_state.id;
        }
        let mut fire_props =
            FireProperties::from_state_id(Block::FIRE.default_state.id, &Block::FIRE);
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
        fire_props.to_state_id(&Block::FIRE)
    }

    pub async fn try_spreading_fire(
        &self,
        world: &Arc<World>,
        pos: &BlockPos,
        spread_factor: i32,
        current_age: u16,
    ) {
        if world.get_fluid(pos).await.name != Fluid::EMPTY.name {
            return; // Skip if there is a fluid
        }
        let spread_chance: i32 = world
            .get_block(pos)
            .await
            .flammable
            .as_ref()
            .map_or(0, |f| f.spread_chance)
            .into();
        if rand::rng().random_range(0..spread_factor) < spread_chance {
            let block = world.get_block(pos).await;
            if rand::rng().random_range(0..current_age + 10) < 5 {
                let new_age = (current_age + rand::rng().random_range(0..5) / 4).min(15);
                let state_id = self.get_state_for_position(world, &Block::FIRE, pos).await;
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

            if block == &Block::TNT {
                TNTBlock::prime(world, pos).await;
            }
        }
    }

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
                total_burn_chance += i32::from(flammable.burn_chance);
            }
        }

        total_burn_chance
    }
}

#[async_trait]
impl PumpkinBlock for FireBlock {
    async fn placed(&self, args: PlacedArgs<'_>) {
        if args.old_state_id == args.state_id {
            // Already a fire
            return;
        }

        let dimension = args.world.dimension_type;
        // First lets check if we are in OverWorld or Nether, its not possible to place an Nether portal in other dimensions in Vanilla
        if dimension == VanillaDimensionType::Overworld
            || dimension == VanillaDimensionType::TheNether
        {
            if let Some(portal) =
                NetherPortal::get_new_portal(args.world, args.location, HorizontalAxis::X).await
            {
                portal.create(args.world).await;
                return;
            }
        }

        args.world
            .schedule_block_tick(
                args.block,
                *args.location,
                Self::get_fire_tick_delay() as u16,
                TickPriority::Normal,
            )
            .await;
    }

    async fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        let base_entity = args.entity.get_entity();
        if !base_entity.entity_type.fire_immune {
            let ticks = base_entity.fire_ticks.load(Ordering::Relaxed);
            if ticks < 0 {
                base_entity.fire_ticks.store(ticks + 1, Ordering::Relaxed);
            } else if base_entity.entity_type == EntityType::PLAYER {
                let rnd_ticks = rand::rng().random_range(1..3);
                base_entity
                    .fire_ticks
                    .store(ticks + rnd_ticks, Ordering::Relaxed);
            }
            if base_entity.fire_ticks.load(Ordering::Relaxed) >= 0 {
                base_entity.set_on_fire_for(8.0);
            }
        }
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if self
            .can_place_at(CanPlaceAtArgs {
                server: None,
                world: Some(args.world),
                block_accessor: args.world,
                block: &Block::FIRE,
                location: args.location,
                direction: BlockDirection::Up,
                player: None,
                use_item_on: None,
            })
            .await
        {
            let old_fire_props = FireProperties::from_state_id(args.state_id, &Block::FIRE);
            let fire_state_id = self
                .get_state_for_position(args.world, &Block::FIRE, args.location)
                .await;
            let mut fire_props = FireProperties::from_state_id(fire_state_id, &Block::FIRE);
            fire_props.age = EnumVariants::from_index(old_fire_props.age.to_index());
            return fire_props.to_state_id(&Block::FIRE);
        }
        Block::AIR.default_state.id
    }

    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let state = args
            .block_accessor
            .get_block_state(&args.location.down())
            .await;
        if state.is_side_solid(BlockDirection::Up) {
            return true;
        }
        self.are_blocks_around_flammable(args.block_accessor, args.location)
            .await
    }

    #[allow(clippy::too_many_lines)]
    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let (world, block, pos) = (args.world, args.block, args.location);
        world
            .schedule_block_tick(
                block,
                *pos,
                Self::get_fire_tick_delay() as u16,
                TickPriority::Normal,
            )
            .await;
        if !Self
            .can_place_at(CanPlaceAtArgs {
                server: None,
                world: Some(world),
                block_accessor: world.as_ref(),
                block,
                location: pos,
                direction: BlockDirection::Up,
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
        //TODO add checks for raining and infiniburn
        let mut fire_props = FireProperties::from_state_id(block_state.id, &Block::FIRE);
        let age = fire_props.age.to_index() + 1;

        let random = rand::rng().random_range(0..3) / 2;
        let new_age = (age + random).min(15);
        if new_age != age {
            fire_props.age = EnumVariants::from_index(new_age);
            let new_state_id = fire_props.to_state_id(&Block::FIRE);
            world
                .set_block_state(pos, new_state_id, BlockFlags::NOTIFY_NEIGHBORS)
                .await;
        }

        if !Self.are_blocks_around_flammable(world.as_ref(), pos).await {
            let block_below_state = world.get_block_state(&pos.down()).await;
            if block_below_state.is_side_solid(BlockDirection::Up) {
                world
                    .set_block_state(
                        pos,
                        Block::AIR.default_state.id,
                        BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
            }
            return;
        }

        if age == 15
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

        Self.try_spreading_fire(
            world,
            &pos.offset(BlockDirection::East.to_offset()),
            300,
            age,
        )
        .await;
        Self.try_spreading_fire(
            world,
            &pos.offset(BlockDirection::West.to_offset()),
            300,
            age,
        )
        .await;
        Self.try_spreading_fire(
            world,
            &pos.offset(BlockDirection::North.to_offset()),
            300,
            age,
        )
        .await;
        Self.try_spreading_fire(
            world,
            &pos.offset(BlockDirection::South.to_offset()),
            300,
            age,
        )
        .await;
        Self.try_spreading_fire(world, &pos.offset(BlockDirection::Up.to_offset()), 250, age)
            .await;
        Self.try_spreading_fire(
            world,
            &pos.offset(BlockDirection::Down.to_offset()),
            250,
            age,
        )
        .await;

        for l in -1..=1 {
            for m in -1..=1 {
                for n in -1..=4 {
                    if l != 0 || n != 0 || m != 0 {
                        let offset_pos = pos.offset(Vector3::new(l, n, m));
                        let burn_chance = Self.get_burn_chance(world, &offset_pos).await;
                        if burn_chance > 0 {
                            let o = 100 + if n > 1 { (n - 1) * 100 } else { 0 };
                            let p: i32 = burn_chance
                                + 40
                                + i32::from(world.level_info.read().await.difficulty.to_int()) * 7
                                    / i32::from(age + 30);

                            if p > 0 && rand::rng().random_range(0..o) <= p {
                                let new_age = (age + rand::rng().random_range(0..5) / 4).min(15);
                                let fire_state_id =
                                    self.get_state_for_position(world, block, &offset_pos).await;
                                let mut new_fire_props =
                                    FireProperties::from_state_id(fire_state_id, &Block::FIRE);
                                new_fire_props.age = EnumVariants::from_index(new_age);

                                //TODO drop items for burned blocks
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
    }

    async fn broken(&self, args: BrokenArgs<'_>) {
        FireBlockBase::broken(args.world.clone(), *args.location).await;
    }
}
