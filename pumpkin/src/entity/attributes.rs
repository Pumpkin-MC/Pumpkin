// Cell is not needed with atomics; removed to avoid unused import warnings
use std::sync::RwLock;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use std::collections::HashMap;
use std::sync::LazyLock;
use uuid::Uuid;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

#[derive(Clone, Debug, Copy)]
#[repr(i8)]
pub enum ModifierOperation {
    Add = 0,         // add value
    MultiplyBase = 1, // multiply base (base * (1 + x))
    MultiplyTotal = 2, // multiply total (applied last)
}

#[derive(Clone, Debug)]
pub struct Modifier {
    pub id: Uuid,
    pub amount: f64,
    pub operation: ModifierOperation,
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
    pub fn new(base_value: f64) -> Self {
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

        self.cached_value.store(value.to_bits(), Ordering::Relaxed);
        self.dirty.store(false, Ordering::Relaxed);

        value
    }

    pub fn add_modifier(&mut self, modifier: Modifier) {
        self.modifiers.push(modifier);
        self.dirty.store(true, Ordering::Relaxed);
    }

    pub fn remove_modifier(&mut self, id: Uuid) {
        if let Some(pos) = self.modifiers.iter().position(|m| m.id == id) {
            self.modifiers.swap_remove(pos);
        }
        self.dirty.store(true, Ordering::Relaxed);
    }
}

/// Send updates for multiple attributes in a single packet for the given living entity.
pub async fn send_attribute_updates_for_living(
    living: &crate::entity::living::LivingEntity,
    attributes: Vec<Attributes>,
) {
    use pumpkin_protocol::java::client::play::Property as JeProperty;
    use pumpkin_protocol::java::client::play::CUpdateAttributes as JePacket;
    use pumpkin_protocol::bedrock::client::update_artributes::{Attribute as BeAttribute, CUpdateAttributes as BePacket};
    use pumpkin_protocol::codec::var_int::VarInt;
    use pumpkin_protocol::codec::{var_uint::VarUInt, var_ulong::VarULong};
    use pumpkin_protocol::java::client::play::AttributeModifier as JeAttrMod;

    let mut je_properties: Vec<JeProperty> = Vec::with_capacity(attributes.len());
    let mut be_attributes: Vec<BeAttribute> = Vec::with_capacity(attributes.len());

    for attribute in attributes.into_iter() {
        let base_value = living.entity.get_attribute_base(&attribute);
        let effective_value = living.entity.get_attribute_value(&attribute);

        // Pull modifiers for this attribute
        let mut modifiers = Vec::new();
        if let Some(inst) = living.entity.attributes.read().unwrap().get(&attribute.id) {
            for mod_inst in &inst.modifiers {
                modifiers.push(JeAttrMod::new(
                    mod_inst.id.to_string(), 
                    mod_inst.amount, 
                    mod_inst.operation.clone() as i8
                ));
            }
        }

        let modifiers_count = modifiers.len();

        // Move modifiers into the property
        je_properties.push(JeProperty::new(VarInt(i32::from(attribute.id)), base_value, modifiers));

        let name = match attribute.id {
            22 => "minecraft:movement".to_string(),
            19 => "minecraft:health".to_string(),
            18 => "minecraft:absorption".to_string(),
            2  => "minecraft:attack_damage".to_string(),
            0  => "minecraft:armor".to_string(),
            16 => "minecraft:knockback_resistance".to_string(),
            17 => "minecraft:luck".to_string(),
            13 => "minecraft:follow_range".to_string(),
            15 => "minecraft:horse.jump_strength".to_string(),
            // Fallback for others
            _ => format!("minecraft:attribute.{}", attribute.id), 
        };

        let be_attribute = BeAttribute {
            min_value: 0.0,
            max_value: 3.402_823_5E38,
            current_value: effective_value as f32,
            default_min_value: 0.0,
            default_max_value: 3.402_823_5E38,
            default_value: base_value as f32,
            name,
            modifiers_list_size: VarUInt(modifiers_count as u32),
        };

        be_attributes.push(be_attribute);
    }

    let je_packet = JePacket::new(living.entity.entity_id.into(), je_properties);

    let runtime_id = living.entity.entity_id as u64;
    let be_packet = BePacket {
        runtime_id: VarULong(runtime_id),
        attributes: be_attributes,
        player_tick: VarULong(0),
    };

    living
        .entity
        .world
        .load()
        .broadcast_editioned(&je_packet, &be_packet)
        .await;
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
/// Internally stores a map from `entity_type.id` -> HashMap<attribute.id, f64> for O(1) lookup.
pub struct AttributeRegistry {
    map: HashMap<u16, HashMap<u8, f64>>,
}

impl AttributeRegistry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Begin registering overrides for an entity type.
    pub fn register<'a>(&'a mut self, entity_type: &'static EntityType) -> RegisterBuilder<'a> {
        let id = entity_type.id;
        self.map.entry(id).or_insert_with(HashMap::new);
        RegisterBuilder { registry: self, id }
    }

    /// Get the base value for `attribute` for the given entity type id.
    /// If no override exists, returns `attribute.default_value`.
    pub fn get_base_value(&self, entity_type_id: u16, attribute: &Attributes) -> f64 {
        if let Some(map) = self.map.get(&entity_type_id) {
            if let Some(val) = map.get(&attribute.id) {
                return *val;
            }
        }
        attribute.default_value
    }

    /// Return a vector of overrides for the given entity type id.
    /// This allows populating per-entity local attribute instances at spawn time.
    pub fn get_overrides_for_entity(&self, entity_type_id: u16) -> Option<Vec<(u8, f64)>> {
        self.map.get(&entity_type_id).map(|m| m.iter().map(|(&k, &v)| (k, v)).collect())
    }
}

