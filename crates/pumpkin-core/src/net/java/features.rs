bitflags::bitflags! {
    /// What a Java client's protocol has beyond the packet format.
    ///
    /// Clients on `CURRENT_MC_VERSION` have [`Self::CURRENT`]
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct JavaConnectionFeatures: u8 {
        /// Login goes through the configuration state.
        const CONFIGURATION_STATE = 1 << 0;
        /// Chunks go in batches the client acknowledges.
        const CHUNK_BATCH_ACKS = 1 << 1;
        /// The client reports when it has loaded the world.
        const PLAYER_LOADED = 1 << 2;
        /// An empty encryption verify token is accepted.
        const OPTIONAL_VERIFY_TOKEN = 1 << 3;
    }
}

impl JavaConnectionFeatures {
    pub const CURRENT: Self = Self::CONFIGURATION_STATE
        .union(Self::CHUNK_BATCH_ACKS)
        .union(Self::PLAYER_LOADED);
}

impl Default for JavaConnectionFeatures {
    fn default() -> Self {
        Self::CURRENT
    }
}
