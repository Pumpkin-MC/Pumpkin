use heck::{ToShoutySnakeCase, ToUpperCamelCase};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{Ident, LitBool, LitFloat, LitInt, LitStr};

include!("../src/tag.rs");

#[derive(Deserialize, Clone, Debug)]
pub struct BlockProperty {
    pub name: String,
    pub values: Vec<String>,
}

fn get_enum_name(props: Vec<String>) -> String {
    // Define the mapping of variant sets to new enum names
    let enum_mappings: &[(&[&str], &str)] = &[
        (&["true", "false"], "Boolean"),
        (&["x", "y", "z"], "Axis"),
        (&["0", "1"], "Level0to1"),
        (&["0", "1", "2", "3", "4"], "Level0to4"),
        (
            &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15",
            ],
            "Level0to15",
        ),
        (&["0", "1", "2", "3"], "Level0to3"),
        (&["1", "2", "3", "4", "5", "6", "7"], "Level1to7"),
        (
            &["north", "east", "south", "west", "up", "down"],
            "Direction",
        ),
        (
            &[
                "harp",
                "basedrum",
                "snare",
                "hat",
                "bass",
                "flute",
                "bell",
                "guitar",
                "chime",
                "xylophone",
                "iron_xylophone",
                "cow_bell",
                "didgeridoo",
                "bit",
                "banjo",
                "pling",
                "zombie",
                "skeleton",
                "creeper",
                "dragon",
                "wither_skeleton",
                "piglin",
                "custom_head",
            ],
            "Instrument",
        ),
        (
            &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15", "16", "17", "18", "19", "20", "21", "22", "23", "24",
            ],
            "Level0to24",
        ),
        (&["north", "south", "west", "east"], "CardinalDirection"),
        (&["head", "foot"], "BedPart"),
        (
            &[
                "north_south",
                "east_west",
                "ascending_east",
                "ascending_west",
                "ascending_north",
                "ascending_south",
            ],
            "RailShape",
        ),
        (&["upper", "lower"], "VerticalHalf"),
        (&["normal", "sticky"], "PistonType"),
        (&["top", "bottom"], "VerticalBlockHalf"),
        (
            &[
                "straight",
                "inner_left",
                "inner_right",
                "outer_left",
                "outer_right",
            ],
            "StairShape",
        ),
        (&["single", "left", "right"], "ChestType"),
        (&["up", "side", "none"], "RedstoneConnection"),
        (&["0", "1", "2", "3", "4", "5", "6", "7"], "Level0to7"),
        (&["left", "right"], "DoorHinge"),
        (
            &[
                "north_south",
                "east_west",
                "ascending_east",
                "ascending_west",
                "ascending_north",
                "ascending_south",
                "south_east",
                "south_west",
                "north_west",
                "north_east",
            ],
            "ExtendedRailShape",
        ),
        (&["floor", "wall", "ceiling"], "AttachmentFace"),
        (&["1", "2", "3", "4", "5", "6", "7", "8"], "Level1to8"),
        (&["x", "z"], "XYAxis"),
        (&["0", "1", "2", "3", "4", "5", "6"], "Level0to6"),
        (&["1", "2", "3", "4"], "Level1to4"),
        (&["top", "bottom", "double"], "SlabType"),
        (&["none", "low", "tall"], "WallHeight"),
        (&["1", "2", "3"], "Level1to3"),
        (&["0", "1", "2"], "Level0to2"),
        (&["compare", "subtract"], "ComparatorMode"),
        (&["down", "north", "south", "west", "east"], "DirectionNoUp"),
        (&["0", "1", "2", "3", "4", "5"], "Level0to5"),
        (
            &[
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15", "16", "17", "18", "19", "20", "21", "22", "23", "24", "25",
            ],
            "Level0to25",
        ),
        (&["none", "small", "large"], "LeafSize"),
        (
            &["floor", "ceiling", "single_wall", "double_wall"],
            "BellAttachment",
        ),
        (&["save", "load", "corner", "data"], "StructureBlockMode"),
        (
            &[
                "down_east",
                "down_north",
                "down_south",
                "down_west",
                "up_east",
                "up_north",
                "up_south",
                "up_west",
                "west_up",
                "east_up",
                "north_up",
                "south_up",
            ],
            "Orientation",
        ),
        (&["0", "1", "2", "3", "4", "5", "6", "7", "8"], "Level0to8"),
        (&["inactive", "active", "cooldown"], "SculkSensorPhase"),
        (
            &["tip_merge", "tip", "frustum", "middle", "base"],
            "DripstoneThickness",
        ),
        (&["up", "down"], "VerticalDirection"),
        (&["none", "unstable", "partial", "full"], "TiltState"),
        (
            &[
                "inactive",
                "waiting_for_players",
                "active",
                "waiting_for_reward_ejection",
                "ejecting_reward",
                "cooldown",
            ],
            "TrialSpawnerState",
        ),
        (
            &["inactive", "active", "unlocking", "ejecting"],
            "VaultState",
        ),
    ];

    // Convert props to a Vec<&str> for comparison
    let props_set: Vec<&str> = props.iter().map(|s| s.as_str()).collect();

    // Find the matching enum by checking if the props match the variant set
    for (variants, enum_name) in enum_mappings {
        if props_set.len() == variants.len() && props_set.iter().all(|p| variants.contains(p)) {
            return enum_name.to_string();
        }
    }

    panic!("UnknownEnum {:?}", props);
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum NormalInvProvider {
    #[serde(rename = "minecraft:uniform")]
    Uniform(UniformIntProvider),
    // TODO: Add more...
}

