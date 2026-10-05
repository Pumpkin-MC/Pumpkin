//! Runtime blocks registered by plugins on top of the generated vanilla blocks.
//!
//! The generated `Block` and `BlockState` tables are compile time only. Blocks added here get the
//! ids `BlockId::COUNT + index` and their states the ids `BlockStateId::COUNT + offset`, both in
//! registration order. The generated lookups (`Block::from_id`, `BlockState::from_id`,
//! `Block::from_registry_key`, ...) fall back to this registry after their static tables miss, so
//! every caller that maps an id or a key to a block also sees dynamic blocks.
//!
//! # State ids
//!
//! The states of a block are numbered like vanilla's `StateDefinition` does it:
//!
//! * the properties are sorted by name (byte order),
//! * the first property is the most significant, the last one changes fastest,
//! * the values of a property are in declaration order: `true` before `false`, integers from the
//!   minimum to the maximum, enum values as listed.
//!
//! `state id = base state id + sum(value index * stride)` where the stride of a property is the
//! product of the value counts of all properties after it.
//!
//! Registration is append only and closes with [`Block::freeze_dynamic_registry`], which the server
//! calls before it accepts connections. Entries are leaked so they can be `&'static`.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{LazyLock, OnceLock, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;

use crate::block_properties::{BlockProperties, COLLISION_SHAPES, NoteblockInstrument};
use crate::block_state::{self as flags, PistonBehavior};
use crate::tag::{RegistryKey, Taggable};
use crate::{Block, BlockDirection, BlockId, BlockState, BlockStateId};

/// Namespace of the generated blocks. It is implicit in their `name`.
const VANILLA_NAMESPACE: &str = "minecraft";
const VANILLA_NAMESPACE_PREFIX: &str = "minecraft:";
/// Vanilla's default friction.
const DEFAULT_SLIPPERINESS: f32 = 0.6;
/// A block with more states than this is rejected. Vanilla's largest block has a few thousand.
///
/// This is only a sanity bound, the hard limit is the 16 bit state id space shared by all blocks:
/// [`BlockStateId`] is a `u16`, so generated and registered states together can not pass
/// `u16::MAX + 1` (65536), which leaves `65536 - BlockStateId::STATE_COUNT` states for plugin
/// blocks in total (29813 with the 35723 states of 1.21.x). The chunk palettes follow the total,
/// see `block_network_max_bits`, they need at most 16 bits.
const MAX_STATES_PER_BLOCK: u32 = 16384;
/// Number of state ids, `BlockStateId` is a `u16`.
const STATE_ID_SPACE: usize = u16::MAX as usize + 1;
const MAX_PROPERTIES: usize = 32;
const MAX_ENUM_VALUES: usize = 256;
const MAX_BOXES: usize = 64;
const MAX_PLACEMENT_RULES: usize = 64;
const MAX_LUMINANCE: u8 = 15;
/// Opacity of a block that occludes light, see `BlockState::opacity`.
const OPAQUE_LIGHT_BLOCK: u8 = 15;
/// Half the width of the 2 pixel wide square in the middle of a face that `isCenterSolid` checks.
const CENTER_MIN: f64 = 0.4375;
const CENTER_MAX: f64 = 0.5625;

/// Tags that vanilla defines as references to other tags, and that the generated (flattened) tables
/// no longer know as references. Adding an entry to the first tag adds it to the listed ones too
/// (`data/minecraft/tags/block/incorrect_for_*_tool.json`), otherwise a block that `needs_diamond_tool`
/// would be mineable with an iron pickaxe.
const IMPLIED_TAGS: &[(&str, &[&str])] = &[
    (
        "minecraft:needs_diamond_tool",
        &[
            "minecraft:incorrect_for_copper_tool",
            "minecraft:incorrect_for_gold_tool",
            "minecraft:incorrect_for_iron_tool",
            "minecraft:incorrect_for_stone_tool",
            "minecraft:incorrect_for_wooden_tool",
        ],
    ),
    (
        "minecraft:needs_iron_tool",
        &[
            "minecraft:incorrect_for_copper_tool",
            "minecraft:incorrect_for_gold_tool",
            "minecraft:incorrect_for_stone_tool",
            "minecraft:incorrect_for_wooden_tool",
        ],
    ),
    (
        "minecraft:needs_stone_tool",
        &[
            "minecraft:incorrect_for_gold_tool",
            "minecraft:incorrect_for_wooden_tool",
        ],
    ),
];

/// The kind and value range of a block property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyKind {
    /// `true` and `false`, in this order.
    Bool,
    /// The integers `min..=max`.
    Int { min: u8, max: u8 },
    /// The listed values, in this order. Lowercase `a-z 0-9 _`.
    Enum(Vec<String>),
}

impl PropertyKind {
    fn values(&self) -> Vec<String> {
        match self {
            Self::Bool => vec!["true".to_string(), "false".to_string()],
            Self::Int { min, max } => (*min..=*max).map(|value| value.to_string()).collect(),
            Self::Enum(values) => values.clone(),
        }
    }

    fn value_count(&self) -> usize {
        match self {
            Self::Bool => 2,
            Self::Int { min, max } => usize::from(*max).saturating_sub(usize::from(*min)) + 1,
            Self::Enum(values) => values.len(),
        }
    }

    fn value_index(&self, value: &str) -> Option<usize> {
        match self {
            Self::Bool => match value {
                "true" => Some(0),
                "false" => Some(1),
                _ => None,
            },
            Self::Int { min, max } => {
                let parsed: u8 = value.parse().ok()?;
                (*min..=*max)
                    .contains(&parsed)
                    .then(|| usize::from(parsed - *min))
            }
            Self::Enum(values) => values.iter().position(|candidate| candidate == value),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyDefinition {
    pub name: String,
    pub kind: PropertyKind,
}

/// A collision or selection shape.
#[derive(Debug, Clone, PartialEq)]
pub enum ShapeDefinition {
    Empty,
    FullCube,
    /// Boxes `[min x, min y, min z, max x, max y, max z]` in block units.
    Boxes(Vec<[f64; 6]>),
}

/// What a neighbour has to be for a connect rule to set its property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectTarget {
    /// A block of the block's own type.
    SameBlock,
    /// A block with this key (`minecraft:` prefix optional for vanilla blocks).
    Block(String),
    /// A block in this tag.
    Tag(String),
}

/// Sets a boolean property to whether the neighbour in `direction` matches `target`. Applied when
/// the block is placed and whenever that neighbour changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectRule {
    pub property: String,
    pub direction: BlockDirection,
    pub target: ConnectTarget,
}

/// Where a placement rule takes the value of its property from. The value is chosen by name: the
/// property has to list the names the source can give.
#[derive(Debug, Clone, PartialEq)]
pub enum PlacementSource {
    /// The horizontal direction the player looks at: `north`, `south`, `east`, `west`.
    HorizontalFacing,
    /// The direction the player looks at, the dominant one of all six: `down`, `up`, `north`,
    /// `south`, `west`, `east`.
    LookingDirection,
    /// The face of the clicked block, which is the direction from the clicked block to the placed
    /// one: `down`, `up`, `north`, `south`, `west`, `east`.
    ClickedFace,
    /// The axis of the clicked face: `x`, `y` or `z`.
    ClickedAxis,
    /// `bottom` or `top`, like vanilla's stairs and slabs: clicking the top face or the lower half
    /// of a side gives `bottom`.
    ClickedHalf,
    /// `up` when the pitch of the player is this many degrees (above 0, at most 90) or more upwards,
    /// `down` when it is as far downwards, and `horizontal` between. A property without a
    /// `horizontal` value is not set in between (the next rule or the default applies).
    VerticalLook(f32),
    /// `true` when the placed block replaces a water source (waterlogged), `false` otherwise.
    InWater,
    /// `true` when the player sneaks, `false` otherwise.
    Sneaking,
    /// Always this value.
    Constant(String),
}

const HORIZONTAL_NAMES: [&str; 4] = ["north", "south", "east", "west"];
const DIRECTION_NAMES: [&str; 6] = ["down", "up", "north", "south", "west", "east"];

const fn direction_name(direction: BlockDirection) -> &'static str {
    match direction {
        BlockDirection::Down => "down",
        BlockDirection::Up => "up",
        BlockDirection::North => "north",
        BlockDirection::South => "south",
        BlockDirection::West => "west",
        BlockDirection::East => "east",
    }
}

const fn picked_direction(direction: BlockDirection, opposite: bool) -> &'static str {
    if opposite {
        direction_name(direction.opposite())
    } else {
        direction_name(direction)
    }
}

const fn bool_name(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

impl PlacementSource {
    /// The value names that a property has to list for this source.
    fn required_values(&self) -> Vec<&str> {
        match self {
            Self::HorizontalFacing => HORIZONTAL_NAMES.to_vec(),
            Self::LookingDirection | Self::ClickedFace => DIRECTION_NAMES.to_vec(),
            Self::ClickedAxis => vec!["x", "y", "z"],
            Self::ClickedHalf => vec!["bottom", "top"],
            Self::VerticalLook(_) => vec!["up", "down"],
            Self::InWater | Self::Sneaking => vec!["true", "false"],
            Self::Constant(value) => vec![value.as_str()],
        }
    }

    /// The value name this source gives for a placement, `None` when it gives none. `opposite`
    /// flips the result: the opposite direction, up and down, top and bottom, true and false.
    fn value(&self, context: &PlacementContext, opposite: bool) -> Option<&str> {
        Some(match self {
            Self::HorizontalFacing => picked_direction(context.horizontal_facing, opposite),
            Self::LookingDirection => picked_direction(context.looking_direction, opposite),
            Self::ClickedFace => picked_direction(context.clicked_face, opposite),
            Self::ClickedAxis => match context.clicked_face {
                BlockDirection::Up | BlockDirection::Down => "y",
                BlockDirection::North | BlockDirection::South => "z",
                BlockDirection::West | BlockDirection::East => "x",
            },
            Self::ClickedHalf => {
                let bottom = context.clicked_face != BlockDirection::Down
                    && (context.clicked_face == BlockDirection::Up || context.cursor_y <= 0.5);
                if bottom != opposite { "bottom" } else { "top" }
            }
            Self::VerticalLook(threshold) => {
                if context.pitch <= -*threshold {
                    if opposite { "down" } else { "up" }
                } else if context.pitch >= *threshold {
                    if opposite { "up" } else { "down" }
                } else {
                    "horizontal"
                }
            }
            Self::InWater => bool_name(context.in_water != opposite),
            Self::Sneaking => bool_name(context.sneaking != opposite),
            Self::Constant(value) => value.as_str(),
        })
    }
}

/// Sets a property when the block is placed, like vanilla's `getStateForPlacement`. Rules are
/// tried in order: the first rule of a property that gives a value wins, a property without a
/// value keeps its default. Connect rules apply after the placement rules.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacementRule {
    pub property: String,
    pub source: PlacementSource,
    /// Use the opposite of the source's value (see [`PlacementSource`]). Not allowed for
    /// `ClickedAxis` and `Constant`.
    pub opposite: bool,
}

