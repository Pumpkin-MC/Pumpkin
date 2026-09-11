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

use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::snbt::SnbtParser;
use crate::command::string_reader::StringReader;

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
                // SAFETY: We traverse disjoint paths in the NBT tree.
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

impl ArgumentType for NbtPathArgumentType {
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
    pub fn get<'a>(
        context: &'a CommandContext,
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
