#![allow(
    clippy::borrow_as_ptr,
    clippy::ref_as_ptr,
    clippy::manual_let_else,
    clippy::undocumented_unsafe_blocks,
    clippy::explicit_counter_loop,
    clippy::collapsible_if,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::redundant_closure_for_method_calls,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::missing_const_for_fn,
    clippy::bool_to_int_with_if,
    clippy::if_not_else
)]

use pumpkin_data::translation;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::text::TextComponent;

use crate::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::context::command_context::CommandContext;
use crate::errors::command_syntax_error::CommandSyntaxError;
use crate::errors::error_types::CommandErrorType;
use crate::snbt::SnbtParser;
use crate::string_reader::StringReader;

pub const ERROR_INVALID_NODE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENTS_NBTPATH_NODE_INVALID,
    translation::java::ARGUMENTS_NBTPATH_NODE_INVALID,
);

pub const ERROR_DATA_TOO_DEEP: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENTS_NBTPATH_TOO_DEEP,
    translation::java::ARGUMENTS_NBTPATH_TOO_DEEP,
);

pub const ERROR_NOTHING_FOUND: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENTS_NBTPATH_NOTHING_FOUND,
    translation::java::ARGUMENTS_NBTPATH_NOTHING_FOUND,
);

pub const ERROR_EXPECTED_LIST: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DATA_MODIFY_EXPECTED_LIST,
    translation::java::COMMANDS_DATA_MODIFY_EXPECTED_LIST,
);

pub const ERROR_INVALID_INDEX: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DATA_MODIFY_INVALID_INDEX,
    translation::java::COMMANDS_DATA_MODIFY_INVALID_INDEX,
);

/// Checks if an NBT tag exceeds the maximum nesting depth (512).
pub fn is_too_deep(tag: &NbtTag, depth: usize) -> bool {
    if depth >= 512 {
        return true;
    }
    match tag {
        NbtTag::Compound(compound) => {
            for child in compound.child_tags.values() {
                if is_too_deep(child, depth + 1) {
                    return true;
                }
            }
        }
        NbtTag::List(list) => {
            for child in list {
                if is_too_deep(child, depth + 1) {
                    return true;
                }
            }
        }
        _ => {}
    }
    false
}

