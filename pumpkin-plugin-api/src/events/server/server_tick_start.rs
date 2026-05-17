use crate::wit::pumpkin::plugin::event::{Event, EventType, ServerTickStartEventData};

use super::super::FromIntoEvent;

/// An event that fires at the start of every server tick (~20 Hz under the
/// default tick rate).
///
/// The associated [`ServerTickStartEventData`] carries the 0-indexed `tick`
/// number of the tick about to run. This event is non-cancellable.
pub struct ServerTickStartEvent;
impl FromIntoEvent for ServerTickStartEvent {
    const EVENT_TYPE: EventType = EventType::ServerTickStartEvent;
    type Data = ServerTickStartEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ServerTickStartEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ServerTickStartEvent(data)
    }
}
