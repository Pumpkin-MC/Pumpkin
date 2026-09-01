use pumpkin_data::enchantment::EnchantmentEntityEffect;

/// Matches vanilla `net.minecraft.world.item.enchantment.effects.AllOf`.
#[derive(Clone, Debug, PartialEq)]
pub struct AllOf<'a> {
    pub effects: &'a [EnchantmentEntityEffect],
}

impl<'a> AllOf<'a> {
    #[must_use]
    pub const fn new(effects: &'a [EnchantmentEntityEffect]) -> Self {
        Self { effects }
    }
}
