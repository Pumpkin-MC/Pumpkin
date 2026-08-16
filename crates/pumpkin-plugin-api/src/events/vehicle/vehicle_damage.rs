use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, VehicleDamageEventData};

/// Event triggered when a vehicle takes damage.
pub struct VehicleDamageEvent;
impl FromIntoEvent for VehicleDamageEvent {
    const EVENT_TYPE: EventType = EventType::VehicleDamageEvent;
    type Data = VehicleDamageEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::VehicleDamageEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::VehicleDamageEvent(data)
    }
}
