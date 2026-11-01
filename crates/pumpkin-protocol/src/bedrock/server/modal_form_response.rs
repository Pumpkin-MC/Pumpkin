// Last verified for v2169

use crate::codec::var_uint::VarUInt;
use crate::serial::{PacketRead, PacketReadSlice};
use pumpkin_macros::packet;
use std::borrow::Cow;

/// Sent by the client in response to a server-sent modal form dialog.
#[derive(Debug, PacketRead, PacketReadSlice)]
#[packet(101)]
pub struct SModalFormResponse<'a> {
    /// Form ID corresponding to the request sent by the server.
    pub form_id: VarUInt,
    /// JSON encoded form response data (inputs, selected buttons), or None if cancelled.
    pub json_response: Option<Cow<'a, str>>,

    // TODO: enum ModalFormCancelReason
    /// Optional cancellation reason code if the form was dismissed without submission.
    pub form_cancel_reason: Option<u8>,
}
