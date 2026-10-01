use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::identifier::Identifier;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::hash::BuildHasher;
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
    /// Vanilla `AttributeModifier.Operation` serialized name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Add => "add_value",
            Self::MultiplyBase => "add_multiplied_base",
            Self::MultiplyTotal => "add_multiplied_total",
        }
    }

    #[must_use]
    pub fn from_name(operation: &str) -> Option<Self> {
        match operation {
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
    /// Mirrors vanilla's permanent modifiers: only these are saved to NBT.
    /// Modifiers from effects and commands are permanent, while ones from
    /// equipment and movement state are transient.
    pub permanent: bool,
}

/// Per-entity attribute instance used at runtime.
#[derive(Debug)]
pub struct AttributeInstance {
    pub base_value: f64,
    pub modifiers: Vec<Modifier>,
    pub cached_value: AtomicU64,
    pub dirty: AtomicBool,
}

impl AttributeInstance {
    #[must_use]
    pub const fn new(base_value: f64) -> Self {
        Self {
            base_value,
            modifiers: Vec::new(),
            cached_value: AtomicU64::new(base_value.to_bits()),
            dirty: AtomicBool::new(false),
        }
    }

    pub fn value(&self) -> f64 {
        if !self.dirty.load(Ordering::Relaxed) {
            return f64::from_bits(self.cached_value.load(Ordering::Relaxed));
        }

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
        self.dirty.store(true, Ordering::Relaxed);
    }

    pub fn remove_modifier(&mut self, id: &str) {
        if let Some(pos) = self.modifiers.iter().position(|m| m.id == id) {
            self.modifiers.swap_remove(pos);
        }
        self.dirty.store(true, Ordering::Relaxed);
    }

    /// Vanilla `AttributeInstance.pack`: only permanent modifiers are persisted.
    #[must_use]
    pub fn pack(&self) -> Vec<Modifier> {
        self.modifiers
            .iter()
            .filter(|modifier| modifier.permanent)
            .cloned()
            .collect()
    }

    /// Vanilla `AttributeInstance.apply`: restore the base value and the saved
    /// permanent modifiers.
    pub fn apply(&mut self, base_value: f64, modifiers: &[Modifier]) {
        self.base_value = base_value;
        for modifier in modifiers {
            self.add_or_replace_modifier(modifier.clone());
        }
        // Vanilla ends with `setDirty()`. A save without modifiers leaves the
        // cached value built from the constructor base, which would otherwise
        // override the restored base.
        self.dirty.store(true, Ordering::Relaxed);
    }
}

/// Writes the entity's attributes in the vanilla `AttributeInstance.Packed`
/// list layout, used as the `attributes` NBT tag.
pub fn write_attributes_nbt<S: BuildHasher>(
    nbt: &mut NbtCompound,
    attributes: &HashMap<u8, AttributeInstance, S>,
) {
    let mut packed_list = Vec::with_capacity(attributes.len());
    for (id, instance) in attributes {
        let Some(attribute) = Attributes::ALL.iter().find(|attr| attr.id == *id) else {
            continue;
        };
        let modifiers: Vec<NbtTag> = instance
            .pack()
            .into_iter()
            .map(|modifier| {
                let mut modifier_nbt = NbtCompound::new();
                modifier_nbt.put_string("id", modifier.id);
                modifier_nbt.put_double("amount", modifier.amount);
                modifier_nbt.put_string("operation", modifier.operation.as_str().to_string());
                NbtTag::Compound(modifier_nbt)
            })
            .collect();
        let mut packed = NbtCompound::new();
        packed.put_string("id", attribute.name.to_string());
        packed.put_double("base", instance.base_value);
        packed.put_list("modifiers", modifiers);
        packed_list.push(NbtTag::Compound(packed));
    }
    nbt.put_list("attributes", packed_list);
}

/// Reads the `attributes` NBT tag into the entity's attribute map, mirroring
/// vanilla `AttributeMap.apply`.
pub fn read_attributes_nbt<S: BuildHasher>(
    nbt: &NbtCompound,
    attributes: &mut HashMap<u8, AttributeInstance, S>,
) {
    let Some(packed_list) = nbt.get_list("attributes") else {
        return;
    };
    for packed in packed_list {
        let NbtTag::Compound(packed) = packed else {
            continue;
        };
        let Some(id) = packed.get_string("id") else {
            continue;
        };
        // Vanilla resolves the id through `Identifier`, so an id without a
        // namespace defaults to `minecraft`.
        let Ok(identifier) = Identifier::parse(id) else {
            continue;
        };
        let id = identifier.to_string();
        let Some(attribute) = Attributes::ALL.iter().find(|attr| attr.name == id) else {
            continue;
        };
        // The packed codec reads `base` with `Codec.DOUBLE`, which accepts any
        // numeric tag and falls back to 0 when the field is absent.
        let base = match packed.get("base") {
            None => 0.0,
            Some(tag) => match tag.as_numeric_double() {
                Some(base) => base,
                // A non-numeric tag fails the codec, dropping the entry.
                None => continue,
            },
        };
        let mut modifiers = Vec::new();
        if let Some(saved_modifiers) = packed.get_list("modifiers") {
            for modifier_nbt in saved_modifiers {
                let NbtTag::Compound(modifier_nbt) = modifier_nbt else {
                    continue;
                };
                let Some(modifier_id) = modifier_nbt.get_string("id") else {
                    continue;
                };
                let Some(operation) = modifier_nbt
                    .get_string("operation")
                    .and_then(ModifierOperation::from_name)
                else {
                    continue;
                };
                let Some(amount) = modifier_nbt
                    .get("amount")
                    .and_then(NbtTag::as_numeric_double)
                else {
                    continue;
                };
                modifiers.push(Modifier {
                    id: modifier_id.to_string(),
                    amount,
                    operation,
                    permanent: true,
                });
            }
        }
        attributes
            .entry(attribute.id)
            .or_insert_with(|| AttributeInstance::new(base))
            .apply(base, &modifiers);
    }
}

/// Send updates for multiple attributes in a single packet for the given living entity.
pub fn send_attribute_updates_for_living(
    living: &crate::entity::living::LivingEntity,
    attributes: Vec<Attributes>,
) {
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

    for attribute in attributes {
        let base_value = living.get_attribute_base(&attribute);
        let effective_value = living.get_attribute_value(&attribute);

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
            min_value: 0.0,
            max_value: 3.402_823_5E38,
            current_value: effective_value as f32,
            default_min_value: 0.0,
            default_max_value: 3.402_823_5E38,
            default_value: base_value as f32,
            name,
            // Bedrock receives the already-computed effective value above. Do not advertise
            // modifier entries until their payload is encoded as well.
            modifiers: Vec::new(),
        };

        be_attributes.push(be_attribute);
    }

    let je_packet = JePacket::new(living.entity.entity_id.into(), je_properties);
    let world = living.entity.world.load();
    if be_attributes.is_empty() {
        world.broadcast_packet_all(&je_packet);
        return;
    }

    let runtime_id = living.entity.entity_id as u64;
    let be_packet = BePacket {
        target_runtime_id: VarULong(runtime_id),
        attribute_list: be_attributes,
        tick: VarULong(0),
    };

    world.broadcast_editioned(&je_packet, &be_packet);
}

