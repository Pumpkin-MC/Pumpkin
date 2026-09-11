use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, SpongeAbsorbEventData};

/// An event that occurs when a sponge absorbs water.
pub struct SpongeAbsorbEvent;
impl FromIntoEvent for SpongeAbsorbEvent {
    const EVENT_TYPE: EventType = EventType::SpongeAbsorbEvent;
    type Data = SpongeAbsorbEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::SpongeAbsorbEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::SpongeAbsorbEvent(data)
    }
}
