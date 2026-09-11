use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, VehicleCollisionEventData};

/// Base event triggered when a vehicle collides.
pub struct VehicleCollisionEvent;
impl FromIntoEvent for VehicleCollisionEvent {
    const EVENT_TYPE: EventType = EventType::VehicleCollisionEvent;
    type Data = VehicleCollisionEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::VehicleCollisionEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::VehicleCollisionEvent(data)
    }
}