impl Clone for AttributeInstance {
    fn clone(&self) -> Self {
        Self {
            base_value: self.base_value,
            modifiers: self.modifiers.clone(),
            cached_value: AtomicU64::new(self.cached_value.load(Ordering::Relaxed)),
            dirty: AtomicBool::new(self.dirty.load(Ordering::Relaxed)),
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
            permanent: false,
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
            permanent: false,
        });
        instance.add_or_replace_modifier(Modifier {
            id: "add_2".to_string(),
            amount: 3.0,
            operation: ModifierOperation::Add,
            permanent: false,
        });
        assert!((instance.value() - 15.0).abs() < f64::EPSILON);

        // MULTIPLY_BASE: 15.0 * (1.0 + 0.5 + 0.2) = 15.0 * 1.7 = 25.5
        instance.add_or_replace_modifier(Modifier {
            id: "mul_base_1".to_string(),
            amount: 0.5,
            operation: ModifierOperation::MultiplyBase,
            permanent: false,
        });
        instance.add_or_replace_modifier(Modifier {
            id: "mul_base_2".to_string(),
            amount: 0.2,
            operation: ModifierOperation::MultiplyBase,
            permanent: false,
        });
        assert!((instance.value() - 25.5).abs() < f64::EPSILON);

        // MULTIPLY_TOTAL: 25.5 * (1.0 + 0.1) * (1.0 + 0.2) = 25.5 * 1.1 * 1.2 = 33.66
        instance.add_or_replace_modifier(Modifier {
            id: "mul_total_1".to_string(),
            amount: 0.1,
            operation: ModifierOperation::MultiplyTotal,
            permanent: false,
        });
        instance.add_or_replace_modifier(Modifier {
            id: "mul_total_2".to_string(),
            amount: 0.2,
            operation: ModifierOperation::MultiplyTotal,
            permanent: false,
        });
        assert!((instance.value() - 33.66).abs() < 1e-9);
    }

    #[test]
    fn attribute_nbt_round_trip() {
        let mut attributes = FxHashMap::default();
        let mut instance = AttributeInstance::new(5.0);
        instance.add_or_replace_modifier(Modifier {
            id: "command_bonus".to_string(),
            amount: 2.0,
            operation: ModifierOperation::Add,
            permanent: true,
        });
        instance.add_or_replace_modifier(Modifier {
            id: "held_weapon".to_string(),
            amount: 3.0,
            operation: ModifierOperation::Add,
            permanent: false,
        });
        attributes.insert(Attributes::ATTACK_DAMAGE.id, instance);

        let mut nbt = NbtCompound::new();
        write_attributes_nbt(&mut nbt, &attributes);

        // Vanilla stores the operation under its named form, not its index.
        let packed_list = nbt.get_list("attributes").unwrap();
        let NbtTag::Compound(packed) = &packed_list[0] else {
            panic!("packed attribute should be a compound");
        };
        assert_eq!(packed.get_string("id"), Some("minecraft:attack_damage"));
        let saved_modifiers = packed.get_list("modifiers").unwrap();
        let NbtTag::Compound(saved_modifier) = &saved_modifiers[0] else {
            panic!("saved modifier should be a compound");
        };
        assert_eq!(saved_modifier.get_string("operation"), Some("add_value"));

        let mut restored = FxHashMap::default();
        read_attributes_nbt(&nbt, &mut restored);
        let restored = restored.get(&Attributes::ATTACK_DAMAGE.id).unwrap();
        assert!((restored.base_value - 5.0).abs() < f64::EPSILON);
        // Only the permanent modifier survives a save/load cycle.
        assert_eq!(restored.modifiers.len(), 1);
        assert_eq!(restored.modifiers[0].id, "command_bonus");
        assert!(restored.modifiers[0].permanent);
    }

    #[test]
    fn apply_restores_base_without_modifiers() {
        let mut instance = AttributeInstance::new(20.0);
        assert!((instance.value() - 20.0).abs() < f64::EPSILON);

        instance.apply(40.0, &[]);
        assert!((instance.value() - 40.0).abs() < f64::EPSILON);
    }

    #[test]
    fn attribute_nbt_reads_vanilla_packed_layout() {
        // `{id:"scale", base:2}`: the bare id and the integer base are both
        // accepted by vanilla's packed-attribute codec.
        let mut packed = NbtCompound::new();
        packed.put_string("id", "scale".to_string());
        packed.put_int("base", 2);
        let mut nbt = NbtCompound::new();
        nbt.put("attributes", NbtTag::List(vec![NbtTag::Compound(packed)]));

        let mut restored = FxHashMap::default();
        read_attributes_nbt(&nbt, &mut restored);

        let restored = restored
            .get(&Attributes::SCALE.id)
            .expect("a bare `scale` id should resolve to minecraft:scale");
        assert!((restored.value() - 2.0).abs() < f64::EPSILON);
    }
}
