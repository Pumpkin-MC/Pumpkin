//! Vehicle events.

#[doc = "Vehicle block collision event."]
pub mod vehicle_block_collision;
#[doc = "Vehicle collision event."]
pub mod vehicle_collision;
#[doc = "Vehicle create event."]
pub mod vehicle_create;
#[doc = "Vehicle damage event."]
pub mod vehicle_damage;
#[doc = "Vehicle destroy event."]
pub mod vehicle_destroy;
#[doc = "Vehicle enter event."]
pub mod vehicle_enter;
#[doc = "Vehicle entity collision event."]
pub mod vehicle_entity_collision;
#[doc = "Vehicle exit event."]
pub mod vehicle_exit;
#[doc = "Vehicle move event."]
pub mod vehicle_move;
#[doc = "Vehicle update event."]
pub mod vehicle_update;

pub use vehicle_block_collision::*;
pub use vehicle_collision::*;
pub use vehicle_create::*;
pub use vehicle_damage::*;
pub use vehicle_destroy::*;
pub use vehicle_enter::*;
pub use vehicle_entity_collision::*;
pub use vehicle_exit::*;
pub use vehicle_move::*;
pub use vehicle_update::*;
