use crate::entity::EntityBase;
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
    /// Vanilla `DragonPhaseInstance.onHurt` (`AbstractDragonPhaseInstance.java:54-57`):
    /// identity by default, called before the head/non-head damage split in
    /// `EnderDragon.hurt` (`EnderDragon.java:451`).
    fn on_hurt(&self, _dragon: &EnderDragonEntity, _source: &dyn EntityBase, damage: f32) -> f32 {
        damage
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

    /// Inverse of the ordinal used by `write_nbt`/`network_id`. Falls back to the
    /// default phase for an out-of-range id, matching vanilla `EnderDragonPhase.getById`'s
    /// fallback-to-a-known-phase behavior for corrupt saves.
    #[must_use]
    pub const fn from_ordinal(id: i32) -> Self {
        match id {
            1 => Self::Strafing,
            2 => Self::Charging,
            3 => Self::FlyToPortal,
            4 => Self::LandingApproach,
            5 => Self::Landing,
            6 => Self::SitAttacking,
            7 => Self::SitBreathing,
            8 => Self::TakingOff,
            9 => Self::Hovering,
            10 => Self::Dying,
            _ => Self::Circling,
        }
    }
}

/// Shared `AbstractDragonSittingPhase.onHurt` body (`AbstractDragonSittingPhase.java:18-26`):
/// arrows, tridents and thrown wind charges deal no damage while sitting and ignite the
/// projectile for a second instead; anything else passes through unchanged. Used by the
/// three Pumpkin phases that model vanilla's `DragonSittingScanningPhase`,
/// `DragonSittingAttackingPhase` and `DragonSittingFlamingPhase`.
pub(super) fn sitting_on_hurt(source: &dyn EntityBase, damage: f32) -> f32 {
    use pumpkin_data::entity::EntityType;

    let id = source.get_entity().entity_type.id;
    let is_arrow_or_wind_charge = id == EntityType::ARROW.id
        || id == EntityType::SPECTRAL_ARROW.id
        || id == EntityType::TRIDENT.id
        || id == EntityType::WIND_CHARGE.id;

    if is_arrow_or_wind_charge {
        source.set_on_fire_for(1.0);
        0.0
    } else {
        damage
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

#[cfg(test)]
mod tests {
    use super::EnderDragonPhase;

    #[test]
    fn from_ordinal_round_trips_all_phases() {
        let phases = [
            EnderDragonPhase::Circling,
            EnderDragonPhase::Strafing,
            EnderDragonPhase::Charging,
            EnderDragonPhase::FlyToPortal,
            EnderDragonPhase::LandingApproach,
            EnderDragonPhase::Landing,
            EnderDragonPhase::SitAttacking,
            EnderDragonPhase::SitBreathing,
            EnderDragonPhase::TakingOff,
            EnderDragonPhase::Hovering,
            EnderDragonPhase::Dying,
        ];
        for phase in phases {
            assert_eq!(EnderDragonPhase::from_ordinal(phase.network_id()), phase);
        }
    }

    #[test]
    fn from_ordinal_falls_back_to_circling_for_corrupt_ids() {
        assert_eq!(
            EnderDragonPhase::from_ordinal(-1),
            EnderDragonPhase::Circling
        );
        assert_eq!(
            EnderDragonPhase::from_ordinal(999),
            EnderDragonPhase::Circling
        );
    }
}
