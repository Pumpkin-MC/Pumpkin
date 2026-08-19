use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerItemConsumeEventData};

/// Event triggered when a player consumes an item.
pub struct PlayerItemConsumeEvent;
impl FromIntoEvent for PlayerItemConsumeEvent {
    const EVENT_TYPE: EventType = EventType::PlayerItemConsumeEvent;
    type Data = PlayerItemConsumeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerItemConsumeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerItemConsumeEvent(data)
    }
}
