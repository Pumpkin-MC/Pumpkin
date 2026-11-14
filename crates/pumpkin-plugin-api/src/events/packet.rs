use super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{
    BedrockConnectionPacketReceivedEventData, BedrockConnectionPacketSentEventData, Event,
    EventType, JavaConnectionPacketReceivedEventData, JavaConnectionPacketSentEventData,
    PacketReceivedEventData, PacketSentEventData,
};
pub use crate::wit::pumpkin::plugin::event::{JavaConnectionState, RawPacket};

/// An event fired when a packet is received from a client
pub struct PacketReceivedEvent;

impl FromIntoEvent for PacketReceivedEvent {
    const EVENT_TYPE: EventType = EventType::PacketReceivedEvent;
    type Data = PacketReceivedEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PacketReceivedEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PacketReceivedEvent(data)
    }
}

/// An event fired when a packet is sent to a client
pub struct PacketSentEvent;

impl FromIntoEvent for PacketSentEvent {
    const EVENT_TYPE: EventType = EventType::PacketSentEvent;
    type Data = PacketSentEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PacketSentEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PacketSentEvent(data)
    }
}

/// An event fired when a packet is received from a Java connection prior to play.
pub struct JavaConnectionPacketReceivedEvent;

impl FromIntoEvent for JavaConnectionPacketReceivedEvent {
    const EVENT_TYPE: EventType = EventType::JavaConnectionPacketReceivedEvent;
    type Data = JavaConnectionPacketReceivedEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::JavaConnectionPacketReceivedEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::JavaConnectionPacketReceivedEvent(data)
    }
}

/// An event fired when a packet is sent to a Java connection prior to play.
pub struct JavaConnectionPacketSentEvent;

impl FromIntoEvent for JavaConnectionPacketSentEvent {
    const EVENT_TYPE: EventType = EventType::JavaConnectionPacketSentEvent;
    type Data = JavaConnectionPacketSentEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::JavaConnectionPacketSentEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::JavaConnectionPacketSentEvent(data)
    }
}

/// An event fired when a packet is received from a Bedrock connection prior to play.
pub struct BedrockConnectionPacketReceivedEvent;

impl FromIntoEvent for BedrockConnectionPacketReceivedEvent {
    const EVENT_TYPE: EventType = EventType::BedrockConnectionPacketReceivedEvent;
    type Data = BedrockConnectionPacketReceivedEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BedrockConnectionPacketReceivedEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BedrockConnectionPacketReceivedEvent(data)
    }
}

/// An event fired when a packet is sent to a Bedrock connection prior to play.
pub struct BedrockConnectionPacketSentEvent;

impl FromIntoEvent for BedrockConnectionPacketSentEvent {
    const EVENT_TYPE: EventType = EventType::BedrockConnectionPacketSentEvent;
    type Data = BedrockConnectionPacketSentEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BedrockConnectionPacketSentEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BedrockConnectionPacketSentEvent(data)
    }
}
