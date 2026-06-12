use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::TextContent::Translate;
use quote::{ToTokens, format_ident, quote};
use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use std::cmp::PartialEq;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::{collections::BTreeMap, fs};

/// helper default used by serde for fields that should be `true` when omitted.
const fn default_true() -> bool {
    true
}

///the structure that contains the display information of an advancement
#[derive(Deserialize, Clone)]
pub struct AdvancementDisplay {
    pub title: TextComponent,
    pub description: TextComponent,
    #[serde(rename = "icon", deserialize_with = "deserialize_icon_id")]
    pub item_icon: ResourceLocation,
    #[serde(default, rename = "frame")]
    pub frame_type: FrameTypeStruct,
    #[serde(default, rename = "background")]
    pub background_texture: Option<ResourceLocation>,
    #[serde(default = "default_true")]
    pub show_toast: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default = "default_true")]
    pub announce_to_chat: bool,
    #[serde(skip)]
    pub x: f32,
    #[serde(skip)]
    pub y: f32,
}

fn as_translate(text: &TextComponent) -> TokenStream {
    let Translate {
        translate,
        bedrock_translate: _,
        with: _,
    } = text.0.content.as_ref()
    else {
        panic!("expected a translatable text component for advancement display")
    };
    quote! { #translate }
}

fn token_option<D>(option: &Option<D>) -> TokenStream
where
    D: ToTokens,
{
    match option {
        Some(x) => quote! { Some(#x) },
        None => quote! { None },
    }
}

impl ToTokens for AdvancementDisplay {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let item_icon = format_ident!(
            "{}",
            self.item_icon
                .strip_prefix("minecraft:")
                .unwrap_or_else(|| {
                    panic!(
                        "expected a vanilla Minecraft item icon, got `{}`",
                        self.item_icon
                    )
                })
                .to_uppercase()
        );
        let frame_type = &self.frame_type;
        let announce_to_chat = &self.announce_to_chat;
        let show_toast = &self.show_toast;
        let hidden = &self.hidden;
        let background_texture = token_option(&self.background_texture);
        let title = as_translate(&self.title);
        let description = as_translate(&self.description);
        let x = self.x;
        let y = self.y;
        tokens.extend(quote! {
            AdvancementDisplay::new(#title,
                #description,
                ItemStack::static_new_java(1,&Item::#item_icon),
                #frame_type,
                #background_texture,
                #show_toast,
                #hidden,
                #announce_to_chat,
                #x,
                #y,
            )
        });
    }
}

///store which type of frame should be use when display
#[derive(Clone, Copy, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FrameTypeStruct {
    #[default]
    Task = 0,
    Challenge = 1,
    Goal = 2,
}

impl ToTokens for FrameTypeStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let t = match self {
            FrameTypeStruct::Task => quote! { FrameType::Task },
            FrameTypeStruct::Challenge => quote! { FrameType::Challenge },
            FrameTypeStruct::Goal => quote! { FrameType::Goal },
        };
        tokens.extend(t);
    }
}

///what it gives you when you complete an advancement
#[derive(Deserialize, Default, Clone)]
pub struct AdvancementRewards {
    #[serde(default)]
    experience: i32,
    //loot: Vec<ResourceLocation> TODO,
    #[serde(default)]
    recipes: Vec<ResourceLocation>,
    //functions: Option<Function> TODO
}

impl ToTokens for AdvancementRewards {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let experience = self.experience;
        let recipes = self.recipes.iter().map(|_recipe| {
            quote! {
                //TODO implement recipe reward
                //Recipe::from_id(#recipe)
            }
        });
        tokens.extend(quote! {
            AdvancementReward {
                experience: #experience,
                recipes: &[#(#recipes),*],
            }
        })
    }
}

/// represent a node in the advancement tree
pub struct AdvancementNode {
    pub children: Vec<usize>,
    pub parent: Option<usize>,
    pub value: AdvancementHolder,
}

impl AdvancementNode {
    #[inline]
    pub fn add_child(&mut self, child: usize) {
        self.children.push(child);
    }

    #[must_use]
    pub fn new(value: AdvancementHolder, parent: Option<usize>) -> Self {
        Self {
            value,
            parent,
            children: Vec::new(),
        }
    }

    #[inline]
    #[must_use]
    pub const fn has_display(&self) -> bool {
        self.value.1.display.is_some()
    }

    #[inline]
    pub const fn set_location(&mut self, x: f32, y: f32) {
        if let Some(val) = self.value.1.display.as_mut() {
            val.x = x;
            val.y = y;
        };
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
        write!(f, "{}", self.value.0)
    }
}
impl Hash for AdvancementNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl ToTokens for AdvancementNode {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let parent = token_option(&self.parent);
        let children = &self.children;
        let value = &self.value;
        tokens.extend(quote! {
            AdvancementNode{
                parent:#parent,
                children: vec![#(#children),*],
                value: #value,
            }
        })
    }
}

