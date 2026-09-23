use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::DataComponentImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{RegistryKey, get_tag_ids};
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::context::command_context::CommandContext;
use crate::errors::command_syntax_error::CommandSyntaxError;
use crate::errors::error_types::CommandErrorType;
use crate::snbt::SnbtParser;
use crate::string_reader::StringReader;
use crate::suggestion::suggestions::{Suggestions, SuggestionsBuilder};

pub const ERROR_UNKNOWN_ITEM: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_ITEM_ID_INVALID,
    translation::java::ARGUMENT_ITEM_ID_INVALID,
);

pub const ERROR_UNKNOWN_TAG: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENTS_ITEM_TAG_UNKNOWN,
    translation::java::ARGUMENTS_ITEM_TAG_UNKNOWN,
);

pub const ERROR_UNKNOWN_COMPONENT: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENTS_ITEM_COMPONENT_UNKNOWN,
    translation::java::ARGUMENTS_ITEM_COMPONENT_UNKNOWN,
);

pub const ERROR_EXPECTED_COMPONENT: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENTS_ITEM_COMPONENT_EXPECTED,
    translation::java::ARGUMENTS_ITEM_COMPONENT_EXPECTED,
);

pub const ERROR_REPEATED_COMPONENT: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENTS_ITEM_COMPONENT_REPEATED,
    translation::java::ARGUMENTS_ITEM_COMPONENT_REPEATED,
);

pub const ERROR_MALFORMED_COMPONENT: CommandErrorType<2> = CommandErrorType::new(
    translation::java::ARGUMENTS_ITEM_COMPONENT_MALFORMED,
    translation::java::ARGUMENTS_ITEM_COMPONENT_MALFORMED,
);

/// A single entry from an item predicate's `[component, !component,
/// component=value, ...]` list.
///
/// `expected: None` is a bare presence check (`minecraft:enchantable` — "has
/// this component at all"); `Some(value)` additionally requires the actual
/// value to match. `negate` inverts whichever of those two checks applies —
/// vanilla's `ComponentPredicateParser` lets `!` prefix any single term in
/// the list.
///
/// This covers the common, concrete cases vanilla's real (post-1.20.5)
/// component grammar supports; it does not implement the separate `{...}`
/// *predicate*-type registry (`minecraft:enchantments`, count ranges, `|`
/// alternatives, ...) that comes after the `[...]` list — since we don't
/// speak that grammar at all, `{...}` isn't consumed here and is left for
/// the command dispatcher to reject as trailing input, rather than
/// misreading it as the pre-1.20.5 raw-NBT predicate it looks similar to.
#[derive(Clone)]
pub struct ComponentConstraint {
    component: DataComponent,
    expected: Option<Box<dyn DataComponentImpl>>,
    negate: bool,
}

impl PartialEq for ComponentConstraint {
    fn eq(&self, other: &Self) -> bool {
        self.component == other.component
            && self.negate == other.negate
            && match (&self.expected, &other.expected) {
                (None, None) => true,
                (Some(a), Some(b)) => a.equal(b.as_ref()),
                _ => false,
            }
    }
}

impl std::fmt::Debug for ComponentConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentConstraint")
            .field("component", &self.component.to_name())
            .field("has_expected_value", &self.expected.is_some())
            .field("negate", &self.negate)
            .finish()
    }
}

impl ComponentConstraint {
    fn matches(&self, stack: &ItemStack) -> bool {
        let actual = get_dyn_component(stack, self.component);
        let holds = match (&self.expected, actual) {
            (None, found) => found.is_some(),
            (Some(expected), Some(found)) => expected.equal(found),
            (Some(_), None) => false,
        };
        holds != self.negate
    }
}

/// Reads `component`'s current value on `stack`, merging the stack's own
/// patch over the item's default components — the same precedence
/// [`ItemStack::get_data_component`] uses, just without needing the
/// concrete Rust type of `component` at compile time (callers here only
/// know it dynamically, from what was parsed).
fn get_dyn_component(
    stack: &ItemStack,
    component: DataComponent,
) -> Option<&dyn DataComponentImpl> {
    for (id, value) in &stack.patch {
        if *id == component {
            return value.as_deref();
        }
    }
    for &(id, value) in stack.item.components {
        if id == component {
            return Some(value);
        }
    }
    None
}

#[derive(Clone, Debug, PartialEq)]
pub enum ItemPredicate {
    Item {
        item: &'static Item,
        components: Vec<ComponentConstraint>,
    },
    Tag {
        item_ids: Vec<u16>,
        components: Vec<ComponentConstraint>,
    },
    Any,
}

