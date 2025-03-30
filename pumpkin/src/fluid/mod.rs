use std::sync::Arc;

use fluids::{lava::FlowingLava, water::FlowingWater};
use registry::FluidRegistry;

mod fluids;

pub mod pumpkin_fluid;
pub mod registry;

#[must_use]
pub fn default_registry() -> Arc<FluidRegistry> {
    let mut manager = FluidRegistry::default();

    manager.register(FlowingWater);
    manager.register(FlowingLava);

    Arc::new(manager)
}
