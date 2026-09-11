use crate::wit::pumpkin::plugin::event::{Event, EventType, HorseJumpEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a horse jumps.
pub struct HorseJumpEvent;
impl FromIntoEvent for HorseJumpEvent {
    const EVENT_TYPE: EventType = EventType::HorseJumpEvent;
    type Data = HorseJumpEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::HorseJumpEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::HorseJumpEvent(data)
    }
}
