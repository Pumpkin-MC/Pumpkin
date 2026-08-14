use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, VehicleExitEventData};

/// Event triggered when an entity exits a vehicle.
pub struct VehicleExitEvent;
impl FromIntoEvent for VehicleExitEvent {
    const EVENT_TYPE: EventType = EventType::VehicleExitEvent;
    type Data = VehicleExitEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::VehicleExitEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::VehicleExitEvent(data)
    }
}
