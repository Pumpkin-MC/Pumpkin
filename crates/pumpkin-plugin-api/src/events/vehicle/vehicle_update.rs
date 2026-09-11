use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, VehicleUpdateEventData};

/// Event triggered when a vehicle is updated per tick.
pub struct VehicleUpdateEvent;
impl FromIntoEvent for VehicleUpdateEvent {
    const EVENT_TYPE: EventType = EventType::VehicleUpdateEvent;
    type Data = VehicleUpdateEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::VehicleUpdateEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::VehicleUpdateEvent(data)
    }
}
