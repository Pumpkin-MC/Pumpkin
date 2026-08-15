use crate::wit::pumpkin::plugin::event::{EnderDragonChangePhaseEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when an Ender Dragon changes its phase.
pub struct EnderDragonChangePhaseEvent;
impl FromIntoEvent for EnderDragonChangePhaseEvent {
    const EVENT_TYPE: EventType = EventType::EnderDragonChangePhaseEvent;
    type Data = EnderDragonChangePhaseEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EnderDragonChangePhaseEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EnderDragonChangePhaseEvent(data)
    }
}
