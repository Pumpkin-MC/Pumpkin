use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{
    Event, EventType, PlayerBedEnterEventData, PlayerBedLeaveEventData,
};

/// Event triggered when a player enters a bed.
pub struct PlayerBedEnterEvent;
impl FromIntoEvent for PlayerBedEnterEvent {
    const EVENT_TYPE: EventType = EventType::PlayerBedEnterEvent;
    type Data = PlayerBedEnterEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerBedEnterEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerBedEnterEvent(data)
    }
}

/// Event triggered when a player leaves a bed.
pub struct PlayerBedLeaveEvent;
impl FromIntoEvent for PlayerBedLeaveEvent {
    const EVENT_TYPE: EventType = EventType::PlayerBedLeaveEvent;
    type Data = PlayerBedLeaveEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerBedLeaveEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerBedLeaveEvent(data)
    }
}
