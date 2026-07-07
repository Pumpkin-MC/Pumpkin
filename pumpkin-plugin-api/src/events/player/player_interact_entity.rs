use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerInteractEntityEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player interacts with a resolved entity.
///
/// The associated [`PlayerInteractEntityEventData`] contains the player, target
/// entity, interaction action, optional target position, sneaking state, and
/// cancellation state. This event is cancellable.
pub struct PlayerInteractEntityEvent;

impl FromIntoEvent for PlayerInteractEntityEvent {
    const EVENT_TYPE: EventType = EventType::PlayerInteractEntityEvent;
    type Data = PlayerInteractEntityEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerInteractEntityEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerInteractEntityEvent(data)
    }
}
