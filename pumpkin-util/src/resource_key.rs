use crate::identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceKey {
    registry_name: Identifier,
    pub identifier: Identifier,
}

impl ResourceKey {
    pub fn new(registry_name: Identifier, identifier: Identifier) -> Self {
        Self {
            registry_name,
            identifier,
        }
    }

    pub fn cast(&self, registry: &Identifier) -> Option<&ResourceKey> {
        if self.registry_name == *registry {
            Some(self)
        } else {
            None
        }
    }
}
