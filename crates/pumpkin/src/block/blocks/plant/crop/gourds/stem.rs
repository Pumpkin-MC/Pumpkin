use crate::block::{
    BlockBehaviour, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, RandomTickArgs,
    blocks::plant::{
        PlantBlockBase,
        crop::{CropBlockBase, get_available_moisture},
    },
};
use crate::world::World;
use pumpkin_data::{
    Block, BlockDirection, BlockId, BlockStateId,
    block_properties::{HorizontalFacing, WallTorchLikeProperties, WheatLikeProperties},
    tag::{self, Taggable},
};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, xoroshiro128::Xoroshiro},
};
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::RngExt;
use std::sync::Arc;

type StemProperties = WheatLikeProperties;
type AttachedStemProperties = WallTorchLikeProperties;

pub struct StemBlock;

impl BlockMetadata for StemBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::PUMPKIN_STEM, BlockId::MELON_STEM].into()
    }
}

impl StemBlock {
    fn state_with_age(block: &Block, state: BlockStateId, age: i32) -> BlockStateId {
        let mut props = StemProperties::from_state_id(state);
        props.age = age as u8;
        props.to_state_id(block)
    }

    fn get_attached_stem(dir: HorizontalFacing, block: &Block) -> BlockStateId {
        let attached_block = match block.id {
            id if id == Block::PUMPKIN_STEM.id => &Block::ATTACHED_PUMPKIN_STEM,
            id if id == Block::MELON_STEM.id => &Block::ATTACHED_MELON_STEM,
            _ => &Block::ATTACHED_MELON_STEM, // Should never happen
        };
        let mut props = AttachedStemProperties::default(attached_block);
        props.facing = dir;
        props.to_state_id(attached_block)
    }

    fn get_gourd(block: &Block) -> &Block {
        match block.id {
            id if id == Block::PUMPKIN_STEM.id => &Block::PUMPKIN,
            id if id == Block::MELON_STEM.id => &Block::MELON,
            _ => &Block::MELON, // Should never happen
        }
    }

    /// Spawn a gourd on a random free horizontal side and attach the stem
    /// to it. Shared by natural random ticks and the bonemeal path - only
    /// the natural path is light-gated.
    fn grow_gourd(&self, world: &Arc<World>, position: &BlockPos, block: &Block) {
        let dir = BlockDirection::random_horizontal(&mut RandomGenerator::Xoroshiro(
            Xoroshiro::from_seed(rand::rng().random()),
        ));
        let plant_block_pos = position.offset(dir.to_offset());
        let plant_block_state = world.get_block_state(&plant_block_pos);
        let under_block: &Block = world.get_block(&plant_block_pos.down());
        if plant_block_state.is_air()
            && (under_block == &Block::FARMLAND || under_block.has_tag(&tag::Block::MINECRAFT_DIRT))
        {
            let attached_stem = Self::get_attached_stem(dir, block);
            let gourd = Self::get_gourd(block);
            world.set_block_state(
                &plant_block_pos,
                gourd.default_state.id,
                BlockFlags::NOTIFY_NEIGHBORS,
            );
            world.set_block_state(position, attached_stem, BlockFlags::NOTIFY_NEIGHBORS);
        }
    }
}

impl BlockBehaviour for StemBlock {
    fn is_valid_bonemeal_target(&self, args: crate::block::BonemealArgs<'_>) -> bool {
        <Self as CropBlockBase>::is_valid_bonemeal_target(self, args.world, args.position)
    }

    fn perform_bonemeal(&self, args: crate::block::BonemealArgs<'_>) {
        <Self as CropBlockBase>::perform_bonemeal(self, args.world, args.position);
        let (block, state) = args.world.get_block_and_state_id(args.position);
        if StemProperties::from_state_id(state).age == 7 {
            // Vanilla spawns the gourd here regardless of light level;
            // routing through the light-gated `random_tick` would suppress
            // bonemeal-grown gourds in the dark.
            self.grow_gourd(args.world, args.position, block);
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

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        // Vanilla `StemBlock.randomTick` requires a raw brightness of at
        // least 9 at the stem for natural growth. The bonemeal path is not
        // light-gated: it calls `grow_gourd` directly (vanilla
        // `BonemealableBlock.performBonemeal` spawns the gourd without the
        // randomTick brightness check).
        if args.world.get_raw_brightness(args.position, 0) < 9 {
            return;
        }
        let f: f32 = get_available_moisture(args.world, args.position, args.block);
        if rand::rng().random_range(0..=(25.0 / f).floor() as i32) == 0 {
            let (block, state) = args.world.get_block_and_state_id(args.position);
            let props = StemProperties::from_state_id(state);
            let age = i32::from(props.age);
            if age < 7 {
                args.world.set_block_state(
                    args.position,
                    Self::state_with_age(block, state, age + 1),
                    BlockFlags::NOTIFY_NEIGHBORS,
                );
            } else {
                self.grow_gourd(args.world, args.position, block);
            }
        }
    }
}

impl PlantBlockBase for StemBlock {
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = block_accessor.get_block(pos);
        if block == &Block::PUMPKIN_STEM {
            block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_PUMPKIN_STEM)
        } else {
            block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_MELON_STEM)
        }
    }
}

impl CropBlockBase for StemBlock {}
