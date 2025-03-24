use pumpkin_data::fluid::{Fluid, FluidState};


pub fn get_fluid(registry_id: &str) -> Option<Fluid> {
    let key = registry_id.replace("minecraft:", "");
    Fluid::from_registry_key(key.as_str())
}

pub fn get_fluid_by_id(id: u16) -> Option<Fluid> {
    Fluid::from_id(id)
}

pub fn get_state_by_state_id(id: u16) -> Option<FluidState> {
    if let Some(fluid) = Fluid::from_state_id(id) {
        let state: &FluidState = fluid.states.iter().find(|state| state.block_state_id == id)?;
        Some(state.clone())
    } else {
        None
    }
}

pub fn get_fluid_by_state_id(id: u16) -> Option<Fluid> {
    Fluid::from_state_id(id)
}

pub fn get_fluid_and_state_by_state_id(id: u16) -> Option<(Fluid, FluidState)> {
    if let Some(fluid) = Fluid::from_state_id(id) {
        let state: &FluidState = fluid.states.iter().find(|state| state.block_state_id == id)?;
        Some((fluid, state.clone()))
    } else {
        None
    }
}


