use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerInteractUnknownEntityEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player targets an entity id that is unknown to the server.
///
/// The associated [`PlayerInteractUnknownEntityEventData`] contains the player, the missing
/// entity id, and the interaction action. This event is cancellable.
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