pub struct RegisterBuilder<'a> {
    registry: &'a mut AttributeRegistry,
    id: u16,
}

impl<'a> RegisterBuilder<'a> {
    pub fn add(self, attribute: Attributes, base: f64) -> Self {
        if let Some(map) = self.registry.map.get_mut(&self.id) {
            map.insert(attribute.id, base);
        } else {
            let mut m = HashMap::new();
            m.insert(attribute.id, base);
            self.registry.map.insert(self.id, m);
        }
        self
    }
}

/// Builder to declaratively assemble attribute overrides for an entity type.
pub struct AttributeBuilder {
    entries: Vec<(Attributes, f64)>,
}

impl AttributeBuilder {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn add(mut self, attribute: Attributes, base: f64) -> Self {
        self.entries.push((attribute, base));
        self
    }

    pub fn build(self) -> Vec<(Attributes, f64)> {
        self.entries
    }
}

impl AttributeRegistry {
    /// Register overrides created by an `AttributeBuilder` for `entity_type`.
    pub fn register_builder(&mut self, entity_type: &'static EntityType, builder: AttributeBuilder) {
        let id = entity_type.id;
        let inner = self.map.entry(id).or_insert_with(HashMap::new);
        for (attr, val) in builder.build() {
            inner.insert(attr.id, val);
        }
    }
}

// Provide a global default registry that can be referenced.
pub static DEFAULT_ATTRIBUTE_REGISTRY: LazyLock<RwLock<AttributeRegistry>> = LazyLock::new(|| {
    RwLock::new(AttributeRegistry::new())
});

/// Initialize the global attribute registry with per-entity registrations.
pub fn init_all_attributes() {
    let mut reg = DEFAULT_ATTRIBUTE_REGISTRY.write().unwrap();

    // Register per-entity attribute builders here. Add entries as modules implement
    // `create_attributes()` on their entity types.
    reg.register_builder(
        &pumpkin_data::entity::EntityType::CREEPER,
        crate::entity::mob::creeper::CreeperEntity::create_attributes(),
    );

    reg.register_builder(
        &pumpkin_data::entity::EntityType::ENDERMAN,
        crate::entity::mob::enderman::EndermanEntity::create_attributes(),
    );

    reg.register_builder(
        &pumpkin_data::entity::EntityType::IRON_GOLEM,
        crate::entity::passive::iron_golem::IronGolemEntity::create_attributes(),
    );

    reg.register_builder(
        &pumpkin_data::entity::EntityType::WOLF,
        crate::entity::passive::wolf::WolfEntity::create_attributes(),
    );

    reg.register_builder(
        &pumpkin_data::entity::EntityType::SNOW_GOLEM,
        crate::entity::passive::snow_golem::SnowGolemEntity::create_attributes(),
    );

    reg.register_builder(
        &pumpkin_data::entity::EntityType::SKELETON,
        crate::entity::mob::skeleton::SkeletonEntityBase::create_attributes(),
    );

    reg.register_builder(
        &pumpkin_data::entity::EntityType::ZOMBIE,
        crate::entity::mob::zombie::ZombieEntity::create_attributes(),
    );

    reg.register_builder(
        &pumpkin_data::entity::EntityType::PLAYER,
        crate::entity::player::Player::create_attributes(),
    );
}