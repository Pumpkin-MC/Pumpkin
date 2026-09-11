use crate::wit::pumpkin::plugin::event::{Event, EventType, SheepRegrowWoolEventData};

use super::super::FromIntoEvent;

/// Event triggered when a sheep regrows its wool.
pub struct SheepRegrowWoolEvent;
impl FromIntoEvent for SheepRegrowWoolEvent {
    const EVENT_TYPE: EventType = EventType::SheepRegrowWoolEvent;
    type Data = SheepRegrowWoolEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::SheepRegrowWoolEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::SheepRegrowWoolEvent(data)
    }
}
