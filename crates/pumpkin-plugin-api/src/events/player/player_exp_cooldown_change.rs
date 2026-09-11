use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerExpCooldownChangeEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player's experience cooldown changes.
pub struct PlayerExpCooldownChangeEvent;
impl FromIntoEvent for PlayerExpCooldownChangeEvent {
    const EVENT_TYPE: EventType = EventType::PlayerExpCooldownChangeEvent;
    type Data = PlayerExpCooldownChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerExpCooldownChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerExpCooldownChangeEvent(data)
    }
}