/// What is known about a placement. The server fills it from the player and the clicked block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlacementContext {
    /// The face of the clicked block, from the clicked block to the placed one.
    pub clicked_face: BlockDirection,
    /// The horizontal direction the player looks at.
    pub horizontal_facing: BlockDirection,
    /// The dominant direction the player looks at, vertical ones included.
    pub looking_direction: BlockDirection,
    /// Pitch of the player in degrees, negative looks up.
    pub pitch: f32,
    /// Height of the click on the clicked block, `0..=1`.
    pub cursor_y: f32,
    /// The placed block replaces a water source.
    pub in_water: bool,
    pub sneaking: bool,
}

#[derive(Debug)]
struct ResolvedPlacementRule {
    position: usize,
    rule: PlacementRule,
}

#[derive(Debug)]
struct PlacementRules {
    rules: Vec<PlacementRule>,
    resolved: Vec<ResolvedPlacementRule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockDrops {
    /// One item of the item linked with `Block::link_dynamic_item`.
    SelfItem,
    Nothing,
    /// The loot table with this key, for example `namespace:blocks/name`.
    LootTable(String),
}

/// Description of a block to add to the registry.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockRegistration {
    /// Full resource location, `namespace:path`. The `minecraft` namespace is reserved.
    pub key: String,
    pub properties: Vec<PropertyDefinition>,
    /// Values of the default state. Properties that are not listed get their first value.
    pub default_state: Vec<(String, String)>,
    /// -1 for unbreakable.
    pub hardness: f32,
    pub blast_resistance: f32,
    pub requires_correct_tool: bool,
    /// Name of the vanilla sound type (`amethyst`, `stone`, ...). Clients pick the sounds, the
    /// server only keeps the name.
    pub sound_type: String,
    /// Light emitted by every state, 0 to 15.
    pub luminance: u8,
    /// Vanilla `canOcclude`, false for `noOcclusion()`.
    pub can_occlude: bool,
    /// Vanilla `isSuffocating`. A suffocating full block hurts an entity whose head is inside.
    pub suffocating: bool,
    pub replaceable: bool,
    pub map_color: u8,
    pub collision_shape: ShapeDefinition,
    pub selection_shape: ShapeDefinition,
    pub connect_rules: Vec<ConnectRule>,
    pub drops: BlockDrops,
    /// Block tags the block joins, `namespace:path`.
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockRegistrationError {
    /// The key is not a valid `namespace:path` resource location.
    InvalidKey(String),
    /// The `minecraft` namespace belongs to the generated blocks.
    ReservedNamespace(String),
    /// A block with this key exists already with another definition.
    Duplicate(String),
    /// The definition is not valid, the text says why.
    Invalid(String),
    /// The block has more states than [`MAX_STATES_PER_BLOCK`].
    TooManyStates(String, u32),
    /// There is no dynamic block with this key.
    UnknownBlock(String),
    /// The block or the item is linked to something else already.
    AlreadyLinked(String),
    /// The registry is closed, see [`Block::freeze_dynamic_registry`].
    Frozen,
    /// There are no block or state ids left.
    Full,
    /// The block needs more state ids than are left of the 16 bit state id space (block key,
    /// states needed, states left).
    OutOfStateIds(String, u32, u32),
    /// The block has other placement rules already (block key).
    PlacementRulesChanged(String),
}

impl fmt::Display for BlockRegistrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidKey(key) => {
                write!(
                    f,
                    "'{key}' is not a valid block key, expected namespace:path"
                )
            }
            Self::ReservedNamespace(key) => {
                write!(
                    f,
                    "'{key}' uses the reserved '{VANILLA_NAMESPACE}' namespace"
                )
            }
            Self::Duplicate(key) => {
                write!(
                    f,
                    "block '{key}' is already registered with another definition"
                )
            }
            Self::Invalid(message) => write!(f, "invalid block definition: {message}"),
            Self::TooManyStates(key, count) => write!(
                f,
                "block '{key}' would have {count} states, the limit is {MAX_STATES_PER_BLOCK}"
            ),
            Self::UnknownBlock(key) => write!(f, "'{key}' is not a registered custom block"),
            Self::AlreadyLinked(message) => write!(f, "{message}"),
            Self::Frozen => write!(f, "blocks can only be registered before players connect"),
            Self::Full => write!(f, "the block registry is full"),
            Self::OutOfStateIds(key, needed, left) => write!(
                f,
                "block '{key}' needs {needed} states but only {left} state ids are left, block state ids are 16 bit and shared by all blocks"
            ),
            Self::PlacementRulesChanged(key) => {
                write!(f, "block '{key}' has other placement rules already")
            }
        }
    }
}

impl std::error::Error for BlockRegistrationError {}

fn invalid(message: impl Into<String>) -> BlockRegistrationError {
    BlockRegistrationError::Invalid(message.into())
}

fn is_valid_namespace(namespace: &str) -> bool {
    !namespace.is_empty()
        && namespace
            .bytes()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.'))
}

fn is_valid_path(path: &str) -> bool {
    !path.is_empty()
        && path
            .bytes()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'/'))
}

fn is_valid_identifier(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|b| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'_'))
}

fn check_key(key: &str) -> Result<(), BlockRegistrationError> {
    let (namespace, path) = key
        .split_once(':')
        .ok_or_else(|| BlockRegistrationError::InvalidKey(key.to_string()))?;
    if !is_valid_namespace(namespace) || !is_valid_path(path) {
        return Err(BlockRegistrationError::InvalidKey(key.to_string()));
    }
    if namespace == VANILLA_NAMESPACE {
        return Err(BlockRegistrationError::ReservedNamespace(key.to_string()));
    }
    Ok(())
}

fn normalize_tag(tag: &str) -> String {
    let tag = tag.strip_prefix('#').unwrap_or(tag);
    if tag.contains(':') {
        tag.to_string()
    } else {
        format!("{VANILLA_NAMESPACE_PREFIX}{tag}")
    }
}

/// Name of a block as `Block::name` has it: no prefix for vanilla blocks.
fn normalize_block_key(key: &str) -> String {
    key.strip_prefix(VANILLA_NAMESPACE_PREFIX)
        .unwrap_or(key)
        .to_string()
}

fn check_box(shape: &[f64; 6]) -> Result<(), BlockRegistrationError> {
    if shape.iter().any(|value| !value.is_finite()) {
        return Err(invalid("a shape box has a value that is not finite"));
    }
    if shape[0] > shape[3] || shape[1] > shape[4] || shape[2] > shape[5] {
        return Err(invalid("a shape box has a minimum above its maximum"));
    }
    Ok(())
}

fn check_shape(shape: &ShapeDefinition) -> Result<(), BlockRegistrationError> {
    if let ShapeDefinition::Boxes(boxes) = shape {
        if boxes.len() > MAX_BOXES {
            return Err(invalid(format!("a shape has more than {MAX_BOXES} boxes")));
        }
        for shape_box in boxes {
            check_box(shape_box)?;
        }
    }
    Ok(())
}

/// Validates a registration and puts it in its canonical form: properties sorted by name, the
/// default state complete and sorted, rules and tags sorted. Two registrations that describe the
/// same block are equal after this, which makes re-registering a plugin idempotent.
fn normalize(mut reg: BlockRegistration) -> Result<BlockRegistration, BlockRegistrationError> {
    check_key(&reg.key)?;

    if !reg.hardness.is_finite() || reg.hardness < -1.0 {
        return Err(invalid("hardness must be finite and at least -1"));
    }
    if !reg.blast_resistance.is_finite() || reg.blast_resistance < 0.0 {
        return Err(invalid("blast resistance must be finite and not negative"));
    }
    if reg.luminance > MAX_LUMINANCE {
        return Err(invalid("luminance is at most 15"));
    }
    if !is_valid_identifier(&reg.sound_type) {
        return Err(invalid("sound type must be a lowercase name like 'stone'"));
    }
    check_shape(&reg.collision_shape)?;
    check_shape(&reg.selection_shape)?;

    if reg.properties.len() > MAX_PROPERTIES {
        return Err(invalid(format!("more than {MAX_PROPERTIES} properties")));
    }
    reg.properties.sort_by(|a, b| a.name.cmp(&b.name));
    let mut states: u32 = 1;
    for (index, property) in reg.properties.iter().enumerate() {
        if !is_valid_identifier(&property.name) {
            return Err(invalid(format!(
                "property name '{}' must be lowercase a-z 0-9 _",
                property.name
            )));
        }
        if index > 0 && reg.properties[index - 1].name == property.name {
            return Err(invalid(format!(
                "property '{}' is listed twice",
                property.name
            )));
        }
        match &property.kind {
            PropertyKind::Bool => {}
            PropertyKind::Int { min, max } => {
                if min > max {
                    return Err(invalid(format!(
                        "property '{}' has a minimum above its maximum",
                        property.name
                    )));
                }
            }
            PropertyKind::Enum(values) => {
                if values.is_empty() || values.len() > MAX_ENUM_VALUES {
                    return Err(invalid(format!(
                        "property '{}' needs 1 to {MAX_ENUM_VALUES} values",
                        property.name
                    )));
                }
                for (value_index, value) in values.iter().enumerate() {
                    if !is_valid_identifier(value) {
                        return Err(invalid(format!(
                            "value '{value}' of property '{}' must be lowercase a-z 0-9 _",
                            property.name
                        )));
                    }
                    if values[..value_index].contains(value) {
                        return Err(invalid(format!(
                            "value '{value}' of property '{}' is listed twice",
                            property.name
                        )));
                    }
                }
            }
        }
        states = states.saturating_mul(property.kind.value_count() as u32);
        if states > MAX_STATES_PER_BLOCK {
            return Err(BlockRegistrationError::TooManyStates(
                reg.key.clone(),
                states,
            ));
        }
    }

    // The default state names every property, in property order.
    for (name, _) in &reg.default_state {
        if !reg.properties.iter().any(|property| property.name == *name) {
            return Err(invalid(format!(
                "default state names unknown property '{name}'"
            )));
        }
    }
    let mut defaults = Vec::with_capacity(reg.properties.len());
    for property in &reg.properties {
        let mut listed = reg
            .default_state
            .iter()
            .filter(|(name, _)| *name == property.name);
        let value = match (listed.next(), listed.next()) {
            (Some(_), Some(_)) => {
                return Err(invalid(format!(
                    "default state lists property '{}' twice",
                    property.name
                )));
            }
            (Some((_, value)), None) => {
                if property.kind.value_index(value).is_none() {
                    return Err(invalid(format!(
                        "'{value}' is not a value of property '{}'",
                        property.name
                    )));
                }
                value.clone()
            }
            // Vanilla's `StateDefinition.any()`: the first value.
            (None, _) => property
                .kind
                .values()
                .into_iter()
                .next()
                .unwrap_or_default(),
        };
        defaults.push((property.name.clone(), value));
    }
    reg.default_state = defaults;

    for rule in &mut reg.connect_rules {
        let Some(property) = reg
            .properties
            .iter()
            .find(|property| property.name == rule.property)
        else {
            return Err(invalid(format!(
                "connect rule names unknown property '{}'",
                rule.property
            )));
        };
        if property.kind != PropertyKind::Bool {
            return Err(invalid(format!(
                "connect rule property '{}' is not a boolean",
                rule.property
            )));
        }
        match &mut rule.target {
            ConnectTarget::SameBlock => {}
            ConnectTarget::Block(key) => *key = normalize_block_key(key),
            ConnectTarget::Tag(tag) => *tag = normalize_tag(tag),
        }
    }
    reg.connect_rules
        .sort_by(|a, b| a.property.cmp(&b.property));
    for pair in reg.connect_rules.windows(2) {
        if pair[0].property == pair[1].property {
            return Err(invalid(format!(
                "property '{}' has more than one connect rule",
                pair[0].property
            )));
        }
    }

    if let BlockDrops::LootTable(key) = &reg.drops
        && key.is_empty()
    {
        return Err(invalid("the loot table key is empty"));
    }

    for tag in &mut reg.tags {
        *tag = normalize_tag(tag);
    }
    reg.tags.sort();
    reg.tags.dedup();
    Ok(reg)
}

