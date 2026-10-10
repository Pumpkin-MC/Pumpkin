// Last verified for v2169

use pumpkin_macros::packet;

use crate::{codec::var_ulong::VarULong, serial::PacketWrite};

/// Sent by the server to update an entity's attribute values (e.g. health, absorption, movement speed, hunger).
#[derive(PacketWrite)]
#[packet(29)]
pub struct CUpdateAttributes {
    /// Runtime entity ID of the target actor.
    pub target_runtime_id: VarULong,
    /// List of attributes with their current bounds, values, and modifiers.
    pub attribute_list: Vec<AttributeData>,
    /// Simulation tick number for sync verification.
    pub tick: VarULong,
}

/// Detailed specification of an entity attribute and its active modifiers.
#[derive(PacketWrite)]
pub struct AttributeData {
    /// Minimum value the attribute can reach.
    pub min_value: f32,
    /// Maximum value the attribute can reach.
    pub max_value: f32,
    /// Current calculated attribute value.
    pub current_value: f32,
    /// Base minimum default value.
    pub default_min_value: f32,
    /// Base maximum default value.
    pub default_max_value: f32,
    /// Base default value without modifiers.
    pub default_value: f32,
    /// Identifier string for the attribute (e.g. `minecraft:health`).
    pub name: String,
    /// Active modifiers altering this attribute.
    pub modifiers: Vec<AttributeModifier>,
}

/// An attribute modifier that adjusts an attribute by addition or multiplication.
#[derive(PacketWrite)]
pub struct AttributeModifier {
    /// Unique identifier for the modifier.
    pub id: String,
    /// Human-readable modifier name.
    pub name: String,
    /// Numeric modifier amount.
    pub amount: f32,
    /// Mathematical operation (addition, multiply base, multiply total).
    pub operation: i32,
    /// Modifier operand type index.
    pub operand: i32,
    /// Whether the modifier persists across save/load sessions.
    pub is_serializable: bool,
}
