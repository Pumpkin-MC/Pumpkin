use crate::entity::living::{LivingEntity, attributes_by_id};
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use rustc_hash::FxHashMap;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum ModifierOperation {
    Add = 0,           // add value
    MultiplyBase = 1,  // multiply base (base * (1 + x))
    MultiplyTotal = 2, // multiply total (applied last)
}

impl ModifierOperation {
    const fn name(self) -> &'static str {
        match self {
            Self::Add => "add_value",
            Self::MultiplyBase => "add_multiplied_base",
            Self::MultiplyTotal => "add_multiplied_total",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        match name {
            "add_value" => Some(Self::Add),
            "add_multiplied_base" => Some(Self::MultiplyBase),
            "add_multiplied_total" => Some(Self::MultiplyTotal),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Modifier {
    pub id: String,
    pub amount: f64,
    pub operation: ModifierOperation,
}

/// Per-entity attribute instance used at runtime.
#[derive(Debug)]
pub struct AttributeInstance {
    pub base_value: f64,
    pub modifiers: Vec<Modifier>,
    pub permanent_modifiers: Vec<String>,
    pub cached_value: AtomicU64,
    pub dirty: AtomicBool,
    /// Vanilla `AttributeMap.attributesToUpdate` membership, drained by the owner's tick.
    pub updated: AtomicBool,
}

impl AttributeInstance {
    #[must_use]
    pub const fn new(base_value: f64) -> Self {
        Self {
            base_value,
            modifiers: Vec::new(),
            permanent_modifiers: Vec::new(),
            cached_value: AtomicU64::new(base_value.to_bits()),
            dirty: AtomicBool::new(true),
            updated: AtomicBool::new(false),
        }
    }

    /// Invalidates the cached value and queues an attribute update for the owner.
    pub fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
        self.updated.store(true, Ordering::Relaxed);
    }

    fn calculate_raw_value(&self) -> f64 {
        let mut value = self.base_value;

        let mut add_sum = 0.0;
        let mut mul_base = 0.0;
        let mut mul_total = 1.0;
        for m in &self.modifiers {
            match m.operation {
                ModifierOperation::Add => add_sum += m.amount,
                ModifierOperation::MultiplyBase => mul_base += m.amount,
                ModifierOperation::MultiplyTotal => mul_total *= 1.0 + m.amount,
            }
        }

        value += add_sum;
        value *= 1.0 + mul_base;
        value *= mul_total;
        value
    }

    /// Computes and caches the attribute value clamped to `attribute.min_value..=attribute.max_value`
    /// (`AttributeInstance::calculateValue` -> `Attribute::sanitizeValue`).
    pub fn sanitized_value(&self, attribute: &Attributes) -> f64 {
        if !self.dirty.load(Ordering::Relaxed) {
            return f64::from_bits(self.cached_value.load(Ordering::Relaxed));
        }

        let value = attribute.sanitize_value(self.calculate_raw_value());
        self.cached_value.store(value.to_bits(), Ordering::Relaxed);
        self.dirty.store(false, Ordering::Relaxed);

        value
    }

    pub fn value(&self) -> f64 {
        if !self.dirty.load(Ordering::Relaxed) {
            return f64::from_bits(self.cached_value.load(Ordering::Relaxed));
        }

        let mut value = self.calculate_raw_value();
        if value.is_nan() || value.is_infinite() {
            value = self.base_value;
        }

        self.cached_value.store(value.to_bits(), Ordering::Relaxed);
        self.dirty.store(false, Ordering::Relaxed);

        value
    }

    pub fn add_or_replace_modifier(&mut self, modifier: Modifier) {
        if let Some(pos) = self.modifiers.iter().position(|m| m.id == modifier.id) {
            self.modifiers.remove(pos);
        }
        self.modifiers.push(modifier);
        self.mark_dirty();
    }

    pub fn add_or_replace_permanent_modifier(&mut self, modifier: Modifier) {
        if !self.permanent_modifiers.contains(&modifier.id) {
            self.permanent_modifiers.push(modifier.id.clone());
        }
        self.add_or_replace_modifier(modifier);
    }

    pub fn remove_modifier(&mut self, id: &str) {
        if let Some(pos) = self.modifiers.iter().position(|m| m.id == id) {
            self.modifiers.swap_remove(pos);
        }
        self.permanent_modifiers.retain(|permanent| permanent != id);
        self.mark_dirty();
    }

    fn pack(&self, attribute: &Attributes) -> NbtCompound {
        let mut packed = NbtCompound::new();
        packed.put_string("id", attribute.name.to_string());
        packed.put_double("base", self.base_value);
        let modifiers: Vec<NbtTag> = self
            .modifiers
            .iter()
            .filter(|modifier| self.permanent_modifiers.contains(&modifier.id))
            .map(|modifier| {
                let mut tag = NbtCompound::new();
                tag.put_string("id", modifier.id.clone());
                tag.put_double("amount", modifier.amount);
                tag.put_string("operation", modifier.operation.name().to_string());
                NbtTag::Compound(tag)
            })
            .collect();
        if !modifiers.is_empty() {
            packed.put_list("modifiers", modifiers);
        }
        packed
    }

    fn apply(&mut self, packed: &NbtCompound) {
        self.base_value = packed.get_double("base").unwrap_or(0.0);
        for modifier in packed.get_list("modifiers").unwrap_or_default() {
            let Some(modifier) = modifier.extract_compound() else {
                continue;
            };
            let (Some(id), Some(amount), Some(operation)) = (
                modifier.get_string("id"),
                modifier.get_double("amount"),
                modifier
                    .get_string("operation")
                    .and_then(ModifierOperation::from_name),
            ) else {
                continue;
            };
            self.add_or_replace_permanent_modifier(Modifier {
                id: id.to_string(),
                amount,
                operation,
            });
        }
        self.mark_dirty();
    }
}

#[must_use]
pub fn pack_attributes(living: &LivingEntity) -> Vec<NbtTag> {
    let attributes = living
        .attributes
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut packed: Vec<(u8, NbtCompound)> = attributes
        .iter()
        .filter_map(|(id, instance)| {
            attributes_by_id(*id).map(|attribute| (*id, instance.pack(attribute)))
        })
        .collect();
    packed.sort_unstable_by_key(|(id, _)| *id);
    packed
        .into_iter()
        .map(|(_, tag)| NbtTag::Compound(tag))
        .collect()
}

pub fn apply_attributes(living: &LivingEntity, packed: &[NbtTag]) {
    let mut attributes = living
        .attributes
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for packed in packed {
        let Some(packed) = packed.extract_compound() else {
            continue;
        };
        let Some(id) = packed.get_string("id") else {
            continue;
        };
        let Some(attribute) = Attributes::ALL.iter().find(|attribute| {
            attribute.name == id || attribute.name.strip_prefix("minecraft:") == Some(id)
        }) else {
            continue;
        };
        if let Some(instance) = attributes.get_mut(&attribute.id) {
            instance.apply(packed);
        }
    }
}

/// Send updates for multiple attributes in a single packet for the given living entity.
pub fn send_attribute_updates_for_living(
    living: &crate::entity::living::LivingEntity,
    attributes: Vec<Attributes>,
) {
    send_attribute_updates(living, attributes, false);
}

pub(crate) fn send_guarded_attribute_updates(living: &LivingEntity, attributes: &[Attributes]) {
    for attribute in attributes {
        send_attribute_updates(living, vec![attribute.clone()], true);
    }
}

fn send_attribute_updates(living: &LivingEntity, attributes: Vec<Attributes>, guard_current: bool) {
    use pumpkin_protocol::bedrock::client::update_attributes::{
        AttributeData as BeAttribute, CUpdateAttributes as BePacket,
    };
    use pumpkin_protocol::codec::var_int::VarInt;
    use pumpkin_protocol::codec::var_ulong::VarULong;
    use pumpkin_protocol::java::client::play::AttributeModifier as JeAttrMod;
    use pumpkin_protocol::java::client::play::CUpdateAttributes as JePacket;
    use pumpkin_protocol::java::client::play::Property as JeProperty;

    let mut je_properties: Vec<JeProperty> = Vec::with_capacity(attributes.len());
    let mut be_attributes: Vec<BeAttribute> = Vec::with_capacity(attributes.len());

    let guarded_attribute = if guard_current {
        attributes.first().cloned()
    } else {
        None
    };
    for attribute in attributes {
        let base_value = living.get_attribute_base(&attribute);
        let effective_value = living.get_attribute_value(&attribute);

        if attribute.client_syncable {
            // Pull modifiers for this attribute
            let mut modifiers = Vec::new();
            if let Some(inst) = living
                .attributes
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(&attribute.id)
            {
                for mod_inst in &inst.modifiers {
                    modifiers.push(JeAttrMod::new(
                        mod_inst.id.clone(),
                        mod_inst.amount,
                        mod_inst.operation as i8,
                    ));
                }
            }

            // Move modifiers into the property
            je_properties.push(JeProperty::new(
                VarInt(i32::from(attribute.id)),
                base_value,
                modifiers,
            ));
        }

        let name = match attribute.id {
            id if id == Attributes::MOVEMENT_SPEED.id => "minecraft:movement".to_string(),
            id if id == Attributes::MAX_HEALTH.id => "minecraft:health".to_string(),
            id if id == Attributes::MAX_ABSORPTION.id => "minecraft:absorption".to_string(),
            id if id == Attributes::ATTACK_DAMAGE.id => "minecraft:attack_damage".to_string(),
            id if id == Attributes::KNOCKBACK_RESISTANCE.id => {
                "minecraft:knockback_resistance".to_string()
            }
            id if id == Attributes::LUCK.id => "minecraft:luck".to_string(),
            id if id == Attributes::FOLLOW_RANGE.id => "minecraft:follow_range".to_string(),
            id if id == Attributes::JUMP_STRENGTH.id => "minecraft:horse.jump_strength".to_string(),
            // Java-only attributes must not be sent under unsupported Bedrock names.
            _ => continue,
        };

        let be_attribute = BeAttribute {
            min_value: attribute.min_value as f32,
            max_value: attribute.max_value as f32,
            current_value: effective_value as f32,
            default_min_value: attribute.min_value as f32,
            default_max_value: attribute.max_value as f32,
            default_value: base_value as f32,
            name,
            // Bedrock receives the already-computed effective value above. Do not advertise
            // modifier entries until their payload is encoded as well.
            modifiers: Vec::new(),
        };

        be_attributes.push(be_attribute);
    }

    if let Some(attribute) = guarded_attribute {
        send_current_shield_attribute(living, &attribute, je_properties, be_attributes);
        return;
    }
    let world = living.entity.world.load();
    match (!je_properties.is_empty(), !be_attributes.is_empty()) {
        (false, false) => {}
        (true, false) => {
            let je_packet = JePacket::new(living.entity.entity_id.into(), je_properties);
            world.broadcast_packet_all(&je_packet);
        }
        (false, true) => {
            let runtime_id = living.entity.entity_id as u64;
            let be_packet = BePacket {
                target_runtime_id: VarULong(runtime_id),
                attribute_list: be_attributes,
                tick: VarULong(0),
            };
            world.broadcast_bedrock_all(&be_packet);
        }
        (true, true) => {
            let je_packet = JePacket::new(living.entity.entity_id.into(), je_properties);
            let runtime_id = living.entity.entity_id as u64;
            let be_packet = BePacket {
                target_runtime_id: VarULong(runtime_id),
                attribute_list: be_attributes,
                tick: VarULong(0),
            };
            world.broadcast_editioned(&je_packet, &be_packet);
        }
    }
}

fn send_current_shield_attribute(
    living: &LivingEntity,
    attribute: &Attributes,
    java: Vec<pumpkin_protocol::java::client::play::Property>,
    bedrock: Vec<pumpkin_protocol::bedrock::client::update_attributes::AttributeData>,
) {
    use pumpkin_protocol::{
        bedrock::client::update_attributes::CUpdateAttributes as BedrockAttributes,
        java::client::play::CUpdateAttributes as JavaAttributes,
    };
    let entity = &living.entity;
    let world = entity.world.load_full();
    let java = JavaAttributes::new(entity.entity_id.into(), java);
    let bedrock = BedrockAttributes {
        target_runtime_id: (entity.entity_id as u64).into(),
        attribute_list: bedrock,
        tick: 0u64.into(),
    };
    for player in world.players.load().iter() {
        let same_world = || {
            !entity.is_removed()
                && std::sync::Arc::ptr_eq(&entity.world.load(), &world)
                && std::sync::Arc::ptr_eq(&player.world(), &world)
        };
        if let Some(client) = player.client.java()
            && let Some(property) = java.properties.first()
            && let Ok(data) = client.serialize_packet(&java)
        {
            let version = client.version.load();
            client.try_enqueue_packet_data_filtering(
                data,
                || {
                    let current = living
                        .attributes
                        .read()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    let matches = current.get(&attribute.id).is_some_and(|value| {
                        value.base_value.to_bits() == property.value.to_bits()
                            && value.modifiers.len() == property.modifiers.len()
                            && value.modifiers.iter().zip(&property.modifiers).all(
                                |(actual, sent)| {
                                    actual.id == sent.id
                                        && actual.amount.to_bits() == sent.amount.to_bits()
                                        && actual.operation as i8 == sent.operation
                                },
                            )
                    });
                    (matches && same_world()).then_some(current)
                },
                |data| filter_java_shield_extra(data, version, java.entity_id, property.id),
            );
        } else if let Some(client) = player.client.bedrock()
            && let Some(property) = bedrock.attribute_list.first()
            && let Ok(data) = client.serialize_packet(&bedrock)
        {
            client.try_enqueue_packet_data_filtering(
                data,
                || {
                    let current = living
                        .attributes
                        .read()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    let matches = current.get(&attribute.id).is_some_and(|value| {
                        (value.base_value as f32).to_bits() == property.default_value.to_bits()
                            && (value.sanitized_value(attribute) as f32).to_bits()
                                == property.current_value.to_bits()
                    });
                    (matches && same_world()).then_some(current)
                },
                |data| {
                    filter_bedrock_shield_extra(
                        client,
                        data,
                        bedrock.target_runtime_id,
                        &property.name,
                    )
                },
            );
        }
    }
}

fn filter_java_shield_extra(
    data: bytes::Bytes,
    version: pumpkin_util::version::JavaMinecraftVersion,
    entity_id: pumpkin_protocol::codec::var_int::VarInt,
    attribute_id: pumpkin_protocol::codec::var_int::VarInt,
) -> Option<bytes::Bytes> {
    use pumpkin_protocol::{
        ServerPacket, java::client::play::CUpdateAttributes as JavaAttributes,
        packet::MultiVersionJavaPacket, ser::NetworkReadExt,
    };
    let mut payload = data.as_ref();
    if !payload
        .get_var_int()
        .is_ok_and(|id| id.0 == JavaAttributes::to_id(version))
    {
        return Some(data);
    }
    let Ok(mut packet) = JavaAttributes::read(&mut payload, &version) else {
        return Some(data);
    };
    if packet.entity_id != entity_id {
        return Some(data);
    }
    let original = packet.properties.len();
    packet.properties.retain(|entry| entry.id != attribute_id);
    match packet.properties.len() {
        0 => None,
        count if count == original => Some(data),
        _ => crate::net::java::JavaClient::serialize_packet_for_version(&packet, version).ok(),
    }
}

fn filter_bedrock_shield_extra(
    client: &crate::net::bedrock::BedrockClient,
    data: bytes::Bytes,
    entity_id: pumpkin_protocol::codec::var_ulong::VarULong,
    attribute_name: &str,
) -> Option<bytes::Bytes> {
    use pumpkin_protocol::{
        bedrock::client::update_attributes::CUpdateAttributes as BedrockAttributes,
        codec::var_uint::VarUInt, packet::Packet, serial::PacketRead,
    };
    let mut payload = data.as_ref();
    if !VarUInt::read(&mut payload).is_ok_and(|id| id.0 == BedrockAttributes::PACKET_ID as u32) {
        return Some(data);
    }
    let Ok(mut packet) = BedrockAttributes::read(&mut payload) else {
        return Some(data);
    };
    if packet.target_runtime_id != entity_id {
        return Some(data);
    }
    let original = packet.attribute_list.len();
    packet
        .attribute_list
        .retain(|entry| entry.name != attribute_name);
    match packet.attribute_list.len() {
        0 => None,
        count if count == original => Some(data),
        _ => client.serialize_packet(&packet).ok(),
    }
}

impl Clone for AttributeInstance {
    fn clone(&self) -> Self {
        Self {
            base_value: self.base_value,
            modifiers: self.modifiers.clone(),
            permanent_modifiers: self.permanent_modifiers.clone(),
            cached_value: AtomicU64::new(self.cached_value.load(Ordering::Relaxed)),
            dirty: AtomicBool::new(self.dirty.load(Ordering::Relaxed)),
            updated: AtomicBool::new(self.updated.load(Ordering::Relaxed)),
        }
    }
}

/// Registry storing per-entity-type base attribute overrides.
/// Internally stores a map from `entity_type.id` -> `FxHashMap`<attribute.id, f64> for O(1) lookup.
#[derive(Default)]
pub struct AttributeRegistry {
    map: FxHashMap<u16, FxHashMap<u8, f64>>,
}

impl AttributeRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the base value for `attribute` for the given entity type id.
    /// If no override exists, returns `attribute.default_value`.
    #[must_use]
    pub fn get_base_value(&self, entity_type_id: u16, attribute: &Attributes) -> f64 {
        self.map
            .get(&entity_type_id)
            .and_then(|map| map.get(&attribute.id))
            .copied()
            .unwrap_or(attribute.default_value)
    }

    /// Return a vector of overrides for the given entity type id.
    /// This allows populating per-entity local attribute instances at spawn time.
    #[must_use]
    pub fn get_overrides_for_entity(&self, entity_type_id: u16) -> Option<Vec<(u8, f64)>> {
        self.map
            .get(&entity_type_id)
            .map(|m| m.iter().map(|(&k, &v)| (k, v)).collect())
    }
}

/// Builder to declaratively assemble attribute overrides for an entity type.
#[derive(Default)]
pub struct AttributeBuilder {
    entries: Vec<(Attributes, f64)>,
}

impl AttributeBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn add(mut self, attribute: Attributes, base: f64) -> Self {
        self.entries.push((attribute, base));
        self
    }

