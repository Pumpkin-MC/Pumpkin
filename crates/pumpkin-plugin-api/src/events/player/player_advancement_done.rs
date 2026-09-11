use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerAdvancementDoneEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player completes an advancement.
pub struct PlayerAdvancementDoneEvent;
impl FromIntoEvent for PlayerAdvancementDoneEvent {
    const EVENT_TYPE: EventType = EventType::PlayerAdvancementDoneEvent;
    type Data = PlayerAdvancementDoneEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerAdvancementDoneEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerAdvancementDoneEvent(data)
    }
}
