use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerDropItemEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player drops an item.
///
/// The associated [`PlayerDropItemEventData`] contains the player, the item stack being dropped,
/// whether the action drops the full held stack, and the cancellation state.
pub struct PlayerDropItemEvent;
impl FromIntoEvent for PlayerDropItemEvent {
    const EVENT_TYPE: EventType = EventType::PlayerDropItemEvent;
    type Data = PlayerDropItemEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerDropItemEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerDropItemEvent(data)
    }
}
