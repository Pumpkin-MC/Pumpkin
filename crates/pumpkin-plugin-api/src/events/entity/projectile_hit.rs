use crate::wit::pumpkin::plugin::event::{Event, EventType, ProjectileHitEventData};

use super::super::FromIntoEvent;

/// Event triggered when a projectile hits an entity or block.
pub struct ProjectileHitEvent;
impl FromIntoEvent for ProjectileHitEvent {
    const EVENT_TYPE: EventType = EventType::ProjectileHitEvent;
    type Data = ProjectileHitEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ProjectileHitEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ProjectileHitEvent(data)
    }
}
