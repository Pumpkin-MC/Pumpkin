use std::sync::Arc;

use pumpkin_data::block_properties::HorizontalAxis;
use pumpkin_util::math::position::BlockPos;

use super::World;

pub mod end;
pub mod nether;
pub mod poi;

pub use nether::{NetherPortal, PortalSearchResult};
pub use poi::PortalPoiStorage;

#[derive(Clone)]
pub struct SourcePortalInfo {
    pub lower_corner: BlockPos,
    pub axis: HorizontalAxis,
    pub width: u32,
    pub height: u32,
}

impl From<&PortalSearchResult> for SourcePortalInfo {
    fn from(result: &PortalSearchResult) -> Self {
        Self {
            lower_corner: result.lower_corner,
            axis: result.axis,
            width: result.width,
            height: result.height,
        }
    }
}

pub struct PortalManager {
    pub portal_delay: u32,
    pub portal_world: Arc<World>,
    pub pos: BlockPos,
    pub ticks_in_portal: u32,
    pub in_portal: bool,
    pub source_portal: Option<SourcePortalInfo>,
}

impl PortalManager {
    pub const fn new(portal_delay: u32, portal_world: Arc<World>, pos: BlockPos) -> Self {
        Self {
            portal_delay,
            portal_world,
            pos,
            ticks_in_portal: 0,
            in_portal: true,
            source_portal: None,
        }
    }

    pub const fn set_source_portal(&mut self, info: SourcePortalInfo) {
        self.source_portal = Some(info);
    }

    pub const fn tick(&mut self) -> bool {
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
