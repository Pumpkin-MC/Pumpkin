// Last verified for v2169

use pumpkin_macros::packet;

use crate::{
    codec::var_ulong::VarULong,
    serial::{PacketRead, PacketWrite},
};

/// Sent by the server to update an entity's attribute values (e.g. health, absorption, movement speed, hunger).
#[derive(PacketRead, PacketWrite)]
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
#[derive(PacketRead, PacketWrite)]
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
#[derive(PacketRead, PacketWrite)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attributes_round_trip_with_modifier_and_tick() -> Result<(), Box<dyn std::error::Error>> {
        let packet = CUpdateAttributes {
            target_runtime_id: VarULong(150),
            attribute_list: vec![AttributeData {
                min_value: 1.0,
                max_value: 16.0,
                current_value: 3.0,
                default_min_value: 2.0,
                default_max_value: 32.0,
                default_value: 4.0,
                name: "minecraft:health".into(),
                modifiers: vec![AttributeModifier {
                    id: "example:test".into(),
                    name: "Test".into(),
                    amount: 0.25,
                    operation: 1,
                    operand: 2,
                    is_serializable: true,
                }],
            }],
            tick: VarULong(300),
        };
        let mut bytes = Vec::new();
        packet.write(&mut bytes)?;
        let mut data = bytes.as_slice();
        let decoded = CUpdateAttributes::read(&mut data)?;
        assert!(data.is_empty());
        assert_eq!(decoded.target_runtime_id, packet.target_runtime_id);
        assert_eq!(decoded.tick, packet.tick);
        let [attribute] = decoded.attribute_list.as_slice() else {
            return Err("expected one attribute".into());
        };
        assert_eq!(attribute.name, "minecraft:health");
        assert_eq!(
            [
                attribute.min_value.to_bits(),
                attribute.max_value.to_bits(),
                attribute.current_value.to_bits(),
                attribute.default_min_value.to_bits(),
                attribute.default_max_value.to_bits(),
                attribute.default_value.to_bits(),
            ],
            [1.0f32, 16.0, 3.0, 2.0, 32.0, 4.0].map(f32::to_bits),
        );
        let [modifier] = attribute.modifiers.as_slice() else {
            return Err("expected one modifier".into());
        };
        assert_eq!(modifier.id, "example:test");
        assert_eq!(modifier.name, "Test");
        assert_eq!(modifier.amount.to_bits(), 0.25f32.to_bits());
        assert_eq!(modifier.operation, 1);
        assert_eq!(modifier.operand, 2);
        assert!(modifier.is_serializable);
        Ok(())
    }
}
