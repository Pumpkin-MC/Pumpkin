use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::SoundCategory;

use super::mob::Mob;

/// A mob that players and dispensers can shear.
pub trait Shearable: Mob {
    fn ready_for_shearing(&self) -> bool;
    fn shear(&self, category: SoundCategory, tool: &ItemStack);
}
