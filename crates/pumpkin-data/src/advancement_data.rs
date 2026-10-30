use crate::item_stack::ItemStack;
use crate::potion_brewing::ItemRecipe;
use crate::translation::java::{
    CHAT_TYPE_ADVANCEMENT_CHALLENGE, CHAT_TYPE_ADVANCEMENT_GOAL, CHAT_TYPE_ADVANCEMENT_TASK,
};
use crate::{ADVANCEMENT_TREE, Advancement};
use pumpkin_util::identifier::Identifier;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Clone)]
pub struct AdvancementDisplay {
    pub title: &'static str,
    pub description: &'static str,
    pub item_icon: ItemStack,
    pub frame_type: FrameType,
    pub background_texture: Option<&'static str>,
    pub show_toast: bool,
    pub hidden: bool,
    pub announce_to_chat: bool,
    pub x: f32,
    pub y: f32,
}

impl AdvancementDisplay {
    #[allow(deprecated)]
    pub fn get_title(&self) -> TextComponent {
        TextComponent::translate(self.title, [])
    }

    #[allow(deprecated)]
    pub fn get_description(&self) -> TextComponent {
        TextComponent::translate(self.description, [])
    }

    pub fn has_background(&self) -> bool {
        self.background_texture.is_some()
    }
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        title: &'static str,
        description: &'static str,
        item_icon: ItemStack,
        frame_type: FrameType,
        background_texture: Option<&'static str>,
        show_toast: bool,
        hidden: bool,
        announce_to_chat: bool,
        x: f32,
        y: f32,
    ) -> Self {
        Self {
            title,
            description,
            frame_type,
            item_icon,
            background_texture,
            show_toast,
            hidden,
            announce_to_chat,
            x,
            y,
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, Default, Debug)]
#[repr(i32)]
#[serde(rename_all = "lowercase")]
pub enum FrameType {
    #[default]
    Task = 0,
    Challenge = 1,
    Goal = 2,
}

impl FrameType {
    pub fn get_color(&self) -> NamedColor {
        match self {
            FrameType::Task => NamedColor::Green,
            FrameType::Challenge => NamedColor::DarkPurple,
            FrameType::Goal => NamedColor::Green,
        }
    }

    pub fn get_translation(&self) -> &'static str {
        match self {
            FrameType::Task => CHAT_TYPE_ADVANCEMENT_TASK,
            FrameType::Challenge => CHAT_TYPE_ADVANCEMENT_CHALLENGE,
            FrameType::Goal => CHAT_TYPE_ADVANCEMENT_GOAL,
        }
    }
}

#[derive(Clone)]
pub struct AdvancementReward {
    pub experience: i32,
    pub recipes: &'static [ItemRecipe],
}

pub struct AdvancementNode {
    pub children: Vec<usize>,
    pub parent: Option<usize>,
    pub value: &'static Advancement,
}

impl AdvancementNode {
    pub fn add_child(&mut self, child: usize) {
        self.children.push(child);
    }

    #[must_use]
    pub fn new(value: &'static Advancement, parent: Option<usize>) -> Self {
        Self {
            value,
            parent,
            children: Vec::new(),
        }
    }

    #[inline]
    #[must_use]
    pub const fn has_display(&self) -> bool {
        self.value.display.is_some()
    }

    pub fn root(&self) -> &AdvancementNode {
        let mut advancement_node = self;
        while let Some(parent) = &advancement_node.parent {
            advancement_node = &ADVANCEMENT_TREE.nodes_vector[*parent];
        }
        advancement_node
    }
}

impl PartialEq<Self> for AdvancementNode {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for AdvancementNode {}

impl Display for AdvancementNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value.id)
    }
}
impl Hash for AdvancementNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

#[derive(Default)]
pub struct AdvancementTree {
    pub nodes: BTreeMap<Identifier, usize>,
    pub nodes_vector: Vec<AdvancementNode>,
    pub roots: Vec<usize>,
    pub tasks: Vec<usize>,
}