/// A boolean property rule with the property resolved to its position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConnectRule {
    position: usize,
    pub direction: BlockDirection,
    pub target: ConnectTarget,
}

#[derive(Debug)]
struct PropertyLayout {
    name: &'static str,
    values: Vec<&'static str>,
    /// How much the state index grows when the value index grows by one.
    stride: u32,
}

/// What the server knows about a registered block beyond its `Block` and `BlockState`s.
#[derive(Debug)]
pub struct DynamicBlockInfo {
    pub id: BlockId,
    /// The registration in canonical form.
    pub registration: BlockRegistration,
    base_state_id: u16,
    state_count: u16,
    default_index: u16,
    properties: Vec<PropertyLayout>,
    connect_rules: Vec<ResolvedConnectRule>,
    /// Set once by [`DynamicBlockRegistry::set_placement_rules`].
    placement: OnceLock<PlacementRules>,
}

fn leak_str(text: &str) -> &'static str {
    Box::leak(text.to_string().into_boxed_str())
}

fn bool_value(value: bool) -> usize {
    // `true` is the first value of a boolean property.
    usize::from(!value)
}

impl DynamicBlockInfo {
    fn build(id: BlockId, registration: BlockRegistration, base_state_id: u16) -> Self {
        let mut properties: Vec<PropertyLayout> = registration
            .properties
            .iter()
            .map(|property| PropertyLayout {
                name: leak_str(&property.name),
                values: property
                    .kind
                    .values()
                    .iter()
                    .map(|value| leak_str(value))
                    .collect(),
                stride: 1,
            })
            .collect();
        // The last property changes fastest.
        let mut stride: u32 = 1;
        for layout in properties.iter_mut().rev() {
            layout.stride = stride;
            stride *= layout.values.len() as u32;
        }
        let state_count = stride;

        let connect_rules = registration
            .connect_rules
            .iter()
            .filter_map(|rule| {
                registration
                    .properties
                    .iter()
                    .position(|property| property.name == rule.property)
                    .map(|position| ResolvedConnectRule {
                        position,
                        direction: rule.direction,
                        target: rule.target.clone(),
                    })
            })
            .collect();

        let mut info = Self {
            id,
            registration,
            base_state_id,
            state_count: state_count as u16,
            default_index: 0,
            properties,
            connect_rules,
            placement: OnceLock::new(),
        };
        let defaults: Vec<(String, String)> = info.registration.default_state.clone();
        let props: Vec<(&str, &str)> = defaults
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect();
        info.default_index = info.index_from_props(&props);
        info
    }

    /// The first state id of the block. Its states follow in index order.
    #[must_use]
    pub const fn base_state_id(&self) -> u16 {
        self.base_state_id
    }

    #[must_use]
    pub const fn state_count(&self) -> u16 {
        self.state_count
    }

    #[must_use]
    pub const fn default_state_index(&self) -> u16 {
        self.default_index
    }

    /// The index of a state of this block, `None` for the state of another block.
    #[must_use]
    pub fn state_index(&self, id: BlockStateId) -> Option<u16> {
        let index = id.as_u16().checked_sub(self.base_state_id)?;
        (index < self.state_count).then_some(index)
    }

    #[must_use]
    pub fn property_names(&self) -> Vec<&'static str> {
        self.properties.iter().map(|layout| layout.name).collect()
    }

    #[must_use]
    pub const fn connect_rules(&self) -> &Vec<ResolvedConnectRule> {
        &self.connect_rules
    }

    fn position_of(&self, name: &str) -> Option<usize> {
        self.properties
            .iter()
            .position(|layout| layout.name == name)
    }

    fn value_index_of(&self, state_index: u16, position: usize) -> usize {
        let layout = &self.properties[position];
        ((u32::from(state_index) / layout.stride) % layout.values.len() as u32) as usize
    }

    fn with_value_index(&self, state_index: u16, position: usize, value_index: usize) -> u16 {
        let layout = &self.properties[position];
        let current = self.value_index_of(state_index, position) as u32;
        (u32::from(state_index) - current * layout.stride + value_index as u32 * layout.stride)
            as u16
    }

    /// Property names and values of a state, in property (name) order.
    #[must_use]
    pub fn to_props(&self, state_index: u16) -> Vec<(&'static str, &'static str)> {
        self.properties
            .iter()
            .enumerate()
            .map(|(position, layout)| {
                (
                    layout.name,
                    layout.values[self.value_index_of(state_index, position)],
                )
            })
            .collect()
    }

    /// The value of one property of a state.
    #[must_use]
    pub fn value(&self, state_index: u16, property: &str) -> Option<&'static str> {
        let position = self.position_of(property)?;
        Some(self.properties[position].values[self.value_index_of(state_index, position)])
    }

    /// The state with one property changed, `None` for an unknown property or value.
    #[must_use]
    pub fn with_value(&self, state_index: u16, property: &str, value: &str) -> Option<u16> {
        let position = self.position_of(property)?;
        let value_index = self.properties[position]
            .values
            .iter()
            .position(|candidate| *candidate == value)?;
        Some(self.with_value_index(state_index, position, value_index))
    }

    /// The state with these values. Unknown properties and values are ignored, properties that are
    /// not listed keep their default.
    #[must_use]
    pub fn index_from_props(&self, props: &[(&str, &str)]) -> u16 {
        let mut index = self.default_index;
        for (name, value) in props {
            if let Some(next) = self.with_value(index, name, value) {
                index = next;
            }
        }
        index
    }

    /// Like [`Self::index_from_props`], but an unknown property or value is `None`.
    #[must_use]
    pub fn strict_index_from_props(&self, props: &[(&str, &str)]) -> Option<u16> {
        let mut index = self.default_index;
        for (name, value) in props {
            index = self.with_value(index, name, value)?;
        }
        Some(index)
    }

    /// Whether `neighbour` makes the rule's property true.
    #[must_use]
    pub fn connects(&self, rule: &ResolvedConnectRule, neighbour: &Block) -> bool {
        match &rule.target {
            ConnectTarget::SameBlock => neighbour.id == self.id,
            ConnectTarget::Block(key) => neighbour.name == key.as_str(),
            ConnectTarget::Tag(tag) => neighbour.is_tagged_with(tag).unwrap_or(false),
        }
    }

    /// The placement rules set with [`DynamicBlockRegistry::set_placement_rules`], empty when there
    /// are none.
    #[must_use]
    pub fn placement_rules(&self) -> &[PlacementRule] {
        match self.placement.get() {
            Some(placement) => placement.rules.as_slice(),
            None => &[],
        }
    }

    /// Checks placement rules against the properties of the block.
    fn resolve_placement_rules(
        &self,
        rules: &[PlacementRule],
    ) -> Result<Vec<ResolvedPlacementRule>, BlockRegistrationError> {
        if rules.len() > MAX_PLACEMENT_RULES {
            return Err(invalid(format!(
                "more than {MAX_PLACEMENT_RULES} placement rules"
            )));
        }
        let mut resolved = Vec::with_capacity(rules.len());
        for rule in rules {
            let Some(position) = self.position_of(&rule.property) else {
                return Err(invalid(format!(
                    "placement rule names unknown property '{}'",
                    rule.property
                )));
            };
            match &rule.source {
                PlacementSource::VerticalLook(threshold) => {
                    if !threshold.is_finite() || *threshold <= 0.0 || *threshold > 90.0 {
                        return Err(invalid(
                            "the pitch of a vertical look rule must be above 0 and at most 90",
                        ));
                    }
                }
                PlacementSource::ClickedAxis | PlacementSource::Constant(_) if rule.opposite => {
                    return Err(invalid(format!(
                        "placement rule for '{}' has no opposite",
                        rule.property
                    )));
                }
                _ => {}
            }
            let layout = &self.properties[position];
            for name in rule.source.required_values() {
                if !layout.values.iter().any(|value| *value == name) {
                    return Err(invalid(format!(
                        "property '{}' has no value '{name}', which its placement rule can give",
                        rule.property
                    )));
                }
            }
            resolved.push(ResolvedPlacementRule {
                position,
                rule: rule.clone(),
            });
        }
        Ok(resolved)
    }

    /// `state_index` with the placement rules applied: the properties they give a value for are
    /// set, the others keep their value.
    #[must_use]
    pub fn state_for_placement(&self, state_index: u16, context: &PlacementContext) -> u16 {
        let Some(placement) = self.placement.get() else {
            return state_index;
        };
        let mut index = state_index;
        // At most 32 properties, one bit each. The first rule that gives a value wins.
        let mut set: u32 = 0;
        for rule in &placement.resolved {
            let bit = 1u32 << rule.position;
            if set & bit != 0 {
                continue;
            }
            let Some(name) = rule.rule.source.value(context, rule.rule.opposite) else {
                continue;
            };
            let value_index = self.properties[rule.position]
                .values
                .iter()
                .position(|value| *value == name);
            if let Some(value_index) = value_index {
                index = self.with_value_index(index, rule.position, value_index);
                set |= bit;
            }
        }
        index
    }

    /// The state to place: `state_index` with every connect rule set from the neighbours.
    pub fn state_for_neighbours(
        &self,
        state_index: u16,
        neighbour: impl Fn(BlockDirection) -> &'static Block,
    ) -> u16 {
        let mut index = state_index;
        for rule in &self.connect_rules {
            let connected = self.connects(rule, neighbour(rule.direction));
            index = self.with_value_index(index, rule.position, bool_value(connected));
        }
        index
    }

    /// The state after the neighbour in `direction` changed to `neighbour`.
    #[must_use]
    pub fn update_connection(
        &self,
        state_index: u16,
        direction: BlockDirection,
        neighbour: &Block,
    ) -> u16 {
        let mut index = state_index;
        for rule in self
            .connect_rules
            .iter()
            .filter(|rule| rule.direction == direction)
        {
            let connected = self.connects(rule, neighbour);
            index = self.with_value_index(index, rule.position, bool_value(connected));
        }
        index
    }
}

/// Properties of a dynamic block. The generated property structs only exist for vanilla blocks.
#[derive(Clone, Copy)]
pub struct DynamicBlockProperties {
    info: &'static DynamicBlockInfo,
    index: u16,
}

impl DynamicBlockProperties {
    #[must_use]
    pub const fn new(info: &'static DynamicBlockInfo, index: u16) -> Self {
        Self { info, index }
    }

