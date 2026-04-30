use pumpkin_plugin_api::packet::java::ServerboundChatMessage;

#[test]
fn chat_message_roundtrip_with_checksum() {
    let message = ServerboundChatMessage {
        message: "hi".into(),
        timestamp: 123,
        salt: 456,
        signature: Some(vec![1, 2, 3, 4]),
        message_count: 7,
        acknowledged: [0xAA, 0xBB, 0xCC],
        checksum: Some(0x5A),
    };

    let payload = message.encode();
    let decoded = ServerboundChatMessage::decode(&payload).expect("decode");
    assert_eq!(decoded, message);
}

#[test]
fn chat_message_roundtrip_without_checksum() {
    let message = ServerboundChatMessage {
        message: "hello".into(),
        timestamp: 111,
        salt: 222,
        signature: None,
        message_count: 3,
        acknowledged: [0x01, 0x02, 0x03],
        checksum: None,
    };

    let payload = message.encode();
    let decoded = ServerboundChatMessage::decode(&payload).expect("decode");
    assert_eq!(decoded, message);
}
