use pumpkin_data::{
    BlockState, BlockStateId,
    item::JavaToBedrockItemMapping,
    tag::{Block as BlockTag, Taggable},
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use super::BlockEntity;

pub struct BedBlockEntity {
    pub position: BlockPos,
}

impl BlockEntity for BedBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(_nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        Self { position }
    }

    fn write_nbt(&self, _nbt: &mut NbtCompound) {}

    fn bedrock_block_actor_data(&self, state_id: BlockStateId) -> Option<NbtCompound> {
        let (block, _) = BlockState::from_id_with_block(state_id);
        if !block.has_tag(&BlockTag::MINECRAFT_BEDS) {
            return None;
        }
        let color = JavaToBedrockItemMapping::from_java_item_id(block.item_id)?
            .bedrock_data
            .try_into()
            .ok()?;
        let mut nbt = NbtCompound::new();
        nbt.put_string("id", "Bed".to_string());
        nbt.put_int("x", self.position.0.x);
        nbt.put_int("y", self.position.0.y);
        nbt.put_int("z", self.position.0.z);
        nbt.put_byte("color", color);
        nbt.put_bool("isMovable", true);
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl BedBlockEntity {
    pub const ID: &'static str = "minecraft:bed";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self { position }
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::Block;
    use pumpkin_util::math::position::BlockPos;

    use super::{BedBlockEntity, BlockEntity};

    #[test]
    fn bedrock_block_actor_uses_the_java_bed_color() {
        let entity = BedBlockEntity::new(BlockPos::new(5, 64, 7));
        let blue = entity
            .bedrock_block_actor_data(Block::BLUE_BED.default_state.id)
            .unwrap();
        let red = entity
            .bedrock_block_actor_data(Block::RED_BED.default_state.id)
            .unwrap();

        assert_eq!(blue.get_string("id"), Some("Bed"));
        assert_eq!(blue.get_byte("color"), Some(11));
        assert_eq!(red.get_byte("color"), Some(14));
        assert_eq!(blue.get_int("x"), Some(5));
        assert_eq!(blue.get_bool("isMovable"), Some(true));
        assert!(
            entity
                .bedrock_block_actor_data(Block::STONE.default_state.id)
                .is_none()
        );
    }
}
