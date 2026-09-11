use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerUnregisterChannelEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a plugin channel is unregistered for a player.
pub struct PlayerUnregisterChannelEvent;
impl FromIntoEvent for PlayerUnregisterChannelEvent {
    const EVENT_TYPE: EventType = EventType::PlayerUnregisterChannelEvent;
    type Data = PlayerUnregisterChannelEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerUnregisterChannelEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerUnregisterChannelEvent(data)
    }
}
