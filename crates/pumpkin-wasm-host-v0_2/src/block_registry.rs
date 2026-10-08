use crate::pumpkin::plugin::block_registry::{
    BlockDefinition as WitBlockDefinition, BlockDrops as WitBlockDrops,
    BlockEntry as WitBlockEntry, BlockShape as WitBlockShape,
    ConnectDirection as WitConnectDirection, ConnectRule as WitConnectRule,
    ConnectTarget as WitConnectTarget, Host, PropertyKind as WitPropertyKind,
    PropertyValue as WitPropertyValue,
};
use pumpkin_core::plugin::permissions;
use pumpkin_data::block_registry::{
    BlockDrops, BlockRegistration, ConnectRule, ConnectTarget, PropertyDefinition, PropertyKind,
    ShapeDefinition,
};
use pumpkin_data::item::Item;
use pumpkin_data::{Block, BlockDirection, BlockId};
use pumpkin_wasm_host_common::state::PluginHostState;
use tracing::info;

const fn from_wit_direction(direction: WitConnectDirection) -> BlockDirection {
    match direction {
        WitConnectDirection::North => BlockDirection::North,
        WitConnectDirection::South => BlockDirection::South,
        WitConnectDirection::East => BlockDirection::East,
        WitConnectDirection::West => BlockDirection::West,
        WitConnectDirection::Up => BlockDirection::Up,
        WitConnectDirection::Down => BlockDirection::Down,
    }
}

fn from_wit_kind(kind: WitPropertyKind) -> PropertyKind {
    match kind {
        WitPropertyKind::Boolean => PropertyKind::Bool,
        WitPropertyKind::IntRange(bounds) => PropertyKind::Int {
            min: bounds.min,
            max: bounds.max,
        },
        WitPropertyKind::Enumeration(values) => PropertyKind::Enum(values),
    }
}

fn from_wit_shape(shape: WitBlockShape) -> ShapeDefinition {
    match shape {
        WitBlockShape::Empty => ShapeDefinition::Empty,
        WitBlockShape::FullCube => ShapeDefinition::FullCube,
        WitBlockShape::Boxes(boxes) => ShapeDefinition::Boxes(
            boxes
                .into_iter()
                .map(|b| [b.min_x, b.min_y, b.min_z, b.max_x, b.max_y, b.max_z])
                .collect(),
        ),
    }
}

fn from_wit_rule(rule: WitConnectRule) -> ConnectRule {
    ConnectRule {
        property: rule.property,
        direction: from_wit_direction(rule.direction),
        target: match rule.target {
            WitConnectTarget::SameBlock => ConnectTarget::SameBlock,
            WitConnectTarget::Block(key) => ConnectTarget::Block(key),
            WitConnectTarget::Tag(tag) => ConnectTarget::Tag(tag),
        },
    }
}

fn from_wit_definition(definition: WitBlockDefinition) -> BlockRegistration {
    BlockRegistration {
        key: definition.key,
        properties: definition
            .properties
            .into_iter()
            .map(|property| PropertyDefinition {
                name: property.name,
                kind: from_wit_kind(property.kind),
            })
            .collect(),
        default_state: definition
            .default_state
            .into_iter()
            .map(|property| (property.name, property.value))
            .collect(),
        hardness: definition.hardness,
        blast_resistance: definition.blast_resistance,
        requires_correct_tool: definition.requires_correct_tool,
        sound_type: definition.sound_type,
        luminance: definition.luminance,
        can_occlude: definition.can_occlude,
        suffocating: definition.suffocating,
        replaceable: definition.replaceable,
        map_color: definition.map_color,
        collision_shape: from_wit_shape(definition.collision_shape),
        selection_shape: from_wit_shape(definition.selection_shape),
        connect_rules: definition
            .connect_rules
            .into_iter()
            .map(from_wit_rule)
            .collect(),
        drops: match definition.drops {
            WitBlockDrops::SelfItem => BlockDrops::SelfItem,
            WitBlockDrops::Nothing => BlockDrops::Nothing,
            WitBlockDrops::LootTable(key) => BlockDrops::LootTable(key),
        },
        tags: definition.tags,
    }
}

fn to_wit_entry(block: &Block) -> Option<WitBlockEntry> {
    let info = block.dynamic_info()?;
    Some(WitBlockEntry {
        key: block.name.to_string(),
        id: u32::from(block.id.as_u16()),
        base_state_id: u32::from(info.base_state_id()),
        state_count: u32::from(info.state_count()),
        item_id: (block.item_id != 0).then_some(u32::from(block.item_id)),
    })
}

/// Fails unless the plugin has the `registry.blocks` permission.
fn check_block_permission(state: &PluginHostState) -> Result<(), String> {
    if state
        .permissions
        .iter()
        .any(|p| p == permissions::REGISTRY_BLOCKS)
    {
        Ok(())
    } else {
        Err(format!(
            "registering blocks needs the '{}' permission",
            permissions::REGISTRY_BLOCKS
        ))
    }
}

