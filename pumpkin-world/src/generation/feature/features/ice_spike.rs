use pumpkin_data::{Block, block_properties::is_air, tag};
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct IceSpikeFeature {}

impl IceSpikeFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut origin = pos.0;

        // Walk down while block is air, staying above minY + 2
        while chunk.is_air(&origin) && origin.y > chunk.bottom_y() as i32 + 2 {
            origin.y -= 1;
        }

        // Only place on snow blocks
        let block_id = GenerationCache::get_block_state(chunk, &origin).to_block_id();
        if block_id != Block::SNOW_BLOCK.id {
            return false;
        }

        origin.y += random.next_bounded_i32(4);
        let height = random.next_bounded_i32(4) + 7;
        let width = height / 4 + random.next_bounded_i32(2);

        // Occasionally create extra-tall spires
        if width > 1 && random.next_bounded_i32(60) == 0 {
            origin.y += 10 + random.next_bounded_i32(30);
        }

        for y_off in 0..height {
            let scale = (1.0f32 - y_off as f32 / height as f32) * width as f32;
            let new_width = scale.ceil() as i32;

            for xo in -new_width..=new_width {
                let dx = xo.abs() as f32 - 0.25f32;

                for zo in -new_width..=new_width {
                    let dz = zo.abs() as f32 - 0.25f32;

                    let in_sphere = (xo == 0 && zo == 0) || (dx * dx + dz * dz <= scale * scale);
                    let edge_ok = (xo != -new_width
                        && xo != new_width
                        && zo != -new_width
                        && zo != new_width)
                        || (random.next_f32() <= 0.75f32);

                    if in_sphere && edge_ok {
                        // Place packed ice upward
                        let place_pos = origin.add(&Vector3::new(xo, y_off, zo));
                        let raw = GenerationCache::get_block_state(chunk, &place_pos);
                        let bid = raw.to_block_id();
                        if is_air(raw.0)
                            || tag::Block::MINECRAFT_DIRT.1.contains(&bid)
                            || bid == Block::SNOW_BLOCK.id
                            || bid == Block::ICE.id
                        {
                            chunk.set_block_state(&place_pos, Block::PACKED_ICE.default_state);
                        }

                        // Mirror downward
                        if y_off != 0 && new_width > 1 {
                            let neg_pos = origin.add(&Vector3::new(xo, -y_off, zo));
                            let raw2 = GenerationCache::get_block_state(chunk, &neg_pos);
                            let bid2 = raw2.to_block_id();
                            if is_air(raw2.0)
                                || tag::Block::MINECRAFT_DIRT.1.contains(&bid2)
                                || bid2 == Block::SNOW_BLOCK.id
                                || bid2 == Block::ICE.id
                            {
                                chunk.set_block_state(&neg_pos, Block::PACKED_ICE.default_state);
                            }
                        }
                    }
                }
            }
        }

        // Build packed-ice roots downward from the spike base
        let pillar_width = (width - 1).clamp(0, 1);

        for xo in -pillar_width..=pillar_width {
            for zo in -pillar_width..=pillar_width {
                let mut ice_block = origin.add(&Vector3::new(xo, -1, zo));
                // Corner positions get a short random run; others run 50 blocks
                let mut run_length: i32 = if xo.abs() == 1 && zo.abs() == 1 {
                    random.next_bounded_i32(5)
                } else {
                    50
                };

                while ice_block.y > 50 {
                    let raw = GenerationCache::get_block_state(chunk, &ice_block);
                    let bid = raw.to_block_id();
                    if !is_air(raw.0)
                        && !tag::Block::MINECRAFT_DIRT.1.contains(&bid)
                        && bid != Block::SNOW_BLOCK.id
                        && bid != Block::ICE.id
                        && bid != Block::PACKED_ICE.id
                    {
                        break;
                    }

                    chunk.set_block_state(&ice_block, Block::PACKED_ICE.default_state);
                    ice_block.y -= 1;
                    run_length -= 1;
                    if run_length <= 0 {
                        ice_block.y -= random.next_bounded_i32(5) + 1;
                        run_length = random.next_bounded_i32(5);
                    }
                }
            }
        }

        true
    }
}
