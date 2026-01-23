use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    RandomTickArgs,
    blocks::plant::{PlantBlockBase, crop::get_available_moisture},
};
use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{
        BlockProperties, EnumVariants, Integer0To7, WallTorchLikeProperties, WheatLikeProperties,
    },
    tag,
    tag::Taggable,
};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, xoroshiro128::Xoroshiro},
};
use pumpkin_world::{
    BlockStateId,
    world::{BlockAccessor, BlockFlags},
};
use rand::Rng;

type StemProperties = WheatLikeProperties;
type AttachedStemProperties = WallTorchLikeProperties;

pub struct StemBlock;

impl BlockMetadata for StemBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[Block::PUMPKIN_STEM.name, Block::MELON_STEM.name]
    }
}

impl StemBlock {
    fn state_with_age(block: &Block, state: u16, age: i32) -> BlockStateId {
        let mut props = StemProperties::from_state_id(state, block);
        props.age = Integer0To7::from_index(age as u16);
        props.to_state_id(block)
    }

    fn get_attached_stem(dir: BlockDirection, block: &Block) -> BlockStateId {
        let attached_block = match block.id {
            id if id == Block::PUMPKIN_STEM.id => &Block::ATTACHED_PUMPKIN_STEM,
            id if id == Block::MELON_STEM.id => &Block::ATTACHED_MELON_STEM,
            _ => &Block::ATTACHED_MELON_STEM, // Should never happen
        };
        let mut props = AttachedStemProperties::default(attached_block);
        props.facing = dir.to_horizontal_facing().unwrap();
        props.to_state_id(attached_block)
    }

    fn get_gourd(block: &Block) -> &Block {
        match block.id {
            id if id == Block::PUMPKIN_STEM.id => &Block::PUMPKIN,
            id if id == Block::MELON_STEM.id => &Block::MELON,
            _ => &Block::MELON, // Should never happen
        }
    }
}

impl BlockBehaviour for StemBlock {
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

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // TODO add light level check
            let f: f32 = get_available_moisture(args.world, args.position, args.block).await;
            if rand::rng().random_range(0..=(25.0 / f).floor() as i32) == 0 {
                let (block, state) = args.world.get_block_and_state_id(args.position).await;
                let props = StemProperties::from_state_id(state, block);
                let age = i32::from(props.age.to_index());
                if age < 7 {
                    args.world
                        .set_block_state(
                            args.position,
                            Self::state_with_age(block, state, age + 1),
                            BlockFlags::NOTIFY_NEIGHBORS,
                        )
                        .await;
                } else {
                    let dir = BlockDirection::random_horizontal(&mut RandomGenerator::Xoroshiro(
                        Xoroshiro::from_seed(rand::rng().random()),
                    ));
                    let plant_block_pos = args.position.offset(dir.to_offset());
                    let plant_block_state = args.world.get_block_state(&plant_block_pos).await;
                    let under_block: &Block = args.world.get_block(&plant_block_pos.down()).await;
                    if plant_block_state.is_air()
                        && (under_block == &Block::FARMLAND
                            || under_block.has_tag(&tag::Block::MINECRAFT_DIRT))
                    {
                        let attached_stem = Self::get_attached_stem(dir, block);
                        let gourd = Self::get_gourd(block);
                        args.world
                            .set_block_state(
                                &plant_block_pos,
                                gourd.default_state.id,
                                BlockFlags::NOTIFY_NEIGHBORS,
                            )
                            .await;
                        args.world
                            .set_block_state(
                                args.position,
                                attached_stem,
                                BlockFlags::NOTIFY_NEIGHBORS,
                            )
                            .await;
                    }
                }
            }
        })
    }
}

impl PlantBlockBase for StemBlock {
    async fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let block = block_accessor.get_block(pos).await;
        block == &Block::FARMLAND
    }
}
