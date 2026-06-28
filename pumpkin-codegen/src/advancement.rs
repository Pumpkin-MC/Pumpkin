use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::TextContent::Translate;
use quote::{ToTokens, format_ident, quote};
use serde::{Deserialize, Deserializer, Serialize};
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
/// Represents a node in the advancement tree used to calculate positions using the Reingold-Tilford algorithm.
///
/// This structure is used internally by the positioning algorithm to compute the layout of advancements
/// as they are displayed by the client, mirroring the behavior of the original Minecraft server.
/// Each node stores positioning information, parent-child relationships, and temporary data used during
/// the tree traversal algorithms.
struct TreeNodePosition {
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

impl TreeNodePosition {
    /// Calculates and sets the x and y positions for all advancement nodes in the tree using the Reingold-Tilford algorithm.
    ///
    /// This method implements the three-pass Reingold-Tilford algorithm to compute an optimal hierarchical layout
    /// for advancement nodes within the advancement tree. The algorithm ensures that the tree is drawn with
    /// minimal width while maintaining a clear parent-child hierarchy.
    ///
    /// # Arguments
    ///
    /// * `tree` - A mutable reference to the `AdvancementTree` containing all the advancement nodes.
    /// The method updates the x and y positions of each node's display information.
    /// * `root_index` - The index of the root node in the tree from which to start the positioning algorithm.
    /// The root must have a display component, otherwise the function will panic.
    ///
    /// # Panics
    ///
    /// Panics if the root node at `root_index` does not have a display component, as the algorithm cannot
    /// position children of invisible nodes.
    ///
    /// # Algorithm Overview
    ///
    /// The positioning is done in three phases:
    /// 1. **First walk**: Assigns preliminary x-coordinates based on subtree placement rules
    /// 2. **Second walk**: Converts preliminary coordinates to final coordinates and calculates the minimum y value
    /// 3. **Third walk**: Adjusts all y-coordinates if necessary to ensure they are non-negative
    pub fn run(tree: &mut AdvancementTree, root_index: usize) {
        let root_node = if let Some(node) = tree.nodes_vector.get(root_index) {
            node
        } else {
            eprintln!("AdvancementNode index out of bounds");
            return;
        };
        if !root_node.has_display() {
            eprintln!("Can't position children of an invisible root!");
            return;
        }
        let mut nodes: Vec<TreeNodePosition> = Vec::with_capacity(32);
        let root_idx = nodes.len();
        nodes.push(TreeNodePosition {
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
        for child in root_node.children.clone() {
            previous_idx = Self::add_child(&mut nodes, tree, root_idx, child, previous_idx);
        }

        Self::first_walk(&mut nodes, root_idx);

        let root_y = nodes[root_idx].y;
        let min = Self::second_walk(&mut nodes, root_idx, 0.0, 0, root_y);

        if min < 0.0 {
            Self::third_walk(&mut nodes, root_idx, -min);
        }

        Self::finalize_position(tree, &nodes, root_idx);
    }

    fn add_child(
        nodes: &mut Vec<TreeNodePosition>,
        tree: &mut AdvancementTree,
        parent_idx: usize,
        adv_node_idx: usize,
        mut previous_idx: Option<usize>,
    ) -> Option<usize> {
        let adv_node = tree.nodes_vector.get(adv_node_idx)?;
        if adv_node.has_display() {
            let child_idx = nodes.len();
            let node = &mut nodes[parent_idx];
            let next_child_index = node.children.len() + 1;
            let depth = node.x + 1;
            node.children.push(child_idx);

            nodes.push(TreeNodePosition {
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

            let mut child_prev = None;
            for child in adv_node.children.clone() {
                child_prev = Self::add_child(nodes, tree, child_idx, child, child_prev);
            }

            Some(child_idx)
        } else {
            for grandchild in &adv_node.children.clone() {
                previous_idx = Self::add_child(nodes, tree, parent_idx, *grandchild, previous_idx);
            }
            previous_idx
        }
    }

    /// First walk of the tree positioning algorithm.
    ///
    /// This function assigns preliminary y-coordinates to nodes based on subtree placement rules.
    /// It performs a post-order traversal (children before parent) to recursively calculate positions.
    ///
    /// For leaf nodes, the y-coordinate is set based on the previous sibling's position.
    /// For nodes with children, y-coordinates are calculated as the midpoint between the
    /// first and last child after applying the apportion algorithm to resolve overlaps.
    ///
    /// # Arguments
    ///
    /// * `nodes` - A mutable reference to the vector of `TreeNodePosition` representing the tree structure.
    /// * `idx` - The index of the current node being processed in the `nodes` vector.
    ///
    /// # Algorithm Details
    ///
    /// - Recursively processes all children first (post-order traversal)
    /// - Uses `apportion()` to detect and resolve overlaps between subtrees
    /// - Executes shifts to propagate positioning adjustments down the tree
    /// - Calculates the node's y-coordinate as either the midpoint of children or positioned
    ///   relative to the previous sibling
    ///
    /// # Note
    ///
    /// This is the first of three passes needed to compute final positions. It sets preliminary
    /// coordinates that will be refined in subsequent passes.
    fn first_walk(nodes: &mut Vec<TreeNodePosition>, idx: usize) {
        let num_children = nodes[idx].children.len();
        if num_children == 0 {
            if let Some(prev_sib) = nodes[idx].previous_sibling {
                nodes[idx].y = nodes[prev_sib].y + 1.0;
            } else {
                nodes[idx].y = 0.0;
            }
        } else {
            let mut default_ancestor: Option<usize> = None;
            for i in 0..num_children {
                let child_idx = nodes[idx].children[i];
                Self::first_walk(nodes, child_idx);
                let arg_ancestor = default_ancestor.unwrap_or(child_idx);
                default_ancestor = Some(Self::apportion(nodes, child_idx, arg_ancestor));
            }

            Self::execute_shifts(nodes, idx);

            let node = &mut nodes[idx];
            let first_child_idx = node.children[0];
            let last_child_idx = node.children[num_children - 1];
            let midpoint = (nodes[first_child_idx].y + nodes[last_child_idx].y) / 2.0;

            if let Some(prev_sib) = nodes[idx].previous_sibling {
                nodes[idx].y = nodes[prev_sib].y + 1.0;
                nodes[idx].mod_field = nodes[idx].y - midpoint;
            } else {
                nodes[idx].y = midpoint;
            }
        }
    }

    /// Second walk of the tree positioning algorithm.
    ///
    /// This function converts preliminary coordinates to final coordinates and calculates
    /// the minimum y value encountered during the traversal. It performs a pre-order traversal
    /// (parent before children) to accumulate modifications from parent nodes to children.
    ///
    /// # Arguments
    ///
    /// * `nodes` - A mutable reference to the vector of `TreeNodePosition` representing the tree structure.
    /// * `idx` - The index of the current node being processed in the `nodes` vector.
    /// * `mod_sum` - The accumulated modification offset from all ancestor nodes. This value is
    /// added to convert preliminary coordinates to final coordinates.
    /// * `depth` - The depth level of the current node in the tree (0 for root, increments for children).
    /// * `mut min` - The minimum y-coordinate encountered so far in the traversal.
    ///
    /// # Returns
    ///
    /// The minimum y-coordinate value found in the current node and all its descendants.
    ///
    /// # Algorithm Details
    ///
    /// - Applies accumulated modifications to convert preliminary y-coordinates to final coordinates
    /// - Sets the x-coordinate (depth) for positioning nodes horizontally
    /// - Tracks the minimum y value to detect if adjustment is needed
    /// - Recursively processes all children, accumulating their modifier offsets
    ///
    /// # Note
    ///
    /// This is the second of three passes. The returned minimum value is used to ensure
    /// all y-coordinates are non-negative in the third walk.
    fn second_walk(
        nodes: &mut Vec<TreeNodePosition>,
        idx: usize,
        mod_sum: f32,
        depth: i32,
        mut min: f32,
    ) -> f32 {
        let node = &mut nodes[idx];
        node.y += mod_sum;
        node.x = depth;

        if node.y < min {
            min = node.y;
        }

        let num_children = node.children.len();
        let current_mod = node.mod_field;

        for i in 0..num_children {
            let child_idx = nodes[idx].children[i];
            min = Self::second_walk(nodes, child_idx, mod_sum + current_mod, depth + 1, min);
        }

        min
    }

    /// Third walk of the tree positioning algorithm.
    ///
    /// This function adjusts all y-coordinates of the tree by adding a uniform offset, ensuring
    /// all coordinates are non-negative. It traverses the tree recursively, applying the same
    /// offset to each node and its descendants.
    ///
    /// # Arguments
    ///
    /// * `nodes` - A mutable reference to the vector of `TreeNodePosition` representing the tree structure.
    /// * `idx` - The index of the current node being processed in the `nodes` vector.
    /// * `offset` - The y-coordinate offset to apply. This is typically the negation of the minimum
    /// y value found in the second walk.
    ///
    /// # Algorithm Details
    ///
    /// - Adds the offset to the current node's y-coordinate
    /// - Recursively applies the same offset to all children
    /// - Uses a simple post-order traversal to ensure uniform adjustment across the entire tree
    ///
    /// # Note
    ///
    /// This is the third of three passes. It only executes if the minimum y value found in
    /// the second walk was negative, ensuring all final positions are non-negative.
    fn third_walk(nodes: &mut Vec<TreeNodePosition>, idx: usize, offset: f32) {
        nodes.iter_mut().for_each(|node| {
            node.y += offset;
        });
    }

    fn execute_shifts(nodes: &mut [TreeNodePosition], idx: usize) {
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
    fn previous_or_thread(nodes: &[TreeNodePosition], idx: usize) -> Option<usize> {
        nodes[idx]
            .thread
            .or_else(|| nodes[idx].children.first().copied())
    }

    #[inline]
    fn next_or_thread(nodes: &[TreeNodePosition], idx: usize) -> Option<usize> {
        nodes[idx]
            .thread
            .or_else(|| nodes[idx].children.last().copied())
    }

    fn apportion(nodes: &mut [TreeNodePosition], idx: usize, mut default_ancestor: usize) -> usize {
        let prev_sib = match nodes[idx].previous_sibling {
            Some(p) => p,
            None => return default_ancestor,
        };
        let parent_idx = nodes[idx].parent.expect("Tree invariant broken: no parent");
        let mut inner_right = idx;
        let mut outer_right = idx;
        let mut inner_left = prev_sib;
        let mut outer_left = nodes[parent_idx].children[0];

        let mod_field = nodes[idx].mod_field;
        let mut shift_inner_right = mod_field;
        let mut shift_outer_right = mod_field;
        let mut shift_inner_left = nodes[inner_left].mod_field;
        let mut shift_outer_left = nodes[outer_left].mod_field;
        while let Some(next_inner_left) = Self::next_or_thread(nodes, inner_left)
            && let Some(next_inner_right) = Self::previous_or_thread(nodes, inner_right)
        {
            inner_left = next_inner_left;
            inner_right = next_inner_right;
            outer_left =
                Self::previous_or_thread(nodes, outer_left).expect("Tree invariant broken");
            outer_right = Self::next_or_thread(nodes, outer_right).expect("Tree invariant broken");

            nodes[outer_right].ancestor = idx;

            let shift = (nodes[inner_left].y + shift_inner_left)
                - (nodes[inner_right].y + shift_inner_right)
                + 1.0;
            if shift > 0.0 {
                let ancestor_idx = Self::get_ancestor(nodes, inner_left, idx, default_ancestor);
                Self::move_subtree(nodes, ancestor_idx, idx, shift);
                shift_inner_right += shift;
                shift_outer_right += shift;
            }

            shift_inner_left += nodes[inner_left].mod_field;
            shift_inner_right += nodes[inner_right].mod_field;
            shift_outer_left += nodes[outer_left].mod_field;
        }

        if let Some(next_inner_left) = Self::next_or_thread(nodes, inner_left)
            && Self::next_or_thread(nodes, outer_right).is_none()
        {
            nodes[outer_right].thread = Some(next_inner_left);
            nodes[outer_right].mod_field += shift_inner_left - shift_outer_right;
        } else {
            if let Some(next_inner_right) = Self::previous_or_thread(nodes, inner_right)
                && Self::previous_or_thread(nodes, outer_left).is_none()
            {
                nodes[outer_left].thread = Some(next_inner_right);
                nodes[outer_left].mod_field += shift_inner_right - shift_outer_left;
            }
            default_ancestor = idx;
        }
        default_ancestor
    }

    fn move_subtree(nodes: &mut [TreeNodePosition], left: usize, right: usize, shift: f32) {
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
        nodes: &[TreeNodePosition],
        idx: usize,
        other: usize,
        default_ancestor: usize,
    ) -> usize {
        let ancestor = nodes[idx].ancestor;
        let parent_idx = nodes[other].parent.unwrap();

        if nodes[parent_idx].children.contains(&ancestor) {
            ancestor
        } else {
            default_ancestor
        }
    }

    /// Final walk of the tree positioning algorithm.
    ///
    /// This function applies the computed positions to the actual advancement nodes in the tree,
    /// finalizing their display locations. It traverses the tree recursively and updates each node's
    /// position coordinates in the tree structure.
    ///
    /// # Arguments
    ///
    /// * `tree` - A mutable reference to the `AdvancementTree`. This tree is updated with the
    ///   computed x and y positions from the `TreeNodePosition` nodes.
    /// * `nodes` - A reference to the vector of `TreeNodePosition` containing the computed positions
    ///   for each node in the tree.
    /// * `idx` - The index of the current node being processed in the `nodes` vector.
    ///
    /// # Algorithm Details
    ///
    /// - Retrieves the computed x and y positions from the `TreeNodePosition` at the given index
    /// - Sets these positions on the corresponding advancement node in the tree
    /// - Recursively processes all children, updating their positions as well
    /// - Uses a post-order traversal to ensure all nodes are properly positioned
    ///
    /// # Note
    ///
    /// This is the fourth and final pass. It should only be called after all three positioning walks
    /// (first, second, and third) have been completed successfully. This function transfers the
    /// computed positions from the internal `TreeNodePosition` structures back to the actual
    /// `AdvancementNode` display information.
    fn finalize_position(tree: &mut AdvancementTree, nodes: &[TreeNodePosition], idx: usize) {
        tree.nodes_vector[nodes[idx].node].set_location(nodes[idx].x as f32, nodes[idx].y);
        for &child_idx in &nodes[idx].children {
            Self::finalize_position(tree, nodes, child_idx);
        }
    }
}

/// The structure that represents an advancement
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
    #[serde(deserialize_with = "deserialize_first_key")]
    pub criteria: Vec<String>,
}

fn deserialize_first_key<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let map = BTreeMap::<String, serde_json::Value>::deserialize(deserializer)?;
    Ok(map.into_keys().collect())
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

/// Entry point for the code generation of advancements.
///
/// Parses the `advancements.json` asset, builds the advancement tree,
/// calculates positions using the Reingold-Tilford algorithm, and generates
/// the final Rust source code.
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
    let mut advancement_list = TokenStream::new();
    let capacity = advancements.len();
    //construct the tree
    let mut tree = AdvancementTree::default();
    tree.add_all(
        advancements
            .into_iter()
            .map(|(key, value)| AdvancementHolder(Identifier::parse(&key).unwrap(), value))
            .collect(),
    );
    for index in tree.roots.clone() {
        if tree.nodes_vector.get(index).unwrap().has_display() {
            TreeNodePosition::run(&mut tree, index);
        }
    }
    let advancement_tree = quote! {
        pub static ADVANCEMENT_TREE : LazyLock<AdvancementTree> = #tree;
    };
    let advancements_holder = tree.nodes_vector.into_iter().map(|node| node.value);
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
        let criteria = advancement.criteria;
        variants.extend([quote! {
            pub const #format_name: &Self = &Self {
                id: Identifier::vanilla_static(#raw_name),
                parent : #parent,
                send_telemetry : #send_telemetry,
                display : #display,
                reward : &#reward,
                requirements: &[#(#requirements),*],
                criteria: &[#(#criteria),*],
            };
        }]);
        let minecraft_name = identifier.to_string();

        name_to_type.extend(quote! { #raw_name => Some(Self::#format_name), });
        minecraft_name_to_type.extend(quote! { #minecraft_name => Some(Self::#format_name), });
        minecraft_namespaces.extend(quote! { Identifier::vanilla_static(#raw_name),});
        advancement_list.extend(quote! {Self::#format_name, });
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
            pub requirements: &'static[&'static[&'static str]],
            pub criteria: &'static[&'static str],
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

            pub fn get_advancements_list() -> [&'static Advancement; #capacity] {
                [#advancement_list]
            }

            pub const fn get_identifier_list() -> [Identifier;#capacity] {
                [#minecraft_namespaces]
            }

            pub const fn is_root(&self) -> bool{
                self.parent.is_none()
            }

        }
        #advancement_tree
    }
}
