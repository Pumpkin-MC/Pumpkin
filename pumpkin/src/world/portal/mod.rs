use std::sync::Arc;

use pumpkin_util::math::position::BlockPos;

use crate::entity::Entity;

use super::World;

pub mod nether;

pub trait Portal: Send + Sync {
    fn get_delay(&self) -> u32 {
        0
    }
    fn get_world(&self, entity: &Entity) -> Arc<World>;
}

pub struct PortalManager {
    pub portal: Arc<dyn Portal>,
    pub pos: BlockPos,
    ticks_in_portal: u32,
    pub in_portal: bool,
}

impl PortalManager {
    pub fn new(portal: Arc<dyn Portal>, pos: BlockPos) -> Self {
        Self {
            portal,
            pos,
            ticks_in_portal: 0,
            in_portal: true,
        }
    }

    pub fn tick(&mut self) -> bool {
        if self.in_portal {
            self.in_portal = false;
            self.ticks_in_portal += 1;
            self.ticks_in_portal >= self.portal.get_delay()
        } else {
            self.ticks_in_portal -= 4;
            false
        }
    }
}
