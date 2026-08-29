use std::sync::Arc;

use crate::{
    block::{
        BlockBehaviour, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, NormalUseArgs,
        OnEntityCollisionArgs, RandomTickArgs, UseWithItemArgs,
        blocks::plant::{PlantBlockBase, crop::CropBlockBase},
        registry::BlockActionResult,
    },
    world::World,
};
use pumpkin_data::{
    Block, BlockStateId,
    block_properties::{BlockProperties, NetherWartLikeProperties},
    damage::DamageType,
    entity::EntityType,
    item::Item,
    item_stack::ItemStack,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::RngExt;

#[pumpkin_block("minecraft:sweet_berry_bush")]
pub struct SweetBerryBushBlock;

impl BlockBehaviour for SweetBerryBushBlock {
    fn is_valid_bonemeal_target(&self, args: crate::block::BonemealArgs<'_>) -> bool {
        <Self as CropBlockBase>::is_valid_bonemeal_target(self, args.world, args.position)
    }

    fn perform_bonemeal(&self, args: crate::block::BonemealArgs<'_>) {
        <Self as CropBlockBase>::perform_bonemeal(self, args.world, args.position);
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position);
        let mut props = NetherWartLikeProperties::from_state_id(state_id, args.block);
        match props.age {
            2 | 3 => {
                let index = props.age;
                props.age = 1;
                let count: u8 = rand::rng().random_range((index - 1)..=(index));
                for _ in 0..count {
                    args.world.drop_stack(
                        args.position,
                        ItemStack::new(1, &Item::SWEET_BERRIES), //
                    );
                }
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(&Block::SWEET_BERRY_BUSH),
                    BlockFlags::NOTIFY_ALL,
                );
                BlockActionResult::SuccessServer
            }
            _ => BlockActionResult::Pass,
        }
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position);
        let props = NetherWartLikeProperties::from_state_id(state_id, &Block::SWEET_BERRY_BUSH);
        if props.age != 3 && args.item_stack.get_item() == &Item::BONE_MEAL {
            BlockActionResult::Pass
        } else {
            BlockActionResult::PassToDefaultBlockAction
        }
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        <Self as PlantBlockBase>::get_state_for_neighbor_update(
            self,
            args.world,
            args.position,
            args.state_id,
        )
    }

    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        let entity = args.entity.get_entity();

        let living_entity_opt = args.entity.get_living_entity();
        let Some(living_entity) = living_entity_opt else {
            return;
        };
        if entity.entity_type == &EntityType::FOX || entity.entity_type == &EntityType::BEE {
            return;
        }
        entity.slow_movement(args.state, Vector3::new(0.8, 0.75, 0.8));
        let mov = if living_entity.is_player() {
            living_entity.get_movement()
        } else {
            entity.last_pos.load() - entity.pos.load()
        };

        let state_id = args.world.get_block_state_id(args.position);
        let props = NetherWartLikeProperties::from_state_id(state_id, args.block);
        if props.age == 0 {
            return;
        }

        if mov.horizontal_length_squared() <= 0.0 || (mov.x.abs() < 0.003 && mov.z.abs() < 0.003) {
            return;
        }

        args.entity
            .damage(args.entity, 1.0, DamageType::SWEET_BERRY_BUSH);
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        if rand::rng().random_range(0..5) == 0 {
            <Self as CropBlockBase>::random_tick(self, args.world, args.position);
        }
    }
}

impl PlantBlockBase for SweetBerryBushBlock {
    fn get_state_for_neighbor_update(
        &self,
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
        block_state: BlockStateId,
    ) -> BlockStateId {
        if !<Self as PlantBlockBase>::can_place_at(self, block_accessor, block_pos) {
            return Block::AIR.default_state.id;
        }
        block_state
    }

    fn can_place_at(&self, block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        <Self as PlantBlockBase>::can_plant_on_top(self, block_accessor, &block_pos.down())
    }
}

impl CropBlockBase for SweetBerryBushBlock {
    fn bonemeal_age_increase(&self) -> i32 {
        1
    }

    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        <Self as PlantBlockBase>::can_plant_on_top(self, block_accessor, pos)
    }

    fn max_age(&self) -> i32 {
        3
    }

    fn get_age(&self, state: BlockStateId, block: &Block) -> i32 {
        let props = NetherWartLikeProperties::from_state_id(state, block);
        i32::from(props.age)
    }

    fn state_with_age(&self, block: &Block, state: BlockStateId, age: i32) -> BlockStateId {
        let mut props = NetherWartLikeProperties::from_state_id(state, block);
        props.age = age as u8;
        props.to_state_id(block)
    }

    fn random_tick(&self, world: &Arc<World>, pos: &BlockPos) {
        let (block, state) = world.get_block_and_state_id(pos);
        let age = self.get_age(state, block);
        if age < self.max_age() {
            let state_above = world.get_block_state(&pos.up());

            if state_above.is_full_cube() || state_above.is_solid() {
                return;
            }
            world.set_block_state(
                pos,
                self.state_with_age(block, state, age + 1),
                BlockFlags::NOTIFY_NEIGHBORS,
            );
        }
    }
}