fn plugin_name(state: &PluginHostState) -> &str {
    state.name.as_deref().unwrap_or("<unknown>")
}

/// State id of a vanilla block with the given property values, `None` for an unknown property or
/// value. Unlisted properties keep their default.
fn vanilla_state_id(block: &'static Block, properties: &[(&str, &str)]) -> Option<u32> {
    let mut merged: Vec<(&str, &str)> = block
        .properties(block.default_state.id)
        .map(|defaults| defaults.to_props())
        .unwrap_or_default();
    for &(name, value) in properties {
        let entry = merged.iter_mut().find(|entry| entry.0 == name)?;
        entry.1 = value;
    }
    block
        .state_from_properties(&merged)
        .map(|state| u32::from(state.id.as_u16()))
}

impl Host for PluginHostState {
    fn register_block(
        &mut self,
        definition: WitBlockDefinition,
    ) -> wasmtime::Result<Result<u32, String>> {
        // The registry itself rejects new blocks once closed, but allows re-registering a known one.
        if let Err(error) = check_block_permission(self) {
            return Ok(Err(error));
        }

        Ok(
            match Block::register_dynamic(from_wit_definition(definition)) {
                Ok(block) => {
                    info!(
                        "Plugin {} registered block {} with id {} and {} states from state id {}",
                        plugin_name(self),
                        block.name,
                        block.id.as_u16(),
                        block.states.len(),
                        block.states.first().map_or(0, |state| state.id.as_u16()),
                    );
                    Ok(u32::from(block.id.as_u16()))
                }
                Err(error) => Err(error.to_string()),
            },
        )
    }

    fn register_block_tag(
        &mut self,
        tag: String,
        entries: Vec<String>,
    ) -> wasmtime::Result<Result<(), String>> {
        if let Err(error) = check_block_permission(self) {
            return Ok(Err(error));
        }
        if Block::is_dynamic_registry_frozen() {
            return Ok(Err(
                "block tags can only be registered before players connect".to_string(),
            ));
        }
        Block::register_dynamic_tag(&tag, &entries);
        Ok(Ok(()))
    }

    fn set_block_item(
        &mut self,
        item_key: String,
        block_key: String,
    ) -> wasmtime::Result<Result<(), String>> {
        if let Err(error) = check_block_permission(self) {
            return Ok(Err(error));
        }
        let Some(item) = Item::from_registry_key(&item_key) else {
            return Ok(Err(format!("unknown item '{item_key}'")));
        };
        if !item.is_dynamic() {
            return Ok(Err(format!(
                "'{item_key}' is a vanilla item, only custom items can place custom blocks"
            )));
        }
        Ok(match Block::link_dynamic_item(&block_key, item.id) {
            Ok(block) => {
                info!(
                    "Plugin {} linked item {} to block {}",
                    plugin_name(self),
                    item_key,
                    block.name
                );
                Ok(())
            }
            Err(error) => Err(error.to_string()),
        })
    }

    fn get_block_id(&mut self, key: String) -> wasmtime::Result<Option<u32>> {
        Ok(Block::from_name(&key).map(|block| u32::from(block.id.as_u16())))
    }

    fn get_block_key(&mut self, id: u32) -> wasmtime::Result<Option<String>> {
        Ok(u16::try_from(id)
            .ok()
            .and_then(BlockId::from_raw)
            .map(|id| Block::from_id(id).resource_location().into_owned()))
    }

    fn get_vanilla_block_count(&mut self) -> wasmtime::Result<u32> {
        Ok(u32::from(Block::vanilla_block_count()))
    }

    fn get_vanilla_state_count(&mut self) -> wasmtime::Result<u32> {
        Ok(u32::from(Block::vanilla_state_count()))
    }

    fn get_state_id(
        &mut self,
        block: String,
        properties: Vec<WitPropertyValue>,
    ) -> wasmtime::Result<Option<u32>> {
        let Some(block) = Block::from_name(&block) else {
            return Ok(None);
        };
        let properties: Vec<(&str, &str)> = properties
            .iter()
            .map(|property| (property.name.as_str(), property.value.as_str()))
            .collect();
        if let Some(info) = block.dynamic_info() {
            return Ok(info
                .strict_index_from_props(&properties)
                .map(|index| u32::from(info.base_state_id()) + u32::from(index)));
        }
        Ok(vanilla_state_id(block, &properties))
    }

    fn get_registered_blocks(&mut self) -> wasmtime::Result<Vec<WitBlockEntry>> {
        Ok(Block::dynamic_blocks()
            .into_iter()
            .filter_map(to_wit_entry)
            .collect())
    }
}
