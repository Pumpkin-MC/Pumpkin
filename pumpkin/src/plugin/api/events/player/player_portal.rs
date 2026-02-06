use pumpkin_macros::{cancellable, Event};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player uses a portal.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerPortalEvent {
    /// The player using the portal.
    pub player: Arc<Player>,

    /// The origin position.
    pub from_position: Vector3<f64>,

    /// The origin world UUID.
    pub from_world_uuid: uuid::Uuid,

    /// The target position.
    pub to_position: Vector3<f64>,

    /// The target world UUID.
    pub to_world_uuid: uuid::Uuid,

    /// The portal cause.
    pub cause: String,

    /// The search radius for destination portals.
    pub search_radius: i32,

    /// Whether a new portal can be created.
    pub can_create_portal: bool,

    /// The portal creation radius.
    pub creation_radius: i32,
}

impl PlayerPortalEvent {
    /// Creates a new instance of `PlayerPortalEvent`.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        player: Arc<Player>,
        from_position: Vector3<f64>,
        from_world_uuid: uuid::Uuid,
        to_position: Vector3<f64>,
        to_world_uuid: uuid::Uuid,
        cause: String,
        search_radius: i32,
        can_create_portal: bool,
        creation_radius: i32,
    ) -> Self {
        Self {
            player,
            from_position,
            from_world_uuid,
            to_position,
            to_world_uuid,
            cause,
            search_radius,
            can_create_portal,
            creation_radius,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerPortalEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
