use std::fmt;
use std::sync::Arc;

use crate::helper::GameTestFunction;
use crate::model::{GameTestDefinition, GameTestRotation, TestType};

#[derive(Clone)]
pub struct BlockBasedTest {
    id: String,
    definition: GameTestDefinition,
    function: Option<Arc<dyn GameTestFunction>>,
}

impl fmt::Debug for BlockBasedTest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockBasedTest")
            .field("id", &self.id)
            .field("definition", &self.definition)
            .field("has_function", &self.function.is_some())
            .finish()
    }
}

impl BlockBasedTest {
    #[must_use]
    pub fn new(id: impl Into<String>, definition: GameTestDefinition) -> Self {
        Self {
            id: id.into(),
            definition,
            function: None,
        }
    }

    #[must_use]
    pub fn with_function(mut self, function: Arc<dyn GameTestFunction>) -> Self {
        self.function = Some(function);
        self
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
    pub const fn instance_type(&self) -> TestType {
        self.definition.instance_type
    }

    #[must_use]
    pub fn function(&self) -> Option<&Arc<dyn GameTestFunction>> {
        self.function.as_ref()
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
