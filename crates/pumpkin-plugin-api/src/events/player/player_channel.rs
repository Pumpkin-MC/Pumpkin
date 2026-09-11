use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerChannelEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player changes a plugin messaging channel.
pub struct PlayerChannelEvent;
impl FromIntoEvent for PlayerChannelEvent {
    const EVENT_TYPE: EventType = EventType::PlayerChannelEvent;
    type Data = PlayerChannelEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerChannelEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerChannelEvent(data)
    }
}
