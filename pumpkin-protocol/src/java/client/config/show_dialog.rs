use crate::IdOr;
use crate::java::client::dialog::DialogNBT;
use pumpkin_data::packet::clientbound::CONFIG_SHOW_DIALOG;
use pumpkin_macros::java_packet;
use serde::Serialize;

#[derive(Serialize)]
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
