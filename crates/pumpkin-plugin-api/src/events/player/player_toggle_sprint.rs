use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerToggleSprintEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player toggles sprint.
pub struct PlayerToggleSprintEvent;
impl FromIntoEvent for PlayerToggleSprintEvent {
    const EVENT_TYPE: EventType = EventType::PlayerToggleSprintEvent;
    type Data = PlayerToggleSprintEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerToggleSprintEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerToggleSprintEvent(data)
    }
}
