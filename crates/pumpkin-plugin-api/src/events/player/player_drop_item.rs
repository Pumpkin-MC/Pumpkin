use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerDropItemEventData};

/// Event triggered when a player drops an item.
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
