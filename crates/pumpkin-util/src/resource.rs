use crate::identifier::Identifier;

/// A registry-scoped resource key identifying an element within a specific registry.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceKey {
    pub registry_name: Identifier,
    /// The unique identifier of the resource.
    pub identifier: Identifier,
}

impl ResourceKey {
    /// Creates a new `ResourceKey` associated with a specific registry name.
    #[must_use]
    pub const fn new(registry_name: Identifier, identifier: Identifier) -> Self {
        Self {
            registry_name,
            identifier,
        }
    }

    /// Casts the resource key if its registry name matches the provided registry identifier.
    #[must_use]
    pub fn cast(&self, registry: &Identifier) -> Option<&Self> {
        (self.registry_name == *registry).then_some(self)
    }
}
