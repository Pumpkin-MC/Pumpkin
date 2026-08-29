use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, VehicleEnterEventData};

/// Event triggered when an entity enters a vehicle.
pub struct VehicleEnterEvent;
impl FromIntoEvent for VehicleEnterEvent {
    const EVENT_TYPE: EventType = EventType::VehicleEnterEvent;
    type Data = VehicleEnterEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::VehicleEnterEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::VehicleEnterEvent(data)
    }
}
