use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_data::{Block, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::generation::structure::template::{BlockStateResolver, PaletteEntry};

use crate::entity::decoration::display::DisplayEntity;
use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture, living::LivingEntity,
};

pub struct BlockDisplayEntity {
    display: DisplayEntity,
    block_state: AtomicCell<BlockStateId>,
}

impl BlockDisplayEntity {
    pub const fn new(entity: Entity) -> Self {
        Self {
            display: DisplayEntity::new(entity),
            block_state: AtomicCell::new(Block::AIR.default_state.id),
        }
    }
}

impl NBTStorage for BlockDisplayEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.display.write_nbt(nbt).await;

            let state = self.block_state.load().to_state();
            let block = self.block_state.load().to_block();

            let mut block_state_compound = NbtCompound::new();
            block_state_compound.put_string("Name", format!("minecraft:{}", block.name));

            if let Some(properties) = block.properties(state.id) {
                let props = properties.to_props();
                if !props.is_empty() {
                    let mut properties_compound = NbtCompound::new();
                    for (key, value) in props {
                        properties_compound.put_string(key, value.to_string());
                    }
                    block_state_compound.put_compound("Properties", properties_compound);
                }
            }

            nbt.put_compound("block_state", block_state_compound);
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.display.read_nbt_non_mut(nbt).await;

            if let Some(block_state_compound) = nbt.get_compound("block_state")
                && let Some(name) = block_state_compound.get_string("Name")
            {
                let properties = block_state_compound.get_compound("Properties").map_or_else(
                    Vec::new,
                    |props_compound| {
                        props_compound
                            .child_tags
                            .iter()
                            .filter_map(|(key, value)| {
                                if let pumpkin_nbt::tag::NbtTag::String(v) = value {
                                    Some((key.to_string(), v.to_string()))
                                } else {
                                    None
                                }
                            })
                            .collect()
                    },
                );

                let entry = PaletteEntry::with_properties(name.to_string(), properties);
                if let Some(state) = BlockStateResolver::resolve_simple(&entry) {
                    self.block_state.store(state.id);
                }
            }
        })
    }
}

impl EntityBase for BlockDisplayEntity {
    fn get_entity(&self) -> &Entity {
        &self.display.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.display.send_metadata();

            let state_id = VarInt(i32::from(self.block_state.load().as_u16()));
            self.display.entity.send_meta_data(
                &[
                    Metadata::new(
                        TrackedData::BLOCK_STATE,
                        MetaDataType::BLOCK_STATE,
                        state_id,
                    ),
                    Metadata::new(
                        TrackedData::BLOCK_STATE_ID,
                        MetaDataType::BLOCK_STATE,
                        state_id,
                    ),
                ],
                None,
            );
        })
    }

    /// `Display.hurtServer` always returns false; display entities cannot be damaged.
    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async { false })
    }
}