    /// The properties of a state of the block `info` describes.
    #[must_use]
    pub fn for_state(info: &'static DynamicBlockInfo, id: BlockStateId) -> Self {
        Self {
            info,
            index: info.state_index(id).unwrap_or(info.default_index),
        }
    }

    #[must_use]
    pub const fn info(&self) -> &'static DynamicBlockInfo {
        self.info
    }

    #[must_use]
    pub fn get(&self, property: &str) -> Option<&'static str> {
        self.info.value(self.index, property)
    }
}

fn info_of(block: &Block) -> &'static DynamicBlockInfo {
    block
        .dynamic_info()
        .expect("DynamicBlockProperties is only used with registered dynamic blocks")
}

impl BlockProperties for DynamicBlockProperties {
    fn to_index(&self) -> u16 {
        self.index
    }

    /// Not available, the index alone does not say which block it belongs to. Use
    /// [`DynamicBlockProperties::new`].
    fn from_index(_index: u16) -> Self
    where
        Self: Sized,
    {
        panic!("DynamicBlockProperties needs its block, use DynamicBlockProperties::new")
    }

    fn handles_block_id(id: BlockId) -> bool
    where
        Self: Sized,
    {
        id.as_u16() >= BlockId::BLOCK_COUNT
    }

    fn to_state_id(&self, block: &Block) -> BlockStateId {
        block
            .states
            .get(usize::from(self.index))
            .map_or(block.default_state.id, |state| state.id)
    }

    fn from_state_id(id: BlockStateId, block: &Block) -> Self
    where
        Self: Sized,
    {
        Self::for_state(info_of(block), id)
    }

    fn default(block: &Block) -> Self
    where
        Self: Sized,
    {
        let info = info_of(block);
        Self::new(info, info.default_index)
    }

    fn to_props(&self) -> Vec<(&'static str, &'static str)> {
        self.info.to_props(self.index)
    }

    fn from_props(props: &[(&str, &str)], block: &Block) -> Self
    where
        Self: Sized,
    {
        let info = info_of(block);
        Self::new(info, info.index_from_props(props))
    }
}

fn unit_box() -> BoundingBox {
    BoundingBox {
        min: Vector3::new(0.0, 0.0, 0.0),
        max: Vector3::new(1.0, 1.0, 1.0),
    }
}

fn shape_boxes(shape: &ShapeDefinition) -> Vec<BoundingBox> {
    match shape {
        ShapeDefinition::Empty => Vec::new(),
        ShapeDefinition::FullCube => vec![unit_box()],
        ShapeDefinition::Boxes(boxes) => boxes
            .iter()
            .map(|b| BoundingBox {
                min: Vector3::new(b[0], b[1], b[2]),
                max: Vector3::new(b[3], b[4], b[5]),
            })
            .collect(),
    }
}

fn same_box(a: &BoundingBox, b: &BoundingBox) -> bool {
    a.min.x == b.min.x
        && a.min.y == b.min.y
        && a.min.z == b.min.z
        && a.max.x == b.max.x
        && a.max.y == b.max.y
        && a.max.z == b.max.z
}

fn is_full_cube(boxes: &[BoundingBox]) -> bool {
    boxes.iter().any(|b| {
        b.min.x <= 0.0
            && b.min.y <= 0.0
            && b.min.z <= 0.0
            && b.max.x >= 1.0
            && b.max.y >= 1.0
            && b.max.z >= 1.0
    })
}

/// `isFaceSturdy` per side: a box that touches the side and covers all of it. The result uses the
/// `side_flags` layout of `BlockState`.
fn side_flags(boxes: &[BoundingBox]) -> u8 {
    let covers_x = |b: &BoundingBox| b.min.x <= 0.0 && b.max.x >= 1.0;
    let covers_y = |b: &BoundingBox| b.min.y <= 0.0 && b.max.y >= 1.0;
    let covers_z = |b: &BoundingBox| b.min.z <= 0.0 && b.max.z >= 1.0;
    let covers_center_x = |b: &BoundingBox| b.min.x <= CENTER_MIN && b.max.x >= CENTER_MAX;
    let covers_center_z = |b: &BoundingBox| b.min.z <= CENTER_MIN && b.max.z >= CENTER_MAX;

    let mut result = 0;
    for b in boxes {
        if b.min.y <= 0.0 && covers_x(b) && covers_z(b) {
            result |= flags::DOWN_SIDE_SOLID;
        }
        if b.max.y >= 1.0 && covers_x(b) && covers_z(b) {
            result |= flags::UP_SIDE_SOLID;
        }
        if b.min.z <= 0.0 && covers_x(b) && covers_y(b) {
            result |= flags::NORTH_SIDE_SOLID;
        }
        if b.max.z >= 1.0 && covers_x(b) && covers_y(b) {
            result |= flags::SOUTH_SIDE_SOLID;
        }
        if b.min.x <= 0.0 && covers_y(b) && covers_z(b) {
            result |= flags::WEST_SIDE_SOLID;
        }
        if b.max.x >= 1.0 && covers_y(b) && covers_z(b) {
            result |= flags::EAST_SIDE_SOLID;
        }
        if b.min.y <= 0.0 && covers_center_x(b) && covers_center_z(b) {
            result |= flags::DOWN_CENTER_SOLID;
        }
        if b.max.y >= 1.0 && covers_center_x(b) && covers_center_z(b) {
            result |= flags::UP_CENTER_SOLID;
        }
    }
    result
}

/// The `state_flags`, `side_flags` and `opacity` that every state of a block shares.
fn derive_state_data(reg: &BlockRegistration) -> (u16, u8, u8) {
    let collision = shape_boxes(&reg.collision_shape);
    let has_collision = !collision.is_empty();
    let full_cube = is_full_cube(&collision);
    let solid_render = reg.can_occlude && full_cube;

    let mut state_flags = 0;
    if reg.requires_correct_tool {
        state_flags |= flags::TOOL_REQUIRED;
    }
    if !reg.can_occlude && full_cube {
        state_flags |= flags::SIDED_TRANSPARENCY;
    }
    if reg.replaceable {
        state_flags |= flags::REPLACEABLE;
    }
    if has_collision {
        state_flags |= flags::IS_SOLID;
    }
    if full_cube {
        state_flags |= flags::IS_FULL_CUBE;
        // Vanilla's default `isRedstoneConductor` is a full collision shape.
        state_flags |= flags::IS_SOLID_BLOCK;
    }
    if solid_render {
        state_flags |= flags::IS_SOLID_RENDER;
    }
    if reg.can_occlude && has_collision {
        state_flags |= flags::CAN_OCCLUDE;
    }
    if !reg.suffocating {
        state_flags |= flags::NEVER_SUFFOCATES;
    }

    let opacity = if solid_render { OPAQUE_LIGHT_BLOCK } else { 0 };
    (state_flags, side_flags(&collision), opacity)
}

/// Append only block table. The global instance backs `Block::from_id` and friends.
pub struct DynamicBlockRegistry {
    frozen: bool,
    blocks: Vec<&'static Block>,
    infos: Vec<&'static DynamicBlockInfo>,
    by_key: HashMap<&'static str, usize>,
    /// Every state of every block, by `state id - BlockStateId::COUNT`.
    states: Vec<&'static BlockState>,
    /// The index into `blocks` of the owner of each entry of `states`.
    state_owners: Vec<usize>,
    /// Item id to the index of the block it places.
    item_blocks: HashMap<u16, usize>,
    /// Collision shapes that are not in `COLLISION_SHAPES`, by `shape index - COLLISION_SHAPES.len()`.
    shapes: Vec<BoundingBox>,
}

