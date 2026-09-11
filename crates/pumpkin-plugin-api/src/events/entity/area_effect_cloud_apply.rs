use crate::wit::pumpkin::plugin::event::{AreaEffectCloudApplyEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when an area effect cloud applies its effect to entities.
pub struct AreaEffectCloudApplyEvent;
impl FromIntoEvent for AreaEffectCloudApplyEvent {
    const EVENT_TYPE: EventType = EventType::AreaEffectCloudApplyEvent;
    type Data = AreaEffectCloudApplyEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::AreaEffectCloudApplyEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::AreaEffectCloudApplyEvent(data)
    }
}
