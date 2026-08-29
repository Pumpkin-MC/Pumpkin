use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, VehicleBlockCollisionEventData};

/// Event triggered when a vehicle collides with a block.
pub struct VehicleBlockCollisionEvent;
impl FromIntoEvent for VehicleBlockCollisionEvent {
    const EVENT_TYPE: EventType = EventType::VehicleBlockCollisionEvent;
    type Data = VehicleBlockCollisionEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::VehicleBlockCollisionEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::VehicleBlockCollisionEvent(data)
    }
}
