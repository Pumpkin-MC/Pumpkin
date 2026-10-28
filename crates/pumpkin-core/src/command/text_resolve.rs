//! Server-side resolution of `score`, `nbt` and `selector` text components.
//!
//! Like vanilla's `ComponentUtils.updateForEntity`. The client cannot resolve these
//! itself, so commands such as `tellraw` and `title` run this before sending.

use std::borrow::Cow;
use std::sync::Arc;

use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::text::{TextComponent, TextComponentBase, TextContent};

use crate::command::CommandSource;
use crate::command::argument_types::argument_type::ArgumentType;
use crate::command::argument_types::entity_selector::parser::EntitySelectorParser;
use crate::command::argument_types::nbt_path::NbtPathArgumentType;
use crate::command::commands::data::{DataAccessor, EntityDataAccessor, StorageDataAccessor};
use crate::command::string_reader::StringReader;
use crate::entity::EntityBase;

/// Vanilla stops resolving nested components past this depth.
const MAX_DEPTH: usize = 100;

/// Resolves every `score`, `nbt` and `selector` component in `component`.
///
/// `target` is the entity the text is shown to; vanilla uses it for the `*` score holder.
#[must_use]
pub fn resolve_component(
    component: &TextComponent,
    source: &CommandSource,
    target: Option<&Arc<dyn EntityBase>>,
) -> TextComponent {
    TextComponent(resolve_base(&component.0, source, target, 0))
}

fn resolve_base(
    base: &TextComponentBase,
    source: &CommandSource,
    target: Option<&Arc<dyn EntityBase>>,
    depth: usize,
) -> TextComponentBase {
    if depth > MAX_DEPTH {
        return base.clone();
    }
    let mut resolved = base.clone();
    let mut prefix = Vec::new();
    match &*base.content {
        TextContent::Score { score } => {
            let text = resolve_score(&score.name, &score.objective, source, target);
            resolved.content = Box::new(TextContent::Text { text: text.into() });
        }
        TextContent::Nbt {
            nbt,
            interpret,
            storage,
            entity,
            ..
        } => {
            let values = resolve_nbt(nbt, storage.as_deref(), entity.as_deref(), source);
            if *interpret {
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        prefix.push(TextComponent::text(", ").0);
                    }
                    let parsed = interpret_tag(value);
                    prefix.push(resolve_base(&parsed.0, source, target, depth + 1));
                }
                resolved.content = Box::new(TextContent::Text {
                    text: Cow::Borrowed(""),
                });
            } else {
                let text: Vec<String> = values.iter().map(tag_to_plain).collect();
                resolved.content = Box::new(TextContent::Text {
                    text: text.join(", ").into(),
                });
            }
        }
        TextContent::EntityNames { selector, .. } => {
            let entities = find_entities(selector, source).unwrap_or_default();
            for (i, entity) in entities.iter().enumerate() {
                if i > 0 {
                    prefix.push(TextComponent::text(", ").0);
                }
                prefix.push(entity.get_display_name().0);
            }
            resolved.content = Box::new(TextContent::Text {
                text: Cow::Borrowed(""),
            });
        }
        TextContent::Translate {
            translate,
            bedrock_translate,
            with,
            fallback,
        } => {
            resolved.content = Box::new(TextContent::Translate {
                translate: translate.clone(),
                bedrock_translate: bedrock_translate.clone(),
                fallback: fallback.clone(),
                with: with
                    .iter()
                    .map(|arg| resolve_base(arg, source, target, depth + 1))
                    .collect(),
            });
        }
        _ => {}
    }
    let extra = base
        .extra
        .iter()
        .map(|child| resolve_base(child, source, target, depth + 1));
    resolved.extra = prefix.into_iter().chain(extra).collect();
    resolved
}

fn find_entities(selector: &str, source: &CommandSource) -> Option<Vec<Arc<dyn EntityBase>>> {
    let mut reader = StringReader::new(selector);
    let selector = EntitySelectorParser::new(&mut reader, true)
        .parse_and_consume()
        .ok()?;
    selector.find_entities(source).ok()
}

fn resolve_score(
    name: &str,
    objective: &str,
    source: &CommandSource,
    target: Option<&Arc<dyn EntityBase>>,
) -> String {
    let holder = if name == "*" {
        match target {
            Some(entity) => entity.get_scoreboard_name(),
            None => return String::new(),
        }
    } else if name.starts_with('@') {
        match find_entities(name, source).as_deref() {
            Some([entity]) => entity.get_scoreboard_name(),
            _ => return String::new(),
        }
    } else {
        name.to_string()
    };
    let Some(world) = source.world.clone().or_else(|| {
        source
            .server
            .as_ref()
            .and_then(|server| server.worlds.load().first().cloned())
    }) else {
        return String::new();
    };
    let scoreboard = world
        .scoreboard
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    scoreboard
        .get_score_value(&holder, objective)
        .map(|value| value.to_string())
        .unwrap_or_default()
}

fn resolve_nbt(
    path: &str,
    storage: Option<&str>,
    entity: Option<&str>,
    source: &CommandSource,
) -> Vec<NbtTag> {
    let mut reader = StringReader::new(path);
    let Ok(path) = <NbtPathArgumentType as ArgumentType<CommandSource>>::parse(
        &NbtPathArgumentType,
        &mut reader,
    ) else {
        return Vec::new();
    };
    let compounds = if let Some(storage) = storage {
        let Some(server) = source.server.clone() else {
            return Vec::new();
        };
        let id = if storage.contains(':') {
            storage.to_string()
        } else {
            format!("minecraft:{storage}")
        };
        StorageDataAccessor::new(id, server)
            .get_data()
            .map(|data| vec![data])
            .unwrap_or_default()
    } else if let Some(selector) = entity {
        find_entities(selector, source)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|entity| EntityDataAccessor::new(entity).get_data().ok())
            .collect()
    } else {
        // Block sources are not resolved yet.
        Vec::new()
    };
    compounds
        .into_iter()
        .flat_map(|compound| path.get(&NbtTag::Compound(compound)).unwrap_or_default())
        .collect()
}

/// Plain text of an NBT value, as vanilla prints it without `interpret`.
fn tag_to_plain(tag: &NbtTag) -> String {
    match tag {
        NbtTag::String(value) => value.to_string(),
        other => other.to_string(),
    }
}

/// Decodes an NBT value as a text component, like vanilla's `interpret: true`.
/// Compounds and lists are components already; strings may hold JSON from
/// older datapacks.
fn interpret_tag(tag: &NbtTag) -> TextComponent {
    match tag {
        NbtTag::String(value) => serde_json::from_str::<TextComponent>(value)
            .unwrap_or_else(|_| TextComponent::text(value.to_string())),
        other => TextComponent::from_nbt(other),
    }
}
