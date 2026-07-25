use crate::IdOr;
use crate::java::client::dialog::DialogNBT;
use pumpkin_data::packet::clientbound::CONFIG_SHOW_DIALOG;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONFIG_SHOW_DIALOG)]
pub struct CConfigShowDialog<'a> {
    pub dialog: IdOr<DialogNBT<'a>>,
}

impl<'a> CConfigShowDialog<'a> {
    #[must_use]
    pub const fn new(dialog: IdOr<DialogNBT<'a>>) -> Self {
        Self { dialog }
    }
}

impl ClientPacket for CConfigShowDialog<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        self.dialog
            .write(&mut write, |w, dialog| dialog.write_packet_data(w, version))?;
        Ok(())
    }
}
