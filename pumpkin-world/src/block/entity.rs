use std::pin::Pin;

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait BlockEntityData {
    fn get_position(&self) -> BlockPos;
    fn get_id(&self) -> u32;
    fn chunk_data_nbt(&self) -> Option<NbtCompound>;
}

#[allow(clippy::len_without_is_empty)]
pub trait BlockEntityCollection: Default + Send + Sync + 'static {
    type BlockEntity: BlockEntityData;
    fn from_nbt_entries(nbt_entries: &[NbtCompound]) -> Self;
    fn to_nbt_entries<'a>(&'a self) -> BoxedFuture<'a, Vec<NbtCompound>>;
    fn len(&self) -> usize;
    fn get_all(&self) -> Vec<Self::BlockEntity>;
}
