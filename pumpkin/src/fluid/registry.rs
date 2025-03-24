use std::{collections::HashMap, sync::Arc};

use pumpkin_data::{fluid::Fluid, item::Item};
use crate::entity::player::Player;
use pumpkin_util::math::position::BlockPos;

use crate::{server::Server, world::World};

use super::pumpkin_fluid::{FluidMetadata, PumpkinFluid};



pub enum FluidActionResult{
    /// Allow other actions to be executed
    Continue,
    /// Block other actions
    Consume,
}

#[derive(Default)]
pub struct FluidRegistry{
    fluids: HashMap<String, Arc<dyn PumpkinFluid>>,
}

impl FluidRegistry{
    pub fn register<T: PumpkinFluid + FluidMetadata + 'static>(&mut self, fluid: T){
        self.fluids.insert(fluid.name(), Arc::new(fluid));
    }

    pub async fn on_use(
        &self,
        fluid: &Fluid,
        player: &Player,
        location: BlockPos,
        server: &Server,
        world: &World,
    ){
        let pumpkin_fluid = self.get_pumpkin_fluid(fluid);
        if let Some(pumpkin_fluid) = pumpkin_fluid{
            pumpkin_fluid
                .normal_use(fluid, player, location, server, world)
                .await;
        }
    }

    pub async fn use_with_item(
        &self,
        fluid: &Fluid,
        player: &Player,
        location: BlockPos,
        item: &Item,
        server: &Server,
        world: &World,
    ) -> FluidActionResult{
        let pumpkin_fluid = self.get_pumpkin_fluid(fluid);
        if let Some(pumpkin_fluid) = pumpkin_fluid{
            return pumpkin_fluid
                .use_with_item(fluid, player, location, item, server, world)
                .await;
        }
        FluidActionResult::Continue
    }

    pub async fn on_placed(
        &self,
        world: &World,
        fluid: &Fluid,
        state_id: u16,
        block_pos: &BlockPos,
        old_state_id: u16,
        notify: bool,
    ) {
        let pumpkin_block = self.get_pumpkin_fluid(fluid);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .placed(world, fluid, state_id, block_pos, old_state_id, notify)
                .await;
        }
    }

    #[must_use]
    pub fn get_pumpkin_fluid(&self, fluid: &Fluid) -> Option<&Arc<dyn PumpkinFluid>>{
        self.fluids.get(format!("{}:{}", "minecraft", fluid.name).as_str())
    }
}

