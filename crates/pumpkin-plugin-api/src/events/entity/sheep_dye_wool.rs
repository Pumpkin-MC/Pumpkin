use crate::wit::pumpkin::plugin::event::{Event, EventType, SheepDyeWoolEventData};

use super::super::FromIntoEvent;

/// Event triggered when a sheep has its wool dyed.
pub struct SheepDyeWoolEvent;
impl FromIntoEvent for SheepDyeWoolEvent {
    const EVENT_TYPE: EventType = EventType::SheepDyeWoolEvent;
    type Data = SheepDyeWoolEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::SheepDyeWoolEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::SheepDyeWoolEvent(data)
    }
}
