use crate::wit::pumpkin::plugin::event::{Event, EventType, VillagerReputationChangeEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a villager's reputation changes.
pub struct VillagerReputationChangeEvent;
impl FromIntoEvent for VillagerReputationChangeEvent {
    const EVENT_TYPE: EventType = EventType::VillagerReputationChangeEvent;
    type Data = VillagerReputationChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::VillagerReputationChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::VillagerReputationChangeEvent(data)
    }
}
