use crate::entity::boss::ender_dragon::EnderDragonEntity;
use futures::future::BoxFuture;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

mod charging;
mod circling;
mod dying;
mod fly_to_portal;
mod hovering;
mod landing;
mod landing_approach;
mod sit_attacking;
mod sit_breathing;
mod strafing;
mod taking_off;

pub use charging::ChargingPhase;
pub use circling::CirclingPhase;
pub use dying::DyingPhase;
pub use fly_to_portal::FlyToPortalPhase;
pub use hovering::HoveringPhase;
pub use landing::LandingPhase;
pub use landing_approach::LandingApproachPhase;
pub use sit_attacking::SitAttackingPhase;
pub use sit_breathing::SitBreathingPhase;
pub use strafing::StrafingPhase;
pub use taking_off::TakingOffPhase;

pub trait Phase: Send + Sync {
    fn get_type(&self) -> EnderDragonPhase;
    fn tick<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()>;
    fn begin<'a>(&'a self, _dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn end<'a>(&'a self, _dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn is_sitting(&self) -> bool {
        false
    }
    fn get_fly_speed(&self) -> f32 {
        0.6
    }
    fn get_turn_speed(&self) -> f32 {
        0.1
    }
    fn get_fly_target_location(&self) -> Option<Vector3<f64>> {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EnderDragonPhase {
    #[default]
    Circling = 0,
    Strafing = 1,
    Charging = 2,
    FlyToPortal = 3,
    LandingApproach = 4,
    Landing = 5,
    SitAttacking = 6,
    SitBreathing = 7,
    TakingOff = 8,
    Hovering = 9,
    Dying = 10,
}

impl EnderDragonPhase {
    #[must_use]
    pub const fn is_sitting(self) -> bool {
        matches!(
            self,
            Self::LandingApproach
                | Self::Landing
                | Self::SitAttacking
                | Self::SitBreathing
                | Self::TakingOff
        )
    }

    #[must_use]
    pub const fn network_id(self) -> i32 {
        self as i32
    }
}

pub struct PhaseManager {
    phases: [Arc<dyn Phase>; 11],
}

impl Default for PhaseManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PhaseManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            phases: [
                Arc::new(CirclingPhase),
                Arc::new(StrafingPhase),
                Arc::new(ChargingPhase),
                Arc::new(FlyToPortalPhase),
                Arc::new(LandingApproachPhase),
                Arc::new(LandingPhase),
                Arc::new(SitAttackingPhase),
                Arc::new(SitBreathingPhase),
                Arc::new(TakingOffPhase),
                Arc::new(HoveringPhase),
                Arc::new(DyingPhase),
            ],
        }
    }

    #[must_use]
    pub fn get_phase(&self, phase_type: EnderDragonPhase) -> Arc<dyn Phase> {
        self.phases[phase_type as usize].clone()
    }
}
