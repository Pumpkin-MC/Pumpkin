use pumpkin_util::identifier::Identifier;
use rustc_hash::FxHashMap;

pub type NetworkId = u32;

pub struct VersionMapping {
    pub(crate) by_identifier: FxHashMap<Identifier, NetworkId>,
    pub(crate) by_network_id: FxHashMap<NetworkId, Identifier>,
}

impl VersionMapping {
    #[must_use]
    pub fn new() -> Self {
        Self {
            by_identifier: FxHashMap::default(),
            by_network_id: FxHashMap::default(),
        }
    }
}

impl Default for VersionMapping {
    fn default() -> Self {
        Self::new()
    }
}
