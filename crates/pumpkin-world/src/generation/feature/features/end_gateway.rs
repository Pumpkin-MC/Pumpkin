use pumpkin_data::Block;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::generation::proto_chunk::GenerationCache;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EndGatewayFeature {
    pub exit: Option<BlockPos>,
    pub exact: bool,
}

impl EndGatewayFeature {
    #[must_use]
    pub const fn known_exit(exit: BlockPos, exact: bool) -> Self {
        Self {
            exit: Some(exit),
            exact,
        }
    }

    #[must_use]
    pub const fn delayed_exit_search() -> Self {
        Self {
            exit: None,
            exact: false,
        }
    }

    pub fn generate<T: GenerationCache>(&self, chunk: &mut T, pos: BlockPos) -> bool {
        for dx in -1i32..=1 {
            for dy in -2i32..=2 {
                for dz in -1i32..=1 {
                    let same_x = dx == 0;
                    let same_y = dy == 0;
                    let same_z = dz == 0;
                    let end = dy.abs() == 2;
                    let target = Vector3::new(pos.0.x + dx, pos.0.y + dy, pos.0.z + dz);

                    if same_x && same_y && same_z {
                        chunk.set_block_state(&target, Block::END_GATEWAY.default_state);
                        if let Some(exit) = self.exit {
                            let mut entity_nbt = NbtCompound::new();
                            entity_nbt.put_string("id", "minecraft:end_gateway".to_string());
                            entity_nbt.put_int("x", target.x);
                            entity_nbt.put_int("y", target.y);
                            entity_nbt.put_int("z", target.z);
                            entity_nbt.put_long("Age", 0);
                            entity_nbt.put_bool("ExactTeleport", self.exact);
                            let mut exit_nbt = NbtCompound::new();
                            exit_nbt.put_int("X", exit.0.x);
                            exit_nbt.put_int("Y", exit.0.y);
                            exit_nbt.put_int("Z", exit.0.z);
                            entity_nbt.put_compound("ExitPortal", exit_nbt);
                            chunk.add_block_entity(&target, entity_nbt);
                        }
                    } else if same_y {
                        chunk.set_block_state(&target, Block::AIR.default_state);
                    } else if (end && same_x && same_z) || ((same_x || same_z) && !end) {
                        chunk.set_block_state(&target, Block::BEDROCK.default_state);
                    } else {
                        chunk.set_block_state(&target, Block::AIR.default_state);
                    }
                }
            }
        }
        true
    }
}
