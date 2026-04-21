#![no_main]
use libfuzzer_sys::fuzz_target;
use pumpkin_protocol::bedrock::packet_encoder::UDPNetworkEncoder;
use pumpkin_protocol::bedrock::SubClient;

fuzz_target!(|data: &[u8]| {
    if data.len() < 10 {
        return;
    }

    let compression_threshold = data[0] as usize;
    let compression_level = (data[1] % 10) as u32;
    let packet_id = u16::from_be_bytes([data[2], data[3]]);
    let sender = match data[4] % 4 {
        0 => SubClient::Main,
        1 => SubClient::SubClient0,
        2 => SubClient::SubClient1,
        _ => SubClient::SubClietn2,
    };
    let target = match data[5] % 4 {
        0 => SubClient::Main,
        1 => SubClient::SubClient0,
        2 => SubClient::SubClient1,
        _ => SubClient::SubClietn2,
    };
    let use_compression = data[6] % 2 == 0;
    let packet_payload = &data[7..];

    let mut encoder = UDPNetworkEncoder::new();

    if use_compression {
        encoder.set_compression((compression_threshold, compression_level));
    }

    let mut out = Vec::new();
    let _ = encoder.write_game_packet(
        packet_id,
        sender,
        target,
        packet_payload,
        &mut out
    );
});
