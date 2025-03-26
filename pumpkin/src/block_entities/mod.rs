use std::sync::Arc;

use chest::Chest;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::block_entity::{BlockEntity, block_entity_from_generic};

pub mod chest;

pub fn block_entity_from_nbt(nbt: &NbtCompound) -> Option<Arc<dyn BlockEntity>> {
    let id = nbt.get_string("id").unwrap();
    match id.as_str() {
        Chest::ID => Some(Arc::new(block_entity_from_generic::<Chest>(nbt))),
        _ => None,
    }
}