///the structure that represent a advancement
#[derive(Deserialize, Default, Clone)]
pub struct AdvancementStruct {
    pub parent: Option<Identifier>,
    #[serde(default)]
    pub display: Option<AdvancementDisplay>,
    //pub criteria : Vec<Criterion>,
    #[serde(default)]
    pub rewards: AdvancementRewards,
    #[serde(default, rename = "sends_telemetry_event")]
    pub sends_telemetry: bool,
    pub requirements: Vec<Vec<String>>,
}
#[derive(Clone)]
pub struct AdvancementHolder(Identifier, AdvancementStruct);

impl PartialEq for AdvancementHolder {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}
impl Eq for AdvancementHolder {}

impl Hash for AdvancementHolder {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl ToTokens for AdvancementHolder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = format_ident!("{}", self.0.path().to_shouty_snake_case());
        tokens.extend(quote! {
            Advancement::#name
        })
    }
}

///the item use for the icon of the display
///
///(doesn't support custom items has the vanilla advancement does not use custom items)
#[derive(Deserialize)]
struct DisplayIcon {
    id: ResourceLocation,
}

fn deserialize_icon_id<'de, D>(deserializer: D) -> Result<ResourceLocation, D::Error>
where
    D: Deserializer<'de>,
{
    let icon = DisplayIcon::deserialize(deserializer)?;
    Ok(icon.id)
}

/// represent the structure that is used to store the different node and linking her id to it's corresponding node.
#[derive(Default)]
pub struct AdvancementTree {
    pub nodes: BTreeMap<Identifier, usize>,
    pub nodes_vector: Vec<AdvancementNode>,
    pub roots: Vec<usize>,
    pub tasks: Vec<usize>,
}

impl AdvancementTree {
    ///Iterate over all advancements until every advancement that can be inserted has been inserted.
    ///
    ///see [`AdvancementTree::try_insert`]
    pub fn add_all(&mut self, advancements: Vec<AdvancementHolder>) {
        let mut advancements_to_add: Vec<AdvancementHolder> = advancements;

        while !advancements_to_add.is_empty() {
            let len_before = advancements_to_add.len();

            advancements_to_add = advancements_to_add
                .into_iter()
                .filter_map(|advancement| self.try_insert(advancement))
                .collect();

            if advancements_to_add.len() == len_before && !advancements_to_add.is_empty() {
                eprintln!(
                    "Couldn't load advancements: {:?}",
                    advancements_to_add.iter().map(|a| &a.0).collect::<Vec<_>>()
                );
                break;
            }
        }
    }

    ///try to insert the advancement in the tree if it has a parent and his has not yet been register fail
    /// and return the owned AdvancementHolder
    pub fn try_insert(&mut self, advancement: AdvancementHolder) -> Option<AdvancementHolder> {
        let parent_id = &advancement.1.parent;
        let parent_idx: Option<usize> = match parent_id {
            Some(id) => match self.nodes.get(id) {
                Some(node) => Some(*node),
                None => return Some(advancement),
            },
            None => None,
        };
        let id = advancement.0.clone();
        let node = AdvancementNode::new(advancement, parent_idx);
        let node_idx = self.nodes_vector.len();
        self.nodes.insert(id, node_idx);
        if let Some(parent) = parent_idx {
            let parent_node = self.nodes_vector.get_mut(parent).unwrap();
            parent_node.add_child(node_idx);
            self.tasks.push(node_idx);
        } else {
            self.roots.push(node_idx);
        }
        self.nodes_vector.push(node);
        None
    }
}

