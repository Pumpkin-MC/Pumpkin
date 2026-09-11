use crate::wit::pumpkin::plugin::event::{Event, EventType, ProjectileLaunchEventData};

use super::super::FromIntoEvent;

/// Event triggered when a projectile is launched.
pub struct ProjectileLaunchEvent;
impl FromIntoEvent for ProjectileLaunchEvent {
    const EVENT_TYPE: EventType = EventType::ProjectileLaunchEvent;
    type Data = ProjectileLaunchEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ProjectileLaunchEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ProjectileLaunchEvent(data)
    }
}
