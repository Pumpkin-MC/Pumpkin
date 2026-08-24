use crate::wit::pumpkin::plugin::event::{Event, EventType, ServerTickEndEventData};

use super::super::FromIntoEvent;

/// An event that fires at the end of every server tick.
///
/// The associated [`ServerTickEndEventData`] carries:
/// - `tick`: the 0-indexed number of the tick that just finished.
/// - `duration_nanos`: how long the tick took, measured from the start of
///   the ticker iteration to just after `Server::tick` returned.
///
/// This event is non-cancellable.
pub struct ServerTickEndEvent;
impl FromIntoEvent for ServerTickEndEvent {
    const EVENT_TYPE: EventType = EventType::ServerTickEndEvent;
    type Data = ServerTickEndEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ServerTickEndEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ServerTickEndEvent(data)
    }
}
