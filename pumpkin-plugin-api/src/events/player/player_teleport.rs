use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerTeleportEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player is teleported.
///
/// The associated [`PlayerTeleportEventData`] contains the player, the origin position,
/// and the destination position. This event is cancellable.
pub struct PlayerTeleportEvent;
impl FromIntoEvent for PlayerTeleportEvent {
    const EVENT_TYPE: EventType = EventType::PlayerTeleportEvent;
    type Data = PlayerTeleportEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerTeleportEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerTeleportEvent(data)
    }
}
