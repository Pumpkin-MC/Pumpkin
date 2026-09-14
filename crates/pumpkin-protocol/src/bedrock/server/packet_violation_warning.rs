// Last verified for v2169

use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketRead};

#[derive(Debug, PacketRead)]
#[packet(156)]
pub struct SPacketViolationWarning {
    // TODO: enum PacketViolationType
    pub violation_type: VarInt,
    // TODO: enum PacketViolationSeverity
    pub violation_severity: VarInt,
    pub violation_packet_id: VarInt,
    pub violation_context: String,
}
