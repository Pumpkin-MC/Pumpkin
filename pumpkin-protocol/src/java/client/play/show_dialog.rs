use crate::java::client::dialog::DialogNBT;
use crate::{ClientPacket, IdOr, ser::WritingError};
use pumpkin_data::packet::clientbound::PLAY_SHOW_DIALOG;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_SHOW_DIALOG)]
pub struct CPlayShowDialog<'a> {
    pub dialog: IdOr<DialogNBT<'a>>,
}

impl<'a> CPlayShowDialog<'a> {
    #[must_use]
    pub const fn new(dialog: IdOr<DialogNBT<'a>>) -> Self {
        Self { dialog }
    }
}

impl ClientPacket for CPlayShowDialog<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        self.dialog
            .write(&mut write, |w, value| value.write_packet_data(w, version))
    }
}