impl DynamicBlockRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            frozen: false,
            blocks: Vec::new(),
            infos: Vec::new(),
            by_key: HashMap::new(),
            states: Vec::new(),
            state_owners: Vec::new(),
            item_blocks: HashMap::new(),
            shapes: Vec::new(),
        }
    }

    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    #[must_use]
    pub const fn is_frozen(&self) -> bool {
        self.frozen
    }

    fn shape_index(&mut self, shape: &BoundingBox) -> Result<u16, BlockRegistrationError> {
        if let Some(index) = COLLISION_SHAPES
            .iter()
            .position(|known| same_box(known, shape))
        {
            return u16::try_from(index).map_err(|_| BlockRegistrationError::Full);
        }
        let known = self.shapes.iter().position(|known| same_box(known, shape));
        let offset = match known {
            Some(offset) => offset,
            None => {
                self.shapes.push(*shape);
                self.shapes.len() - 1
            }
        };
        u16::try_from(COLLISION_SHAPES.len() + offset).map_err(|_| BlockRegistrationError::Full)
    }

    fn shape_indices(
        &mut self,
        shape: &ShapeDefinition,
    ) -> Result<&'static [u16], BlockRegistrationError> {
        let mut indices = Vec::new();
        for shape_box in shape_boxes(shape) {
            indices.push(self.shape_index(&shape_box)?);
        }
        Ok(Box::leak(indices.into_boxed_slice()))
    }

    fn build_states(
        &mut self,
        info: &DynamicBlockInfo,
    ) -> Result<&'static [BlockState], BlockRegistrationError> {
        let reg = &info.registration;
        let (state_flags, side_flags, opacity) = derive_state_data(reg);
        let collision_shapes = self.shape_indices(&reg.collision_shape)?;
        let outline_shapes = self.shape_indices(&reg.selection_shape)?;

        let states: Vec<BlockState> = (0..info.state_count)
            .map(|index| BlockState {
                id: BlockStateId::from_dynamic_raw(info.base_state_id + index),
                state_flags,
                side_flags,
                instrument: NoteblockInstrument::Harp,
                luminance: reg.luminance,
                piston_behavior: PistonBehavior::Normal,
                hardness: reg.hardness,
                collision_shapes,
                outline_shapes,
                opacity,
                block_entity_type: u16::MAX,
            })
            .collect();
        Ok(Box::leak(states.into_boxed_slice()))
    }

    /// Adds a block. Registering a key again with an identical definition returns the existing
    /// block, even when the registry is closed, so a plugin can be reloaded. A different definition
    /// for a known key is [`BlockRegistrationError::Duplicate`].
    pub fn register(
        &mut self,
        registration: BlockRegistration,
    ) -> Result<&'static Block, BlockRegistrationError> {
        let registration = normalize(registration)?;

        if let Some(&index) = self.by_key.get(registration.key.as_str()) {
            return if self.infos[index].registration == registration {
                Ok(self.blocks[index])
            } else {
                Err(BlockRegistrationError::Duplicate(registration.key))
            };
        }
        if self.frozen {
            return Err(BlockRegistrationError::Frozen);
        }

        let index = self.blocks.len();
        let block_id = usize::from(BlockId::BLOCK_COUNT) + index;
        let base_state_id = usize::from(BlockStateId::STATE_COUNT) + self.states.len();
        let id = u16::try_from(block_id).map_err(|_| BlockRegistrationError::Full)?;
        let base = u16::try_from(base_state_id).map_err(|_| BlockRegistrationError::Full)?;

        let key: &'static str = leak_str(&registration.key);
        let info = DynamicBlockInfo::build(BlockId::from_dynamic_raw(id), registration, base);
        // The last state id has to fit as well: all states of all blocks share the 16 bit ids.
        if base_state_id + usize::from(info.state_count) > STATE_ID_SPACE {
            return Err(BlockRegistrationError::OutOfStateIds(
                key.to_string(),
                u32::from(info.state_count),
                (STATE_ID_SPACE - base_state_id) as u32,
            ));
        }

        let states = self.build_states(&info)?;
        let default_state: &'static BlockState = &states[usize::from(info.default_index)];
        let block: &'static Block = Box::leak(Box::new(Block {
            id: info.id,
            name: key,
            hardness: info.registration.hardness,
            blast_resistance: info.registration.blast_resistance,
            map_color: info.registration.map_color,
            slipperiness: DEFAULT_SLIPPERINESS,
            velocity_multiplier: 1.0,
            jump_velocity_multiplier: 1.0,
            item_id: 0,
            default_state,
            states,
            flammable: None,
            experience: None,
        }));
        let info: &'static DynamicBlockInfo = Box::leak(Box::new(info));

        self.blocks.push(block);
        self.infos.push(info);
        self.by_key.insert(key, index);
        for state in states {
            self.states.push(state);
            self.state_owners.push(index);
        }
        DYNAMIC_BLOCK_COUNT.store(self.blocks.len() as u32, Ordering::Release);
        DYNAMIC_STATE_COUNT.store(self.states.len() as u32, Ordering::Release);
        Ok(block)
    }

    /// Makes `item_id` the item that places the block `block_key`, and the block's `item_id`. Doing
    /// it again with the same pair is fine, also when the registry is closed.
    pub fn link_item(
        &mut self,
        block_key: &str,
        item_id: u16,
    ) -> Result<&'static Block, BlockRegistrationError> {
        let index = *self
            .by_key
            .get(block_key)
            .ok_or_else(|| BlockRegistrationError::UnknownBlock(block_key.to_string()))?;
        if item_id == 0 {
            return Err(BlockRegistrationError::AlreadyLinked(
                "item id 0 is air and cannot place a block".to_string(),
            ));
        }
        let current = self.blocks[index];
        if current.item_id == item_id {
            return Ok(current);
        }
        if current.item_id != 0 {
            return Err(BlockRegistrationError::AlreadyLinked(format!(
                "block '{block_key}' is linked to item {} already",
                current.item_id
            )));
        }
        if let Some(&other) = self.item_blocks.get(&item_id) {
            return Err(BlockRegistrationError::AlreadyLinked(format!(
                "item {item_id} places '{}' already",
                self.blocks[other].name
            )));
        }
        if self.frozen {
            return Err(BlockRegistrationError::Frozen);
        }

        // The states are shared, only the `Block` itself is replaced.
        let mut updated = Block::clone(current);
        updated.item_id = item_id;
        let updated: &'static Block = Box::leak(Box::new(updated));
        self.blocks[index] = updated;
        self.item_blocks.insert(item_id, index);
        Ok(updated)
    }

    /// Sets how placing the block `block_key` chooses its property values. Setting the same rules
    /// again is fine, also when the registry is closed, other rules for a block that has some are
    /// [`BlockRegistrationError::PlacementRulesChanged`].
    pub fn set_placement_rules(
        &mut self,
        block_key: &str,
        rules: Vec<PlacementRule>,
    ) -> Result<&'static DynamicBlockInfo, BlockRegistrationError> {
        let index = *self
            .by_key
            .get(block_key)
            .ok_or_else(|| BlockRegistrationError::UnknownBlock(block_key.to_string()))?;
        let info = self.infos[index];
        let resolved = info.resolve_placement_rules(&rules)?;
        if let Some(existing) = info.placement.get() {
            return if existing.rules == rules {
                Ok(info)
            } else {
                Err(BlockRegistrationError::PlacementRulesChanged(
                    block_key.to_string(),
                ))
            };
        }
        if self.frozen {
            return Err(BlockRegistrationError::Frozen);
        }
        // The registry is locked for writing, nobody else can have set them in the meantime.
        let _ = info.placement.set(PlacementRules { rules, resolved });
        Ok(info)
    }

    #[must_use]
    pub fn get_by_id(&self, id: u16) -> Option<&'static Block> {
        let index = usize::from(id).checked_sub(usize::from(BlockId::BLOCK_COUNT))?;
        self.blocks.get(index).copied()
    }

    #[must_use]
    pub fn get_by_key(&self, key: &str) -> Option<&'static Block> {
        self.by_key.get(key).map(|&index| self.blocks[index])
    }

    #[must_use]
    pub fn get_info(&self, id: u16) -> Option<&'static DynamicBlockInfo> {
        let index = usize::from(id).checked_sub(usize::from(BlockId::BLOCK_COUNT))?;
        self.infos.get(index).copied()
    }

    #[must_use]
    pub fn get_state(&self, id: u16) -> Option<&'static BlockState> {
        let index = usize::from(id).checked_sub(usize::from(BlockStateId::STATE_COUNT))?;
        self.states.get(index).copied()
    }

    #[must_use]
    pub fn get_state_block(&self, id: u16) -> Option<&'static Block> {
        let index = usize::from(id).checked_sub(usize::from(BlockStateId::STATE_COUNT))?;
        let owner = *self.state_owners.get(index)?;
        self.blocks.get(owner).copied()
    }

    #[must_use]
    pub fn get_by_item(&self, item_id: u16) -> Option<&'static Block> {
        self.item_blocks
            .get(&item_id)
            .map(|&index| self.blocks[index])
    }

    /// Registered blocks in registration (and so id) order.
    #[must_use]
    pub fn blocks(&self) -> &[&'static Block] {
        &self.blocks
    }
}

impl Default for DynamicBlockRegistry {
    fn default() -> Self {
        Self::new()
    }
}

static REGISTRY: LazyLock<RwLock<DynamicBlockRegistry>> =
    LazyLock::new(|| RwLock::new(DynamicBlockRegistry::new()));
/// Lets the id checks skip the lock. Updated after the registry changed.
static DYNAMIC_BLOCK_COUNT: AtomicU32 = AtomicU32::new(0);
static DYNAMIC_STATE_COUNT: AtomicU32 = AtomicU32::new(0);

fn read_registry() -> RwLockReadGuard<'static, DynamicBlockRegistry> {
    REGISTRY.read().unwrap_or_else(PoisonError::into_inner)
}

fn write_registry() -> RwLockWriteGuard<'static, DynamicBlockRegistry> {
    REGISTRY.write().unwrap_or_else(PoisonError::into_inner)
}

/// Number of blocks, generated and registered.
pub(crate) fn total_block_count() -> u32 {
    u32::from(BlockId::BLOCK_COUNT) + DYNAMIC_BLOCK_COUNT.load(Ordering::Acquire)
}

/// Number of block states, generated and registered.
pub(crate) fn total_state_count() -> u32 {
    u32::from(BlockStateId::STATE_COUNT) + DYNAMIC_STATE_COUNT.load(Ordering::Acquire)
}

/// Collision shape by index into `COLLISION_SHAPES`, or into the shapes of registered blocks.
pub(crate) fn collision_shape(index: u16) -> BoundingBox {
    if let Some(shape) = COLLISION_SHAPES.get(usize::from(index)) {
        return *shape;
    }
    let offset = usize::from(index) - COLLISION_SHAPES.len();
    read_registry()
        .shapes
        .get(offset)
        .copied()
        .unwrap_or(BoundingBox {
            min: Vector3::new(0.0, 0.0, 0.0),
            max: Vector3::new(0.0, 0.0, 0.0),
        })
}

impl Block {
    /// Number of generated blocks. Dynamic blocks get the ids `vanilla_block_count() + index`.
    #[must_use]
    pub const fn vanilla_block_count() -> u16 {
        BlockId::BLOCK_COUNT
    }

    /// Number of generated block states. The states of dynamic blocks follow.
    #[must_use]
    pub const fn vanilla_state_count() -> u16 {
        BlockStateId::STATE_COUNT
    }

    /// Whether this block was registered at runtime instead of being generated.
    #[must_use]
    pub const fn is_dynamic(&self) -> bool {
        self.id.as_u16() >= BlockId::BLOCK_COUNT
    }

    /// The full resource location, `minecraft:` prefixed for vanilla blocks.
    #[must_use]
    pub fn resource_location(&self) -> Cow<'static, str> {
        if self.is_dynamic() {
            Cow::Borrowed(self.name)
        } else {
            Cow::Owned(format!("{VANILLA_NAMESPACE_PREFIX}{}", self.name))
        }
    }

    /// Registers a block in the global registry and returns it, see
    /// [`DynamicBlockRegistry::register`]. The block joins the tags of its definition.
    pub fn register_dynamic(
        registration: BlockRegistration,
    ) -> Result<&'static Self, BlockRegistrationError> {
        let key = registration.key.clone();
        let tags = registration.tags.clone();
        let block = write_registry().register(registration)?;
        for tag in tags {
            Self::register_dynamic_tag(&tag, std::slice::from_ref(&key));
        }
        Ok(block)
    }

    /// Adds entries (block keys or `#tag` references) to a block tag, and to the tags that vanilla
    /// defines as including it.
    pub fn register_dynamic_tag(tag: &str, entries: &[String]) {
        crate::dynamic_tag::register_tag(RegistryKey::Block, tag, entries.iter().cloned());
        let full = normalize_tag(tag);
        for (child, parents) in IMPLIED_TAGS {
            if *child == full {
                for parent in *parents {
                    crate::dynamic_tag::register_tag(
                        RegistryKey::Block,
                        parent,
                        entries.iter().cloned(),
                    );
                }
            }
        }
    }

    /// Makes an item the item of a registered block, see [`DynamicBlockRegistry::link_item`].
    pub fn link_dynamic_item(
        block_key: &str,
        item_id: u16,
    ) -> Result<&'static Self, BlockRegistrationError> {
        write_registry().link_item(block_key, item_id)
    }

    /// Sets the placement rules of a registered block, see
    /// [`DynamicBlockRegistry::set_placement_rules`].
    pub fn set_dynamic_placement_rules(
        block_key: &str,
        rules: Vec<PlacementRule>,
    ) -> Result<(), BlockRegistrationError> {
        write_registry().set_placement_rules(block_key, rules)?;
        Ok(())
    }

    /// Closes the global registry. Later registrations fail with [`BlockRegistrationError::Frozen`].
    pub fn freeze_dynamic_registry() {
        write_registry().freeze();
    }

    #[must_use]
    pub fn is_dynamic_registry_frozen() -> bool {
        read_registry().is_frozen()
    }

    /// Blocks registered at runtime, in id order.
    #[must_use]
    pub fn dynamic_blocks() -> Vec<&'static Self> {
        read_registry().blocks().to_vec()
    }

    /// What the registry keeps for a dynamic block, `None` for a generated one.
    #[must_use]
    pub fn dynamic_info(&self) -> Option<&'static DynamicBlockInfo> {
        if !self.is_dynamic() {
            return None;
        }
        read_registry().get_info(self.id.as_u16())
    }

    /// The block tags for the client tag sync (generated plus runtime). `None` when there are no
    /// runtime block tags, so the generated tables can be sent as they are.
    #[must_use]
    pub fn network_tags() -> Option<Vec<(String, Vec<u16>)>> {
        crate::dynamic_tag::merged_tags(RegistryKey::Block, |key| {
            Self::from_name(key).map(|block| block.id.as_u16())
        })
    }

    /// Fallback of the generated `from_id` for ids past the vanilla range.
    #[must_use]
    pub fn from_dynamic_id(id: BlockId) -> &'static Self {
        read_registry().get_by_id(id.as_u16()).unwrap_or(&Self::AIR)
    }

    /// Fallback of the generated `from_state_id` for state ids past the vanilla range.
    #[must_use]
    pub fn from_dynamic_state_id(id: BlockStateId) -> &'static Self {
        read_registry()
            .get_state_block(id.as_u16())
            .unwrap_or(&Self::AIR)
    }

    /// Fallback of the generated `from_registry_key` for keys that are not vanilla. Only
    /// namespaced keys can match, vanilla keys never carry a namespace here.
    #[must_use]
    pub fn from_dynamic_registry_key(key: &str) -> Option<&'static Self> {
        if !key.contains(':') {
            return None;
        }
        read_registry().get_by_key(key)
    }

    /// Fallback of the generated `from_item_id`.
    #[must_use]
    pub fn from_dynamic_item_id(item_id: u16) -> Option<&'static Self> {
        read_registry().get_by_item(item_id)
    }

    /// The properties of a state of a dynamic block, `None` for a block without properties.
    #[must_use]
    pub fn dynamic_properties(&self, state_id: BlockStateId) -> Option<Box<dyn BlockProperties>> {
        let info = self.dynamic_info()?;
        if info.properties.is_empty() {
            return None;
        }
        Some(Box::new(DynamicBlockProperties::for_state(info, state_id)))
    }

    /// `from_properties` for a dynamic block.
    #[must_use]
    pub fn dynamic_from_properties(
        &self,
        props: &[(&str, &str)],
    ) -> Option<Box<dyn BlockProperties>> {
        let info = self.dynamic_info()?;
        Some(Box::new(DynamicBlockProperties::new(
            info,
            info.index_from_props(props),
        )))
    }
}

