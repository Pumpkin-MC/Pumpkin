use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerInteractUnknownEntityEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player interacts with an entity ID that the server cannot resolve.
///
/// The associated [`PlayerInteractUnknownEntityEventData`] contains the player, the unknown
/// entity ID, and the attempted interaction action. This event is cancellable.
pub struct PlayerInteractUnknownEntityEvent;

impl FromIntoEvent for PlayerInteractUnknownEntityEvent {
    const EVENT_TYPE: EventType = EventType::PlayerInteractUnknownEntityEvent;
    type Data = PlayerInteractUnknownEntityEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerInteractUnknownEntityEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerInteractUnknownEntityEvent(data)
    }
}
