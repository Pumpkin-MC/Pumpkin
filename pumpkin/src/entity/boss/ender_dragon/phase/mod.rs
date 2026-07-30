use crate::entity::boss::ender_dragon::EnderDragonEntity;
use crate::entity::player::Player;
use futures::future::BoxFuture;
use std::sync::Arc;

mod charging;
mod dying;
mod holding_pattern;
mod hovering;
mod landing;
mod landing_approach;
mod sit_attacking;
mod sit_breathing;
mod sit_scanning;
mod strafing;
mod taking_off;

pub use charging::ChargingPhase;
pub use dying::DyingPhase;
pub use holding_pattern::HoldingPatternPhase;
pub use hovering::HoveringPhase;
pub use landing::LandingPhase;
pub use landing_approach::LandingApproachPhase;
pub use sit_attacking::SitAttackingPhase;
pub use sit_breathing::SitFlamingPhase;
pub use sit_scanning::SitScanningPhase;
pub use strafing::StrafePlayerPhase;
pub use taking_off::TakeoffPhase;

pub trait Phase: Send + Sync {
    fn get_type(&self) -> EnderDragonPhase;
    fn tick<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()>;
    fn begin<'a>(&'a self, _dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn end<'a>(&'a self, _dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn is_sitting_or_hovering(&self) -> bool {
        false
    }
    fn get_max_y_acceleration(&self) -> f32 {
        0.6
    }
    /// Java: `getFlySpeed()` — controls how fast `moveRelative` accelerates
    /// the dragon toward its target.  Different from `get_max_y_acceleration`
    /// which only clamps the Y component.  Default 0.6 matches
    /// `AbstractDragonPhaseInstance.getFlySpeed()`.
    fn get_fly_speed(&self) -> f32 {
        0.6
    }
    fn get_yaw_acceleration<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, f32> {
        Box::pin(async move {
            let vel = dragon.mob_entity.living_entity.entity.velocity.load();
            let f = vel.horizontal_length() as f32 + 1.0;
            let g = f.min(40.0);
            0.7 / g / f
        })
    }
    fn modify_damage_taken(&self, amount: f32) -> f32 {
        // TODO: Sitting phases (AbstractSittingPhase in Java) override this to
        // return 0.0 for projectile damage (PersistentProjectileEntity, WindChargeEntity)
        // and set the projectile on fire.  Blocked on implementing projectile type
        // detection on DamageSource.
        amount
    }
    /// Called when an end crystal is destroyed, matching
    /// `Phase.crystalDestroyed(EndCrystalEntity, BlockPos, DamageSource, PlayerEntity)`.
    /// `player` is whoever destroyed the crystal (the attacker, or the closest player to
    /// it if there was none), already resolved by `EnderDragonEntity::crystal_destroyed`.
    fn crystal_destroyed<'a>(
        &'a self,
        _dragon: &'a EnderDragonEntity,
        _player: Option<Arc<Player>>,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EnderDragonPhase {
    #[default]
    HoldingPattern = 0,
    StrafePlayer = 1,
    LandingApproach = 2,
    Landing = 3,
    Takeoff = 4,
    SittingFlaming = 5,
    SittingScanning = 6,
    SittingAttacking = 7,
    ChargingPlayer = 8,
    Dying = 9,
    Hover = 10,
}

impl EnderDragonPhase {
    #[must_use]
    pub const fn is_sitting_or_hovering(self) -> bool {
        matches!(
            self,
            Self::SittingFlaming | Self::SittingScanning | Self::SittingAttacking | Self::Hover
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
                Arc::new(HoldingPatternPhase),
                Arc::new(StrafePlayerPhase),
                Arc::new(LandingApproachPhase),
                Arc::new(LandingPhase),
                Arc::new(TakeoffPhase),
                Arc::new(SitFlamingPhase),
                Arc::new(SitScanningPhase),
                Arc::new(SitAttackingPhase),
                Arc::new(ChargingPhase),
                Arc::new(DyingPhase),
                Arc::new(HoveringPhase),
            ],
        }
    }

    #[must_use]
    pub fn get_phase(&self, phase_type: EnderDragonPhase) -> Arc<dyn Phase> {
        self.phases[phase_type as usize].clone()
    }
}
