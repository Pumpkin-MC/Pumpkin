use crate::wit::pumpkin::plugin::event::{Event, EventType, VillagerCareerChangeEventData};

use super::super::FromIntoEvent;

/// Event triggered when a villager changes its profession.
pub struct VillagerCareerChangeEvent;
impl FromIntoEvent for VillagerCareerChangeEvent {
    const EVENT_TYPE: EventType = EventType::VillagerCareerChangeEvent;
    type Data = VillagerCareerChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::VillagerCareerChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::VillagerCareerChangeEvent(data)
    }
}