/// Recursively compares whether `target` satisfies the `pattern`.
pub fn compare_nbt(pattern: &NbtTag, target: &NbtTag) -> bool {
    if pattern == target {
        return true;
    }
    match (pattern, target) {
        (NbtTag::Compound(pattern_compound), NbtTag::Compound(target_compound)) => {
            for (key, pattern_value) in &pattern_compound.child_tags {
                match target_compound.child_tags.get(key) {
                    Some(target_value) => {
                        if !compare_nbt(pattern_value, target_value) {
                            return false;
                        }
                    }
                    None => return false,
                }
            }
            true
        }
        (NbtTag::List(pattern_list), NbtTag::List(target_list)) => {
            if pattern_list.is_empty() {
                return target_list.is_empty();
            }
            // Matches vanilla's `NbtUtils.compareNbt`: a pattern element can
            // match any target element without "consuming" it, so a target
            // shorter than the pattern could otherwise let one target
            // element satisfy several pattern elements at once (e.g.
            // pattern `[A, A]` against target `[A]`) — reject that up
            // front, the same way vanilla's own size check does, rather
            // than let the search below quietly allow it.
            if target_list.len() < pattern_list.len() {
                return false;
            }
            for pattern_elem in pattern_list {
                let mut matched = false;
                for target_elem in target_list {
                    if compare_nbt(pattern_elem, target_elem) {
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    return false;
                }
            }
            true
        }
        _ => pattern == target,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NbtPathNode {
    AllElements,
    CompoundChild(String),
    IndexedElement(i32),
    MatchElement(NbtCompound),
    MatchObject(String, NbtCompound),
    MatchRootObject(NbtCompound),
}

impl NbtPathNode {
    pub fn get_tag(&self, parent: &NbtTag, output: &mut Vec<NbtTag>) {
        match self {
            Self::AllElements => match parent {
                NbtTag::List(list) => output.extend(list.iter().cloned()),
                NbtTag::ByteArray(arr) => {
                    output.extend(arr.iter().map(|&b| NbtTag::Byte(b)));
                }
                NbtTag::IntArray(arr) => {
                    output.extend(arr.iter().map(|&i| NbtTag::Int(i)));
                }
                NbtTag::LongArray(arr) => {
                    output.extend(arr.iter().map(|&l| NbtTag::Long(l)));
                }
                _ => {}
            },
            Self::CompoundChild(name) => {
                if let NbtTag::Compound(compound) = parent {
                    if let Some(tag) = compound.child_tags.get(name.as_str()) {
                        output.push(tag.clone());
                    }
                }
            }
            Self::IndexedElement(index) => match parent {
                NbtTag::List(list) => {
                    let size = list.len() as i32;
                    let actual_index = if *index < 0 { size + index } else { *index };
                    if actual_index >= 0 && (actual_index as usize) < list.len() {
                        output.push(list[actual_index as usize].clone());
                    }
                }
                NbtTag::ByteArray(arr) => {
                    let size = arr.len() as i32;
                    let actual_index = if *index < 0 { size + index } else { *index };
                    if actual_index >= 0 && (actual_index as usize) < arr.len() {
                        output.push(NbtTag::Byte(arr[actual_index as usize]));
                    }
                }
                NbtTag::IntArray(arr) => {
                    let size = arr.len() as i32;
                    let actual_index = if *index < 0 { size + index } else { *index };
                    if actual_index >= 0 && (actual_index as usize) < arr.len() {
                        output.push(NbtTag::Int(arr[actual_index as usize]));
                    }
                }
                NbtTag::LongArray(arr) => {
                    let size = arr.len() as i32;
                    let actual_index = if *index < 0 { size + index } else { *index };
                    if actual_index >= 0 && (actual_index as usize) < arr.len() {
                        output.push(NbtTag::Long(arr[actual_index as usize]));
                    }
                }
                _ => {}
            },
            Self::MatchElement(pattern) => {
                if let NbtTag::List(list) = parent {
                    let pattern_tag = NbtTag::Compound(pattern.clone());
                    for elem in list {
                        if compare_nbt(&pattern_tag, elem) {
                            output.push(elem.clone());
                        }
                    }
                }
            }
            Self::MatchObject(name, pattern) => {
                if let NbtTag::Compound(compound) = parent {
                    if let Some(tag) = compound.child_tags.get(name.as_str()) {
                        let pattern_tag = NbtTag::Compound(pattern.clone());
                        if compare_nbt(&pattern_tag, tag) {
                            output.push(tag.clone());
                        }
                    }
                }
            }
            Self::MatchRootObject(pattern) => {
                if let NbtTag::Compound(_) = parent {
                    let pattern_tag = NbtTag::Compound(pattern.clone());
                    if compare_nbt(&pattern_tag, parent) {
                        output.push(parent.clone());
                    }
                }
            }
        }
    }

    pub fn create_preferred_parent_tag(&self) -> NbtTag {
        match self {
            Self::AllElements | Self::IndexedElement(_) | Self::MatchElement(_) => {
                NbtTag::List(Vec::new())
            }
            Self::CompoundChild(_) | Self::MatchObject(_, _) | Self::MatchRootObject(_) => {
                NbtTag::Compound(NbtCompound::new())
            }
        }
    }

    pub fn set_tag(&self, parent: &mut NbtTag, to_add: &mut impl FnMut() -> NbtTag) -> i32 {
        match self {
            Self::AllElements => {
                if let NbtTag::List(list) = parent {
                    let size = list.len();
                    if size == 0 {
                        list.push(to_add());
                        return 1;
                    }
                    let new_val = to_add();
                    let changed_count = list.iter().filter(|&x| x != &new_val).count() as i32;
                    if changed_count == 0 {
                        return 0;
                    }
                    list.clear();
                    list.push(new_val);
                    for _ in 1..size {
                        list.push(to_add());
                    }
                    changed_count
                } else {
                    0
                }
            }
            Self::CompoundChild(name) => {
                if let NbtTag::Compound(compound) = parent {
                    let new_val = to_add();
                    let prev = compound
                        .child_tags
                        .insert(name.clone().into(), new_val.clone());
                    i32::from(prev.as_ref() != Some(&new_val))
                } else {
                    0
                }
            }
            Self::IndexedElement(index) => {
                if let NbtTag::List(list) = parent {
                    let size = list.len() as i32;
                    let actual_index = if *index < 0 { size + index } else { *index };
                    if actual_index >= 0 && (actual_index as usize) < list.len() {
                        let new_val = to_add();
                        if list[actual_index as usize] != new_val {
                            list[actual_index as usize] = new_val;
                            return 1;
                        }
                    }
                }
                0
            }
            Self::MatchElement(pattern) => {
                let mut changed_count = 0;
                if let NbtTag::List(list) = parent {
                    let pattern_tag = NbtTag::Compound(pattern.clone());
                    if list.is_empty() {
                        list.push(to_add());
                        changed_count += 1;
                    } else {
                        for elem in list.iter_mut() {
                            if compare_nbt(&pattern_tag, elem) {
                                let new_val = to_add();
                                if *elem != new_val {
                                    *elem = new_val;
                                    changed_count += 1;
                                }
                            }
                        }
                    }
                }
                changed_count
            }
            Self::MatchObject(name, pattern) => {
                if let NbtTag::Compound(compound) = parent {
                    let pattern_tag = NbtTag::Compound(pattern.clone());
                    if let Some(curr) = compound.child_tags.get_mut(name.as_str()) {
                        if compare_nbt(&pattern_tag, curr) {
                            let new_val = to_add();
                            if *curr != new_val {
                                *curr = new_val;
                                return 1;
                            }
                        }
                    }
                }
                0
            }
            Self::MatchRootObject(_) => 0,
        }
    }

    pub fn remove_tag(&self, parent: &mut NbtTag) -> i32 {
        match self {
            Self::AllElements => {
                if let NbtTag::List(list) = parent {
                    let size = list.len() as i32;
                    if size > 0 {
                        list.clear();
                        return size;
                    }
                }
                0
            }
            Self::CompoundChild(name) => {
                if let NbtTag::Compound(compound) = parent {
                    if compound.child_tags.remove(name.as_str()).is_some() {
                        return 1;
                    }
                }
                0
            }
            Self::IndexedElement(index) => {
                if let NbtTag::List(list) = parent {
                    let size = list.len() as i32;
                    let actual_index = if *index < 0 { size + index } else { *index };
                    if actual_index >= 0 && (actual_index as usize) < list.len() {
                        list.remove(actual_index as usize);
                        return 1;
                    }
                }
                0
            }
            Self::MatchElement(pattern) => {
                let mut changed_count = 0;
                if let NbtTag::List(list) = parent {
                    let pattern_tag = NbtTag::Compound(pattern.clone());
                    let mut i = 0;
                    while i < list.len() {
                        if compare_nbt(&pattern_tag, &list[i]) {
                            list.remove(i);
                            changed_count += 1;
                        } else {
                            i += 1;
                        }
                    }
                }
                changed_count
            }
            Self::MatchObject(name, pattern) => {
                if let NbtTag::Compound(compound) = parent {
                    let pattern_tag = NbtTag::Compound(pattern.clone());
                    if let Some(curr) = compound.child_tags.get(name.as_str()) {
                        if compare_nbt(&pattern_tag, curr) {
                            compound.child_tags.remove(name.as_str());
                            return 1;
                        }
                    }
                }
                0
            }
            Self::MatchRootObject(_) => 0,
        }
    }
}

/// Represents a parsed NBT path.
#[derive(Clone, Debug, PartialEq)]
pub struct NbtPath {
    original: String,
    nodes: Vec<NbtPathNode>,
    node_to_original_position: Vec<usize>,
}

impl NbtPath {
    #[must_use]
    pub fn new(
        original: String,
        nodes: Vec<NbtPathNode>,
        node_to_original_position: Vec<usize>,
    ) -> Self {
        Self {
            original,
            nodes,
            node_to_original_position,
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.original
    }

    #[must_use]
    pub fn nodes(&self) -> &[NbtPathNode] {
        &self.nodes
    }

    fn create_not_found_exception(&self, node_index: usize) -> CommandSyntaxError {
        let index = self
            .node_to_original_position
            .get(node_index)
            .copied()
            .unwrap_or(self.original.len());
        ERROR_NOTHING_FOUND
            .create_without_context(TextComponent::text(self.original[..index].to_string()))
    }

    /// Gets matching tags from the given root tag.
    pub fn get(&self, tag: &NbtTag) -> Result<Vec<NbtTag>, CommandSyntaxError> {
        let mut current = vec![tag.clone()];
        for (i, node) in self.nodes.iter().enumerate() {
            let mut next = Vec::new();
            for parent in &current {
                node.get_tag(parent, &mut next);
            }
            if next.is_empty() {
                return Err(self.create_not_found_exception(i));
            }
            current = next;
        }
        Ok(current)
    }

    /// Counts how many elements match the path.
    pub fn count_matching(&self, tag: &NbtTag) -> usize {
        let mut current = vec![tag.clone()];
        for node in &self.nodes {
            let mut next = Vec::new();
            for parent in &current {
                node.get_tag(parent, &mut next);
            }
            if next.is_empty() {
                return 0;
            }
            current = next;
        }
        current.len()
    }

    // SAFETY (applies to every `unsafe { &mut *parent_ptr }` fed by this
    // function's return value, in `set`/`insert`/`remove` as well as here):
    // every pointer in `current` at the start of a loop iteration points at
    // a *distinct* location reachable from the root `tag`, and `NbtTag`/
    // `NbtCompound` are plain owned trees (a `Vec`/`HashMap` of owned
    // values, no `Rc`/shared aliasing) — so two different tree positions can
    // never alias the same memory. Mutating through one pointer this
    // iteration therefore can't invalidate another pointer already held for
    // a different position. The one way that guarantee could break is a
    // `Vec`/`HashMap` reallocating out from under an *already-collected*
    // pointer into one of its own elements: this is avoided because a
    // container is only grown (`list.push`, `child_tags.insert`) *before*
    // any pointer into that same container is taken for this iteration,
    // never after (see the `MatchElement`/`MatchObject` `found`/
    // `contains_key` guards below, and the single unconditional push in
    // `AllElements` happening only while the list is still empty). Changing
    // that ordering in any node's match arm — or adding a node kind that
    // doesn't follow it — would reintroduce a dangling-pointer hazard with
    // no compiler check to catch it; `set_on_a_freshly_created_match_element
    // _also_applies_the_final_field` and `set_through_all_elements_then_a
    // _field_does_not_cross_contaminate_siblings` in the tests below exist
    // specifically to catch a regression here.
    fn get_or_create_parents(
        &self,
        tag: &mut NbtTag,
    ) -> Result<Vec<*mut NbtTag>, CommandSyntaxError> {
        let mut current: Vec<*mut NbtTag> = vec![tag as *mut NbtTag];
        for i in 0..self.nodes.len().saturating_sub(1) {
            let node = &self.nodes[i];
            let next_node = &self.nodes[i + 1];
            let mut next = Vec::new();
            for &parent_ptr in &current {
                let parent = unsafe { &mut *parent_ptr };
                match node {
                    NbtPathNode::CompoundChild(name) => {
                        if let NbtTag::Compound(compound) = parent {
                            if !compound.child_tags.contains_key(name.as_str()) {
                                compound.child_tags.insert(
                                    name.clone().into(),
                                    next_node.create_preferred_parent_tag(),
                                );
                            }
                            if let Some(child) = compound.child_tags.get_mut(name.as_str()) {
                                next.push(child as *mut NbtTag);
                            }
                        }
                    }
                    NbtPathNode::AllElements => {
                        if let NbtTag::List(list) = parent {
                            if list.is_empty() {
                                list.push(next_node.create_preferred_parent_tag());
                            }
                            for elem in list.iter_mut() {
                                next.push(elem as *mut NbtTag);
                            }
                        }
                    }
                    NbtPathNode::IndexedElement(index) => {
                        if let NbtTag::List(list) = parent {
                            let size = list.len() as i32;
                            let actual_index = if *index < 0 { size + index } else { *index };
                            if actual_index >= 0 && (actual_index as usize) < list.len() {
                                next.push(&mut list[actual_index as usize] as *mut NbtTag);
                            }
                        }
                    }
                    NbtPathNode::MatchElement(pattern) => {
                        if let NbtTag::List(list) = parent {
                            let pattern_tag = NbtTag::Compound(pattern.clone());
                            let mut found = false;
                            for elem in list.iter_mut() {
                                if compare_nbt(&pattern_tag, elem) {
                                    next.push(elem as *mut NbtTag);
                                    found = true;
                                }
                            }
                            if !found {
                                list.push(pattern_tag.clone());
                                if let Some(last) = list.last_mut() {
                                    next.push(last as *mut NbtTag);
                                }
                            }
                        }
                    }
                    NbtPathNode::MatchObject(name, pattern) => {
                        if let NbtTag::Compound(compound) = parent {
                            let pattern_tag = NbtTag::Compound(pattern.clone());
                            if !compound.child_tags.contains_key(name.as_str()) {
                                compound
                                    .child_tags
                                    .insert(name.clone().into(), pattern_tag.clone());
                            }
                            if let Some(child) = compound.child_tags.get_mut(name.as_str()) {
                                if compare_nbt(&pattern_tag, child) {
                                    next.push(child as *mut NbtTag);
                                }
                            }
                        }
                    }
                    NbtPathNode::MatchRootObject(pattern) => {
                        let pattern_tag = NbtTag::Compound(pattern.clone());
                        if compare_nbt(&pattern_tag, parent) {
                            next.push(parent as *mut NbtTag);
                        }
                    }
                }
            }
            if next.is_empty() {
                return Err(self.create_not_found_exception(i));
            }
            current = next;
        }
        Ok(current)
    }

    /// Sets the value at this path on the given root tag.
    pub fn set(&self, tag: &mut NbtTag, to_add: NbtTag) -> Result<i32, CommandSyntaxError> {
        if is_too_deep(&to_add, self.nodes.len()) {
            return Err(ERROR_DATA_TOO_DEEP.create_without_context());
        }
        let parents = self.get_or_create_parents(tag)?;
        if parents.is_empty() {
            return Ok(0);
        }
        let last_node = match self.nodes.last() {
            Some(node) => node,
            None => return Ok(0),
        };
        let mut changed_count = 0;
        for &parent_ptr in &parents {
            // SAFETY: see `get_or_create_parents`, which produced `parents`.
            let parent = unsafe { &mut *parent_ptr };
            let val_clone = to_add.clone();
            changed_count += last_node.set_tag(parent, &mut || val_clone.clone());
        }
        Ok(changed_count)
    }

    /// Inserts elements at the given index into list targets matching this path.
    pub fn insert(
        &self,
        index: i32,
        target: &mut NbtTag,
        to_insert: &[NbtTag],
    ) -> Result<i32, CommandSyntaxError> {
        for tag in to_insert {
            if is_too_deep(tag, self.nodes.len()) {
                return Err(ERROR_DATA_TOO_DEEP.create_without_context());
            }
        }
        let parents = self.get_or_create_parents(target)?;
        let last_node = match self.nodes.last() {
            Some(node) => node,
            None => return Ok(0),
        };

        let mut modified_count = 0;
        for &parent_ptr in &parents {
            // SAFETY: see `get_or_create_parents`, which produced `parents`.
            let parent = unsafe { &mut *parent_ptr };
            let target_tags = match last_node {
                NbtPathNode::CompoundChild(name) => {
                    if let NbtTag::Compound(compound) = parent {
                        if !compound.child_tags.contains_key(name.as_str()) {
                            compound
                                .child_tags
                                .insert(name.clone().into(), NbtTag::List(Vec::new()));
                        }
                        compound.child_tags.get_mut(name.as_str())
                    } else {
                        None
                    }
                }
                NbtPathNode::MatchRootObject(_) => Some(parent),
                _ => None,
            };

            if let Some(target_tag) = target_tags {
                match target_tag {
                    NbtTag::List(list) => {
                        let mut modified = false;
                        let size = list.len() as i32;
                        let mut actual_index = if index < 0 { size + index + 1 } else { index };
                        for source_tag in to_insert {
                            if actual_index < 0 || (actual_index as usize) > list.len() {
                                return Err(ERROR_INVALID_INDEX.create_without_context(
                                    TextComponent::text(actual_index.to_string()),
                                ));
                            }
                            list.insert(actual_index as usize, source_tag.clone());
                            actual_index += 1;
                            modified = true;
                        }
                        if modified {
                            modified_count += 1;
                        }
                    }
                    _ => {
                        return Err(ERROR_EXPECTED_LIST.create_without_context(
                            TextComponent::text(format!("{target_tag:?}")),
                        ));
                    }
                }
            }
        }

        Ok(modified_count)
    }

    /// Removes tags at this path from the given root tag.
    pub fn remove(&self, tag: &mut NbtTag) -> i32 {
        let parents = match self.get_or_create_parents(tag) {
            Ok(p) => p,
            Err(_) => return 0,
        };
        let last_node = match self.nodes.last() {
            Some(node) => node,
            None => return 0,
        };
        let mut total_removed = 0;
        for &parent_ptr in &parents {
            // SAFETY: see `get_or_create_parents`, which produced `parents`.
            let parent = unsafe { &mut *parent_ptr };
            total_removed += last_node.remove_tag(parent);
        }
        total_removed
    }
}

impl std::fmt::Display for NbtPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.original)
    }
}

/// Argument type for parsing NBT Paths.
pub struct NbtPathArgumentType;

impl<S: crate::source::CommandSource> ArgumentType<S> for NbtPathArgumentType {
    type Item = NbtPath;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let mut nodes = Vec::new();
        let start = reader.cursor();
        let mut node_to_original_position = Vec::new();
        let mut first_node = true;

        while reader.can_read_char() && reader.peek() != Some(' ') {
            let node = parse_node(reader, first_node)?;
            nodes.push(node);
            node_to_original_position.push(reader.cursor() - start);
            first_node = false;
            if reader.can_read_char() {
                let next = reader.peek().unwrap();
                if next != ' ' && next != '[' && next != '{' {
                    reader.expect('.')?;
                }
            }
        }

        if nodes.is_empty() {
            return Err(ERROR_INVALID_NODE.create(reader));
        }

        let original = reader.string()[start..reader.cursor()].to_string();
        Ok(NbtPath::new(original, nodes, node_to_original_position))
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::NbtPath
    }

    fn examples(&self) -> Vec<String> {
        examples!("foo", "foo.bar", "foo[0]", "[0]", "[]", "{foo:\"bar\"}")
    }
}

