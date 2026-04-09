use pumpkin_plugin_api::packet::codec::{PacketReader, PacketWriter};

#[test]
fn varint_roundtrip() {
    let values = [0, 1, 2, 127, 128, 255, 1024, 65535, 2_097_151];
    for value in values {
        let mut writer = PacketWriter::new();
        writer.write_varint(value);
        let bytes = writer.into_inner();

        let mut reader = PacketReader::new(&bytes);
        let decoded = reader.read_varint().expect("read varint");
        assert_eq!(decoded, value);
        assert_eq!(reader.remaining(), 0);
    }
}

#[test]
fn string_roundtrip() {
    let message = "hello pumpkin";
    let mut writer = PacketWriter::new();
    writer.write_string(message);
    let bytes = writer.into_inner();

    let mut reader = PacketReader::new(&bytes);
    let decoded = reader.read_string(256).expect("read string");
    assert_eq!(decoded, message);
    assert_eq!(reader.remaining(), 0);
}