impl BlockState {
    /// Fallback of the generated `from_id` for ids past the vanilla range.
    #[must_use]
    pub fn from_dynamic_id(id: BlockStateId) -> &'static Self {
        read_registry()
            .get_state(id.as_u16())
            .unwrap_or(Block::AIR.default_state)
    }
}

impl BlockId {
    /// Fallback of the generated `from_state_id` for state ids past the vanilla range.
    #[must_use]
    pub fn from_dynamic_state_id(id: BlockStateId) -> Self {
        Block::from_dynamic_state_id(id).id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_properties::BlockProperties as _;

    fn bool_property(name: &str) -> PropertyDefinition {
        PropertyDefinition {
            name: name.to_string(),
            kind: PropertyKind::Bool,
        }
    }

    fn registration(key: &str) -> BlockRegistration {
        BlockRegistration {
            key: key.to_string(),
            properties: Vec::new(),
            default_state: Vec::new(),
            hardness: 1.5,
            blast_resistance: 6.0,
            requires_correct_tool: false,
            sound_type: "stone".to_string(),
            luminance: 0,
            can_occlude: true,
            suffocating: true,
            replaceable: false,
            map_color: 0,
            collision_shape: ShapeDefinition::FullCube,
            selection_shape: ShapeDefinition::FullCube,
            connect_rules: Vec::new(),
            drops: BlockDrops::SelfItem,
            tags: Vec::new(),
        }
    }

    /// The Lonsdaleite wardframe: six booleans, connecting to its own kind.
    fn wardframe(key: &str) -> BlockRegistration {
        let sides = [
            ("north", BlockDirection::North),
            ("south", BlockDirection::South),
            ("east", BlockDirection::East),
            ("west", BlockDirection::West),
            ("up", BlockDirection::Up),
            ("down", BlockDirection::Down),
        ];
        BlockRegistration {
            properties: sides.iter().map(|(name, _)| bool_property(name)).collect(),
            default_state: sides
                .iter()
                .map(|(name, _)| ((*name).to_string(), "false".to_string()))
                .collect(),
            hardness: 5.0,
            blast_resistance: 1200.0,
            requires_correct_tool: true,
            sound_type: "amethyst".to_string(),
            luminance: 7,
            can_occlude: false,
            suffocating: false,
            connect_rules: sides
                .iter()
                .map(|(name, direction)| ConnectRule {
                    property: (*name).to_string(),
                    direction: *direction,
                    target: ConnectTarget::SameBlock,
                })
                .collect(),
            tags: vec![
                "minecraft:mineable/pickaxe".to_string(),
                "minecraft:needs_diamond_tool".to_string(),
            ],
            ..registration(key)
        }
    }

    #[test]
    fn ids_follow_the_vanilla_counts_in_registration_order() {
        let mut registry = DynamicBlockRegistry::new();
        let first = registry.register(registration("test_order:first")).unwrap();
        let mut second = registration("test_order:second");
        second.properties = vec![PropertyDefinition {
            name: "level".to_string(),
            kind: PropertyKind::Int { min: 0, max: 14 },
        }];
        let second = registry.register(second).unwrap();
        assert_eq!(first.id.as_u16(), Block::vanilla_block_count());
        assert_eq!(second.id.as_u16(), Block::vanilla_block_count() + 1);
        // One state for the first block, then 15 for the second.
        assert_eq!(
            first.default_state.id.as_u16(),
            Block::vanilla_state_count()
        );
        assert_eq!(
            second.states[0].id.as_u16(),
            Block::vanilla_state_count() + 1
        );
        assert_eq!(second.states.len(), 15);
        assert_eq!(registry.get_by_id(first.id.as_u16()), Some(first));
        assert_eq!(registry.get_by_key("test_order:second"), Some(second));
        assert_eq!(
            registry.get_state_block(second.states[14].id.as_u16()),
            Some(second)
        );
        assert!(first.is_dynamic());
        assert!(!Block::STONE.is_dynamic());
        assert_eq!(first.resource_location(), "test_order:first");
        assert_eq!(Block::STONE.resource_location(), "minecraft:stone");
        assert!(
            registry
                .get_by_id(Block::vanilla_block_count() + 2)
                .is_none()
        );
    }

    #[test]
    fn state_ids_follow_vanilla_property_order() {
        let mut registry = DynamicBlockRegistry::new();
        let block = registry.register(wardframe("test_order:ward")).unwrap();
        let info = registry.get_info(block.id.as_u16()).unwrap();
        assert_eq!(block.states.len(), 64);
        // Sorted by name, whatever the declaration order was.
        assert_eq!(
            info.property_names(),
            vec!["down", "east", "north", "south", "up", "west"]
        );
        // The first state is all true, the last all false, true before false.
        assert!(info.to_props(0).iter().all(|(_, value)| *value == "true"));
        assert!(info.to_props(63).iter().all(|(_, value)| *value == "false"));
        // The default state is all false: index 63 of 64, like the Java mod's manifest says.
        assert_eq!(info.default_state_index(), 63);
        assert_eq!(block.default_state.id, block.states[63].id);
        // The last property (west) changes fastest, the first (down) is the most significant.
        assert_eq!(info.value(1, "west"), Some("false"));
        assert_eq!(info.value(1, "up"), Some("true"));
        assert_eq!(info.value(32, "down"), Some("false"));
        assert_eq!(info.value(32, "east"), Some("true"));
        // Index 18 = 16 + 2: east and up are the second value (false), the rest the first.
        assert_eq!(
            info.to_props(18),
            vec![
                ("down", "true"),
                ("east", "false"),
                ("north", "true"),
                ("south", "true"),
                ("up", "false"),
                ("west", "true"),
            ]
        );
    }

    #[test]
    fn properties_map_to_state_indices_and_back() {
        let mut registry = DynamicBlockRegistry::new();
        let mut definition = registration("test_props:mixed");
        definition.properties = vec![
            PropertyDefinition {
                name: "mode".to_string(),
                kind: PropertyKind::Enum(vec![
                    "off".to_string(),
                    "on".to_string(),
                    "auto".to_string(),
                ]),
            },
            PropertyDefinition {
                name: "age".to_string(),
                kind: PropertyKind::Int { min: 2, max: 4 },
            },
            bool_property("lit"),
        ];
        definition.default_state = vec![("mode".to_string(), "auto".to_string())];
        let block = registry.register(definition).unwrap();
        let info = registry.get_info(block.id.as_u16()).unwrap();
        // age (3 values), lit (2), mode (3): 18 states, age most significant.
        assert_eq!(block.states.len(), 18);
        assert_eq!(info.property_names(), vec!["age", "lit", "mode"]);
        // Unlisted properties take their first value: age 2, lit true, mode auto.
        assert_eq!(info.default_state_index(), 2);
        for index in 0..info.state_count() {
            let props = info.to_props(index);
            assert_eq!(info.index_from_props(&props), index);
            assert_eq!(info.strict_index_from_props(&props), Some(index));
        }
        assert_eq!(
            info.strict_index_from_props(&[("mode", "on"), ("age", "4"), ("lit", "false")]),
            Some(2 * 6 + 3 + 1)
        );
        assert_eq!(info.strict_index_from_props(&[("mode", "nope")]), None);
        assert_eq!(info.strict_index_from_props(&[("missing", "true")]), None);
        // The lenient mapping keeps the default for what it does not know.
        assert_eq!(
            info.index_from_props(&[("mode", "nope")]),
            info.default_state_index()
        );
        assert_eq!(
            info.with_value(info.default_state_index(), "lit", "false"),
            Some(5)
        );
        assert_eq!(info.value(info.default_state_index(), "age"), Some("2"));
    }

    #[test]
    fn rejects_bad_definitions() {
        let mut registry = DynamicBlockRegistry::new();
        for bad in ["nocolon", "Upper:case", "ns:", ":path", "ns:sp ace"] {
            assert_eq!(
                registry.register(registration(bad)).err(),
                Some(BlockRegistrationError::InvalidKey(bad.to_string())),
            );
        }
        assert_eq!(
            registry.register(registration("minecraft:thing")).err(),
            Some(BlockRegistrationError::ReservedNamespace(
                "minecraft:thing".to_string()
            )),
        );

        let mut luminous = registration("test_bad:light");
        luminous.luminance = 16;
        assert!(matches!(
            registry.register(luminous).err(),
            Some(BlockRegistrationError::Invalid(_))
        ));

        let mut twice = registration("test_bad:twice");
        twice.properties = vec![bool_property("a"), bool_property("a")];
        assert!(matches!(
            registry.register(twice).err(),
            Some(BlockRegistrationError::Invalid(_))
        ));

        let mut unknown_default = registration("test_bad:default");
        unknown_default.properties = vec![bool_property("a")];
        unknown_default.default_state = vec![("a".to_string(), "maybe".to_string())];
        assert!(matches!(
            registry.register(unknown_default).err(),
            Some(BlockRegistrationError::Invalid(_))
        ));

        let mut not_bool = registration("test_bad:rule");
        not_bool.properties = vec![PropertyDefinition {
            name: "level".to_string(),
            kind: PropertyKind::Int { min: 0, max: 3 },
        }];
        not_bool.connect_rules = vec![ConnectRule {
            property: "level".to_string(),
            direction: BlockDirection::North,
            target: ConnectTarget::SameBlock,
        }];
        assert!(matches!(
            registry.register(not_bool).err(),
            Some(BlockRegistrationError::Invalid(_))
        ));

        // 2^15 states.
        let mut huge = registration("test_bad:huge");
        huge.properties = (0..15).map(|i| bool_property(&format!("p{i}"))).collect();
        assert!(matches!(
            registry.register(huge).err(),
            Some(BlockRegistrationError::TooManyStates(..))
        ));

        let mut bad_box = registration("test_bad:box");
        bad_box.collision_shape = ShapeDefinition::Boxes(vec![[0.5, 0.0, 0.0, 0.1, 1.0, 1.0]]);
        assert!(matches!(
            registry.register(bad_box).err(),
            Some(BlockRegistrationError::Invalid(_))
        ));

        // Nothing of the above was added.
        assert!(registry.blocks().is_empty());
    }

    #[test]
    fn duplicates_idempotence_and_frozen() {
        let mut registry = DynamicBlockRegistry::new();
        let first = registry.register(wardframe("test_dup:a")).unwrap();
        // The same definition, written in another order, is the same block.
        let mut shuffled = wardframe("test_dup:a");
        shuffled.properties.reverse();
        shuffled.default_state.reverse();
        shuffled.connect_rules.reverse();
        shuffled.tags.reverse();
        assert_eq!(registry.register(shuffled).unwrap(), first);
        let mut different = wardframe("test_dup:a");
        different.hardness = 1.0;
        assert_eq!(
            registry.register(different).err(),
            Some(BlockRegistrationError::Duplicate("test_dup:a".to_string())),
        );
        assert_eq!(registry.blocks().len(), 1);

        registry.freeze();
        assert_eq!(
            registry.register(registration("test_dup:b")).err(),
            Some(BlockRegistrationError::Frozen),
        );
        // Re-registering a known block still works for plugin reloads.
        assert_eq!(registry.register(wardframe("test_dup:a")).unwrap(), first);
        assert_eq!(registry.blocks().len(), 1);
    }

    #[test]
    fn items_link_once_and_replace_the_block() {
        let mut registry = DynamicBlockRegistry::new();
        let block = registry.register(registration("test_item:a")).unwrap();
        registry.register(registration("test_item:b")).unwrap();
        assert_eq!(block.item_id, 0);
        assert_eq!(
            registry.link_item("test_item:missing", 5).err(),
            Some(BlockRegistrationError::UnknownBlock(
                "test_item:missing".to_string()
            )),
        );
        let linked = registry.link_item("test_item:a", 5000).unwrap();
        assert_eq!(linked.item_id, 5000);
        assert_eq!(linked.id, block.id);
        // The states are shared with the block that was replaced.
        assert_eq!(linked.default_state.id, block.default_state.id);
        assert_eq!(registry.get_by_key("test_item:a"), Some(linked));
        assert_eq!(registry.get_by_item(5000), Some(linked));
        assert_eq!(
            registry.get_state_block(block.default_state.id.as_u16()),
            Some(linked)
        );
        // Same pair again is fine, other pairs are not.
        assert_eq!(registry.link_item("test_item:a", 5000).unwrap(), linked);
        assert!(registry.link_item("test_item:a", 5001).is_err());
        assert!(registry.link_item("test_item:b", 5000).is_err());
        assert!(registry.link_item("test_item:b", 0).is_err());
    }

    #[test]
    fn derived_state_data_matches_vanilla_glass_like_blocks() {
        let mut registry = DynamicBlockRegistry::new();
        let block = registry.register(wardframe("test_flags:ward")).unwrap();
        let state = block.default_state;
        assert_eq!(state.luminance, 7);
        assert_eq!(state.hardness, 5.0);
        assert!(state.tool_required());
        assert!(state.is_full_cube());
        assert!(state.is_solid());
        assert!(!state.can_occlude());
        assert!(!state.is_solid_render());
        assert!(!state.is_air());
        assert!(!state.is_liquid());
        assert!(!state.replaceable());
        assert!(state.never_suffocates());
        assert_eq!(state.opacity, 0);
        assert_eq!(state.collision_shapes, &[0]);
        assert_eq!(state.outline_shapes, &[0]);
        assert!(state.is_side_solid(BlockDirection::Up));
        assert!(state.is_center_solid(BlockDirection::Down));
        assert_eq!(block.hardness, 5.0);
        assert_eq!(block.blast_resistance, 1200.0);

        let stone_like = registry.register(registration("test_flags:stone")).unwrap();
        let state = stone_like.default_state;
        assert!(state.can_occlude() && state.is_solid_render() && state.is_solid_block());
        assert!(!state.never_suffocates());
        assert_eq!(state.opacity, 15);
        // Vanilla stone has the same flags apart from the tool requirement.
        assert_eq!(
            state.state_flags,
            Block::STONE.default_state.state_flags & !4
        );

        let mut ghost = registration("test_flags:ghost");
        ghost.collision_shape = ShapeDefinition::Empty;
        ghost.selection_shape = ShapeDefinition::FullCube;
        ghost.replaceable = true;
        let ghost = registry.register(ghost).unwrap();
        assert!(ghost.default_state.collision_shapes.is_empty());
        assert!(!ghost.default_state.is_solid());
        assert!(!ghost.default_state.is_full_cube());
        assert!(ghost.default_state.replaceable());
        assert_eq!(ghost.default_state.side_flags, 0);
    }

    #[test]
    fn custom_shapes_get_table_indices() {
        let mut registry = DynamicBlockRegistry::new();
        let mut slab = registration("test_shape:slab");
        slab.collision_shape = ShapeDefinition::Boxes(vec![[0.0, 0.0, 0.0, 1.0, 0.5, 1.0]]);
        slab.selection_shape = slab.collision_shape.clone();
        let slab = registry.register(slab).unwrap();
        let state = slab.default_state;
        assert!(!state.is_full_cube());
        assert!(state.is_side_solid(BlockDirection::Down));
        assert!(!state.is_side_solid(BlockDirection::Up));
        let shapes = state.collision_shapes;
        assert_eq!(shapes.len(), 1);

        // An odd box that vanilla does not have is added behind the generated shapes.
        let mut odd = registration("test_shape:odd");
        odd.collision_shape = ShapeDefinition::Boxes(vec![[0.123, 0.0, 0.0, 0.456, 0.789, 1.0]]);
        odd.selection_shape = ShapeDefinition::Empty;
        let odd = registry.register(odd).unwrap();
        let index = odd.default_state.collision_shapes[0];
        assert!(usize::from(index) >= COLLISION_SHAPES.len());
        let shape = registry.shapes[usize::from(index) - COLLISION_SHAPES.len()];
        assert_eq!(shape.min.x, 0.123);
        assert_eq!(shape.max.y, 0.789);
        // The same box again reuses the entry.
        let shape_count = registry.shapes.len();
        let mut again = registration("test_shape:again");
        again.collision_shape = ShapeDefinition::Boxes(vec![[0.123, 0.0, 0.0, 0.456, 0.789, 1.0]]);
        let again = registry.register(again).unwrap();
        assert_eq!(again.default_state.collision_shapes, &[index]);
        assert_eq!(registry.shapes.len(), shape_count);
    }

    #[test]
    fn connect_rules_follow_the_neighbours() {
        let mut registry = DynamicBlockRegistry::new();
        let block = registry.register(wardframe("test_connect:ward")).unwrap();
        let info = registry.get_info(block.id.as_u16()).unwrap();
        // Another block of the same kind is the only thing that connects.
        let neighbour = |direction: BlockDirection| match direction {
            BlockDirection::North | BlockDirection::Up => block,
            _ => &Block::STONE,
        };
        let placed = info.state_for_neighbours(info.default_state_index(), neighbour);
        let props = info.to_props(placed);
        for (name, value) in props {
            let expected = matches!(name, "north" | "up");
            assert_eq!(value, if expected { "true" } else { "false" }, "{name}");
        }
        // A neighbour update only changes the property of its direction.
        let after = info.update_connection(placed, BlockDirection::South, block);
        assert_eq!(info.value(after, "south"), Some("true"));
        assert_eq!(info.value(after, "north"), Some("true"));
        assert_eq!(info.value(after, "west"), Some("false"));
        let cleared = info.update_connection(after, BlockDirection::North, &Block::AIR);
        assert_eq!(info.value(cleared, "north"), Some("false"));
        assert_eq!(info.value(cleared, "south"), Some("true"));
        assert_eq!(
            info.update_connection(cleared, BlockDirection::West, &Block::AIR),
            cleared
        );

        // Rules can target a block or a tag as well.
        let mut fence = registration("test_connect:fence");
        fence.properties = vec![bool_property("north")];
        fence.default_state = vec![("north".to_string(), "false".to_string())];
        fence.connect_rules = vec![ConnectRule {
            property: "north".to_string(),
            direction: BlockDirection::North,
            target: ConnectTarget::Block("minecraft:stone".to_string()),
        }];
        let fence = registry.register(fence).unwrap();
        let fence_info = registry.get_info(fence.id.as_u16()).unwrap();
        let on = fence_info.update_connection(
            fence_info.default_state_index(),
            BlockDirection::North,
            &Block::STONE,
        );
        assert_eq!(fence_info.value(on, "north"), Some("true"));
        let off = fence_info.update_connection(on, BlockDirection::North, &Block::DIRT);
        assert_eq!(fence_info.value(off, "north"), Some("false"));

        let mut tagged = registration("test_connect:tagged");
        tagged.properties = vec![bool_property("up")];
        tagged.default_state = vec![("up".to_string(), "false".to_string())];
        tagged.connect_rules = vec![ConnectRule {
            property: "up".to_string(),
            direction: BlockDirection::Up,
            target: ConnectTarget::Tag("minecraft:logs".to_string()),
        }];
        let tagged = registry.register(tagged).unwrap();
        let tagged_info = registry.get_info(tagged.id.as_u16()).unwrap();
        let on = tagged_info.update_connection(
            tagged_info.default_state_index(),
            BlockDirection::Up,
            &Block::OAK_LOG,
        );
        assert_eq!(tagged_info.value(on, "up"), Some("true"));
        let off = tagged_info.update_connection(on, BlockDirection::Up, &Block::STONE);
        assert_eq!(tagged_info.value(off, "up"), Some("false"));
    }

    #[test]
    fn global_lookups_cover_dynamic_blocks() {
        let block = Block::register_dynamic(wardframe("test_global:ward")).unwrap();
        let again = Block::register_dynamic(wardframe("test_global:ward")).unwrap();
        assert_eq!(block, again);

        assert_eq!(Block::from_id(block.id), block);
        assert_eq!(Block::from_registry_key("test_global:ward"), Some(block));
        assert_eq!(Block::from_name("test_global:ward"), Some(block));
        assert_eq!(Block::from_name("ward"), None);
        assert_eq!(Block::from_name("minecraft:stone"), Some(&Block::STONE));
        assert!(Block::dynamic_blocks().contains(&block));

        for state in block.states {
            assert_eq!(BlockState::from_id(state.id), state);
            assert_eq!(Block::from_state_id(state.id), block);
            assert_eq!(BlockId::from_state_id(state.id), block.id);
            assert_eq!(BlockStateId::from_raw(state.id.as_u16()), Some(state.id));
        }
        assert_eq!(BlockId::from_raw(block.id.as_u16()), Some(block.id));
        assert!(BlockId::from_raw(u16::MAX).is_none());
        assert!(BlockStateId::from_raw(u16::MAX).is_none());
        assert_eq!(BlockStateId::from_raw_or_air(u16::MAX), BlockStateId::AIR);
        assert!(u32::from(BlockStateId::COUNT) + 64 <= BlockStateId::total_count());
        assert!(u32::from(BlockId::COUNT) < BlockId::total_count());
        // The vanilla constructors still stop at the vanilla ranges.
        assert!(BlockStateId::new(block.default_state.id.as_u16()).is_none());

        // Properties through the generic interface.
        let state = block.states[0];
        let props = block.properties(state.id).unwrap().to_props();
        assert_eq!(props.len(), 6);
        assert!(props.iter().all(|(_, value)| *value == "true"));
        let from_props = block.from_properties(&[("north", "true")]);
        let id = from_props.to_state_id(block);
        let reread = block.properties(id).unwrap().to_props();
        assert!(reread.contains(&("north", "true")));
        assert!(reread.contains(&("down", "false")));
        assert_eq!(
            block
                .state_from_properties(&[
                    ("down", "true"),
                    ("east", "true"),
                    ("north", "true"),
                    ("south", "true"),
                    ("up", "true"),
                    ("west", "true"),
                ])
                .map(|state| state.id),
            Some(block.states[0].id)
        );
        assert!(!block.is_waterloggable());
        assert!(!block.is_waterlogged(block.default_state.id));
    }

    #[test]
    fn linked_items_place_blocks_and_tags_work() {
        let block = Block::register_dynamic(wardframe("test_tagged:ward")).unwrap();
        assert!(Block::from_item_id(60_000).is_none());
        let linked = Block::link_dynamic_item("test_tagged:ward", 60_000).unwrap();
        assert_eq!(Block::from_item_id(60_000), Some(linked));
        assert_eq!(Block::from_registry_key("test_tagged:ward"), Some(linked));
        assert_eq!(Block::from_id(block.id).item_id, 60_000);

        // Tag membership through the string interface that tool rules use.
        assert_eq!(
            linked.is_tagged_with("minecraft:mineable/pickaxe"),
            Some(true)
        );
        assert_eq!(
            linked.is_tagged_with("#minecraft:needs_diamond_tool"),
            Some(true)
        );
        assert_eq!(linked.is_tagged_with("minecraft:mineable/axe"), Some(false));
        assert_eq!(
            Block::STONE.is_tagged_with("minecraft:mineable/pickaxe"),
            Some(true)
        );
        // A diamond tier block is also wrong for every lower tier.
        assert_eq!(
            linked.is_tagged_with("minecraft:incorrect_for_iron_tool"),
            Some(true)
        );
        assert_eq!(
            linked.is_tagged_with("minecraft:incorrect_for_wooden_tool"),
            Some(true)
        );
        assert_ne!(
            linked.is_tagged_with("minecraft:incorrect_for_diamond_tool"),
            Some(true)
        );
        assert_eq!(
            Block::OBSIDIAN.is_tagged_with("minecraft:incorrect_for_iron_tool"),
            Some(true)
        );
        // And the client tag sync sees it.
        let tags = Block::network_tags().unwrap();
        let ids = |name: &str| &tags.iter().find(|(n, _)| n == name).unwrap().1;
        assert!(ids("minecraft:needs_diamond_tool").contains(&linked.id.as_u16()));
        assert!(ids("minecraft:needs_diamond_tool").contains(&Block::OBSIDIAN.id.as_u16()));
        assert!(ids("minecraft:mineable/pickaxe").contains(&linked.id.as_u16()));
    }

    fn enum_property(name: &str, values: &[&str]) -> PropertyDefinition {
        PropertyDefinition {
            name: name.to_string(),
            kind: PropertyKind::Enum(values.iter().map(|value| (*value).to_string()).collect()),
        }
    }

    fn rule(property: &str, source: PlacementSource, opposite: bool) -> PlacementRule {
        PlacementRule {
            property: property.to_string(),
            source,
            opposite,
        }
    }

    fn placement(face: BlockDirection) -> PlacementContext {
        PlacementContext {
            clicked_face: face,
            horizontal_facing: BlockDirection::East,
            looking_direction: BlockDirection::Down,
            pitch: 70.0,
            cursor_y: 0.75,
            in_water: true,
            sneaking: false,
        }
    }

    #[test]
    fn a_block_may_have_thousands_of_states() {
        let mut registry = DynamicBlockRegistry::new();
        // 2^14 states, the limit of one block.
        let mut big = registration("test_big:states");
        big.properties = (0..14).map(|i| bool_property(&format!("p{i}"))).collect();
        let big = registry.register(big).unwrap();
        assert_eq!(big.states.len(), 16384);
        // 6400 states, as a block with two big integer properties has.
        let mut wide = registration("test_big:wide");
        wide.properties = vec![
            PropertyDefinition {
                name: "a".to_string(),
                kind: PropertyKind::Int { min: 0, max: 79 },
            },
            PropertyDefinition {
                name: "b".to_string(),
                kind: PropertyKind::Int { min: 0, max: 79 },
            },
        ];
        let wide = registry.register(wide).unwrap();
        assert_eq!(wide.states.len(), 6400);
        assert_eq!(
            wide.states[6399].id.as_u16(),
            wide.states[0].id.as_u16() + 6399
        );
    }

    #[test]
    fn placement_rules_choose_property_values() {
        let mut registry = DynamicBlockRegistry::new();
        let mut reg = registration("test_place:block");
        reg.properties = vec![
            enum_property("facing", &["north", "south", "east", "west"]),
            bool_property("waterlogged"),
            enum_property("half", &["bottom", "top"]),
            enum_property("axis", &["x", "y", "z"]),
            enum_property("look", &["up", "down"]),
            bool_property("sneak"),
        ];
        reg.default_state = vec![
            ("waterlogged".to_string(), "false".to_string()),
            ("sneak".to_string(), "false".to_string()),
        ];
        let block = registry.register(reg).unwrap();
        let info = registry.get_info(block.id.as_u16()).unwrap();
        let default = info.default_state_index();
        // Without rules the state is the default one.
        assert_eq!(
            info.state_for_placement(default, &placement(BlockDirection::North)),
            default
        );

        let rules = vec![
            rule("facing", PlacementSource::HorizontalFacing, true),
            rule("waterlogged", PlacementSource::InWater, false),
            rule("half", PlacementSource::ClickedHalf, false),
            rule("axis", PlacementSource::ClickedAxis, false),
            rule("look", PlacementSource::VerticalLook(45.0), false),
            rule("sneak", PlacementSource::Sneaking, true),
        ];
        registry
            .set_placement_rules("test_place:block", rules.clone())
            .unwrap();
        // The same rules again are fine, also when the registry is closed.
        registry.freeze();
        registry
            .set_placement_rules("test_place:block", rules.clone())
            .unwrap();
        assert!(matches!(
            registry.set_placement_rules("test_place:block", Vec::new()),
            Err(BlockRegistrationError::PlacementRulesChanged(_))
        ));
        assert_eq!(info.placement_rules(), rules.as_slice());

        let state = info.state_for_placement(default, &placement(BlockDirection::North));
        assert_eq!(info.value(state, "facing"), Some("west"));
        assert_eq!(info.value(state, "waterlogged"), Some("true"));
        // Clicked on a side, above the middle.
        assert_eq!(info.value(state, "half"), Some("top"));
        assert_eq!(info.value(state, "axis"), Some("z"));
        assert_eq!(info.value(state, "look"), Some("down"));
        // Not sneaking, opposite.
        assert_eq!(info.value(state, "sneak"), Some("true"));

        let top_face = info.state_for_placement(default, &placement(BlockDirection::Up));
        assert_eq!(info.value(top_face, "half"), Some("bottom"));
        assert_eq!(info.value(top_face, "axis"), Some("y"));
    }

    #[test]
    fn placement_rules_fall_back_in_order() {
        let mut registry = DynamicBlockRegistry::new();
        let mut reg = registration("test_place:fallback");
        reg.properties = vec![enum_property(
            "facing",
            &["north", "south", "east", "west", "up", "down", "none"],
        )];
        reg.default_state = vec![("facing".to_string(), "none".to_string())];
        let block = registry.register(reg).unwrap();
        let info = registry.get_info(block.id.as_u16()).unwrap();
        registry
            .set_placement_rules(
                "test_place:fallback",
                vec![
                    // Looking 80 degrees or more up or down wins. In between the rule gives
                    // `horizontal`, which the property lacks, so the next rule applies.
                    rule("facing", PlacementSource::VerticalLook(80.0), false),
                    rule("facing", PlacementSource::HorizontalFacing, false),
                    rule(
                        "facing",
                        PlacementSource::Constant("none".to_string()),
                        false,
                    ),
                ],
            )
            .unwrap();
        let default = info.default_state_index();
        let mut steep = placement(BlockDirection::North);
        steep.pitch = 85.0;
        let state = info.state_for_placement(default, &steep);
        assert_eq!(info.value(state, "facing"), Some("down"));
        let state = info.state_for_placement(default, &placement(BlockDirection::North));
        assert_eq!(info.value(state, "facing"), Some("east"));
    }

    #[test]
    fn bad_placement_rules_are_rejected() {
        let mut registry = DynamicBlockRegistry::new();
        let mut reg = registration("test_place:bad");
        reg.properties = vec![
            enum_property("facing", &["north", "south"]),
            bool_property("lit"),
            enum_property("axis", &["x", "y", "z"]),
        ];
        let block = registry.register(reg).unwrap();
        let info = registry.get_info(block.id.as_u16()).unwrap();
        let mut rejected = |rules: Vec<PlacementRule>| {
            matches!(
                registry.set_placement_rules("test_place:bad", rules),
                Err(BlockRegistrationError::Invalid(_))
            )
        };
        // A facing property that does not list all four directions.
        assert!(rejected(vec![rule(
            "facing",
            PlacementSource::HorizontalFacing,
            false
        )]));
        // Unknown property.
        assert!(rejected(vec![rule(
            "nope",
            PlacementSource::Sneaking,
            false
        )]));
        // Not a boolean.
        assert!(rejected(vec![rule(
            "axis",
            PlacementSource::InWater,
            false
        )]));
        // Constant that is no value.
        assert!(rejected(vec![rule(
            "facing",
            PlacementSource::Constant("up".to_string()),
            false
        )]));
        // No opposite of an axis.
        assert!(rejected(vec![rule(
            "axis",
            PlacementSource::ClickedAxis,
            true
        )]));
        // Pitch out of range.
        assert!(rejected(vec![rule(
            "axis",
            PlacementSource::VerticalLook(0.0),
            false
        )]));
        // Nothing of the above was kept.
        assert!(info.placement_rules().is_empty());
        // Rules for a block that is not registered.
        assert!(matches!(
            registry.set_placement_rules("test_place:missing", Vec::new()),
            Err(BlockRegistrationError::UnknownBlock(_))
        ));
    }
}
