use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, VehicleCreateEventData};

/// Event triggered when a vehicle is created.
pub struct VehicleCreateEvent;
impl FromIntoEvent for VehicleCreateEvent {
    const EVENT_TYPE: EventType = EventType::VehicleCreateEvent;
    type Data = VehicleCreateEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::VehicleCreateEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::VehicleCreateEvent(data)
    }
}
