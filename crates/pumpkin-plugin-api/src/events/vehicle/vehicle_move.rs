use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, VehicleMoveEventData};

/// Event triggered when a vehicle moves.
pub struct VehicleMoveEvent;
impl FromIntoEvent for VehicleMoveEvent {
    const EVENT_TYPE: EventType = EventType::VehicleMoveEvent;
    type Data = VehicleMoveEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::VehicleMoveEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::VehicleMoveEvent(data)
    }
}
