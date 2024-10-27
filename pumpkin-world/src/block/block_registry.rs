use std::sync::LazyLock;

use serde::Deserialize;

use super::BlockState;

pub static BLOCKS: LazyLock<TopLevel> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../assets/blocks.json"))
        .expect("Could not parse blocks.json registry.")
});

pub fn get_block(registry_id: &str) -> Option<&Block> {
    BLOCKS
        .blocks
        .iter()
        .find(|&block| block.name == registry_id)
}

pub fn get_block_by_item<'a>(item_id: u16) -> Option<&'a Block> {
    BLOCKS.blocks.iter().find(|&block| block.item_id == item_id)
}
#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct TopLevel {
    pub blocks: Vec<Block>,
    shapes: Vec<Shape>,
    block_entity_types: Vec<BlockEntityKind>,
}
#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
pub struct Block {
    pub id: u16,
    pub item_id: u16,
    wall_variant_id: Option<u16>,
    translation_key: String,
    pub name: String,
    properties: Vec<Property>,
    pub default_state_id: u16,
    states: Vec<State>,
}
#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
struct BlockEntityKind {
    id: u32,
    ident: String,
    name: String,
}
#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
struct Property {
    name: String,
    values: Vec<String>,
}
#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
struct State {
    id: u16,
    luminance: u8,
    opaque: bool,
    replaceable: bool,
    collision_shapes: Vec<u16>,
    block_entity_type: Option<u32>,
}
#[expect(dead_code)]
#[derive(Deserialize, Clone, Debug)]
struct Shape {
    min_x: f64,
    min_y: f64,
    min_z: f64,
    max_x: f64,
    max_y: f64,
    max_z: f64,
}

#[derive(Default, Copy, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct BlockId {
    pub data: u16,
}

impl BlockId {
    pub fn is_air(&self) -> bool {
        self.data == 0 || self.data == 12959 || self.data == 12958
    }

    pub fn get_id_mojang_repr(&self) -> i32 {
        self.data as i32
    }

    pub fn get_id(&self) -> u16 {
        self.data
    }
}

impl From<BlockState> for BlockId {
    fn from(value: BlockState) -> Self {
        Self {
            data: value.get_id(),
        }
    }
}
