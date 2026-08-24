use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{
    Event, EventType, PlayerBucketEmptyEventData, PlayerBucketFillEventData,
};

/// Event triggered when a player empties a bucket.
pub struct PlayerBucketEmptyEvent;
impl FromIntoEvent for PlayerBucketEmptyEvent {
    const EVENT_TYPE: EventType = EventType::PlayerBucketEmptyEvent;
    type Data = PlayerBucketEmptyEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerBucketEmptyEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerBucketEmptyEvent(data)
    }
}

/// Event triggered when a player fills a bucket.
pub struct PlayerBucketFillEvent;
impl FromIntoEvent for PlayerBucketFillEvent {
    const EVENT_TYPE: EventType = EventType::PlayerBucketFillEvent;
    type Data = PlayerBucketFillEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerBucketFillEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerBucketFillEvent(data)
    }
}