impl ItemPredicate {
    #[must_use]
    pub fn test(&self, stack: &ItemStack) -> bool {
        let (type_matches, components) = match self {
            Self::Any => return true,
            Self::Item { item, components } => (stack.item.id == item.id, components),
            Self::Tag {
                item_ids,
                components,
            } => (item_ids.contains(&stack.item.id), components),
        };
        type_matches && components.iter().all(|c| c.matches(stack))
    }
}

fn parse_components(
    reader: &mut StringReader,
) -> Result<Vec<ComponentConstraint>, CommandSyntaxError> {
    let mut components = Vec::new();
    if reader.peek() != Some('[') {
        return Ok(components);
    }
    reader.skip();
    reader.skip_whitespace();

    while reader.can_read_char() && reader.peek() != Some(']') {
        let negate = if reader.peek() == Some('!') {
            reader.skip();
            true
        } else {
            false
        };

        let start = reader.cursor();
        while let Some(c) = reader.peek() {
            if c.is_alphanumeric() || c == '_' || c == ':' || c == '.' || c == '-' {
                reader.skip();
            } else {
                break;
            }
        }
        let key = reader.string()[start..reader.cursor()].to_string();
        if key.is_empty() {
            return Err(ERROR_EXPECTED_COMPONENT.create(reader));
        }
        let component = DataComponent::try_from_name(&key).ok_or_else(|| {
            ERROR_UNKNOWN_COMPONENT.create(reader, TextComponent::text(key.clone()))
        })?;
        if components
            .iter()
            .any(|c: &ComponentConstraint| c.component == component)
        {
            return Err(ERROR_REPEATED_COMPONENT.create(reader, TextComponent::text(key)));
        }

        reader.skip_whitespace();
        let expected = if reader.peek() == Some('=') {
            reader.skip();
            reader.skip_whitespace();
            let value_start = reader.cursor();
            let value = SnbtParser::parse_for_commands(reader)?;
            let decoded = pumpkin_data::data_component_impl::read_data(component, &value)
                .ok_or_else(|| {
                    // The component name is already resolved at this point — a
                    // failure here means the value doesn't decode into what
                    // that component expects, not that the component is
                    // unknown, so this needs vanilla's malformed-component
                    // error rather than a repeat of the unknown-component one.
                    let raw_value = reader.string()[value_start..reader.cursor()].to_string();
                    ERROR_MALFORMED_COMPONENT.create(
                        reader,
                        TextComponent::text(key.clone()),
                        TextComponent::text(raw_value),
                    )
                })?;
            Some(decoded)
        } else {
            None
        };

        components.push(ComponentConstraint {
            component,
            expected,
            negate,
        });

        reader.skip_whitespace();
        if reader.peek() == Some(',') {
            reader.skip();
            reader.skip_whitespace();
        } else {
            break;
        }
    }

    reader.expect(']')?;
    Ok(components)
}

pub struct ItemPredicateArgumentType;

impl<S: crate::source::CommandSource> ArgumentType<S> for ItemPredicateArgumentType {
    type Item = ItemPredicate;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        if reader.peek() == Some('#') {
            reader.skip();
            let start = reader.cursor();
            while let Some(c) = reader.peek() {
                if c.is_alphanumeric() || c == '_' || c == ':' || c == '/' || c == '.' || c == '-' {
                    reader.skip();
                } else {
                    break;
                }
            }
            let tag_str = &reader.string()[start..reader.cursor()];
            let normalized = if tag_str.contains(':') {
                tag_str.to_string()
            } else {
                format!("minecraft:{tag_str}")
            };
            let stripped = normalized.strip_prefix("minecraft:").unwrap_or(&normalized);

            let tag_ids = get_tag_ids(RegistryKey::Item, &normalized)
                .or_else(|| get_tag_ids(RegistryKey::Item, stripped));

            let item_ids = match tag_ids {
                Some(ids) => ids.to_vec(),
                None => {
                    return Err(ERROR_UNKNOWN_TAG.create(reader, TextComponent::text(normalized)));
                }
            };

            let components = parse_components(reader)?;

            Ok(ItemPredicate::Tag {
                item_ids,
                components,
            })
        } else {
            let start = reader.cursor();
            while let Some(c) = reader.peek() {
                if c.is_alphanumeric()
                    || c == '_'
                    || c == ':'
                    || c == '/'
                    || c == '.'
                    || c == '-'
                    || c == '*'
                {
                    reader.skip();
                } else {
                    break;
                }
            }
            let item_str = &reader.string()[start..reader.cursor()];
            if item_str == "*" {
                return Ok(ItemPredicate::Any);
            }

            let normalized = if item_str.contains(':') {
                item_str.to_string()
            } else {
                format!("minecraft:{item_str}")
            };
            let stripped = normalized.strip_prefix("minecraft:").unwrap_or(&normalized);

            let item = Item::from_registry_key(&normalized)
                .or_else(|| Item::from_registry_key(stripped))
                .ok_or_else(|| {
                    ERROR_UNKNOWN_ITEM.create(reader, TextComponent::text(normalized))
                })?;

            let components = parse_components(reader)?;

            Ok(ItemPredicate::Item { item, components })
        }
    }

    fn client_side_parser(&self) -> JavaClientArgumentType {
        JavaClientArgumentType::ItemPredicate
    }

    fn examples(&self) -> Vec<String> {
        vec![
            "stick".to_string(),
            "minecraft:stick".to_string(),
            "#stick".to_string(),
            "stick[minecraft:enchantments]".to_string(),
        ]
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        builder.build()
    }
}

