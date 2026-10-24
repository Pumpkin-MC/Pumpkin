// Last verified for v2169

use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketRead};

/// Sent by the client to report malformed or unexpected packets received from the server.
#[derive(Debug, PacketRead)]
#[packet(156)]
pub struct SPacketViolationWarning {
    // TODO: enum PacketViolationType
    /// Violation category (e.g. unknown packet or malformed payload).
    pub violation_type: VarInt,
    // TODO: enum PacketViolationSeverity
    /// Severity level of the violation (warning vs fatal).
    pub violation_severity: VarInt,
    /// Packet ID that triggered the violation.
    pub violation_packet_id: VarInt,
    /// Diagnostic description of the violation context.
    pub violation_context: String,
}
