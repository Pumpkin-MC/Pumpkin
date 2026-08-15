use crate::wit::pumpkin::plugin::event::{EntityTargetLivingEntityEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an entity targets a living entity.
pub struct EntityTargetLivingEntityEvent;
impl FromIntoEvent for EntityTargetLivingEntityEvent {
    const EVENT_TYPE: EventType = EventType::EntityTargetLivingEntityEvent;
    type Data = EntityTargetLivingEntityEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityTargetLivingEntityEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityTargetLivingEntityEvent(data)
    }
}
