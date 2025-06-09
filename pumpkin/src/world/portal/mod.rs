use std::sync::Arc;

use pumpkin_util::math::position::BlockPos;

use super::World;

pub mod end;
pub mod nether;

pub struct PortalManager {
    pub portal_delay: u32,
    pub portal_world: Arc<World>,
    pub pos: BlockPos,
    pub ticks_in_portal: u32,
    pub in_portal: bool,
}

impl PortalManager {
    pub fn new(portal_delay: u32, portal_world: Arc<World>, pos: BlockPos) -> Self {
        Self {
            portal_delay,
            portal_world,
            pos,
            ticks_in_portal: 0,
            in_portal: true,
        }
    }

    pub fn tick(&mut self) -> bool {
        if self.in_portal {
            self.in_portal = false;
            self.ticks_in_portal += 1;
            self.ticks_in_portal >= self.portal_delay
        } else {
            if self.ticks_in_portal < 4 {
                self.ticks_in_portal = 0;
            } else {
                self.ticks_in_portal -= 4;
            }
            false
        }
    }
}
