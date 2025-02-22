use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    block::{
        BlockDirection,
        registry::{BLOCKS, Block, State},
    },
    item::ItemStack,
};

pub(crate) mod age;
pub(crate) mod attachment;
pub(crate) mod axis;
pub(crate) mod cardinal;
pub(crate) mod face;
pub(crate) mod facing;
pub(crate) mod half;
pub(crate) mod layers;
pub(crate) mod open;
pub(crate) mod powered;
pub(crate) mod signal_fire;
pub(crate) mod slab_type;
pub(crate) mod stair_shape;
pub(crate) mod waterlog;

use crate::world::World;

#[derive(Clone, Debug, PartialEq)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

pub trait BlockPropertyMetadata: Sync + Send {
    fn name(&self) -> &'static str;
    fn value(&self) -> String;
    fn from_value(value: String) -> Self
    where
        Self: Sized;
}

#[async_trait]
#[allow(clippy::too_many_arguments)]
pub trait BlockProperty: Sync + Send + BlockPropertyMetadata {
    async fn on_place(
        &self,
        _world: &World,
        _block: &Block,
        _face: &BlockDirection,
        _block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        _player_direction: &Direction,
        _properties: &BlockProperties,
        _other: bool,
    ) -> String {
        self.value()
    }

    async fn can_update(
        &self,
        _value: String,
        _block: &Block,
        block_state: &State,
        _face: &BlockDirection,
        _use_item_on: &SUseItemOn,
        _other: bool,
    ) -> bool {
        block_state.replaceable
    }

    async fn on_interact(&self, value: String, _block: &Block, _item: &ItemStack) -> String {
        value
    }
}

#[derive(Default)]
pub struct BlockPropertiesManager {
    properties_registry: HashMap<u16, BlockProperties>,
    // Properties that are implemented
    registered_properties: HashMap<String, Arc<dyn BlockProperty>>,
}

pub struct BlockProperties {
    // Mappings from property state strings -> offset
    state_mappings: HashMap<Vec<String>, u16>,
    // Mappings from offset -> property state strings
    property_mappings: HashMap<u16, Vec<String>>,
}

impl BlockPropertiesManager {
    pub fn build_properties_registry(&mut self) {
        for block in &BLOCKS.blocks {
            let properties = &block.properties;
            if properties.is_empty() {
                continue;
            }
            let total_combinations: usize = properties.iter().map(|p| p.values.len()).product();

            let mut forward_map = HashMap::with_capacity(total_combinations);
            let mut reverse_map = HashMap::with_capacity(total_combinations);

            for i in 0..total_combinations {
                let mut current = i;
                let mut combination = Vec::with_capacity(properties.len());

                for property in properties.iter().rev() {
                    let property_size = property.values.len();
                    combination.push(current % property_size);
                    current /= property_size;
                }

                combination.reverse();

                let key: Vec<String> = combination
                    .iter()
                    .enumerate()
                    .map(|(prop_idx, &state_idx)| properties[prop_idx].values[state_idx].clone())
                    .collect();

                forward_map.insert(key.clone(), i as u16);
                reverse_map.insert(i as u16, key);
            }
            self.properties_registry.insert(
                block.id,
                BlockProperties {
                    state_mappings: forward_map,
                    property_mappings: reverse_map,
                },
            );
        }
    }

    pub fn register<T: BlockProperty + 'static>(&mut self, property: T) {
        self.registered_properties
            .insert(property.name().to_string(), Arc::new(property));
    }

    pub async fn can_update(
        &self,
        block: &Block,
        block_state: &State,
        face: &BlockDirection,
        use_item_on: &SUseItemOn,
        other: bool,
    ) -> bool {
        if let Some(properties) = self.properties_registry.get(&block.id) {
            let key = block_state.id - block.states[0].id;
            let property_states = properties.property_mappings.get(&key).unwrap();
            for (i, property_value) in property_states.iter().enumerate() {
                let property_name = block.properties[i].name.clone();
                if let Some(property) = self.registered_properties.get(&property_name) {
                    if property
                        .can_update(
                            property_value.clone(),
                            block,
                            block_state,
                            face,
                            use_item_on,
                            other,
                        )
                        .await
                    {
                        return true;
                    }
                }
            }
        }
        false
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn on_place_state(
        &self,
        world: &World,
        block: &Block,
        face: &BlockDirection,
        block_pos: &BlockPos,
        use_item_on: &SUseItemOn,
        player_direction: &Direction,
        other: bool,
    ) -> u16 {
        if let Some(properties) = self.properties_registry.get(&block.id) {
            let mut hmap_key: Vec<String> = Vec::with_capacity(block.properties.len());

            for raw_property in &block.properties {
                let property = self.registered_properties.get(&raw_property.name);
                if let Some(property) = property {
                    let state = property
                        .on_place(
                            world,
                            block,
                            face,
                            block_pos,
                            use_item_on,
                            player_direction,
                            properties,
                            other,
                        )
                        .await;
                    hmap_key.push(state);
                } else {
                    log::warn!("Unknown Block Property: {}", &raw_property.name);
                    // if one property is not found everything will not work
                    return block.default_state_id;
                }
            }
            // Base state id plus offset
            let mapping = properties.state_mappings.get(&hmap_key);
            if let Some(mapping) = mapping {
                return block.states[0].id + mapping;
            }
            log::error!("Failed to get Block Properties mapping for {}", block.name);
        }
        block.default_state_id
    }

    pub async fn on_interact(&self, block: &Block, block_state: &State, item: &ItemStack) -> u16 {
        if let Some(properties) = self.properties_registry.get(&block.id) {
            if let Some(states) = properties
                .property_mappings
                .get(&(block_state.id - block.states[0].id))
            {
                let mut hmap_key: Vec<String> = Vec::with_capacity(block.properties.len());

                for (i, raw_property) in block.properties.iter().enumerate() {
                    let property = self.registered_properties.get(&raw_property.name);
                    if let Some(property) = property {
                        let state = property.on_interact(states[i].clone(), block, item).await;
                        hmap_key.push(state);
                    } else {
                        log::warn!("Unknown Block Property: {}", &raw_property.name);
                        // if one property is not found everything will not work
                        return block.default_state_id;
                    }
                }
                // Base state id plus offset
                let mapping = properties.state_mappings.get(&hmap_key);
                if let Some(mapping) = mapping {
                    return block.states[0].id + mapping;
                }
                log::error!("Failed to get Block Properties mapping for {}", block.name);
            }
        }
        block_state.id
    }
}
