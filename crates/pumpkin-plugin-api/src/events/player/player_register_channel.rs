use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerRegisterChannelEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a plugin channel is registered for a player.
pub struct PlayerRegisterChannelEvent;
impl FromIntoEvent for PlayerRegisterChannelEvent {
    const EVENT_TYPE: EventType = EventType::PlayerRegisterChannelEvent;
    type Data = PlayerRegisterChannelEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerRegisterChannelEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerRegisterChannelEvent(data)
    }
}