    #[must_use]
    pub fn build(self) -> Vec<(Attributes, f64)> {
        self.entries
    }
}

impl AttributeRegistry {
    /// Register overrides created by an `AttributeBuilder` for `entity_type`.
    pub fn register_builder(
        &mut self,
        entity_type: &'static EntityType,
        builder: AttributeBuilder,
    ) {
        let inner = self.map.entry(entity_type.id).or_default();
        for (attr, val) in builder.build() {
            inner.insert(attr.id, val);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::attributes::Attributes;
    use pumpkin_data::entity::EntityType;

    #[test]
    fn player_base_attributes() {
        let speed_attr = EntityType::PLAYER
            .attributes
            .iter()
            .find(|(attr, _)| attr.id == Attributes::MOVEMENT_SPEED.id);
        assert!(speed_attr.is_some());
        let (_, base_speed) = speed_attr.unwrap();
        assert!((base_speed - 0.1).abs() < 1e-4);
    }

    #[test]
    fn sprinting_modifier_calculation() {
        let mut instance = AttributeInstance::new(0.1);
        assert!((instance.value() - 0.1).abs() < f64::EPSILON);

        let sprinting_mod = Modifier {
            id: "minecraft:sprinting".to_string(),
            amount: 0.300_000_011_920_928_96,
            operation: ModifierOperation::MultiplyTotal,
        };

        instance.add_or_replace_modifier(sprinting_mod);
        let sprinting_value = instance.value();
        // 0.1 * (1.0 + 0.30000001192092896) = 0.1300000011920929
        assert!((sprinting_value - 0.130_000_001_192_092_9).abs() < 1e-9);

        instance.remove_modifier("minecraft:sprinting");
        assert!((instance.value() - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn attribute_modifier_operations() {
        let mut instance = AttributeInstance::new(10.0);

        // ADD: 10.0 + 2.0 + 3.0 = 15.0
        instance.add_or_replace_modifier(Modifier {
            id: "add_1".to_string(),
            amount: 2.0,
            operation: ModifierOperation::Add,
        });
        instance.add_or_replace_modifier(Modifier {
            id: "add_2".to_string(),
            amount: 3.0,
            operation: ModifierOperation::Add,
        });
        assert!((instance.value() - 15.0).abs() < f64::EPSILON);

        // MULTIPLY_BASE: 15.0 * (1.0 + 0.5 + 0.2) = 15.0 * 1.7 = 25.5
        instance.add_or_replace_modifier(Modifier {
            id: "mul_base_1".to_string(),
            amount: 0.5,
            operation: ModifierOperation::MultiplyBase,
        });
        instance.add_or_replace_modifier(Modifier {
            id: "mul_base_2".to_string(),
            amount: 0.2,
            operation: ModifierOperation::MultiplyBase,
        });
        assert!((instance.value() - 25.5).abs() < f64::EPSILON);

        // MULTIPLY_TOTAL: 25.5 * (1.0 + 0.1) * (1.0 + 0.2) = 25.5 * 1.1 * 1.2 = 33.66
        instance.add_or_replace_modifier(Modifier {
            id: "mul_total_1".to_string(),
            amount: 0.1,
            operation: ModifierOperation::MultiplyTotal,
        });
        instance.add_or_replace_modifier(Modifier {
            id: "mul_total_2".to_string(),
            amount: 0.2,
            operation: ModifierOperation::MultiplyTotal,
        });
        assert!((instance.value() - 33.66).abs() < 1e-9);
    }

    #[test]
    fn sanitized_value_clamps_to_ranged_attribute_bounds() {
        // ARMOR has bounds 0.0..=30.0
        let mut armor = AttributeInstance::new(10.0);
        armor.add_or_replace_modifier(Modifier {
            id: "overflow".to_string(),
            amount: 50.0,
            operation: ModifierOperation::Add,
        });
        assert!((armor.sanitized_value(&Attributes::ARMOR) - 30.0).abs() < f64::EPSILON);

        armor.add_or_replace_modifier(Modifier {
            id: "underflow".to_string(),
            amount: -100.0,
            operation: ModifierOperation::Add,
        });
        assert!((armor.sanitized_value(&Attributes::ARMOR) - 0.0).abs() < f64::EPSILON);

        // NaN sanitizes to min_value (RangedAttribute::sanitizeValue)
        let nan_instance = AttributeInstance::new(f64::NAN);
        assert!((nan_instance.sanitized_value(&Attributes::MAX_HEALTH) - 1.0).abs() < f64::EPSILON);
    }
}
