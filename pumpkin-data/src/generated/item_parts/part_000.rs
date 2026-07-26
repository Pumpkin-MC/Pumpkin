use crate::Block;
use crate::attributes::Attributes;
use crate::data_component::DataComponent;
#[allow(
    clippy::wildcard_imports,
    clippy::enum_glob_use,
    clippy::too_many_lines
)]
use crate::data_component::DataComponent::*;
use crate::data_component_impl::IDSet::{IDs, Tag};
use crate::data_component_impl::*;
use crate::effect::StatusEffect;
use crate::sound::Sound;
use crate::tag::{RegistryKey, Taggable};
use crate::{AttributeModifierSlot, tag};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_util::text::TextComponent;
use std::borrow::Cow;
use std::hash::{Hash, Hasher};
#[derive(Clone)]
pub struct Item {
    pub id: u16,
    pub registry_key: &'static str,
    pub components: &'static [(DataComponent, &'static dyn DataComponentImpl)],
}
impl Eq for Item {}
impl Hash for Item {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
#[derive(Clone)]
pub enum BedrockItemVersion {
    Legacy,
    DataDriven,
    None,
}
#[derive(Clone)]
pub struct BedrockItem {
    pub id: i16,
    pub registry_key: &'static str,
    pub version: BedrockItemVersion,
    pub component_based: bool,
    pub definition_components: &'static [u8],
}
impl Eq for BedrockItem {}
impl Hash for BedrockItem {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl PartialEq for BedrockItem {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
#[derive(Clone)]
pub struct JavaToBedrockItemMapping {
    pub java_item: &'static Item,
    pub bedrock_item: &'static BedrockItem,
    pub bedrock_data: u32,
    pub bedrock_block_state: u16,
}
impl Eq for JavaToBedrockItemMapping {}
impl Hash for JavaToBedrockItemMapping {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.java_item.id.hash(state);
    }
}
impl PartialEq for JavaToBedrockItemMapping {
    fn eq(&self, other: &Self) -> bool {
        self.java_item.id == other.java_item.id
    }
}
