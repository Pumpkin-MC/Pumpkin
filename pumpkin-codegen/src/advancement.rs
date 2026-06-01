use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::TextContent::Translate;
use quote::{format_ident, quote, ToTokens};
use serde::{Deserialize, Deserializer, Serialize};
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::{collections::BTreeMap, fs};

const fn r#true() -> bool {
    true
}

#[derive(Deserialize)]
pub struct AdvancementDisplay {
    pub title: TextComponent,
    pub description: TextComponent,
    #[serde(rename = "icon", deserialize_with = "deserialize_icon_id")]
    pub item_icon: ResourceLocation,
    #[serde(default, rename = "frame")]
    pub frame_type: FrameTypeStruct,
    #[serde(default, rename = "background")]
    pub background_texture: Option<ResourceLocation>,
    #[serde(default = "r#true")]
    pub show_toast: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default = "r#true")]
    pub announce_to_chat: bool,
    #[serde(skip)]
    pub x:f32,
    #[serde(skip)]
    pub y:f32,
}

fn as_translate(text: &TextComponent) -> TokenStream {
    let Translate { translate, bedrock_translate : _ , with: _ } = text.0.content.as_ref() else {
        panic!()
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
                .unwrap()
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
                ItemStack::new(1,&Item::#item_icon),
                #frame_type,
                #background_texture,
                #show_toast,
                #hidden,
                #announce_to_chat,
                #x,
                #y
            )
        });
    }
}

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

