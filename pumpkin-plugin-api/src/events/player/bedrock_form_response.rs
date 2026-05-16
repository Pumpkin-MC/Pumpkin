use crate::wit::pumpkin::plugin::event::{BedrockFormResponseEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a player responds to a Bedrock custom form.
pub struct BedrockFormResponseEvent;
impl FromIntoEvent for BedrockFormResponseEvent {
    const EVENT_TYPE: EventType = EventType::BedrockFormResponseEvent;
    type Data = BedrockFormResponseEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BedrockFormResponseEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BedrockFormResponseEvent(data)
    }
}
