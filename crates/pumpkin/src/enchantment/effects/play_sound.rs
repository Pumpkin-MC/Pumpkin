/// Matches vanilla `net.minecraft.world.item.enchantment.effects.PlaySound`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaySound {
    pub sound: &'static str,
}

impl PlaySound {
    #[must_use]
    pub const fn new(sound: &'static str) -> Self {
        Self { sound }
    }
}
