#![no_main]
use libfuzzer_sys::fuzz_target;
use pumpkin_protocol::query::{RawQueryPacket, SHandshake, SStatusRequest};
use tokio::runtime::Runtime;

fuzz_target!(|data: &[u8]| {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        if let Ok(mut raw) = RawQueryPacket::decode(data.to_vec()).await {
            match raw.packet_type {
                pumpkin_protocol::query::PacketType::Handshake => {
                    let _ = SHandshake::decode(&mut raw).await;
                }
                pumpkin_protocol::query::PacketType::Status => {
                    let _ = SStatusRequest::decode(&mut raw).await;
                }
            }
        }
    });
});
