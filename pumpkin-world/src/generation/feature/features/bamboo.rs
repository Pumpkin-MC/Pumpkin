use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{
        BambooLeaves, BambooLikeProperties, BlockProperties, Integer0To1, get_state_by_state_id,
    },
    tag::Tagable,
};
use pumpkin_util::{
    math::{position::BlockPos, vector2::Vector2},
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::{ProtoChunk, world::BlockRegistryExt};

#[derive(Deserialize)]
pub struct BambooFeature {
    probability: f32,
}

impl BambooFeature {
    #[expect(clippy::too_many_arguments)]
    pub async fn generate(
        &self,
        chunk: &mut ProtoChunk<'_>,
        block_registry: &dyn BlockRegistryExt,
        _min_y: i8,
        _height: u16,
        _feature: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut i = 0;
        if chunk.is_air(&pos.0) {
            if block_registry
                .can_place_at(&Block::BAMBOO, chunk, &pos, BlockDirection::Up)
                .await
            {
                let height = random.next_bounded_i32(12) + 5;
                if random.next_f32() < self.probability {
                    let rnd = random.next_bounded_i32(4) + 1;
                    for x in pos.0.x - rnd..pos.0.x + rnd {
                        for z in pos.0.z - rnd..pos.0.z + rnd {
                            let block_below = BlockPos::new(
                                x,
                                chunk.top_block_height_exclusive(&Vector2::new(x, z)) as i32 - 1,
                                z,
                            );
                            let block = chunk.get_block_state(&block_below.0);
                            if !block.to_block().is_tagged_with("minecraft:dirt").unwrap() {
                                continue;
                            }
                            chunk.set_block_state(
                                &block_below.0,
                                &get_state_by_state_id(Block::PODZOL.id).unwrap(),
                            );
                        }
                    }
                }
                let mut bpos = pos;
                let bamboo = get_state_by_state_id(Block::BAMBOO.default_state_id).unwrap();
                for _ in 0..height {
                    if chunk.is_air(&bpos.0) {
                        chunk.set_block_state(&bpos.0, &bamboo);
                        bpos = bpos.up();
                    } else {
                        break;
                    }
                }
                // Top block
                if bpos.0.y - pos.0.y >= 3 {
                    let mut props = BambooLikeProperties::default(&Block::BAMBOO);
                    props.leaves = BambooLeaves::Large;
                    props.stage = Integer0To1::L1;

                    chunk.set_block_state(
                        &bpos.0,
                        &get_state_by_state_id(props.to_state_id(&Block::BAMBOO)).unwrap(),
                    );
                    props.stage = Integer0To1::L0;

                    chunk.set_block_state(
                        &bpos.down().0,
                        &get_state_by_state_id(props.to_state_id(&Block::BAMBOO)).unwrap(),
                    );
                    props.leaves = BambooLeaves::Small;

                    chunk.set_block_state(
                        &bpos.down().down().0,
                        &get_state_by_state_id(props.to_state_id(&Block::BAMBOO)).unwrap(),
                    );
                }
            }
            i += 1;
        }
        i > 0
    }
}
