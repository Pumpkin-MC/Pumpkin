// Last verified for v2169

use crate::{codec::var_uint::VarUInt, serial::PacketWrite};
use pumpkin_macros::packet;

/// Requests the client to open a server-defined UI form (modal dialog, list menu, or custom form).
#[packet(100)]
#[derive(PacketWrite)]
pub struct CModalFormRequest {
    /// Unique identifier assigned by the server to distinguish responses when the form is submitted.
    pub form_id: VarUInt,
    /// JSON-encoded definition describing the layout, controls, and contents of the form window.
    pub form_ui_json: String,
}
