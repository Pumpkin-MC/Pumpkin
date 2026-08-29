use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerRespawnEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player respawns.
///
/// The associated [`PlayerRespawnEventData`] contains the player, the world they
/// respawned from, the world they respawned into, and the destination position,
/// yaw and pitch. This event is not cancellable.
pub struct PlayerRespawnEvent;
impl FromIntoEvent for PlayerRespawnEvent {
    const EVENT_TYPE: EventType = EventType::PlayerRespawnEvent;
    type Data = PlayerRespawnEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerRespawnEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerRespawnEvent(data)
    }
}
