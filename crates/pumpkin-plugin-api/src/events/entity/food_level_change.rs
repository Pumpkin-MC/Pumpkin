use crate::wit::pumpkin::plugin::event::{Event, EventType, FoodLevelChangeEventData};

use super::super::FromIntoEvent;

/// Event triggered when an entity's food level changes.
pub struct FoodLevelChangeEvent;
impl FromIntoEvent for FoodLevelChangeEvent {
    const EVENT_TYPE: EventType = EventType::FoodLevelChangeEvent;
    type Data = FoodLevelChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::FoodLevelChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::FoodLevelChangeEvent(data)
    }
}
