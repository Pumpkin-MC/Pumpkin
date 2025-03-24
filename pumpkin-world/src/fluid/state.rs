use crate::chunk::format::PaletteEntry;

use super::registry::{get_fluid, get_fluid_by_id};

#[derive(Clone, Copy, Debug, Eq)]
pub struct ChunkFluidState {
    pub state_id: u16,
    pub fluid_id: u16,
}

impl PartialEq for ChunkFluidState {
    fn eq(&self, other: &Self) -> bool {
        self.state_id == other.state_id
    }
}

impl ChunkFluidState {
    pub const EMPTY: ChunkFluidState = ChunkFluidState {
        state_id: 0,
        fluid_id: 0,
    };

    /// Get a Fluid from the Vanilla Fluid registry at Runtime
    pub fn new(registry_id: &str) -> Option<Self> {
        let fluid = get_fluid(registry_id);
        fluid.map(|fluid| Self {
            state_id: fluid.default_state_index,
            fluid_id: fluid.id,
        })
    }

    pub fn new_by_id(fluid_id: u16) -> Option<Self> {
        let fluid = get_fluid_by_id(fluid_id as u16);
        fluid.map(|fluid| Self {
            state_id: fluid.default_state_index,
            fluid_id: fluid.id,
        })
    }

    pub fn from_palette(palette: &PaletteEntry) -> Self {
        let fluid = get_fluid(palette.name.as_str());

        if let Some(fluid) = fluid {
            let mut state_id = fluid.default_state_index;

            if let Some(properties) = palette.properties.clone() {
                let mut properties_vec = Vec::new();
                for (key, value) in properties {
                    properties_vec.push((key.clone(), value.clone()));
                }
                let fluid_properties = fluid.from_properties(properties_vec).unwrap();
                state_id = fluid_properties.to_state_id(&fluid);
            }

            return Self {
                state_id,
                fluid_id: fluid.id,
            };
        }

        ChunkFluidState::EMPTY
    }

    pub fn get_id(&self) -> u16 {
        self.state_id
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.state_id == 0
    }

    #[inline]
    pub fn of_fluid(&self, fluid_id: u16) -> bool {
        self.fluid_id == fluid_id
    }
}

#[cfg(test)]
mod tests {
    use super::ChunkFluidState;

    #[test]
    fn not_existing() {
        let state = ChunkFluidState::new("this_fluid_does_not_exist");
        assert!(state.is_none());
    }

    #[test]
    fn does_exist() {
        let state = ChunkFluidState::new("water");
        assert!(state.is_some());
    }
}
