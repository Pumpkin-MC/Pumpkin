use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerLinksSendEventData};

use super::super::FromIntoEvent;

/// An event that occurs when server links are sent to a player.
pub struct PlayerLinksSendEvent;
impl FromIntoEvent for PlayerLinksSendEvent {
    const EVENT_TYPE: EventType = EventType::PlayerLinksSendEvent;
    type Data = PlayerLinksSendEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerLinksSendEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerLinksSendEvent(data)
    }
}
