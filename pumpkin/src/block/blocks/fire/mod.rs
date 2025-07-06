use std::sync::Arc;

use pumpkin_data::tag::Tagable;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_registry::VanillaDimensionType;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::RandomGenerator;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use rand::Rng;
use soul_fire::SoulFireBlock;

use crate::block::blocks::fire::fire::FireBlock;
use crate::block::pumpkin_block::{CanPlaceAtArgs, PumpkinBlock};
use crate::world::World;
use crate::world::portal::nether::NetherPortal;

#[expect(clippy::module_inception)]
pub mod fire;
pub mod soul_fire;

pub struct FireBlockBase;

impl FireBlockBase {
    pub async fn get_fire_type(world: &World, pos: &BlockPos) -> Block {
        let (block, _block_state) = world.get_block_and_block_state(&pos.down()).await;
        if SoulFireBlock::is_soul_base(block) {
            return Block::SOUL_FIRE;
        }
        Block::FIRE
    }

    #[must_use]
    pub fn can_place_on(block: &Block) -> bool {
        // Make sure the block below is not a fire block or fluid block
        block != &Block::SOUL_FIRE
            && block != &Block::FIRE
            && block != &Block::WATER
            && block != &Block::LAVA
    }

    pub async fn is_soul_fire(world: &Arc<World>, block_pos: &BlockPos) -> bool {
        let block = world.get_block(&block_pos.down()).await;
        block.is_tagged_with("minecraft:soul_fire_base_blocks") == Some(true)
    }

    pub async fn can_place_at(world: &Arc<World>, block_pos: &BlockPos) -> bool {
        let block_state = world.get_block_state(block_pos).await;
        if !block_state.is_air() {
            return false;
        }
        if Self::is_soul_fire(world, block_pos).await {
            SoulFireBlock
                .can_place_at(CanPlaceAtArgs {
                    server: None,
                    world: Some(world),
                    block_accessor: world.as_ref(),
                    block: &Block::SOUL_FIRE,
                    location: block_pos,
                    direction: BlockDirection::Up,
                    player: None,
                    use_item_on: None,
                })
                .await
        } else {
            FireBlock
                .can_place_at(CanPlaceAtArgs {
                    server: None,
                    world: Some(world),
                    block_accessor: world.as_ref(),
                    block: &Block::FIRE,
                    location: block_pos,
                    direction: BlockDirection::Up,
                    player: None,
                    use_item_on: None,
                })
                .await
                || Self::should_light_portal_at(world, block_pos, BlockDirection::Up).await
        }
    }

    pub async fn should_light_portal_at(
        world: &Arc<World>,
        block_pos: &BlockPos,
        direction: BlockDirection,
    ) -> bool {
        let dimension = world.dimension_type;
        if dimension != VanillaDimensionType::Overworld
            && dimension != VanillaDimensionType::TheNether
        {
            return false;
        }
        let mut found = false;

        for dir in BlockDirection::all() {
            if world.get_block(&block_pos.offset(dir.to_offset())).await == &Block::OBSIDIAN {
                found = true;
                break;
            }
        }

        if !found {
            return false;
        }

        let dir = if direction.is_horizontal() {
            direction.rotate_counter_clockwise()
        } else {
            BlockDirection::random_horizontal(&mut RandomGenerator::Xoroshiro(
                Xoroshiro::from_seed(rand::rng().random()),
            ))
        };
        return NetherPortal::get_new_portal(world, block_pos, dir.to_horizontal_axis().unwrap())
            .await
            .is_some();
    }

    async fn broken(world: Arc<World>, block_pos: BlockPos) {
        world
            .sync_world_event(WorldEvent::FireExtinguished, block_pos, 0)
            .await;
    }
}
