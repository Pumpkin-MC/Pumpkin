use pumpkin_data::packet::clientbound::config::DISCONNECT;
use pumpkin_macros::java_packet;
use pumpkin_util::text::TextComponent;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;

#[java_packet(DISCONNECT)]
pub struct CConfigDisconnect<'a> {
    pub reason: &'a str,
}

impl<'a> CConfigDisconnect<'a> {
    #[must_use]
    pub const fn new(reason: &'a str) -> Self {
        Self { reason }
    }
}

impl ClientPacket for CConfigDisconnect<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        // Text component, not a plain string.
        write.write_component(&TextComponent::text(self.reason.to_string()))
    }
}
