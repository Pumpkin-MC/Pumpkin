use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerBucketEntityEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player captures an entity with a bucket.
pub struct PlayerBucketEntityEvent;
impl FromIntoEvent for PlayerBucketEntityEvent {
    const EVENT_TYPE: EventType = EventType::PlayerBucketEntityEvent;
    type Data = PlayerBucketEntityEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerBucketEntityEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerBucketEntityEvent(data)
    }
}
