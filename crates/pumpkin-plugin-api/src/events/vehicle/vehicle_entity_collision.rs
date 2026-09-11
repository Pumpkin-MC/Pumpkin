use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, VehicleEntityCollisionEventData};

/// Event triggered when a vehicle collides with another entity.
pub struct VehicleEntityCollisionEvent;
impl FromIntoEvent for VehicleEntityCollisionEvent {
    const EVENT_TYPE: EventType = EventType::VehicleEntityCollisionEvent;
    type Data = VehicleEntityCollisionEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::VehicleEntityCollisionEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::VehicleEntityCollisionEvent(data)
    }
}
