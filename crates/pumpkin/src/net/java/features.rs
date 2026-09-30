/// What a Java client's protocol has beyond the packet format.
///
/// Clients on `CURRENT_MC_VERSION` have [`Self::CURRENT`]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct JavaConnectionFeatures {
    /// Login goes through the configuration state.
    pub configuration_state: bool,
    /// Chunks go in batches the client acknowledges.
    pub chunk_batch_acks: bool,
    /// The client reports when it has loaded the world.
    pub player_loaded: bool,
    /// An empty encryption verify token is accepted.
    pub optional_verify_token: bool,
}

impl JavaConnectionFeatures {
    pub const CURRENT: Self = Self {
        configuration_state: true,
        chunk_batch_acks: true,
        player_loaded: true,
        optional_verify_token: false,
    };
}

impl Default for JavaConnectionFeatures {
    fn default() -> Self {
        Self::CURRENT
    }
}
