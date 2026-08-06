/// Entity combust (catch fire) event.
pub mod entity_combust;
/// Entity damage event.
pub mod entity_damage;
/// Entity death and player death events.
pub mod entity_death;
/// Entity health regeneration event.
pub mod entity_regain_health;
/// Entity spawn event.
pub mod entity_spawn;

pub use entity_combust::*;
pub use entity_damage::*;
pub use entity_death::*;
pub use entity_regain_health::*;
pub use entity_spawn::*;
