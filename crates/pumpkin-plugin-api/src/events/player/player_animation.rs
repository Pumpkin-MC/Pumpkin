use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerAnimationEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player performs an animation.
pub struct PlayerAnimationEvent;
impl FromIntoEvent for PlayerAnimationEvent {
    const EVENT_TYPE: EventType = EventType::PlayerAnimationEvent;
    type Data = PlayerAnimationEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerAnimationEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerAnimationEvent(data)
    }
}
