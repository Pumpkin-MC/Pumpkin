use crate::IdOr;
use crate::java::client::dialog::DialogNBT;
use pumpkin_data::packet::clientbound::PLAY_SHOW_DIALOG;
use pumpkin_macros::java_packet;
use serde::Serialize;

#[derive(Serialize)]
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
