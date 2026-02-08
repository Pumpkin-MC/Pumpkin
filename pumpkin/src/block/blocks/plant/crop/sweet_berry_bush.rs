use std::sync::Arc;

use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, EnumVariants, Integer0To3, NetherWartLikeProperties},
    damage::DamageType,
    entity::EntityType,
    item::Item,
    tag::{self, Taggable},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::{
    position::BlockPos,
    vector3::Vector3
};
use pumpkin_world::{
    BlockStateId,
    item::ItemStack,
    world::{BlockAccessor, BlockFlags},
};
use rand::RngExt;
use pumpkin_data::world::WorldEvent;
use crate::{
    block::{
        BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, NormalUseArgs,
        OnEntityCollisionArgs, RandomTickArgs, UseWithItemArgs,
        blocks::plant::{PlantBlockBase, crop::CropBlockBase},
        registry::BlockActionResult,
    },
    world::World,
};

#[pumpkin_block("minecraft:sweet_berry_bush")]
pub struct SweetBerryBushBlock;

impl BlockBehaviour for SweetBerryBushBlock {
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state_id = args.world.get_block_state_id(args.position).await;
            let mut props = NetherWartLikeProperties::from_state_id(state_id, args.block);
            match props.age {
                Integer0To3::L2 | Integer0To3::L3 => {
                    let index = props.age.to_index() as u8;
                    props.age = Integer0To3::L1;
                    let count: u8 = rand::rng().random_range((index - 1)..=(index));
                    for _ in 0..count {
                        args.world
                            .drop_stack(
                                args.position,
                                ItemStack::new(1, &Item::SWEET_BERRIES), //
                            )
                            .await;
                    }
                    args.world
                        .set_block_state(
                            args.position,
                            props.to_state_id(&Block::SWEET_BERRY_BUSH),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                    BlockActionResult::SuccessServer
                }
                _ => BlockActionResult::Pass,
            }
        })
    }

    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state_id = args.world.get_block_state_id(args.position).await;
            let props = NetherWartLikeProperties::from_state_id(state_id, &Block::SWEET_BERRY_BUSH);
            let mut lock = args.item_stack.lock().await;
            if props.age != Integer0To3::L3
                && lock.get_item() == &Item::BONE_MEAL
            {
                <Self as PlantBlockBase>::grow(self, args.world, args.position).await;
                lock.decrement(1);
                args.world.sync_world_event(WorldEvent::BoneMealUsed, *args.position, 15).await;
                BlockActionResult::Success
            } else {
                BlockActionResult::PassToDefaultBlockAction
            }
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            <Self as PlantBlockBase>::get_state_for_neighbor_update(
                self,
                args.world,
                args.position,
                args.state_id,
            )
            .await
        })
    }

    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let entity = args.entity.get_entity();
            let living_entity_opt = args.entity.get_living_entity();
            if living_entity_opt.is_none() || entity.entity_type == &EntityType::FOX || entity.entity_type == &EntityType::BEE {
                return;
            }
            let living_entity = living_entity_opt.expect("Living entity should exist");

            living_entity.fall_distance.store(0f32);
            entity.movement_multiplier.store(Vector3::new(0.8, 0.75, 0.8));
            let mov = if living_entity.is_controlled_by_player() {
                living_entity.get_movement()
            } else {
                entity.last_pos.load() - entity.pos.load()
            };
            let state_id = args.world.get_block_state_id(args.position).await;
            let props = NetherWartLikeProperties::from_state_id(state_id, args.block);
            if props.age == Integer0To3::L0 {
                return;
            }

            if mov.horizontal_length_squared() <= 0.0
                || (mov.x.abs() < 0.003 && mov.z.abs() < 0.003)
            {
                return;
            }

            args.entity
                .damage(args.entity, 1.0, DamageType::SWEET_BERRY_BUSH)
                .await;
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if rand::rng().random_range(0..5) == 0 {
                <Self as CropBlockBase>::random_tick(self, args.world, args.position).await;
            }
        })
    }
}

impl PlantBlockBase for SweetBerryBushBlock {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = block_accessor.get_block(pos).await;
        block.has_tag(&tag::Block::MINECRAFT_DIRT)
    }

    async fn get_state_for_neighbor_update(
        &self,
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
        block_state: BlockStateId,
    ) -> BlockStateId {
        if !<Self as PlantBlockBase>::can_place_at(self, block_accessor, block_pos).await {
            return Block::AIR.default_state.id;
        }
        block_state
    }

    async fn can_place_at(&self, block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        <Self as PlantBlockBase>::can_plant_on_top(self, block_accessor, &block_pos.down()).await
    }

    async fn grow(&self, world: &Arc<World>, pos: &BlockPos) {
        let (block, state) = world.get_block_and_state_id(pos).await;
        let age = self.get_age(state, block);
        world
            .set_block_state(
                pos,
                self.state_with_age(block, state, 3.min(age + 1)),
                BlockFlags::NOTIFY_LISTENERS,
            )
            .await;
    }
}

impl CropBlockBase for SweetBerryBushBlock {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        <Self as PlantBlockBase>::can_plant_on_top(self, block_accessor, pos).await
    }

    fn max_age(&self) -> i32 {
        3
    }

    fn get_age(&self, state: u16, block: &Block) -> i32 {
        let props = NetherWartLikeProperties::from_state_id(state, block);
        i32::from(props.age.to_index())
    }

    fn state_with_age(&self, block: &Block, state: u16, age: i32) -> BlockStateId {
        let mut props = NetherWartLikeProperties::from_state_id(state, block);
        props.age = Integer0To3::from_index(age as u16);
        props.to_state_id(block)
    }

    async fn random_tick(&self, world: &Arc<World>, pos: &BlockPos) {
        let (block, state) = world.get_block_and_state_id(pos).await;
        let age = self.get_age(state, block);
        if age < self.max_age() {
            let state_above = world.get_block_state(&pos.up()).await;

            if state_above.is_full_cube() || state_above.is_solid() {
                return;
            }
            if rand::rng().random_range(0..=25) == 0 {
                world
                    .set_block_state(
                        pos,
                        self.state_with_age(block, state, age + 1),
                        BlockFlags::NOTIFY_NEIGHBORS,
                    )
                    .await;
            }
        }
    }
}
