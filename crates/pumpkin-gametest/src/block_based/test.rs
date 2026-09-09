use crate::model::{GameTestDefinition, GameTestRotation};

#[derive(Clone, Debug)]
pub struct BlockBasedTest {
    id: String,
    definition: GameTestDefinition,
}

impl BlockBasedTest {
    #[must_use]
    pub fn new(id: impl Into<String>, definition: GameTestDefinition) -> Self {
        Self {
            id: id.into(),
            definition,
        }
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub const fn definition(&self) -> &GameTestDefinition {
        &self.definition
    }

    #[must_use]
    pub const fn max_ticks(&self) -> u32 {
        self.definition.max_ticks as u32
    }

    #[must_use]
    pub const fn setup_ticks(&self) -> u32 {
        self.definition.setup_ticks as u32
    }

    #[must_use]
    pub const fn max_attempts(&self) -> u32 {
        self.definition.max_attempts as u32
    }

    #[must_use]
    pub const fn required_successes(&self) -> u32 {
        self.definition.required_successes as u32
    }

    #[must_use]
    pub const fn is_required(&self) -> bool {
        self.definition.required
    }

    #[must_use]
    pub const fn rotation(&self) -> GameTestRotation {
        self.definition.rotation
    }
}