impl ToTokens for NormalInvProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            NormalInvProvider::Uniform(uniform) => {
                tokens.extend(quote! {
                    NormalInvProvider::Uniform(#uniform)
                });
            }
        }
    }
}
#[derive(Deserialize, Clone, Debug)]
pub struct UniformIntProvider {
    min_inclusive: i32,
    max_inclusive: i32,
}

impl ToTokens for UniformIntProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let min_inclusive = LitInt::new(&self.min_inclusive.to_string(), Span::call_site());
        let max_inclusive = LitInt::new(&self.max_inclusive.to_string(), Span::call_site());

        tokens.extend(quote! {
            UniformIntProvider { min_inclusive: #min_inclusive, max_inclusive: #max_inclusive }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum InvProvider {
    Object(NormalInvProvider),
    Constant(i32),
}
impl ToTokens for InvProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            InvProvider::Object(inv_provider) => {
                tokens.extend(quote! {
                    InvProvider::Object(#inv_provider)
                });
            }
            InvProvider::Constant(i) => tokens.extend(quote! {
                InvProvider::Constant(#i)
            }),
        }
    }
}
#[derive(Deserialize, Clone, Debug)]
pub struct Experience {
    pub experience: InvProvider,
}

impl ToTokens for Experience {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let experience = self.experience.to_token_stream();

        tokens.extend(quote! {
            Experience { experience: #experience }
        });
    }
}
#[derive(Deserialize, Clone, Debug)]
pub struct PropertyStruct {
    pub name: String,
    pub values: Vec<String>,
}

impl ToTokens for PropertyStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = Ident::new(&self.name, Span::call_site());

        let mut prefix = "";

        if name.to_string().contains("Level") {
            prefix = "L";
        }

        let variant_count = self.values.clone().len() as u16;
        let values_index = (0..self.values.clone().len() as u16).collect::<Vec<_>>();

        let values = self.values.iter().map(|value| {
            Ident::new(
                &(prefix.to_owned() + value).to_upper_camel_case(),
                Span::call_site(),
            )
        });

        let values_2 = values.clone();
        let values_3 = values.clone();

        tokens.extend(quote! {
            #[derive(Clone, Copy)]
            pub enum #name {
                #(#values),*
            }

            impl EnumVariants for #name {
                fn variant_count() -> u16 {
                    #variant_count
                }

                fn to_index(&self) -> u16 {
                    match self {
                        #(Self::#values_2 => #values_index),*
                    }
                }

                fn from_index(index: u16) -> Self {
                    match index {
                        #(#values_index => Self::#values_3,)*
                        _ => panic!("Invalid index: {}", index),
                    }
                }
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct BlockPropertyStruct {
    pub block_name: String,
    pub entires: Vec<(String, String)>,
}

impl ToTokens for BlockPropertyStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = Ident::new(
            &(self.block_name.clone() + "_props").to_upper_camel_case(),
            Span::call_site(),
        );
        let block_name = self.block_name.clone();
        let values = self.entires.iter().map(|(key, value)| {
            let key = Ident::new_raw(&key.to_owned(), Span::call_site());
            let value = Ident::new(&value, Span::call_site());

            quote! {
                #key: #value
            }
        });

        let field_names: Vec<_> = self
            .entires
            .iter()
            .map(|(key, _)| Ident::new_raw(key, Span::call_site()))
            .collect();

        let field_types: Vec<_> = self
            .entires
            .iter()
            .map(|(_, ty)| Ident::new(ty, Span::call_site()))
            .collect();

        tokens.extend(quote! {
            pub struct #name {
                #(#values),*
            }

            impl BlockProperties for #name {
                #[allow(unused_assignments)]
                fn to_index(&self) -> u16 {
                    let mut index = 0;
                    let mut multiplier = 1;

                    #(
                        index += self.#field_names.to_index() * multiplier;
                        multiplier *= #field_types::variant_count();
                    )*

                    index
                }

                #[allow(unused_assignments)]
                fn from_index(mut index: u16) -> Self {
                    Self {
                        #(
                            #field_names: {
                                let value = index % #field_types::variant_count();
                                index /= #field_types::variant_count();
                                #field_types::from_index(value)
                            }
                        ),*
                    }
                }

                fn to_state_id(&self) -> u16 {
                    Block::from_registry_key(#block_name).unwrap().states[0].id + self.to_index()
                }

                fn from_state_id(state_id: u16) -> Option<Self> {
                    let block = Block::from_registry_key(#block_name).unwrap();
                    if state_id >= block.states[0].id && state_id <= block.states.last().unwrap().id {
                        let index = state_id - block.states[0].id;
                        Some(Self::from_index(index))
                    } else {
                        None
                    }
                }
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct CollisionShape {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

impl ToTokens for CollisionShape {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let min_x = LitFloat::new(&self.min[0].to_string(), Span::call_site());
        let min_y = LitFloat::new(&self.min[1].to_string(), Span::call_site());
        let min_z = LitFloat::new(&self.min[2].to_string(), Span::call_site());

        let max_x = LitFloat::new(&self.max[0].to_string(), Span::call_site());
        let max_y = LitFloat::new(&self.max[1].to_string(), Span::call_site());
        let max_z = LitFloat::new(&self.max[2].to_string(), Span::call_site());

        tokens.extend(quote! {
            CollisionShape {
                min: [#min_x as f64, #min_y as f64, #min_z as f64],
                max: [#max_x as f64, #max_y as f64, #max_z as f64],
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct BlockState {
    pub id: u16,
    pub air: bool,
    pub luminance: u8,
    pub burnable: bool,
    pub tool_required: bool,
    pub hardness: f32,
    pub sided_transparency: bool,
    pub replaceable: bool,
    pub collision_shapes: Vec<u16>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct BlockStateRef {
    pub id: u16,
    pub state_idx: u16,
}

impl ToTokens for BlockState {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        //let id = LitInt::new(&self.id.to_string(), Span::call_site());
        let air = LitBool::new(self.air, Span::call_site());
        let luminance = LitInt::new(&self.luminance.to_string(), Span::call_site());
        let burnable = LitBool::new(self.burnable, Span::call_site());
        let tool_required = LitBool::new(self.tool_required, Span::call_site());
        let hardness = LitFloat::new(&self.hardness.to_string(), Span::call_site());
        let sided_transparency = LitBool::new(self.sided_transparency, Span::call_site());
        let replaceable = LitBool::new(self.replaceable, Span::call_site());

        let collision_shapes = self
            .collision_shapes
            .iter()
            .map(|shape_id| LitInt::new(&shape_id.to_string(), Span::call_site()));

        tokens.extend(quote! {
            PartialBlockState {
                air: #air,
                luminance: #luminance,
                burnable: #burnable,
                tool_required: #tool_required,
                hardness: #hardness as f32,
                sided_transparency: #sided_transparency,
                replaceable: #replaceable,
                collision_shapes: &[#(#collision_shapes),*],
            }
        });
    }
}

impl ToTokens for BlockStateRef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = LitInt::new(&self.id.to_string(), Span::call_site());
        let state_idx = LitInt::new(&self.state_idx.to_string(), Span::call_site());

        tokens.extend(quote! {
            BlockStateRef {
                id: #id,
                state_idx: #state_idx,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct LootTable {
    r#type: LootTableType,
    random_sequence: Option<String>,
    pools: Option<Vec<LootPool>>,
}

impl ToTokens for LootTable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let dase = self.r#type.to_token_stream();
        let random_sequence = match &self.random_sequence {
            Some(seq) => quote! { Some(#seq) },
            None => quote! { None },
        };
        let pools = match &self.pools {
            Some(pools) => {
                let pool_tokens: Vec<_> = pools.iter().map(|pool| pool.to_token_stream()).collect();
                quote! { Some(&[#(#pool_tokens),*]) }
            }
            None => quote! { None },
        };

        tokens.extend(quote! {
            LootTable {
                r#type: #dase,
                random_sequence: #random_sequence,
                pools: #pools,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct LootPool {
    entries: Vec<LootPoolEntry>,
    rolls: f32, // TODO
    bonus_rolls: f32,
}

impl ToTokens for LootPool {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let entries_tokens: Vec<_> = self
            .entries
            .iter()
            .map(|entry| entry.to_token_stream())
            .collect();
        let rolls = LitFloat::new(&self.rolls.to_string(), Span::call_site());
        let bonus_rolls = LitFloat::new(&self.bonus_rolls.to_string(), Span::call_site());

        tokens.extend(quote! {
            LootPool {
                entries: &[#(#entries_tokens),*],
                rolls: #rolls as f32,
                bonus_rolls: #bonus_rolls as f32,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ItemEntry {
    name: String,
}

impl ToTokens for ItemEntry {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = LitStr::new(&self.name, Span::call_site());

        tokens.extend(quote! {
            ItemEntry {
                name: #name,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct AlternativeEntry {
    children: Vec<LootPoolEntry>,
}

impl ToTokens for AlternativeEntry {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let children = self.children.iter().map(|entry| entry.to_token_stream());

        tokens.extend(quote! {
            AlternativeEntry {
                children: &[#(#children),*],
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum LootPoolEntryTypes {
    #[serde(rename = "minecraft:empty")]
    Empty,
    #[serde(rename = "minecraft:item")]
    Item(ItemEntry),
    #[serde(rename = "minecraft:loot_table")]
    LootTable,
    #[serde(rename = "minecraft:dynamic")]
    Dynamic,
    #[serde(rename = "minecraft:tag")]
    Tag,
    #[serde(rename = "minecraft:alternatives")]
    Alternatives(AlternativeEntry),
    #[serde(rename = "minecraft:sequence")]
    Sequence,
    #[serde(rename = "minecraft:group")]
    Group,
}

impl ToTokens for LootPoolEntryTypes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            LootPoolEntryTypes::Empty => {
                tokens.extend(quote! { LootPoolEntryTypes::Empty });
            }
            LootPoolEntryTypes::Item(item) => {
                tokens.extend(quote! { LootPoolEntryTypes::Item(#item) });
            }
            LootPoolEntryTypes::LootTable => {
                tokens.extend(quote! { LootPoolEntryTypes::LootTable });
            }
            LootPoolEntryTypes::Dynamic => {
                tokens.extend(quote! { LootPoolEntryTypes::Dynamic });
            }
            LootPoolEntryTypes::Tag => {
                tokens.extend(quote! { LootPoolEntryTypes::Tag });
            }
            LootPoolEntryTypes::Alternatives(alt) => {
                tokens.extend(quote! { LootPoolEntryTypes::Alternatives(#alt) });
            }
            LootPoolEntryTypes::Sequence => {
                tokens.extend(quote! { LootPoolEntryTypes::Sequence });
            }
            LootPoolEntryTypes::Group => {
                tokens.extend(quote! { LootPoolEntryTypes::Group });
            }
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "condition")]
pub enum LootCondition {
    #[serde(rename = "minecraft:inverted")]
    Inverted,
    #[serde(rename = "minecraft:any_of")]
    AnyOf,
    #[serde(rename = "minecraft:all_of")]
    AllOf,
    #[serde(rename = "minecraft:random_chance")]
    RandomChance,
    #[serde(rename = "minecraft:random_chance_with_enchanted_bonus")]
    RandomChanceWithEnchantedBonus,
    #[serde(rename = "minecraft:entity_properties")]
    EntityProperties,
    #[serde(rename = "minecraft:killed_by_player")]
    KilledByPlayer,
    #[serde(rename = "minecraft:entity_scores")]
    EntityScores,
    #[serde(rename = "minecraft:block_state_property")]
    BlockStateProperty,
    #[serde(rename = "minecraft:match_tool")]
    MatchTool,
    #[serde(rename = "minecraft:table_bonus")]
    TableBonus,
    #[serde(rename = "minecraft:survives_explosion")]
    SurvivesExplosion,
    #[serde(rename = "minecraft:damage_source_properties")]
    DamageSourceProperties,
    #[serde(rename = "minecraft:location_check")]
    LocationCheck,
    #[serde(rename = "minecraft:weather_check")]
    WeatherCheck,
    #[serde(rename = "minecraft:reference")]
    Reference,
    #[serde(rename = "minecraft:time_check")]
    TimeCheck,
    #[serde(rename = "minecraft:value_check")]
    ValueCheck,
    #[serde(rename = "minecraft:enchantment_active_check")]
    EnchantmentActiveCheck,
}

impl ToTokens for LootCondition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            LootCondition::Inverted => quote! { LootCondition::Inverted },
            LootCondition::AnyOf => quote! { LootCondition::AnyOf },
            LootCondition::AllOf => quote! { LootCondition::AllOf },
            LootCondition::RandomChance => quote! { LootCondition::RandomChance },
            LootCondition::RandomChanceWithEnchantedBonus => {
                quote! { LootCondition::RandomChanceWithEnchantedBonus }
            }
            LootCondition::EntityProperties => quote! { LootCondition::EntityProperties },
            LootCondition::KilledByPlayer => quote! { LootCondition::KilledByPlayer },
            LootCondition::EntityScores => quote! { LootCondition::EntityScores },
            LootCondition::BlockStateProperty => quote! { LootCondition::BlockStateProperty },
            LootCondition::MatchTool => quote! { LootCondition::MatchTool },
            LootCondition::TableBonus => quote! { LootCondition::TableBonus },
            LootCondition::SurvivesExplosion => quote! { LootCondition::SurvivesExplosion },
            LootCondition::DamageSourceProperties => {
                quote! { LootCondition::DamageSourceProperties }
            }
            LootCondition::LocationCheck => quote! { LootCondition::LocationCheck },
            LootCondition::WeatherCheck => quote! { LootCondition::WeatherCheck },
            LootCondition::Reference => quote! { LootCondition::Reference },
            LootCondition::TimeCheck => quote! { LootCondition::TimeCheck },
            LootCondition::ValueCheck => quote! { LootCondition::ValueCheck },
            LootCondition::EnchantmentActiveCheck => {
                quote! { LootCondition::EnchantmentActiveCheck }
            }
        };

        tokens.extend(name);
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct LootPoolEntry {
    #[serde(flatten)]
    content: LootPoolEntryTypes,
    conditions: Option<Vec<LootCondition>>,
}

impl ToTokens for LootPoolEntry {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let content = &self.content;
        let conditions_tokens = match &self.conditions {
            Some(conds) => {
                let cond_tokens: Vec<_> = conds.iter().map(|c| c.to_token_stream()).collect();
                quote! { Some(&[#(#cond_tokens),*]) }
            }
            None => quote! { None },
        };

        tokens.extend(quote! {
            LootPoolEntry {
                content: #content,
                conditions: #conditions_tokens,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename = "snake_case")]
pub enum LootTableType {
    #[serde(rename = "minecraft:empty")]
    /// Nothing will be dropped
    Empty,
    #[serde(rename = "minecraft:block")]
    /// A Block will be dropped
    Block,
    #[serde(rename = "minecraft:chest")]
    /// A Item will be dropped
    Chest,
}

impl ToTokens for LootTableType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            LootTableType::Empty => quote! { LootTableType::Empty },
            LootTableType::Block => quote! { LootTableType::Block },
            LootTableType::Chest => quote! { LootTableType::Chest },
        };

        tokens.extend(name);
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Block {
    pub id: u16,
    pub name: String,
    pub translation_key: String,
    pub hardness: f32,
    pub blast_resistance: f32,
    pub item_id: u16,
    pub loot_table: Option<LootTable>,
    pub slipperiness: f32,
    pub velocity_multiplier: f32,
    pub jump_velocity_multiplier: f32,
    pub properties: Vec<BlockProperty>,
    pub default_state_id: u16,
    pub states: Vec<BlockState>,
    pub experience: Option<Experience>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct OptimizedBlock {
    pub id: u16,
    pub name: String,
    pub translation_key: String,
    pub hardness: f32,
    pub blast_resistance: f32,
    pub item_id: u16,
    pub loot_table: Option<LootTable>,
    pub slipperiness: f32,
    pub velocity_multiplier: f32,
    pub jump_velocity_multiplier: f32,
    pub default_state_id: u16,
    pub states: Vec<BlockStateRef>,
    pub experience: Option<Experience>,
}

impl ToTokens for OptimizedBlock {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = LitInt::new(&self.id.to_string(), Span::call_site());
        let name = LitStr::new(&self.name, Span::call_site());
        let translation_key = LitStr::new(&self.translation_key, Span::call_site());
        let hardness = LitFloat::new(&self.hardness.to_string(), Span::call_site());
        let blast_resistance = LitFloat::new(&self.blast_resistance.to_string(), Span::call_site());
        let item_id = LitInt::new(&self.item_id.to_string(), Span::call_site());
        let default_state_id = LitInt::new(&self.default_state_id.to_string(), Span::call_site());
        let slipperiness = LitFloat::new(&self.slipperiness.to_string(), Span::call_site());
        let velocity_multiplier =
            LitFloat::new(&self.velocity_multiplier.to_string(), Span::call_site());
        let jump_velocity_multiplier = LitFloat::new(
            &self.jump_velocity_multiplier.to_string(),
            Span::call_site(),
        );
        let experience = match &self.experience {
            Some(exp) => {
                let exp_tokens = exp.to_token_stream();
                quote! { Some(#exp_tokens) }
            }
            None => quote! { None },
        };
        // Generate state tokens
        let states = self.states.iter().map(|state| state.to_token_stream());
        let loot_table = match &self.loot_table {
            Some(table) => {
                let table_tokens = table.to_token_stream();
                quote! { Some(#table_tokens) }
            }
            None => quote! { None },
        };

        tokens.extend(quote! {
            Block {
                id: #id,
                name: #name,
                translation_key: #translation_key,
                hardness: #hardness as f32,
                blast_resistance: #blast_resistance as f32,
                slipperiness: #slipperiness as f32,
                velocity_multiplier: #velocity_multiplier as f32,
                jump_velocity_multiplier: #jump_velocity_multiplier as f32,
                item_id: #item_id,
                default_state_id: #default_state_id,
                states: &[#(#states),*],
                loot_table: #loot_table,
                experience: #experience,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct BlockAssets {
    pub blocks: Vec<Block>,
    pub shapes: Vec<CollisionShape>,
    pub block_entity_types: Vec<String>,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/blocks.json");

    let blocks_assets: BlockAssets = serde_json::from_str(include_str!("../../assets/blocks.json"))
        .expect("Failed to parse blocks.json");

    let mut type_from_raw_id_arms = TokenStream::new();
    let mut type_from_name = TokenStream::new();
    let mut block_from_state_id = TokenStream::new();
    let mut block_from_item_id = TokenStream::new();
    let mut constants = TokenStream::new();

    let mut unique_states = Vec::new();
    for block in blocks_assets.blocks.clone() {
        for state in block.states.clone() {
            // Check if this state is already in unique_states by comparing all fields except id
            let already_exists = unique_states.iter().any(|s: &BlockState| {
                s.air == state.air
                    && s.luminance == state.luminance
                    && s.burnable == state.burnable
                    && s.tool_required == state.tool_required
                    && s.hardness == state.hardness
                    && s.sided_transparency == state.sided_transparency
                    && s.replaceable == state.replaceable
                    && s.collision_shapes == state.collision_shapes
            });

            if !already_exists {
                unique_states.push(state);
            }
        }
    }

    // Create a map of block name to Block for easier access
    let mut blocks_map: HashMap<String, OptimizedBlock> = HashMap::new();
    let mut block_props: Vec<BlockPropertyStruct> = Vec::new();
    let mut properties: Vec<PropertyStruct> = Vec::new();
    for block in blocks_assets.blocks.clone() {
        blocks_map.insert(
            block.name.clone(),
            OptimizedBlock {
                id: block.id,
                name: block.name.clone(),
                translation_key: block.translation_key.clone(),
                hardness: block.hardness,
                blast_resistance: block.blast_resistance,
                item_id: block.item_id,
                default_state_id: block.default_state_id,
                slipperiness: block.slipperiness,
                velocity_multiplier: block.velocity_multiplier,
                jump_velocity_multiplier: block.jump_velocity_multiplier,
                loot_table: block.loot_table,
                experience: block.experience,
                states: block
                    .states
                    .iter()
                    .map(|state| {
                        // Find the index in unique_states by comparing all fields except id
                        let state_idx = unique_states
                            .iter()
                            .position(|s| {
                                s.air == state.air
                                    && s.luminance == state.luminance
                                    && s.burnable == state.burnable
                                    && s.tool_required == state.tool_required
                                    && s.hardness == state.hardness
                                    && s.sided_transparency == state.sided_transparency
                                    && s.replaceable == state.replaceable
                                    && s.collision_shapes == state.collision_shapes
                            })
                            .unwrap() as u16;

                        BlockStateRef {
                            id: state.id,
                            state_idx,
                        }
                    })
                    .collect(),
            },
        );
        let mut props: HashMap<String, Vec<String>> = HashMap::new();
        for prop in block.properties.clone() {
            props.insert(prop.name.clone(), prop.values.clone());
        }
        if props.len() != 0 {
            block_props.push(BlockPropertyStruct {
                block_name: block.name.clone(),
                entires: props
                    .into_iter()
                    .map(|(key, values)| (key, get_enum_name(values)))
                    .collect(),
            });
        }
        for prop in block.properties.clone() {
            if !properties
                .iter()
                .any(|p| p.name == get_enum_name(prop.values.clone()))
            {
                properties.push(PropertyStruct {
                    name: get_enum_name(prop.values.clone()),
                    values: prop.values.clone(),
                });
            }
        }
    }

    // Generate collision shapes array
    let shapes = blocks_assets
        .shapes
        .iter()
        .map(|shape| shape.to_token_stream());

    let unique_states = unique_states.iter().map(|state| state.to_token_stream());

    let block_props = block_props.iter().map(|prop| prop.to_token_stream());
    let properties = properties.iter().map(|prop| prop.to_token_stream());

    // Generate block entity types array
    let block_entity_types = blocks_assets
        .block_entity_types
        .iter()
        .map(|entity_type| LitStr::new(entity_type, Span::call_site()));

    // Generate constants and match arms for each block
    for (name, block) in blocks_map {
        let const_ident = format_ident!("{}", name.to_shouty_snake_case());
        let block_tokens = block.to_token_stream();
        let id_lit = LitInt::new(&block.id.to_string(), Span::call_site());
        let state_start = block.states.iter().map(|state| state.id).min().unwrap();
        let state_end = block.states.iter().map(|state| state.id).max().unwrap();
        let item_id = block.item_id;

        constants.extend(quote! {
            pub const #const_ident: Block = #block_tokens;
        });

        type_from_raw_id_arms.extend(quote! {
            #id_lit => Some(Self::#const_ident),
        });

        type_from_name.extend(quote! {
            #name => Some(Self::#const_ident),
        });

        block_from_state_id.extend(quote! {
            #state_start..=#state_end => Some(Self::#const_ident),
        });

        if item_id != 0 {
            block_from_item_id.extend(quote! {
                #item_id => Some(Self::#const_ident),
            });
        }
    }

    quote! {
        use crate::tag::{Tagable, RegistryKey};
        use pumpkin_util::math::int_provider::{UniformIntProvider, InvProvider, NormalInvProvider};



        #[derive(Clone, Debug)]
        pub struct Experience {
            pub experience: InvProvider,
        }

        #[derive(Clone, Debug)]
        pub struct PartialBlockState {
            pub air: bool,
            pub luminance: u8,
            pub burnable: bool,
            pub tool_required: bool,
            pub hardness: f32,
            pub sided_transparency: bool,
            pub replaceable: bool,
            pub collision_shapes: &'static [u16],
        }

        #[derive(Clone, Debug)]
        pub struct BlockState {
            pub id: u16,
            pub air: bool,
            pub luminance: u8,
            pub burnable: bool,
            pub tool_required: bool,
            pub hardness: f32,
            pub sided_transparency: bool,
            pub replaceable: bool,
            pub collision_shapes: &'static [u16],
        }

        #[derive(Clone, Debug)]
        pub struct BlockStateRef {
            pub id: u16,
            pub state_idx: u16,
        }

        #[allow(dead_code)]
        #[derive(Clone, Debug)]
        pub struct LootTable {
            r#type: LootTableType,
            random_sequence: Option<&'static str>,
            pools: Option<&'static [LootPool]>,
        }

        #[allow(dead_code)]
        #[derive(Clone, Debug)]
        pub struct LootPool {
            entries: &'static [LootPoolEntry],
            rolls: f32, // TODO
            bonus_rolls: f32,
        }

        #[allow(dead_code)]
        #[derive(Clone, Debug)]
        pub struct ItemEntry {
            name: &'static str,
        }

        #[allow(dead_code)]
        #[derive(Clone, Debug)]
        pub struct AlternativeEntry {
            children: &'static [LootPoolEntry],
        }

        #[allow(dead_code)]
        #[derive(Clone, Debug)]
        pub enum LootPoolEntryTypes {
            Empty,
            Item(ItemEntry),
            LootTable,
            Dynamic,
            Tag,
            Alternatives(AlternativeEntry),
            Sequence,
            Group,
        }

        #[allow(dead_code)]
        #[derive(Clone, Debug)]
        pub enum LootCondition {
            Inverted,
            AnyOf,
            AllOf,
            RandomChance,
            RandomChanceWithEnchantedBonus,
            EntityProperties,
            KilledByPlayer,
            EntityScores,
            BlockStateProperty,
            MatchTool,
            TableBonus,
            SurvivesExplosion,
            DamageSourceProperties,
            LocationCheck,
            WeatherCheck,
            Reference,
            TimeCheck,
            ValueCheck,
            EnchantmentActiveCheck,
        }

        #[allow(dead_code)]
        #[derive(Clone, Debug)]
        pub struct LootPoolEntry {
            content: LootPoolEntryTypes,
            conditions: Option<&'static [LootCondition]>,
        }

        #[allow(dead_code)]
        #[derive(Clone, Debug)]
        pub enum LootTableType {
            /// Nothing will be dropped
            Empty,
            /// A Block will be dropped
            Block,
            /// A Item will be dropped
            Chest,
        }

        #[derive(Clone, Debug)]
        pub struct Block {
            pub id: u16,
            pub name: &'static str,
            pub translation_key: &'static str,
            pub hardness: f32,
            pub blast_resistance: f32,
            pub slipperiness: f32,
            pub velocity_multiplier: f32,
            pub jump_velocity_multiplier: f32,
            pub item_id: u16,
            pub default_state_id: u16,
            pub states: &'static [BlockStateRef],
            pub loot_table: Option<LootTable>,
            pub experience: Option<Experience>,
        }

        impl PartialEq for Block {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct BlockProperty {
            pub name: &'static str,
            pub values: &'static [&'static str],
        }

        #[derive(Clone, Copy, Debug)]
        pub struct CollisionShape {
            pub min: [f64; 3],
            pub max: [f64; 3],
        }

        #[derive(Clone, Copy, Debug)]
        pub struct BlockStateData {
            pub air: bool,
            pub luminance: u8,
            pub burnable: bool,
            pub tool_required: bool,
            pub hardness: f32,
            pub sided_transparency: bool,
            pub replaceable: bool,
            pub collision_shapes: &'static [u16],
        }


        pub trait BlockProperties {
            // Convert properties to an index (0 to N-1)
            fn to_index(&self) -> u16;
            // Convert an index back to properties
            fn from_index(index: u16) -> Self;

            // Convert properties to a state id
            fn to_state_id(&self) -> u16;
            // Convert a state id back to properties
            fn from_state_id(state_id: u16) -> Option<Self> where Self: Sized;
        }

        pub trait EnumVariants {
            fn variant_count() -> u16;
            fn to_index(&self) -> u16;
            fn from_index(index: u16) -> Self;
        }



        pub static COLLISION_SHAPES: &[CollisionShape] = &[
            #(#shapes),*
        ];

        pub static BLOCK_STATES: &[PartialBlockState] = &[
            #(#unique_states),*
        ];

        pub static BLOCK_ENTITY_TYPES: &[&str] = &[
            #(#block_entity_types),*
        ];



        impl Block {
            #constants

            #[doc = r" Try to parse a Block from a resource location string"]
            pub fn from_registry_key(name: &str) -> Option<Self> {
                match name {
                    #type_from_name
                    _ => None
                }
            }

            #[doc = r" Try to parse a Block from a raw id"]
            pub const fn from_id(id: u16) -> Option<Self> {
                match id {
                    #type_from_raw_id_arms
                    _ => None
                }
            }

            #[doc = r" Try to parse a Block from a state id"]
            pub const fn from_state_id(id: u16) -> Option<Self> {
                match id {
                    #block_from_state_id
                    _ => None
                }
            }

            #[doc = r" Try to parse a Block from an item id"]
            pub const fn from_item_id(id: u16) -> Option<Self> {
                #[allow(unreachable_patterns)]
                match id {
                    #block_from_item_id
                    _ => None
                }
            }
        }

        #(#properties)*

        #(#block_props)*

        impl BlockStateRef {
            pub fn get_state(&self) -> BlockState {
                let partial_state = &BLOCK_STATES[self.state_idx as usize];
                BlockState {
                    id: self.id,
                    air: partial_state.air,
                    luminance: partial_state.luminance,
                    burnable: partial_state.burnable,
                    tool_required: partial_state.tool_required,
                    hardness: partial_state.hardness,
                    sided_transparency: partial_state.sided_transparency,
                    replaceable: partial_state.replaceable,
                    collision_shapes: partial_state.collision_shapes,
                }
            }
        }

        impl Tagable for Block {
            #[inline]
            fn tag_key() -> RegistryKey {
                RegistryKey::Block
            }

            #[inline]
            fn registry_key(&self) -> &str {
                self.name
            }
        }
    }
}
