use crate::identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceKey {
    registry_name: Identifier,
    pub identifier: Identifier,
}

impl ResourceKey {
    #[must_use]
    pub const fn new(registry_name: Identifier, identifier: Identifier) -> Self {
        Self {
            registry_name,
            identifier,
        }
    }

    #[must_use]
    pub fn cast(&self, registry: &Identifier) -> Option<&Self> {
        (self.registry_name == *registry).then_some(self)
    }
}
