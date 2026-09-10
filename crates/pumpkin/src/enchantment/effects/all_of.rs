use pumpkin_data::enchantment::EnchantmentEntityEffect;

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

impl super::EnchantmentEntityEffectExt for AllOf<'_> {
    fn apply(
        &self,
        world: &std::sync::Arc<crate::world::World>,
        enchantment_level: i32,
        owner: Option<&std::sync::Arc<crate::entity::player::Player>>,
        entity: Option<&crate::entity::Entity>,
        position: pumpkin_util::math::vector3::Vector3<f64>,
    ) {
        for effect in self.effects {
            effect.apply(world, enchantment_level, owner, entity, position);
        }
    }
}
