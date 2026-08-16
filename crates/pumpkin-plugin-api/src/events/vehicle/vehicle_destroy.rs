use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, VehicleDestroyEventData};

/// Event triggered when a vehicle is destroyed.
pub struct VehicleDestroyEvent;
impl FromIntoEvent for VehicleDestroyEvent {
    const EVENT_TYPE: EventType = EventType::VehicleDestroyEvent;
    type Data = VehicleDestroyEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::VehicleDestroyEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::VehicleDestroyEvent(data)
    }
}
