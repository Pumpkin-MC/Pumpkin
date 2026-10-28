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
    fn third_walk(nodes: &mut Vec<TreeNodePosition>, _idx: usize, offset: f32) {
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

/// The criterion definition with trigger and condition
#[derive(Deserialize, Clone, Debug, Default)]
pub struct CriterionStruct {
    #[serde(default)]
    pub trigger: String,
    #[serde(default)]
    pub conditions: serde_json::Value,
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
    #[serde(default)]
    pub criteria: BTreeMap<String, CriterionStruct>,
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
        let node_count = self.nodes.len();
        let nodes = self.nodes.iter().map(|(k, v)| {
            let key = identifier_to_tokens(k);
            quote! { (#key, #v) }
        });
        let nodes_vector = &self.nodes_vector;
        let roots = &self.roots;
        let tasks = &self.tasks;
        tokens.extend(quote! {
            LazyLock::new(|| {
                const NODE_IDS: [(Identifier, usize); #node_count] = [#(#nodes),*];
                let nodes: BTreeMap<Identifier, usize> = NODE_IDS.into_iter().collect();
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

fn collect_advancements(
    base: &std::path::Path,
    dir: &std::path::Path,
    result: &mut BTreeMap<String, AdvancementStruct>,
) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_advancements(base, &path, result);
        } else if path.extension().is_some_and(|ext| ext == "json") {
            let rel = path.strip_prefix(base).unwrap();
            let rel_str = rel
                .with_extension("")
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            let key = format!("minecraft:{rel_str}");
            if let Ok(content) = fs::read_to_string(&path)
                && let Ok(adv) = serde_json::from_str::<AdvancementStruct>(&content)
            {
                result.insert(key, adv);
            }
        }
    }
}

fn opt_str_tokens(opt: Option<&str>) -> TokenStream {
    match opt {
        Some(s) => quote! { Some(#s) },
        None => quote! { None },
    }
}

fn opt_f64_tokens(opt: Option<f64>) -> TokenStream {
    match opt {
        Some(v) => quote! { Some(#v) },
        None => quote! { None },
    }
}

fn opt_i64_tokens(opt: Option<i64>) -> TokenStream {
    match opt {
        Some(v) => quote! { Some(#v) },
        None => quote! { None },
    }
}

fn opt_i32_tokens(opt: Option<i32>) -> TokenStream {
    match opt {
        Some(v) => quote! { Some(#v) },
        None => quote! { None },
    }
}

fn opt_bool_tokens(opt: Option<bool>) -> TokenStream {
    match opt {
        Some(v) => quote! { Some(#v) },
        None => quote! { None },
    }
}

fn generate_item_condition_tokens(val: &serde_json::Value) -> TokenStream {
    let mut items: Vec<&str> = Vec::new();
    let mut tag = None;
    if let Some(items_val) = val.get("items") {
        match items_val {
            serde_json::Value::String(s) => {
                if let Some(t) = s.strip_prefix('#') {
                    tag = Some(t);
                } else {
                    items.push(s.as_str());
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    if let Some(s) = item.as_str() {
                        if let Some(t) = s.strip_prefix('#') {
                            tag = Some(t);
                        } else {
                            items.push(s);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    let count_min = val.get("count").and_then(|c| match c {
        serde_json::Value::Number(n) => n.as_i64(),
        serde_json::Value::Object(m) => m.get("min").and_then(|v| v.as_i64()),
        _ => None,
    });
    let count_max = val.get("count").and_then(|c| match c {
        serde_json::Value::Number(n) => n.as_i64(),
        serde_json::Value::Object(m) => m.get("max").and_then(|v| v.as_i64()),
        _ => None,
    });
    let tag_tok = opt_str_tokens(tag);
    let cmin_tok = opt_i64_tokens(count_min);
    let cmax_tok = opt_i64_tokens(count_max);

    quote! {
        ItemCondition {
            items: &[#(#items),*],
            tag: #tag_tok,
            count_min: #cmin_tok,
            count_max: #cmax_tok,
            durability_min: None,
            durability_max: None,
            enchantments: &[],
        }
    }
}

fn generate_entity_condition_tokens(val: &serde_json::Value) -> TokenStream {
    let val = val.as_array().and_then(|a| a.first()).unwrap_or(val);
    let entity_pred = if let Some(pred) = val.get("predicate") {
        pred
    } else if let Some(ent) = val.get("entity") {
        if ent.is_object() {
            ent.get("predicate").unwrap_or(ent)
        } else {
            val.get("predicate").unwrap_or(val)
        }
    } else {
        val
    };
    let mut entity_type = None;
    let mut tag = None;
    if let Some(type_val) = entity_pred
        .get("minecraft:entity_type")
        .or_else(|| entity_pred.get("type"))
    {
        if let Some(s) = type_val.as_str() {
            if let Some(t) = s.strip_prefix('#') {
                tag = Some(t);
            } else if !matches!(
                s,
                "minecraft:all_of"
                    | "minecraft:any_of"
                    | "minecraft:entity_properties"
                    | "minecraft:inverted"
                    | "minecraft:location_check"
                    | "minecraft:match_block"
                    | "minecraft:match_tool"
            ) {
                entity_type = Some(s);
            }
        }
    }
    let is_baby = entity_pred
        .get("minecraft:flags")
        .or_else(|| entity_pred.get("flags"))
        .and_then(|f| f.get("is_baby"))
        .and_then(|b| b.as_bool());
    let dist_h = entity_pred
        .get("minecraft:distance")
        .or_else(|| entity_pred.get("distance"))
        .and_then(|d| d.get("horizontal"))
        .and_then(|h| h.get("min"))
        .and_then(|v| v.as_f64());
    let dist_a = entity_pred
        .get("minecraft:distance")
        .or_else(|| entity_pred.get("distance"))
        .and_then(|d| d.get("absolute"))
        .and_then(|a| a.get("min"))
        .and_then(|v| v.as_f64());

    let mut vehicle_type = None;
    let mut vehicle_tag = None;
    let mut passenger_type = None;
    if let Some(veh) = entity_pred
        .get("minecraft:vehicle")
        .or_else(|| entity_pred.get("vehicle"))
    {
        if let Some(vtype) = veh
            .get("minecraft:entity_type")
            .or_else(|| veh.get("type"))
            .and_then(|t| t.as_str())
        {
            if let Some(t) = vtype.strip_prefix('#') {
                vehicle_tag = Some(t);
            } else {
                vehicle_type = Some(vtype);
            }
        }
        if let Some(pass) = veh
            .get("minecraft:passenger")
            .or_else(|| veh.get("passenger"))
        {
            passenger_type = pass
                .get("minecraft:entity_type")
                .or_else(|| pass.get("type"))
                .and_then(|t| t.as_str());
        }
    }

    let stepping_on = entity_pred
        .get("minecraft:stepping_on")
        .or_else(|| entity_pred.get("stepping_on"))
        .and_then(|s| s.get("block").or_else(|| s.get("blocks")))
        .and_then(|b| b.get("blocks").or_else(|| Some(b)))
        .and_then(|b| b.as_str());

    let looking_at = entity_pred
        .get("minecraft:type_specific/player")
        .and_then(|p| p.get("looking_at"))
        .and_then(|l| {
            l.get("minecraft:entity_type")
                .or_else(|| l.get("type"))
                .and_then(|t| t.as_str())
        });

    let feet_item = entity_pred
        .get("minecraft:equipment")
        .or_else(|| entity_pred.get("equipment"))
        .and_then(|eq| eq.get("feet"))
        .and_then(|f| f.get("items"))
        .and_then(|i| i.as_str());

    let val_str = val.to_string();
    let safe_armor_disallowed = (val_str.contains("piglin_safe_armor")
        && (val_str.contains("inverted") || val_str.contains("minecraft:inverted")))
    .then_some(true);

    let etype_tok = opt_str_tokens(entity_type);
    let tag_tok = opt_str_tokens(tag);
    let baby_tok = opt_bool_tokens(is_baby);
    let dh_tok = opt_f64_tokens(dist_h);
    let da_tok = opt_f64_tokens(dist_a);
    let vtype_tok = opt_str_tokens(vehicle_type);
    let vtag_tok = opt_str_tokens(vehicle_tag);
    let ptype_tok = opt_str_tokens(passenger_type);
    let step_tok = opt_str_tokens(stepping_on);
    let look_tok = opt_str_tokens(looking_at);
    let feet_tok = opt_str_tokens(feet_item);
    let safe_arm_tok = opt_bool_tokens(safe_armor_disallowed);

    quote! {
        EntityCondition {
            entity_type: #etype_tok,
            tag: #tag_tok,
            is_baby: #baby_tok,
            distance_horizontal_min: #dh_tok,
            distance_absolute_min: #da_tok,
            vehicle_type: #vtype_tok,
            vehicle_tag: #vtag_tok,
            passenger_type: #ptype_tok,
            stepping_on: #step_tok,
            looking_at: #look_tok,
            feet_item: #feet_tok,
            safe_armor_disallowed: #safe_arm_tok,
        }
    }
}

fn generate_location_condition_tokens(val: &serde_json::Value) -> TokenStream {
    let loc_val = val
        .get("player")
        .and_then(|p| p.get("predicate"))
        .and_then(|pred| {
            pred.get("minecraft:location")
                .or_else(|| pred.get("location"))
        })
        .or_else(|| val.get("location"))
        .unwrap_or(val);

    let biome = loc_val
        .get("biomes")
        .or_else(|| loc_val.get("biome"))
        .and_then(|b| b.as_str());
    let dimension = loc_val.get("dimension").and_then(|d| d.as_str());
    let block = loc_val
        .get("block")
        .or_else(|| loc_val.get("blocks"))
        .and_then(|b| {
            b.as_str()
                .or_else(|| b.get("blocks").and_then(|s| s.as_str()))
        });
    let can_see_sky = loc_val.get("can_see_sky").and_then(|b| b.as_bool());
    let smokey = loc_val.get("smokey").and_then(|b| b.as_bool());
    let structure = loc_val
        .get("structures")
        .or_else(|| loc_val.get("structure"))
        .and_then(|s| s.as_str());

    let player_pred = val.get("player").and_then(|p| p.get("predicate"));
    let stepping_on = player_pred
        .and_then(|p| {
            p.get("minecraft:stepping_on")
                .or_else(|| p.get("stepping_on"))
        })
        .and_then(|s| s.get("block").or_else(|| s.get("blocks")))
        .and_then(|b| b.get("blocks").or_else(|| Some(b)))
        .and_then(|b| b.as_str());
    let feet_item = player_pred
        .and_then(|p| p.get("minecraft:equipment").or_else(|| p.get("equipment")))
        .and_then(|eq| eq.get("feet"))
        .and_then(|f| f.get("items"))
        .and_then(|i| i.as_str());

    let biome_tok = opt_str_tokens(biome);
    let dim_tok = opt_str_tokens(dimension);
    let block_tok = opt_str_tokens(block);
    let sky_tok = opt_bool_tokens(can_see_sky);
    let smokey_tok = opt_bool_tokens(smokey);
    let struct_tok = opt_str_tokens(structure);
    let step_tok = opt_str_tokens(stepping_on);
    let feet_tok = opt_str_tokens(feet_item);

    quote! {
        &LocationCondition {
            biome: #biome_tok,
            dimension: #dim_tok,
            block: #block_tok,
            can_see_sky: #sky_tok,
            smokey: #smokey_tok,
            structure: #struct_tok,
            stepping_on: #step_tok,
            feet_item: #feet_tok,
        }
    }
}

fn generate_item_used_on_block_condition_tokens(val: &serde_json::Value) -> TokenStream {
    let mut blocks: Vec<&str> = Vec::new();
    let mut block_tag = None;
    let mut items: Vec<&str> = Vec::new();
    let mut item_tag = None;
    let mut state_key = None;
    let mut state_value = None;
    let mut smokey = None;
    let mut biome = None;

    let loc = val.get("location").unwrap_or(val);
    let mut terms: Vec<&serde_json::Value> = Vec::new();
    if let Some(arr) = loc.get("terms").and_then(|t| t.as_array()) {
        terms.extend(arr);
    } else {
        terms.push(loc);
    }

    for term in terms {
        let pred = term.get("predicate").unwrap_or(term);
        let blk = pred.get("block").unwrap_or(pred);
        if let Some(b_val) = blk.get("blocks").or_else(|| blk.get("block")) {
            if let Some(s) = b_val.as_str() {
                if let Some(t) = s.strip_prefix('#') {
                    block_tag = Some(t);
                } else {
                    blocks.push(s);
                }
            } else if let Some(arr) = b_val.as_array() {
                for b in arr {
                    if let Some(s) = b.as_str() {
                        if let Some(t) = s.strip_prefix('#') {
                            block_tag = Some(t);
                        } else {
                            blocks.push(s);
                        }
                    }
                }
            }
        }
        if let Some(st) = blk.get("state").and_then(|s| s.as_object()) {
            for (k, v) in st {
                state_key = Some(k.as_str());
                state_value = v.as_str();
            }
        }
        if let Some(sm) = pred.get("smokey").and_then(|s| s.as_bool()) {
            smokey = Some(sm);
        }
        if let Some(bio) = pred
            .get("biomes")
            .or_else(|| pred.get("biome"))
            .and_then(|b| b.as_str())
        {
            biome = Some(bio);
        }
        if let Some(i_val) = pred.get("items").or_else(|| pred.get("item")) {
            if let Some(s) = i_val.as_str() {
                if let Some(t) = s.strip_prefix('#') {
                    item_tag = Some(t);
                } else {
                    items.push(s);
                }
            } else if let Some(arr) = i_val.as_array() {
                for i in arr {
                    if let Some(s) = i.as_str() {
                        if let Some(t) = s.strip_prefix('#') {
                            item_tag = Some(t);
                        } else {
                            items.push(s);
                        }
                    }
                }
            }
        }
    }

    let btag_tok = opt_str_tokens(block_tag);
    let itag_tok = opt_str_tokens(item_tag);
    let sk_tok = opt_str_tokens(state_key);
    let sv_tok = opt_str_tokens(state_value);
    let sm_tok = opt_bool_tokens(smokey);
    let bio_tok = opt_str_tokens(biome);

    quote! {
        &ItemUsedOnBlockCondition {
            blocks: &[#(#blocks),*],
            block_tag: #btag_tok,
            items: &[#(#items),*],
            item_tag: #itag_tok,
            state_key: #sk_tok,
            state_value: #sv_tok,
            smokey: #sm_tok,
            biome: #bio_tok,
        }
    }
}

fn extract_damage_condition_tokens(val: &serde_json::Value) -> TokenStream {
    let damage_val = val.get("damage").unwrap_or(val);
    let blocked = damage_val.get("blocked").and_then(|b| b.as_bool());
    let dealt_min = damage_val
        .get("dealt")
        .and_then(|d| d.get("min"))
        .and_then(|v| v.as_f64());
    let taken_min = damage_val
        .get("taken")
        .and_then(|t| t.get("min"))
        .and_then(|v| v.as_f64());
    let is_projectile = damage_val
        .get("type")
        .and_then(|t| t.get("tags"))
        .and_then(|tags| tags.as_array())
        .map(|arr| {
            arr.iter().any(|tag| {
                tag.get("id").and_then(|s| s.as_str()) == Some("#minecraft:is_projectile")
            })
        });
    let b_tok = opt_bool_tokens(blocked);
    let proj_tok = opt_bool_tokens(is_projectile);
    let dealt_tok = opt_f64_tokens(dealt_min);
    let taken_tok = opt_f64_tokens(taken_min);
    quote! {
        DamageCondition {
            blocked: #b_tok,
            is_projectile: #proj_tok,
            dealt_min: #dealt_tok,
            taken_min: #taken_tok,
        }
    }
}

fn generate_condition_tokens(trigger: &str, val: &serde_json::Value) -> TokenStream {
    if val.is_null() || val.as_object().is_some_and(|m| m.is_empty()) {
        return match trigger {
            "minecraft:slept_in_bed" => quote! { AdvancementCondition::SleptInBed },
            "minecraft:entity_killed_player" => {
                quote! { AdvancementCondition::EntityKilledPlayer }
            }
            "minecraft:used_ender_eye" => quote! { AdvancementCondition::UsedEnderEye },
            "minecraft:cured_zombie_villager" => {
                quote! { AdvancementCondition::CuredZombieVillager }
            }
            "minecraft:villager_trade" => quote! { AdvancementCondition::VillagerTrade },
            "minecraft:tick" => quote! { AdvancementCondition::Tick },
            "minecraft:killed_by_arrow" => quote! { AdvancementCondition::KilledByArrow },
            "minecraft:enchanted_item" => quote! { AdvancementCondition::EnchantedItem },
            "minecraft:brewed_potion" => quote! { AdvancementCondition::BrewedPotion },
            "minecraft:avoid_vibration" => quote! { AdvancementCondition::AvoidVibration },
            "minecraft:kill_mob_near_sculk_catalyst" => {
                quote! { AdvancementCondition::KillMobNearSculkCatalyst }
            }
            "minecraft:hero_of_the_village" => quote! { AdvancementCondition::HeroOfTheVillage },
            "minecraft:started_riding" => quote! { AdvancementCondition::StartedRiding },
            "minecraft:effects_changed" => quote! { AdvancementCondition::EffectsChanged },
            "minecraft:impossible" => quote! { AdvancementCondition::Impossible },
            _ => quote! { AdvancementCondition::Empty },
        };
    }

    match trigger {
        "minecraft:player_generates_container_loot" => {
            let lt = val
                .get("loot_tables")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            quote! { AdvancementCondition::PlayerGeneratesContainerLoot(#lt) }
        }
        "minecraft:recipe_unlocked" => {
            let r = val
                .get("recipes")
                .or_else(|| val.get("recipe"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            quote! { AdvancementCondition::RecipeUnlocked(#r) }
        }
        "minecraft:recipe_crafted" => {
            let r = val.get("recipes").and_then(|v| v.as_str()).unwrap_or("");
            quote! { AdvancementCondition::RecipeCrafted(#r) }
        }
        "minecraft:inventory_changed" => {
            let items_toks: Vec<_> = val
                .get("items")
                .and_then(|i| i.as_array())
                .map_or_else(Vec::new, |arr| {
                    arr.iter().map(generate_item_condition_tokens).collect()
                });
            quote! {
                AdvancementCondition::InventoryChanged(&InventoryChangedCondition {
                    items: &[#(#items_toks),*],
                    slots: None,
                })
            }
        }
        "minecraft:consume_item" => {
            let item_val = val.get("item").unwrap_or(val);
            let item_tok = generate_item_condition_tokens(item_val);
            quote! { AdvancementCondition::ConsumeItem(#item_tok) }
        }
        "minecraft:placed_block" => {
            let mut block = val
                .get("block")
                .or_else(|| val.get("blocks"))
                .and_then(|v| v.as_str());
            if block.is_none() {
                if let Some(loc) = val.get("location") {
                    block = loc
                        .get("blocks")
                        .or_else(|| loc.get("block"))
                        .and_then(|v| v.as_str());
                    if block.is_none() {
                        let loc_str = loc.to_string();
                        if loc_str.contains("chiseled_bookshelf") {
                            block = Some("minecraft:chiseled_bookshelf");
                        } else if loc_str.contains("comparator") {
                            block = Some("minecraft:comparator");
                        } else if loc_str.contains("creaking_heart") {
                            block = Some("minecraft:creaking_heart");
                        } else if loc_str.contains("pale_oak_logs") {
                            block = Some("minecraft:pale_oak_log");
                        }
                    }
                }
            }
            let b_tok = opt_str_tokens(block);
            quote! {
                AdvancementCondition::PlacedBlock {
                    block: #b_tok,
                    item: None,
                }
            }
        }
        "minecraft:item_used_on_block" => {
            let cond_tok = generate_item_used_on_block_condition_tokens(val);
            quote! { AdvancementCondition::ItemUsedOnBlock(#cond_tok) }
        }
        "minecraft:allay_drop_item_on_block" => {
            let cond_tok = generate_item_used_on_block_condition_tokens(val);
            quote! { AdvancementCondition::AllayDropItemOnBlock(#cond_tok) }
        }
        "minecraft:entity_hurt_player" => {
            let dmg_tok = extract_damage_condition_tokens(val);
            quote! { AdvancementCondition::EntityHurtPlayer(#dmg_tok) }
        }
        "minecraft:player_hurt_entity" => {
            let dmg_tok = extract_damage_condition_tokens(val);
            quote! { AdvancementCondition::PlayerHurtEntity(#dmg_tok) }
        }
        "minecraft:enter_block" => {
            let b = val
                .get("blocks")
                .or_else(|| val.get("block"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            quote! { AdvancementCondition::EnterBlock(#b) }
        }
        "minecraft:slide_down_block" => {
            let b = val
                .get("blocks")
                .or_else(|| val.get("block"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            quote! { AdvancementCondition::SlideDownBlock(#b) }
        }
        "minecraft:filled_bucket" => {
            let item = val
                .get("item")
                .and_then(|i| i.get("items"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            quote! { AdvancementCondition::FilledBucket(#item) }
        }
        "minecraft:fishing_rod_hooked" => {
            let item_val = val.get("item").unwrap_or(val);
            let item_tok = generate_item_condition_tokens(item_val);
            quote! { AdvancementCondition::FishingRodHooked(#item_tok) }
        }
        "minecraft:using_item" => {
            let item_val = val.get("item").unwrap_or(val);
            let item_tok = generate_item_condition_tokens(item_val);
            let looking_at = val
                .get("player")
                .and_then(|p| p.get("predicate"))
                .and_then(|pred| pred.get("minecraft:type_specific/player"))
                .and_then(|p| p.get("looking_at"))
                .and_then(|l| {
                    l.get("minecraft:entity_type")
                        .or_else(|| l.get("type"))
                        .and_then(|t| t.as_str())
                });
            let look_tok = opt_str_tokens(looking_at);
            quote! {
                AdvancementCondition::UsingItem {
                    item: #item_tok,
                    looking_at: #look_tok,
                }
            }
        }
        "minecraft:used_totem" => {
            let item_val = val.get("item").unwrap_or(val);
            let item_tok = generate_item_condition_tokens(item_val);
            quote! { AdvancementCondition::UsedTotem(#item_tok) }
        }
        "minecraft:changed_dimension" => {
            let from = val.get("from").and_then(|v| v.as_str());
            let to = val.get("to").and_then(|v| v.as_str());
            let from_tok = opt_str_tokens(from);
            let to_tok = opt_str_tokens(to);
            quote! { AdvancementCondition::ChangedDimension { from: #from_tok, to: #to_tok } }
        }
        "minecraft:construct_beacon" => {
            let lvl = val.get("level").and_then(|l| match l {
                serde_json::Value::Number(n) => n.as_i64().map(|v| v as i32),
                serde_json::Value::Object(m) => {
                    m.get("min").and_then(|v| v.as_i64()).map(|v| v as i32)
                }
                _ => None,
            });
            let lvl_tok = opt_i32_tokens(lvl);
            quote! { AdvancementCondition::ConstructBeacon(BeaconCondition { level_min: #lvl_tok }) }
        }
        "minecraft:target_hit" => {
            let sig = val
                .get("signal_strength")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32);
            let dist = val
                .get("projectile")
                .and_then(|p| p.get("distance"))
                .and_then(|d| d.get("horizontal"))
                .and_then(|h| h.get("min"))
                .and_then(|v| v.as_f64());
            let sig_tok = opt_i32_tokens(sig);
            let dist_tok = opt_f64_tokens(dist);
            quote! {
                AdvancementCondition::TargetHit(TargetHitCondition {
                    signal_strength: #sig_tok,
                    distance_min: #dist_tok,
                })
            }
        }
        "minecraft:player_killed_entity" => {
            let entity_tok = generate_entity_condition_tokens(val);
            quote! { AdvancementCondition::PlayerKilledEntity(#entity_tok) }
        }
        "minecraft:tame_animal" => {
            let entity_tok = generate_entity_condition_tokens(val);
            quote! { AdvancementCondition::TameAnimal(#entity_tok) }
        }
        "minecraft:thrown_item_picked_up_by_player" => {
            let entity_tok = generate_entity_condition_tokens(val);
            quote! { AdvancementCondition::ThrownItemPickedUpByPlayer(#entity_tok) }
        }
        "minecraft:bred_animals" => {
            let parent_tok = val.get("parent").map(generate_entity_condition_tokens);
            let partner_tok = val.get("partner").map(generate_entity_condition_tokens);
            let child_tok = val.get("child").map(generate_entity_condition_tokens);
            let p_tok = match parent_tok {
                Some(t) => quote! { Some(#t) },
                None => quote! { None },
            };
            let pr_tok = match partner_tok {
                Some(t) => quote! { Some(#t) },
                None => quote! { None },
            };
            let c_tok = match child_tok {
                Some(t) => quote! { Some(#t) },
                None => quote! { None },
            };
            quote! {
                AdvancementCondition::BredAnimals(&BredAnimalsCondition {
                    parent: #p_tok,
                    partner: #pr_tok,
                    child: #c_tok,
                })
            }
        }
        "minecraft:location" => {
            let loc_tok = generate_location_condition_tokens(val);
            quote! { AdvancementCondition::Location(#loc_tok) }
        }
        "minecraft:enchanted_item" => quote! { AdvancementCondition::EnchantedItem },
        "minecraft:brewed_potion" => quote! { AdvancementCondition::BrewedPotion },
        "minecraft:item_durability_changed" => {
            let item_val = val.get("item").unwrap_or(val);
            let item_tok = generate_item_condition_tokens(item_val);
            let vtype = val
                .get("player")
                .and_then(|p| p.get("predicate"))
                .and_then(|pred| {
                    pred.get("minecraft:vehicle")
                        .or_else(|| pred.get("vehicle"))
                })
                .and_then(|v| v.get("minecraft:entity_type").or_else(|| v.get("type")))
                .and_then(|t| t.as_str());
            let vt_tok = opt_str_tokens(vtype);
            quote! {
                AdvancementCondition::ItemDurabilityChanged {
                    item: #item_tok,
                    vehicle_type: #vt_tok,
                }
            }
        }
        "minecraft:levitation" => {
            let dist = val
                .get("distance")
                .and_then(|d| d.get("y"))
                .and_then(|y| y.get("min"))
                .and_then(|v| v.as_f64());
            let d_tok = opt_f64_tokens(dist);
            quote! { AdvancementCondition::Levitation { distance_y_min: #d_tok } }
        }
        "minecraft:fall_from_height" => {
            let dist = val
                .get("distance")
                .and_then(|d| d.get("y"))
                .and_then(|y| y.get("min"))
                .and_then(|v| v.as_f64());
            let d_tok = opt_f64_tokens(dist);
            quote! { AdvancementCondition::FallFromHeight { distance_y_min: #d_tok } }
        }
        "minecraft:fall_after_explosion" => {
            let dist = val
                .get("distance")
                .and_then(|d| d.get("y"))
                .and_then(|y| y.get("min"))
                .and_then(|v| v.as_f64());
            let cause = val
                .get("cause")
                .and_then(|c| c.get("predicate"))
                .and_then(|p| {
                    p.get("minecraft:entity_type")
                        .or_else(|| p.get("entity_type"))
                })
                .and_then(|v| v.as_str());
            let d_tok = opt_f64_tokens(dist);
            let c_tok = opt_str_tokens(cause);
            quote! {
                AdvancementCondition::FallAfterExplosion {
                    distance_y_min: #d_tok,
                    cause_entity: #c_tok,
                }
            }
        }
        "minecraft:nether_travel" => {
            let dist = val
                .get("distance")
                .and_then(|d| d.get("horizontal"))
                .and_then(|h| h.get("min"))
                .and_then(|v| v.as_f64());
            let d_tok = opt_f64_tokens(dist);
            quote! { AdvancementCondition::NetherTravel { distance_horizontal_min: #d_tok } }
        }
        "minecraft:ride_entity_in_lava" => {
            let dist = val
                .get("distance")
                .and_then(|d| d.get("horizontal"))
                .and_then(|h| h.get("min"))
                .and_then(|v| v.as_f64());
            let dim = val
                .get("player")
                .and_then(|p| p.get("predicate"))
                .and_then(|pred| {
                    pred.get("minecraft:location")
                        .or_else(|| pred.get("location"))
                })
                .and_then(|l| l.get("dimension"))
                .and_then(|d| d.as_str());
            let vtype = val
                .get("player")
                .and_then(|p| p.get("predicate"))
                .and_then(|pred| {
                    pred.get("minecraft:vehicle")
                        .or_else(|| pred.get("vehicle"))
                })
                .and_then(|v| v.get("minecraft:entity_type").or_else(|| v.get("type")))
                .and_then(|t| t.as_str());
            let d_tok = opt_f64_tokens(dist);
            let dim_tok = opt_str_tokens(dim);
            let vt_tok = opt_str_tokens(vtype);
            quote! {
                AdvancementCondition::RideEntityInLava {
                    distance_horizontal_min: #d_tok,
                    dimension: #dim_tok,
                    vehicle_type: #vt_tok,
                }
            }
        }
        "minecraft:shot_crossbow" => {
            let item_val = val.get("item").unwrap_or(val);
            let item_tok = generate_item_condition_tokens(item_val);
            quote! { AdvancementCondition::ShotCrossbow(#item_tok) }
        }
        "minecraft:avoid_vibration" => quote! { AdvancementCondition::AvoidVibration },
        "minecraft:bee_nest_destroyed" => {
            let block = val
                .get("blocks")
                .or_else(|| val.get("block"))
                .and_then(|v| v.as_str());
            let bees = val
                .get("num_bees_inside")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32);
            let b_tok = opt_str_tokens(block);
            let bees_tok = opt_i32_tokens(bees);
            quote! {
                AdvancementCondition::BeeNestDestroyed {
                    block: #b_tok,
                    num_bees_inside: #bees_tok,
                }
            }
        }
        "minecraft:channeled_lightning" => {
            let victim = val
                .get("victims")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
                .and_then(|e| e.get("predicate"))
                .and_then(|p| {
                    p.get("minecraft:entity_type")
                        .or_else(|| p.get("entity_type"))
                })
                .and_then(|v| v.as_str());
            let v_tok = opt_str_tokens(victim);
            quote! { AdvancementCondition::ChanneledLightning { victim_type: #v_tok } }
        }
        "minecraft:lightning_strike" => {
            let bystander = val
                .get("bystander")
                .and_then(|b| b.get("predicate"))
                .and_then(|p| {
                    p.get("minecraft:entity_type")
                        .or_else(|| p.get("entity_type"))
                })
                .and_then(|v| v.as_str());
            let fire = val
                .get("lightning")
                .and_then(|l| l.get("predicate"))
                .and_then(|p| p.get("minecraft:type_specific/lightning"))
                .and_then(|t| t.get("blocks_set_on_fire"))
                .and_then(|v| v.as_i64())
                .map(|v| v as i32);
            let max_dist = val
                .get("lightning")
                .and_then(|l| l.get("predicate"))
                .and_then(|p| p.get("minecraft:distance"))
                .and_then(|d| d.get("absolute"))
                .and_then(|a| a.get("max"))
                .and_then(|v| v.as_f64());
            let b_tok = opt_str_tokens(bystander);
            let f_tok = opt_i32_tokens(fire);
            let md_tok = opt_f64_tokens(max_dist);
            quote! {
                AdvancementCondition::LightningStrike {
                    bystander_type: #b_tok,
                    blocks_set_on_fire: #f_tok,
                    distance_max: #md_tok,
                }
            }
        }
        "minecraft:kill_mob_near_sculk_catalyst" => {
            quote! { AdvancementCondition::KillMobNearSculkCatalyst }
        }
        "minecraft:hero_of_the_village" => quote! { AdvancementCondition::HeroOfTheVillage },
        "minecraft:player_interacted_with_entity" => {
            let ent_tok = val.get("entity").map(generate_entity_condition_tokens);
            let item_tok = val.get("item").map(generate_item_condition_tokens);
            let player_tok = val.get("player").map(generate_entity_condition_tokens);
            let e_tok = match ent_tok {
                Some(t) => quote! { Some(#t) },
                None => quote! { None },
            };
            let i_tok = match item_tok {
                Some(t) => quote! { Some(#t) },
                None => quote! { None },
            };
            let p_tok = match player_tok {
                Some(t) => quote! { Some(#t) },
                None => quote! { None },
            };
            quote! {
                AdvancementCondition::PlayerInteractedWithEntity {
                    entity: #e_tok,
                    item: #i_tok,
                    player: #p_tok,
                }
            }
        }
        "minecraft:thrown_item_picked_up_by_entity" => {
            let ent_tok = val.get("entity").map(generate_entity_condition_tokens);
            let item_tok = val.get("item").map(generate_item_condition_tokens);
            let player_tok = val.get("player").map(generate_entity_condition_tokens);
            let e_tok = match ent_tok {
                Some(t) => quote! { Some(#t) },
                None => quote! { None },
            };
            let i_tok = match item_tok {
                Some(t) => quote! { Some(#t) },
                None => quote! { None },
            };
            let p_tok = match player_tok {
                Some(t) => quote! { Some(#t) },
                None => quote! { None },
            };
            quote! {
                AdvancementCondition::ThrownItemPickedUpByEntity {
                    entity: #e_tok,
                    item: #i_tok,
                    player: #p_tok,
                }
            }
        }
        "minecraft:player_sheared_equipment" => {
            let ent_tok = val.get("entity").map(generate_entity_condition_tokens);
            let item_tok = val.get("item").map(generate_item_condition_tokens);
            let e_tok = match ent_tok {
                Some(t) => quote! { Some(#t) },
                None => quote! { None },
            };
            let i_tok = match item_tok {
                Some(t) => quote! { Some(#t) },
                None => quote! { None },
            };
            quote! {
                AdvancementCondition::PlayerShearedEquipment {
                    entity: #e_tok,
                    item: #i_tok,
                }
            }
        }
        "minecraft:started_riding" => {
            let player_tok = val.get("player").map(generate_entity_condition_tokens);
            let p_tok = match player_tok {
                Some(t) => quote! { Some(#t) },
                None => quote! { None },
            };
            quote! { AdvancementCondition::StartedRiding(#p_tok) }
        }
        "minecraft:summoned_entity" => {
            let ent_val = val.get("entity").unwrap_or(val);
            let ent_tok = generate_entity_condition_tokens(ent_val);
            quote! { AdvancementCondition::SummonedEntity(#ent_tok) }
        }
        "minecraft:crafter_recipe_crafted" => {
            let r = val
                .get("recipes")
                .or_else(|| val.get("recipe"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            quote! { AdvancementCondition::CrafterRecipeCrafted(#r) }
        }
        "minecraft:effects_changed" => quote! { AdvancementCondition::EffectsChanged },
        "minecraft:spear_mobs" => {
            let cnt = val.get("count").and_then(|v| v.as_i64()).map(|v| v as i32);
            let c_tok = opt_i32_tokens(cnt);
            quote! { AdvancementCondition::SpearMobs { count: #c_tok } }
        }
        "minecraft:impossible" => quote! { AdvancementCondition::Impossible },
        "minecraft:killed_by_arrow" => quote! { AdvancementCondition::KilledByArrow },
        "minecraft:slept_in_bed" => quote! { AdvancementCondition::SleptInBed },
        "minecraft:entity_killed_player" => quote! { AdvancementCondition::EntityKilledPlayer },
        "minecraft:used_ender_eye" => quote! { AdvancementCondition::UsedEnderEye },
        "minecraft:cured_zombie_villager" => quote! { AdvancementCondition::CuredZombieVillager },
        "minecraft:villager_trade" => quote! { AdvancementCondition::VillagerTrade },
        "minecraft:tick" => quote! { AdvancementCondition::Tick },
        _ => {
            let raw = serde_json::to_string(val).unwrap_or_default();
            quote! { AdvancementCondition::Custom(#raw) }
        }
    }
}

/// Entry point for the code generation of advancements.
///
/// Parses the advancement files from the 26.2 datapack, builds the advancement tree,
/// calculates positions using the Reingold-Tilford algorithm, and generates
/// the final Rust source code.
pub(crate) fn build() -> TokenStream {
    let base_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/datapack/data/minecraft/advancement");
    let mut advancements: BTreeMap<String, AdvancementStruct> = BTreeMap::new();
    collect_advancements(&base_path, &base_path, &mut advancements);

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
        let criteria_names: Vec<&String> = advancement.criteria.keys().collect();
        let criteria_defs = advancement.criteria.iter().map(|(name, crit)| {
            let trigger = &crit.trigger;
            let condition_tokens = generate_condition_tokens(trigger, &crit.conditions);
            quote! {
                AdvancementCriterion {
                    name: #name,
                    trigger: #trigger,
                    conditions: #condition_tokens,
                }
            }
        });
        variants.extend([quote! {
            pub const #format_name: &Self = &Self {
                id: Identifier::vanilla_static(#raw_name),
                parent : #parent,
                send_telemetry : #send_telemetry,
                display : #display,
                reward : &#reward,
                requirements: &[#(#requirements),*],
                criteria: &[#(#criteria_names),*],
                criteria_definitions: &[#(#criteria_defs),*],
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
            pub criteria_definitions: &'static [AdvancementCriterion],
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

            const IDENTIFIERS: [Identifier; #capacity] = [#minecraft_namespaces];

            pub const fn get_identifier_list() -> [Identifier;#capacity] {
                Self::IDENTIFIERS
            }

            pub const fn is_root(&self) -> bool{
                self.parent.is_none()
            }

            pub fn get_criterion(&self, name: &str) -> Option<&'static AdvancementCriterion> {
                self.criteria_definitions.iter().find(|c| c.name == name)
            }

        }
        #advancement_tree
    }
}
