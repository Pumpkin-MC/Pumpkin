#![no_main]
use libfuzzer_sys::fuzz_target;
use pumpkin::net::rcon::packet::Packet;

fuzz_target!(|data: &[u8]| {
    let mut incoming = data.to_vec();
    let _ = Packet::deserialize(&mut incoming);
});