impl NbtPathArgumentType {
    pub fn get<'a, S: crate::source::CommandSource>(
        context: &'a CommandContext<S>,
        name: &'_ str,
    ) -> Result<&'a NbtPath, CommandSyntaxError> {
        context.get_argument(name)
    }
}

fn parse_node(
    reader: &mut StringReader,
    first_node: bool,
) -> Result<NbtPathNode, CommandSyntaxError> {
    let peek = match reader.peek() {
        Some(c) => c,
        None => return Err(ERROR_INVALID_NODE.create(reader)),
    };
    match peek {
        '"' | '\'' => {
            let name = reader.read_string()?;
            read_object_node(reader, name)
        }
        '[' => {
            reader.skip();
            let next = match reader.peek() {
                Some(c) => c,
                None => return Err(ERROR_INVALID_NODE.create(reader)),
            };
            if next == '{' {
                let pattern = parse_compound_pattern(reader)?;
                reader.expect(']')?;
                Ok(NbtPathNode::MatchElement(pattern))
            } else if next == ']' {
                reader.skip();
                Ok(NbtPathNode::AllElements)
            } else {
                let index = reader.read_int()?;
                reader.expect(']')?;
                Ok(NbtPathNode::IndexedElement(index))
            }
        }
        '{' => {
            if !first_node {
                return Err(ERROR_INVALID_NODE.create(reader));
            }
            let pattern = parse_compound_pattern(reader)?;
            Ok(NbtPathNode::MatchRootObject(pattern))
        }
        _ => {
            let name = read_unquoted_name(reader)?;
            read_object_node(reader, name)
        }
    }
}