impl AdvancementTree {
    pub fn get_node_from_id(&self, id: &Identifier) -> Option<&AdvancementNode> {
        if let Some(idx) = self.nodes.get(id) {
            self.nodes_vector.get(*idx)
        } else {
            None
        }
    }

    pub fn get_node_from_idx(&self, idx: usize) -> Option<&AdvancementNode> {
        self.nodes_vector.get(idx)
    }

    pub fn get_idx(&self, id: &Identifier) -> Option<usize> {
        self.nodes.get(id).copied()
    }
}

#[derive(Serialize)]
pub struct AdvancementProgressData {
    pub id: Identifier,
    pub progress: Vec<Criteria>,
}

#[derive(Serialize)]
pub struct Criteria {
    pub criterion_id: Arc<str>,
    pub achieve_date: Option<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdvancementCriterion {
    pub name: &'static str,
    pub trigger: &'static str,
    pub conditions: AdvancementCondition,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum AdvancementCondition {
    #[default]
    Empty,
    InventoryChanged(&'static InventoryChangedCondition),
    RecipeUnlocked(&'static str),
    RecipeCrafted(&'static str),
    PlayerGeneratesContainerLoot(&'static str),
    ChangedDimension {
        from: Option<&'static str>,
        to: Option<&'static str>,
    },
    ConsumeItem(ItemCondition),
    PlacedBlock {
        block: Option<&'static str>,
        item: Option<&'static str>,
    },
    EnterBlock(&'static str),
    SlideDownBlock(&'static str),
    PlayerKilledEntity(EntityCondition),
    BredAnimals(&'static BredAnimalsCondition),
    TameAnimal(EntityCondition),
    FilledBucket(&'static str),
    FishingRodHooked(ItemCondition),
    UsingItem {
        item: ItemCondition,
        looking_at: Option<&'static str>,
    },
    UsedTotem(ItemCondition),
    EntityHurtPlayer(DamageCondition),
    PlayerHurtEntity(DamageCondition),
    TargetHit(TargetHitCondition),
    ConstructBeacon(BeaconCondition),
    AllayDropItemOnBlock(&'static ItemUsedOnBlockCondition),
    ThrownItemPickedUpByPlayer(EntityCondition),
    ItemUsedOnBlock(&'static ItemUsedOnBlockCondition),
    Location(&'static LocationCondition),
    KilledByArrow,
    SleptInBed,
    EntityKilledPlayer,
    UsedEnderEye,
    CuredZombieVillager,
    VillagerTrade,
    EnchantedItem,
    BrewedPotion,
    ItemDurabilityChanged {
        item: ItemCondition,
        vehicle_type: Option<&'static str>,
    },
    Levitation {
        distance_y_min: Option<f64>,
    },
    FallFromHeight {
        distance_y_min: Option<f64>,
    },
    FallAfterExplosion {
        distance_y_min: Option<f64>,
        cause_entity: Option<&'static str>,
    },
    NetherTravel {
        distance_horizontal_min: Option<f64>,
    },
    RideEntityInLava {
        distance_horizontal_min: Option<f64>,
        dimension: Option<&'static str>,
        vehicle_type: Option<&'static str>,
    },
    ShotCrossbow(ItemCondition),
    AvoidVibration,
    BeeNestDestroyed {
        block: Option<&'static str>,
        num_bees_inside: Option<i32>,
    },
    ChanneledLightning {
        victim_type: Option<&'static str>,
    },
    LightningStrike {
        bystander_type: Option<&'static str>,
        blocks_set_on_fire: Option<i32>,
        distance_max: Option<f64>,
    },
    KillMobNearSculkCatalyst,
    HeroOfTheVillage,
    PlayerInteractedWithEntity {
        entity: Option<EntityCondition>,
        item: Option<ItemCondition>,
        player: Option<EntityCondition>,
    },
    ThrownItemPickedUpByEntity {
        entity: Option<EntityCondition>,
        item: Option<ItemCondition>,
        player: Option<EntityCondition>,
    },
    PlayerShearedEquipment {
        entity: Option<EntityCondition>,
        item: Option<ItemCondition>,
    },
    StartedRiding(Option<EntityCondition>),
    SummonedEntity(EntityCondition),
    CrafterRecipeCrafted(&'static str),
    EffectsChanged,
    SpearMobs {
        count: Option<i32>,
    },
    Impossible,
    Tick,
    Custom(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct InventoryChangedCondition {
    pub items: &'static [ItemCondition],
    pub slots: Option<InventorySlotsCondition>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ItemCondition {
    pub items: &'static [&'static str],
    pub tag: Option<&'static str>,
    pub count_min: Option<i64>,
    pub count_max: Option<i64>,
    pub durability_min: Option<i64>,
    pub durability_max: Option<i64>,
    pub enchantments: &'static [EnchantmentCondition],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnchantmentCondition {
    pub enchantment: &'static str,
    pub min_level: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct InventorySlotsCondition {
    pub empty_min: Option<i64>,
    pub empty_max: Option<i64>,
    pub full_min: Option<i64>,
    pub full_max: Option<i64>,
    pub occupied_min: Option<i64>,
    pub occupied_max: Option<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ItemUsedOnBlockCondition {
    pub blocks: &'static [&'static str],
    pub block_tag: Option<&'static str>,
    pub items: &'static [&'static str],
    pub item_tag: Option<&'static str>,
    pub state_key: Option<&'static str>,
    pub state_value: Option<&'static str>,
    pub smokey: Option<bool>,
    pub biome: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct LocationCondition {
    pub biome: Option<&'static str>,
    pub dimension: Option<&'static str>,
    pub block: Option<&'static str>,
    pub can_see_sky: Option<bool>,
    pub smokey: Option<bool>,
    pub structure: Option<&'static str>,
    pub stepping_on: Option<&'static str>,
    pub feet_item: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct EntityCondition {
    pub entity_type: Option<&'static str>,
    pub tag: Option<&'static str>,
    pub is_baby: Option<bool>,
    pub distance_horizontal_min: Option<f64>,
    pub distance_absolute_min: Option<f64>,
    pub vehicle_type: Option<&'static str>,
    pub vehicle_tag: Option<&'static str>,
    pub passenger_type: Option<&'static str>,
    pub stepping_on: Option<&'static str>,
    pub looking_at: Option<&'static str>,
    pub feet_item: Option<&'static str>,
    pub safe_armor_disallowed: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct BredAnimalsCondition {
    pub parent: Option<EntityCondition>,
    pub partner: Option<EntityCondition>,
    pub child: Option<EntityCondition>,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct DamageCondition {
    pub blocked: Option<bool>,
    pub is_projectile: Option<bool>,
    pub dealt_min: Option<f64>,
    pub taken_min: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct TargetHitCondition {
    pub signal_strength: Option<i32>,
    pub distance_min: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct BeaconCondition {
    pub level_min: Option<i32>,
}
/// Represents the requirements needed to complete an advancement.
/// To mark as done, at least one criterion from each requirement group must be satisfied.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AdvancementRequirement {
    pub requirements: Vec<Vec<Arc<str>>>,
}

impl AdvancementRequirement {
    pub fn from_const(static_requirements: &[&[&str]]) -> Self {
        let requirements = static_requirements
            .iter()
            .map(|inner| inner.iter().map(|s| Arc::from(*s)).collect())
            .collect();
        AdvancementRequirement { requirements }
    }

    /// test if the requirements is complete
    pub fn test(&self, predicate: impl Fn(&str) -> bool) -> bool {
        if self.requirements.is_empty() {
            false
        } else {
            for requirement in &self.requirements {
                if !Self::any_match(requirement, &predicate) {
                    return false;
                }
            }
            true
        }
    }

    /// check if any test pass
    fn any_match(requirements: &Vec<Arc<str>>, predicate: impl Fn(&str) -> bool) -> bool {
        for requirement in requirements {
            if predicate(requirement) {
                return true;
            }
        }
        false
    }

    pub fn names(&self) -> Vec<Arc<str>> {
        self.requirements.iter().flatten().cloned().collect()
    }
}
pub trait Criterion {}