#[derive(Deserialize, Default)]
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
        let recipes = self.recipes.iter().map(|recipe| {
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

pub struct AdvancementNode {
    pub children: Vec<usize>,
    pub parent: Option<usize>,
    pub value: AdvancementHolder,
}

impl AdvancementNode {
    pub fn add_child(&mut self, child: usize) {
        self.children.push(child);
    }

    #[must_use]
    pub fn new(value:AdvancementHolder, parent: Option<usize>) -> Self {
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
    pub const fn set_location(&mut self,x:f32,y:f32) {
        if let Some(val) = self.value.1.display.as_mut() {
            val.x=x;
            val.y=y;
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

struct LayoutNode {
    node: usize,
    parent: Option<usize>,
    previous_sibling: Option<usize>,
    child_index: usize,
    children: Vec<usize>,
    ancestor: usize,
    thread: Option<usize>,
    x: i32,
    y: f32,
    mod_field: f32,
    change: f32,
    shift: f32,
}

pub struct TreePositioner;

impl TreePositioner {
    pub fn run(tree:&mut AdvancementTree ,root_index: usize) {
        let root_node = tree.nodes_vector[root_index];
        if !root_node.has_display() {
            panic!("Can't position children of an invisible root!");
        }
        let mut nodes: Vec<LayoutNode> = Vec::with_capacity(32);

        let root_idx = nodes.len();
        nodes.push(LayoutNode {
            node: root_index,
            parent: None,
            previous_sibling: None,
            child_index: 1,
            children: Vec::new(),
            ancestor: root_idx,
            thread: None,
            x: 0,
            y: -1.0,
            mod_field: 0.0,
            change: 0.0,
            shift: 0.0,
        });

        let mut previous_idx = None;
        for child in root_node.children {
            previous_idx = Self::add_child(&mut nodes,tree, root_idx, child, previous_idx);
        }

        Self::first_walk(&mut nodes, root_idx);

        let root_y = nodes[root_idx].y;
        let min = Self::second_walk(&mut nodes, root_idx, 0.0, 0, root_y);

        if min < 0.0 {
            Self::third_walk(&mut nodes, root_idx, -min);
        }

        Self::finalize_position(tree,&nodes, root_idx);
    }

    fn add_child(
        nodes: &mut Vec<LayoutNode>,
        tree: &mut AdvancementTree,
        parent_idx: usize,
        adv_node_idx: usize,
        mut previous_idx: Option<usize>,
    ) -> Option<usize> {
        let adv_node = tree.nodes_vector[adv_node_idx];
        if adv_node.has_display() {
            let child_idx = nodes.len();
            let next_child_index = nodes[parent_idx].children.len() + 1;
            let depth = nodes[parent_idx].x + 1;

            nodes.push(LayoutNode {
                node: adv_node_idx,
                parent: Some(parent_idx),
                previous_sibling: previous_idx,
                child_index: next_child_index,
                children: Vec::new(),
                ancestor: child_idx,
                thread: None,
                x: depth,
                y: -1.0,
                mod_field: 0.0,
                change: 0.0,
                shift: 0.0,
            });

            nodes[parent_idx].children.push(child_idx);

            let mut child_prev = None;
            for child in adv_node.children {
                child_prev = Self::add_child(nodes,tree, child_idx, child, child_prev);
            }

            Some(child_idx)
        } else {
            for grandchild in adv_node.children {
                previous_idx = Self::add_child(nodes,tree, parent_idx, grandchild, previous_idx);
            }
            previous_idx
        }
    }

    fn first_walk(nodes: &mut Vec<LayoutNode>, idx: usize) {
        let num_children = nodes[idx].children.len();

        if num_children == 0 {
            if let Some(prev_sib) = nodes[idx].previous_sibling {
                nodes[idx].y = nodes[prev_sib].y + 1.0;
            } else {
                nodes[idx].y = 0.0;
            }
        } else {
            let mut default_ancestor = None;
            for i in 0..num_children {
                let child_idx = nodes[idx].children[i];
                Self::first_walk(nodes, child_idx);
                let arg_ancestor = default_ancestor.unwrap_or(child_idx);
                default_ancestor = Some(Self::apportion(nodes, child_idx, arg_ancestor));
            }

            Self::execute_shifts(nodes, idx);

            let first_child_idx = nodes[idx].children[0];
            let last_child_idx = nodes[idx].children[num_children - 1];
            let midpoint = (nodes[first_child_idx].y + nodes[last_child_idx].y) / 2.0;

            if let Some(prev_sib) = nodes[idx].previous_sibling {
                nodes[idx].y = nodes[prev_sib].y + 1.0;
                nodes[idx].mod_field = nodes[idx].y - midpoint;
            } else {
                nodes[idx].y = midpoint;
            }
        }
    }

    fn second_walk<>(
        nodes: &mut Vec<LayoutNode>,
        idx: usize,
        mod_sum: f32,
        depth: i32,
        mut min: f32,
    ) -> f32 {
        nodes[idx].y += mod_sum;
        nodes[idx].x = depth;

        if nodes[idx].y < min {
            min = nodes[idx].y;
        }

        let num_children = nodes[idx].children.len();
        let current_mod = nodes[idx].mod_field;

        for i in 0..num_children {
            let child_idx = nodes[idx].children[i];
            min = Self::second_walk(nodes, child_idx, mod_sum + current_mod, depth + 1, min);
        }

        min
    }

    fn third_walk(nodes: &mut Vec<LayoutNode>, idx: usize, offset: f32) {
        nodes[idx].y += offset;

        let num_children = nodes[idx].children.len();
        for i in 0..num_children {
            let child_idx = nodes[idx].children[i];
            Self::third_walk(nodes, child_idx, offset);
        }
    }

    fn execute_shifts(nodes: &mut [LayoutNode], idx: usize) {
        let mut shift = 0.0;
        let mut change = 0.0;

        for &child_idx in nodes[idx].children.iter().rev() {
            nodes[child_idx].y += shift;
            nodes[child_idx].mod_field += shift;
            change += nodes[child_idx].change;
            shift += nodes[child_idx].shift + change;
        }
    }

    #[inline]
    fn previous_or_thread(nodes: &[LayoutNode], idx: usize) -> Option<usize> {
        nodes[idx].thread.or_else(|| nodes[idx].children.first().copied())
    }

    #[inline]
    fn next_or_thread(nodes: &[LayoutNode], idx: usize) -> Option<usize> {
        nodes[idx].thread.or_else(|| nodes[idx].children.last().copied())
    }

    fn apportion(
        nodes: &mut [LayoutNode],
        idx: usize,
        mut default_ancestor: usize,
    ) -> usize {
        let prev_sib = match nodes[idx].previous_sibling {
            Some(p) => p,
            None => return default_ancestor,
        };

        let mut vir = idx;
        let mut vor = idx;
        let mut vil = prev_sib;

        let parent_idx = nodes[idx].parent.expect("Tree invariant broken: no parent");
        let mut vol = nodes[parent_idx].children[0];

        let mut sir = nodes[vir].mod_field;
        let mut sor = nodes[vor].mod_field;
        let mut sil = nodes[vil].mod_field;
        let mut sol = nodes[vol].mod_field;

        while let (Some(next_vil), Some(next_vir)) = (Self::next_or_thread(nodes, vil), Self::previous_or_thread(nodes, vir)) {
            vil = next_vil;
            vir = next_vir;
            vol = Self::previous_or_thread(nodes, vol).expect("Tree invariant broken");
            vor = Self::next_or_thread(nodes, vor).expect("Tree invariant broken");

            nodes[vor].ancestor = idx;

            let shift = (nodes[vil].y + sil) - (nodes[vir].y + sir) + 1.0;
            if shift > 0.0 {
                let ancestor_idx = Self::get_ancestor(nodes, vil, idx, default_ancestor);
                Self::move_subtree(nodes, ancestor_idx, idx, shift);
                sir += shift;
                sor += shift;
            }

            sil += nodes[vil].mod_field;
            sir += nodes[vir].mod_field;
            sol += nodes[vol].mod_field;
            sor += nodes[vor].mod_field;
        }

        if Self::next_or_thread(nodes, vil).is_some() && Self::next_or_thread(nodes, vor).is_none() {
            nodes[vor].thread = Self::next_or_thread(nodes, vil);
            nodes[vor].mod_field += sil - sor;
        } else {
            if Self::previous_or_thread(nodes, vir).is_some() && Self::previous_or_thread(nodes, vol).is_none() {
                nodes[vol].thread = Self::previous_or_thread(nodes, vir);
                nodes[vol].mod_field += sir - sol;
            }
            default_ancestor = idx;
        }

        default_ancestor
    }

    fn move_subtree(nodes: &mut [LayoutNode], left: usize, right: usize, shift: f32) {
        let subtrees = (nodes[right].child_index as f32) - (nodes[left].child_index as f32);
        if subtrees != 0.0 {
            nodes[right].change -= shift / subtrees;
            nodes[left].change += shift / subtrees;
        }
        nodes[right].shift += shift;
        nodes[right].y += shift;
        nodes[right].mod_field += shift;
    }

    fn get_ancestor(
        nodes: &[LayoutNode],
        vil: usize,
        idx: usize,
        default_ancestor: usize,
    ) -> usize {
        let ancestor = nodes[vil].ancestor;
        let parent_idx = nodes[idx].parent.unwrap();

        if nodes[parent_idx].children.contains(&ancestor) {
            ancestor
        } else {
            default_ancestor
        }
    }

    fn finalize_position(tree:&mut AdvancementTree, nodes: &[LayoutNode], idx: usize) {
        tree.nodes_vector[nodes[idx].node].set_location(nodes[idx].x as f32, nodes[idx].y);
        for &child_idx in &nodes[idx].children {
            Self::finalize_position(tree,nodes, child_idx);
        }
    }
}

#[derive(Deserialize, Default)]
pub struct AdvancementStruct {
    pub parent: Option<Identifier>,
    #[serde(default)]
    pub display: Option<AdvancementDisplay>,
    //pub criteria : Vec<Criterion>,
    #[serde(default)]
    pub rewards: AdvancementRewards,
    #[serde(default, rename = "sends_telemetry_event")]
    pub sends_telemetry: bool,
}
pub struct AdvancementHolder(Identifier,AdvancementStruct);

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

#[derive(Default)]
pub struct AdvancementTree {
    pub nodes: HashMap<Identifier, usize>,
    pub nodes_vector: Vec<AdvancementNode>,
    pub roots: Vec<usize>,
    pub tasks: Vec<usize>
}

impl AdvancementTree {
    pub fn add_all(&mut self, advancements: impl IntoIterator<Item = AdvancementHolder>) {
        let mut advancements_to_add : Vec<AdvancementHolder> = advancements
            .into_iter()
            .collect();

        while !advancements_to_add.is_empty() {
            let len_before = advancements_to_add.len();
            advancements_to_add.retain(|&val| !self.try_insert(val));
            if len_before == advancements_to_add.len() {
                eprintln!("Couldn't load advancements: {:?}",
                    advancements_to_add.iter().map(|a| &a.0).collect::<Vec<_>>()
                );
                break;
            }
        }
    }

    pub fn try_insert(&mut self, advancement: AdvancementHolder) -> bool {
        let parent_id = &advancement.1.parent;
        let parent_node : Option<usize> = match parent_id {
            Some(id) => match self.nodes.get(id) {
                Some(node) => Some(*node),
                None => return false,
            },
            None => None,
        };
        let id = advancement.0.clone();
        let node = AdvancementNode::new(advancement, parent_node.clone());
        let node_idx = self.nodes_vector.len();
        self.nodes.insert(id, node_idx);
        if let Some(parent)  = self.nodes_vector.get_mut(parent_node.unwrap()) {
            parent.add_child(node_idx);
            self.tasks.push(node_idx);
        } else {
            self.roots.push(node_idx);
        }
        self.nodes_vector.push(node);
        true
    }
}

pub(crate) fn build() -> TokenStream {
    let advancements: BTreeMap<String, AdvancementStruct> =
        serde_json::from_str(&fs::read_to_string("../assets/advancements.json").unwrap())
            .expect("Failed to parse advancements.json");

    let mut variants = TokenStream::new();
    let mut name_to_type = TokenStream::new();
    let mut minecraft_name_to_type = TokenStream::new();
    let mut minecraft_namespaces = TokenStream::new();
    let capacity = advancements.len();
    for (minecraft_name, advancement) in advancements {
        let raw_name = minecraft_name.strip_prefix("minecraft:").unwrap();
        let format_name = format_ident!("{}", raw_name.to_shouty_snake_case());

        let parent = if let Some(parent) = &advancement.parent {
            quote!{Some(Identifier::parse_static(#parent))}
        } else {
            quote!{ None }
        };
        let send_telemetry = advancement.sends_telemetry;
        let display = match &advancement.display {
            Some(display) => quote! { Some(&#display) },
            None => quote! { None },
        };
        let reward = advancement.rewards;
        variants.extend([quote! {
            pub const #format_name: &Self = &Self {
                id: Identifier::vanilla_static(#raw_name),
                parent : #parent,
                send_telemetry : #send_telemetry,
                display : #display,
                reward : &#reward,
            };
        }]);

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

        pub struct Advancement {
            pub id : Identifier,
            pub parent : Option<Identifier>,
            pub send_telemetry : bool,
            pub display : Option<&'static AdvancementDisplay>,
            pub reward : &'static AdvancementReward,
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
        }
    }
}