fn read_object_node(
    reader: &mut StringReader,
    name: String,
) -> Result<NbtPathNode, CommandSyntaxError> {
    if name.is_empty() {
        return Err(ERROR_INVALID_NODE.create(reader));
    }
    if reader.peek() == Some('{') {
        let pattern = parse_compound_pattern(reader)?;
        Ok(NbtPathNode::MatchObject(name, pattern))
    } else {
        Ok(NbtPathNode::CompoundChild(name))
    }
}

fn read_unquoted_name(reader: &mut StringReader) -> Result<String, CommandSyntaxError> {
    let start = reader.cursor();
    while let Some(c) = reader.peek() {
        if is_allowed_in_unquoted_name(c) {
            reader.skip();
        } else {
            break;
        }
    }
    if reader.cursor() == start {
        return Err(ERROR_INVALID_NODE.create(reader));
    }
    Ok(reader.string()[start..reader.cursor()].to_string())
}

const fn is_allowed_in_unquoted_name(c: char) -> bool {
    !matches!(c, ' ' | '"' | '\'' | '[' | ']' | '.' | '{' | '}')
}

fn parse_compound_pattern(reader: &mut StringReader) -> Result<NbtCompound, CommandSyntaxError> {
    match SnbtParser::parse_for_commands(reader)? {
        NbtTag::Compound(compound) => Ok(compound),
        _ => Err(ERROR_INVALID_NODE.create(reader)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string_reader::StringReader;

    fn parse(input: &str) -> Result<NbtPath, CommandSyntaxError> {
        let mut reader = StringReader::new(input);
        ArgumentType::<crate::source::DummySource>::parse(&NbtPathArgumentType, &mut reader)
    }

    // --- parsing ---

    #[test]
    fn parses_a_single_unquoted_name() {
        let path = parse("foo").unwrap();
        assert_eq!(path.nodes(), &[NbtPathNode::CompoundChild("foo".into())]);
    }

    #[test]
    fn parses_a_dotted_chain_of_names() {
        let path = parse("foo.bar.baz").unwrap();
        assert_eq!(
            path.nodes(),
            &[
                NbtPathNode::CompoundChild("foo".into()),
                NbtPathNode::CompoundChild("bar".into()),
                NbtPathNode::CompoundChild("baz".into()),
            ]
        );
    }

    #[test]
    fn parses_a_quoted_name_containing_a_space() {
        let path = parse("\"foo bar\"").unwrap();
        assert_eq!(
            path.nodes(),
            &[NbtPathNode::CompoundChild("foo bar".into())]
        );
    }

    #[test]
    fn parses_an_indexed_element_positive_and_negative() {
        let path = parse("foo[0]").unwrap();
        assert_eq!(
            path.nodes(),
            &[
                NbtPathNode::CompoundChild("foo".into()),
                NbtPathNode::IndexedElement(0)
            ]
        );
        let path = parse("foo[-1]").unwrap();
        assert_eq!(
            path.nodes(),
            &[
                NbtPathNode::CompoundChild("foo".into()),
                NbtPathNode::IndexedElement(-1)
            ]
        );
    }

    #[test]
    fn parses_an_all_elements_wildcard() {
        let path = parse("foo[]").unwrap();
        assert_eq!(
            path.nodes(),
            &[
                NbtPathNode::CompoundChild("foo".into()),
                NbtPathNode::AllElements
            ]
        );
    }

    #[test]
    fn parses_a_match_element_pattern() {
        let mut expected = NbtCompound::new();
        expected.put_int("a", 1);
        let path = parse("foo[{a:1}]").unwrap();
        assert_eq!(
            path.nodes(),
            &[
                NbtPathNode::CompoundChild("foo".into()),
                NbtPathNode::MatchElement(expected)
            ]
        );
    }

    #[test]
    fn parses_a_match_object_pattern() {
        let mut expected = NbtCompound::new();
        expected.put_int("a", 1);
        let path = parse("foo{a:1}").unwrap();
        assert_eq!(
            path.nodes(),
            &[NbtPathNode::MatchObject("foo".into(), expected)]
        );
    }

    #[test]
    fn parses_a_match_root_object_pattern_as_the_first_node() {
        let mut expected = NbtCompound::new();
        expected.put_int("a", 1);
        let path = parse("{a:1}").unwrap();
        assert_eq!(path.nodes(), &[NbtPathNode::MatchRootObject(expected)]);
    }

    #[test]
    fn rejects_a_match_root_object_pattern_that_is_not_the_first_node() {
        // Two match-object-shaped nodes back to back with no `.` between
        // them: the second one starts with `{` but isn't the first node of
        // the whole path, which only `MatchRootObject` is allowed to do.
        let mut reader = StringReader::new("foo{a:1}{b:2}");
        assert!(
            ArgumentType::<crate::source::DummySource>::parse(&NbtPathArgumentType, &mut reader)
                .is_err()
        );
    }

    #[test]
    fn rejects_an_empty_path() {
        let mut reader = StringReader::new("");
        assert!(
            ArgumentType::<crate::source::DummySource>::parse(&NbtPathArgumentType, &mut reader)
                .is_err()
        );
    }

    #[test]
    fn rejects_an_unclosed_index() {
        let mut reader = StringReader::new("foo[0");
        assert!(
            ArgumentType::<crate::source::DummySource>::parse(&NbtPathArgumentType, &mut reader)
                .is_err()
        );
    }

    // --- get / count_matching ---

    #[test]
    fn get_returns_the_value_at_a_matching_path() {
        let mut root = NbtCompound::new();
        root.put_int("foo", 42);
        let path = parse("foo").unwrap();
        assert_eq!(
            path.get(&NbtTag::Compound(root)).unwrap(),
            vec![NbtTag::Int(42)]
        );
    }

    #[test]
    fn get_fails_with_nothing_found_on_a_missing_path() {
        let root = NbtCompound::new();
        let path = parse("foo").unwrap();
        assert!(path.get(&NbtTag::Compound(root)).is_err());
    }

    #[test]
    fn count_matching_counts_every_wildcard_hit() {
        let mut root = NbtCompound::new();
        root.put_list(
            "items",
            vec![NbtTag::Int(1), NbtTag::Int(2), NbtTag::Int(3)],
        );
        let path = parse("items[]").unwrap();
        assert_eq!(path.count_matching(&NbtTag::Compound(root)), 3);
    }

    // --- set ---

    #[test]
    fn set_creates_missing_intermediate_compounds() {
        let mut root = NbtTag::Compound(NbtCompound::new());
        let path = parse("a.b.c").unwrap();
        let changed = path.set(&mut root, NbtTag::Int(7)).unwrap();
        assert_eq!(changed, 1);
        assert_eq!(path.get(&root).unwrap(), vec![NbtTag::Int(7)]);
    }

    #[test]
    fn set_on_all_elements_overwrites_every_item_with_the_same_value() {
        let mut compound = NbtCompound::new();
        compound.put_list(
            "items",
            vec![NbtTag::Int(1), NbtTag::Int(2), NbtTag::Int(3)],
        );
        let mut root = NbtTag::Compound(compound);
        let path = parse("items[]").unwrap();

        let changed = path.set(&mut root, NbtTag::Int(9)).unwrap();
        assert_eq!(changed, 3);
        assert_eq!(
            path.get(&root).unwrap(),
            vec![NbtTag::Int(9), NbtTag::Int(9), NbtTag::Int(9)]
        );

        // Setting the same value again changes nothing.
        let changed_again = path.set(&mut root, NbtTag::Int(9)).unwrap();
        assert_eq!(changed_again, 0);
    }

    #[test]
    fn set_on_negative_index_addresses_from_the_end() {
        let mut compound = NbtCompound::new();
        compound.put_list(
            "items",
            vec![NbtTag::Int(1), NbtTag::Int(2), NbtTag::Int(3)],
        );
        let mut root = NbtTag::Compound(compound);
        let path = parse("items[-1]").unwrap();
        path.set(&mut root, NbtTag::Int(99)).unwrap();
        assert_eq!(path.get(&root).unwrap(), vec![NbtTag::Int(99)]);
        // Confirm it was the *last* element, not some other one, that changed.
        let all = parse("items[]").unwrap().get(&root).unwrap();
        assert_eq!(all, vec![NbtTag::Int(1), NbtTag::Int(2), NbtTag::Int(99)]);
    }

    #[test]
    fn set_on_a_freshly_created_match_element_also_applies_the_final_field() {
        // Regression test for the `get_or_create_parents` raw-pointer chain:
        // `list[{a:1}]` on an empty list has to fall back to *creating* a
        // new `{a:1}` element (no existing element matches), and the next
        // node (`.b`) then has to set a field on that same brand-new
        // element through the pointer collected during that fallback. If
        // the pointer ever dangled or aliased a sibling, this would either
        // panic, silently do nothing, or corrupt another element.
        let mut compound = NbtCompound::new();
        compound.put_list("list", Vec::new());
        let mut root = NbtTag::Compound(compound);

        let path = parse("list[{a:1}].b").unwrap();
        let changed = path.set(&mut root, NbtTag::Int(5)).unwrap();
        assert_eq!(changed, 1);

        let NbtTag::Compound(root_compound) = &root else {
            panic!("expected a compound")
        };
        let list = root_compound.get_list("list").unwrap();
        assert_eq!(list.len(), 1);
        let NbtTag::Compound(created) = &list[0] else {
            panic!("expected the fallback-created element to be a compound");
        };
        assert_eq!(created.get_int("a"), Some(1));
        assert_eq!(created.get_int("b"), Some(5));
    }

    #[test]
    fn set_through_all_elements_then_a_field_does_not_cross_contaminate_siblings() {
        // Each list element is its own compound; writing `value` through
        // each of the (disjoint) pointers collected for `[]` must only ever
        // touch that one element's own map, never a sibling's.
        let mut first = NbtCompound::new();
        first.put_int("id", 1);
        let mut second = NbtCompound::new();
        second.put_int("id", 2);
        let mut compound = NbtCompound::new();
        compound.put_list(
            "list",
            vec![NbtTag::Compound(first), NbtTag::Compound(second)],
        );
        let mut root = NbtTag::Compound(compound);

        let path = parse("list[].value").unwrap();
        let changed = path.set(&mut root, NbtTag::Int(100)).unwrap();
        assert_eq!(changed, 2);

        let NbtTag::Compound(root_compound) = &root else {
            panic!("expected a compound")
        };
        let list = root_compound.get_list("list").unwrap();
        for (i, elem) in list.iter().enumerate() {
            let NbtTag::Compound(c) = elem else {
                panic!("expected a compound")
            };
            assert_eq!(c.get_int("id"), Some(i as i32 + 1), "id got clobbered");
            assert_eq!(c.get_int("value"), Some(100));
        }
    }

    // --- insert ---

    #[test]
    fn insert_creates_the_list_if_the_key_did_not_exist() {
        let mut root = NbtTag::Compound(NbtCompound::new());
        let path = parse("items").unwrap();
        let changed = path
            .insert(0, &mut root, &[NbtTag::Int(1), NbtTag::Int(2)])
            .unwrap();
        assert_eq!(changed, 1);
        assert_eq!(
            path.get(&root).unwrap(),
            vec![NbtTag::List(vec![NbtTag::Int(1), NbtTag::Int(2)])]
        );
    }

    #[test]
    fn insert_at_a_negative_index_counts_from_the_end() {
        let mut compound = NbtCompound::new();
        compound.put_list("items", vec![NbtTag::Int(1), NbtTag::Int(3)]);
        let mut root = NbtTag::Compound(compound);
        let path = parse("items").unwrap();
        // Matches vanilla's `NbtPathArgument.Node.insert`: actualIndex =
        // size + index + 1, so -1 is "append at the very end" (size + 1 - 1
        // == size) and -2 is "one slot before that" — not "before the last
        // element" in the everyday negative-index sense.
        path.insert(-2, &mut root, &[NbtTag::Int(2)]).unwrap();
        assert_eq!(
            path.get(&root).unwrap(),
            vec![NbtTag::List(vec![
                NbtTag::Int(1),
                NbtTag::Int(2),
                NbtTag::Int(3)
            ])]
        );
    }

    #[test]
    fn insert_into_a_non_list_target_fails() {
        let mut compound = NbtCompound::new();
        compound.put_int("items", 5);
        let mut root = NbtTag::Compound(compound);
        let path = parse("items").unwrap();
        assert!(path.insert(0, &mut root, &[NbtTag::Int(1)]).is_err());
    }

    #[test]
    fn insert_out_of_range_fails() {
        let mut compound = NbtCompound::new();
        compound.put_list("items", vec![NbtTag::Int(1)]);
        let mut root = NbtTag::Compound(compound);
        let path = parse("items").unwrap();
        assert!(path.insert(5, &mut root, &[NbtTag::Int(2)]).is_err());
    }

    // --- remove ---

    #[test]
    fn remove_deletes_a_compound_child() {
        let mut compound = NbtCompound::new();
        compound.put_int("foo", 1);
        let mut root = NbtTag::Compound(compound);
        let path = parse("foo").unwrap();
        assert_eq!(path.remove(&mut root), 1);
        assert!(path.get(&root).is_err());
    }

    #[test]
    fn remove_all_elements_clears_the_list_and_reports_how_many() {
        let mut compound = NbtCompound::new();
        compound.put_list(
            "items",
            vec![NbtTag::Int(1), NbtTag::Int(2), NbtTag::Int(3)],
        );
        let mut root = NbtTag::Compound(compound);
        let path = parse("items[]").unwrap();
        assert_eq!(path.remove(&mut root), 3);
        let NbtTag::Compound(root_compound) = &root else {
            panic!("expected a compound")
        };
        assert!(root_compound.get_list("items").unwrap().is_empty());
    }

    #[test]
    fn remove_match_element_only_removes_matching_entries() {
        let mut a = NbtCompound::new();
        a.put_int("keep", 1);
        let mut b = NbtCompound::new();
        b.put_int("drop", 1);
        let mut c = NbtCompound::new();
        c.put_int("keep", 1);
        let mut compound = NbtCompound::new();
        compound.put_list(
            "items",
            vec![
                NbtTag::Compound(a),
                NbtTag::Compound(b.clone()),
                NbtTag::Compound(c),
            ],
        );
        let mut root = NbtTag::Compound(compound);

        let path = parse("items[{drop:1}]").unwrap();
        assert_eq!(path.remove(&mut root), 1);

        let NbtTag::Compound(root_compound) = &root else {
            panic!("expected a compound")
        };
        let remaining = root_compound.get_list("items").unwrap();
        assert_eq!(remaining.len(), 2);
        for elem in remaining {
            let NbtTag::Compound(c) = elem else {
                panic!("expected a compound")
            };
            assert_eq!(c.get_int("keep"), Some(1));
        }
    }

    // --- is_too_deep ---

    #[test]
    fn is_too_deep_is_false_for_a_shallow_tag() {
        let mut compound = NbtCompound::new();
        compound.put_int("a", 1);
        assert!(!is_too_deep(&NbtTag::Compound(compound), 0));
    }

    #[test]
    fn is_too_deep_is_true_past_the_512_depth_limit() {
        let mut tag = NbtTag::Compound(NbtCompound::new());
        for _ in 0..600 {
            let mut wrapper = NbtCompound::new();
            wrapper.put("inner", tag);
            tag = NbtTag::Compound(wrapper);
        }
        assert!(is_too_deep(&tag, 0));
    }

    // --- compare_nbt ---

    #[test]
    fn compare_nbt_matches_a_compound_subset() {
        let mut pattern = NbtCompound::new();
        pattern.put_int("a", 1);
        let mut target = NbtCompound::new();
        target.put_int("a", 1);
        target.put_int("b", 2);
        assert!(compare_nbt(
            &NbtTag::Compound(pattern),
            &NbtTag::Compound(target)
        ));
    }

    #[test]
    fn compare_nbt_rejects_a_mismatched_value() {
        let mut pattern = NbtCompound::new();
        pattern.put_int("a", 1);
        let mut target = NbtCompound::new();
        target.put_int("a", 2);
        assert!(!compare_nbt(
            &NbtTag::Compound(pattern),
            &NbtTag::Compound(target)
        ));
    }

    #[test]
    fn compare_nbt_list_pattern_only_needs_each_element_matched_somewhere() {
        let pattern = NbtTag::List(vec![NbtTag::Int(1)]);
        let target = NbtTag::List(vec![NbtTag::Int(1), NbtTag::Int(2)]);
        assert!(compare_nbt(&pattern, &target));
    }

    #[test]
    fn compare_nbt_a_shorter_target_list_cannot_satisfy_a_longer_pattern() {
        // Real bug this session found and fixed: without a length check,
        // the "does some target element match" search lets one target
        // element satisfy multiple pattern elements at once, so `[1, 1]`
        // against a target of just `[1]` would wrongly match twice against
        // the same lone element. Vanilla's `NbtUtils.compareNbt` guards
        // against exactly this with an explicit size check.
        let pattern = NbtTag::List(vec![NbtTag::Int(1), NbtTag::Int(1)]);
        let target = NbtTag::List(vec![NbtTag::Int(1)]);
        assert!(!compare_nbt(&pattern, &target));
    }

    #[test]
    fn compare_nbt_empty_pattern_list_only_matches_an_empty_target_list() {
        let pattern = NbtTag::List(Vec::new());
        assert!(compare_nbt(&pattern, &NbtTag::List(Vec::new())));
        assert!(!compare_nbt(&pattern, &NbtTag::List(vec![NbtTag::Int(1)])));
    }
}