impl ItemPredicateArgumentType {
    pub fn get<S: crate::source::CommandSource>(
        context: &CommandContext<S>,
        name: &str,
    ) -> Result<ItemPredicate, CommandSyntaxError> {
        Ok(context.get_argument::<ItemPredicate>(name)?.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string_reader::StringReader;

    fn parse(input: &str) -> Result<ItemPredicate, CommandSyntaxError> {
        let mut reader = StringReader::new(input);
        ArgumentType::<crate::source::DummySource>::parse(&ItemPredicateArgumentType, &mut reader)
    }

    #[test]
    fn a_bare_item_name_matches_only_that_item() {
        let predicate = parse("minecraft:stick").unwrap();
        assert!(predicate.test(&ItemStack::new(1, &Item::STICK)));
        assert!(!predicate.test(&ItemStack::new(1, &Item::DIAMOND)));
    }

    #[test]
    fn a_lone_star_matches_anything() {
        let predicate = parse("*").unwrap();
        assert!(predicate.test(&ItemStack::new(1, &Item::STICK)));
        assert!(predicate.test(&ItemStack::new(1, &Item::DIAMOND)));
    }

    #[test]
    fn rejects_an_unknown_component_name() {
        assert_parse_err_reset!(
            &mut StringReader::new("stick[not_a_real_component]"),
            ItemPredicateArgumentType,
            &ERROR_UNKNOWN_COMPONENT
        );
    }

    #[test]
    fn rejects_a_repeated_component() {
        assert_parse_err_reset!(
            &mut StringReader::new("stick[minecraft:enchantable,minecraft:enchantable]"),
            ItemPredicateArgumentType,
            &ERROR_REPEATED_COMPONENT
        );
    }

    #[test]
    fn a_bare_component_requires_presence_only() {
        // Real bug this session found and fixed: components used to be
        // parsed (into nothing at all — `skip_components_and_nbt` threw the
        // whole bracket away) and never checked. A stick has no
        // `minecraft:food` component (it isn't edible); a predicate on it
        // would previously have matched anyway since the check was a no-op.
        let predicate = parse("stick[minecraft:enchantments]").unwrap();
        assert!(predicate.test(&ItemStack::new(1, &Item::STICK)));

        let predicate = parse("stick[minecraft:food]").unwrap();
        assert!(!predicate.test(&ItemStack::new(1, &Item::STICK)));
    }

    #[test]
    fn a_negated_component_requires_absence() {
        let predicate = parse("stick[!minecraft:food]").unwrap();
        assert!(predicate.test(&ItemStack::new(1, &Item::STICK)));

        let predicate = parse("stick[!minecraft:enchantments]").unwrap();
        assert!(!predicate.test(&ItemStack::new(1, &Item::STICK)));
    }

    #[test]
    fn a_valued_component_requires_the_exact_value() {
        let predicate = parse("stick[minecraft:max_stack_size=64]").unwrap();
        assert!(predicate.test(&ItemStack::new(1, &Item::STICK)));

        let predicate = parse("stick[minecraft:max_stack_size=1]").unwrap();
        assert!(!predicate.test(&ItemStack::new(1, &Item::STICK)));
    }

    #[test]
    fn tag_form_parses_and_matches_by_membership() {
        let predicate = parse("#minecraft:doors").unwrap();
        assert!(predicate.test(&ItemStack::new(1, &Item::OAK_DOOR)));
        assert!(!predicate.test(&ItemStack::new(1, &Item::STICK)));
    }

    #[test]
    fn a_trailing_curly_brace_is_left_unconsumed() {
        // Real vanilla 26.3 has no raw-NBT predicate syntax for items
        // anymore (`ItemParser` only knows `[components]`) — `{...}` here
        // isn't ours to interpret, so it's left for the dispatcher's own
        // trailing-input check to reject rather than silently accepted as
        // a legacy NBT match.
        let mut reader = StringReader::new("stick{foo:'bar'}");
        ArgumentType::<crate::source::DummySource>::parse(&ItemPredicateArgumentType, &mut reader)
            .unwrap();
        assert_eq!(reader.remaining_part(), "{foo:'bar'}");
    }
}