impl ToTokens for AdvancementTree {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let nodes = self.nodes.iter().map(|(k, v)| {
            let key = identifier_to_tokens(k);
            quote! {
                nodes.insert(#key, #v);
            }
        });
        let nodes_vector = &self.nodes_vector;
        let roots = &self.roots;
        let tasks = &self.tasks;
        tokens.extend(quote! {
            LazyLock::new(|| {
                let mut nodes = BTreeMap::new();
                #(#nodes)*
                let nodes_vector = vec![#(#nodes_vector),*];
                let roots = vec![#(#roots),*];
                let tasks = vec![#(#tasks),*];
                AdvancementTree {
                    nodes,
                    nodes_vector,
                    roots,
                    tasks,
                }
            })
        })
    }
}

///Convert a identifier to its token form
fn identifier_to_tokens(identifier: &Identifier) -> TokenStream {
    let namespace = identifier.namespace();
    let path = identifier.path();
    quote! {
        Identifier::from_static(#namespace, #path)
    }
}

pub(crate) fn build() -> TokenStream {
    let advancements_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/advancements.json");
    let advancements_json = fs::read_to_string(&advancements_path)
        .unwrap_or_else(|err| panic!("Failed to read {}: {err}", advancements_path.display()));
    let advancements: BTreeMap<String, AdvancementStruct> =
        serde_json::from_str(&advancements_json)
            .unwrap_or_else(|err| panic!("Failed to parse {}: {err}", advancements_path.display()));

    let mut variants = TokenStream::new();
    let mut name_to_type = TokenStream::new();
    let mut minecraft_name_to_type = TokenStream::new();
    let mut minecraft_namespaces = TokenStream::new();
    let capacity = advancements.len();
    //construct the tree
    let mut tree = AdvancementTree::default();
    tree.add_all(
        advancements
            .into_iter()
            .map(|(key, value)| AdvancementHolder(Identifier::parse(&key).unwrap(), value))
            .collect(),
    );
    let advancement_tree = quote! {
        pub static ADVANCEMENT_TREE : LazyLock<AdvancementTree> = #tree;
    };
    let advancements_holder: Vec<AdvancementHolder> = tree
        .nodes_vector
        .into_iter()
        .map(|node| node.value)
        .collect();
    for AdvancementHolder(identifier, advancement) in advancements_holder {
        let raw_name = identifier.path();
        let format_name = format_ident!("{}", raw_name.to_shouty_snake_case());

        let parent = if let Some(identifier) = &advancement.parent {
            let parent = identifier_to_tokens(identifier);
            quote! {Some(#parent)}
        } else {
            quote! { None }
        };
        let send_telemetry = advancement.sends_telemetry;
        let display = match &advancement.display {
            Some(display) => quote! { Some(&#display) },
            None => quote! { None },
        };
        let reward = advancement.rewards;
        let requirements = advancement.requirements.iter().map(|inner_req| {
            quote! { &[#(#inner_req),*]}
        });
        variants.extend([quote! {
            pub const #format_name: &Self = &Self {
                id: Identifier::vanilla_static(#raw_name),
                parent : #parent,
                send_telemetry : #send_telemetry,
                display : #display,
                reward : &#reward,
                requirements: AdvancementRequirement{
                    requirements : &[#(#requirements),*]
                }
            };
        }]);
        let minecraft_name = identifier.to_string();

        name_to_type.extend(quote! { #raw_name => Some(Self::#format_name), });
        minecraft_name_to_type.extend(quote! { #minecraft_name => Some(Self::#format_name), });
        minecraft_namespaces.extend(quote! { Identifier::vanilla_static(#raw_name),})
    }

    quote! {
        use pumpkin_util::text::TextComponent;
        use crate::item_stack::ItemStack;
        use crate::item::Item;
        use crate::advancement_data::*;
        use std::sync::LazyLock;
        use pumpkin_util::identifier::Identifier;
        use pumpkin_util::text::{color::NamedColor,
            style::Style,
            hover::HoverEvent,
            color::Color};
        use std::hash::{Hash,Hasher};
        use std::fmt::Display;
        use std::collections::BTreeMap;

        pub struct Advancement {
            pub id : Identifier,
            pub parent : Option<Identifier>,
            pub send_telemetry : bool,
            pub display : Option<&'static AdvancementDisplay>,
            pub reward : &'static AdvancementReward,
            pub requirements: AdvancementRequirement,
        }

        impl Display for Advancement {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", self.id)
            }
        }

        impl Hash for Advancement {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.id.hash(state);
            }
        }

        impl PartialEq<Self> for Advancement {
            fn eq(&self, other: &Self) -> bool {
                other.id == self.id
            }
        }

        impl Eq for Advancement {}

        impl Advancement {
            #variants

            pub fn option_name(&self) -> Option<TextComponent> {
                match self.display {
                    Some(display) => {
                        let mut over = display.get_title();
                        let color = Color::Named(display.frame_type.get_color());
                        *over.0.style = Style::default().color(color);
                        over = over.add_text("\n").add_child(display.get_description());
                        let mut text = display.get_title();
                        text.0.style.hover_event = Some(HoverEvent::show_text(over));
                        Some(text.wrap_in_square_brackets().color(color))
                    }
                    None => None
                }
            }

            pub fn name(&self) -> TextComponent {
                self.option_name().unwrap_or(TextComponent::text(self.id.to_string()))
            }

            pub fn from_name(name: &str) -> Option<&'static Self> {
                    match name {
                        #name_to_type
                        _ => None
                    }
                }


            pub fn from_minecraft_name(name: &str) -> Option<&'static Self> {
                match name {
                    #minecraft_name_to_type
                    _ => None
                }
            }

            pub const fn get_list() -> [Identifier;#capacity] {
                [#minecraft_namespaces]
            }

            pub const fn is_root(&self) -> bool{
                self.parent.is_none()
            }

        }
        #advancement_tree
    }
}
