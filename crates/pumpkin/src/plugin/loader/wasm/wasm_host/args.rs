use std::{collections::HashMap, sync::Arc};

use pumpkin_util::text::TextComponent;

use crate::{
    command::{
        argument_types::{
            argument_type::JavaClientArgumentType, coordinates::Coordinates,
            entity_anchor::EntityAnchor,
        },
        context::command_context::CommandContext,
        node::attached::AttachedNode,
    },
    entity::player::Player,
};
use pumpkin_util::math::{
    position::BlockPos,
    vector2::Vector2,
    vector3::{Axis, Vector3},
};

#[derive(Clone, Copy, Debug)]
pub enum Number {
    F64(f64),
    F32(f32),
    I32(i32),
    I64(i64),
}

#[derive(Clone, Copy, Debug)]
pub enum NotInBounds {
    LowerBound(Number, Number),
    UpperBound(Number, Number),
}

#[derive(Clone)]
pub enum OwnedArg {
    Entities(Vec<Arc<dyn crate::entity::EntityBase>>),
    Entity(Arc<dyn crate::entity::EntityBase>),
    Players(Vec<Arc<Player>>),
    GameProfiles(Vec<crate::net::GameProfile>),
    BlockPos(pumpkin_util::math::position::BlockPos),
    Pos3D(pumpkin_util::math::vector3::Vector3<f64>),
    Pos2D(pumpkin_util::math::vector2::Vector2<f64>),
    Rotation(f32, bool, f32, bool),
    GameMode(pumpkin_util::GameMode),
    Difficulty(pumpkin_util::Difficulty),
    Item(String),
    ItemPredicate(String),
    ResourceLocation(String),
    Block(String),
    BlockPredicate(String),
    BossbarColor(crate::world::bossbar::BossbarColor),
    BossbarStyle(crate::world::bossbar::BossbarDivisions),
    Particle(pumpkin_data::particle::Particle),
    Msg(String),
    TextComponent(TextComponent),
    Time(i32),
    Num(Result<Number, NotInBounds>),
    Bool(bool),
    Simple(String),
    SoundCategory(pumpkin_data::sound::SoundCategory),
    DamageType(pumpkin_data::damage::DamageType),
    Effect(&'static pumpkin_data::effect::StatusEffect),
    Enchantment(&'static pumpkin_data::Enchantment),
    EntityAnchor(EntityAnchor),
    Advancement(&'static pumpkin_data::Advancement),
}

#[must_use]
pub fn build_consumed_args_from_context(context: &CommandContext) -> HashMap<String, OwnedArg> {
    let mut map = HashMap::new();
    for (name, parsed) in &context.arguments {
        let res = &parsed.result;
        if let Some(&b) = res.downcast_ref::<bool>() {
            map.insert(name.clone(), OwnedArg::Bool(b));
        } else if let Some(&i) = res.downcast_ref::<i32>() {
            map.insert(name.clone(), OwnedArg::Num(Ok(Number::I32(i))));
        } else if let Some(&i) = res.downcast_ref::<i64>() {
            map.insert(name.clone(), OwnedArg::Num(Ok(Number::I64(i))));
        } else if let Some(&f) = res.downcast_ref::<f32>() {
            map.insert(name.clone(), OwnedArg::Num(Ok(Number::F32(f))));
        } else if let Some(&f) = res.downcast_ref::<f64>() {
            map.insert(name.clone(), OwnedArg::Num(Ok(Number::F64(f))));
        } else if let Some(s) = res.downcast_ref::<String>() {
            map.insert(name.clone(), OwnedArg::Simple(s.clone()));
        } else if let Some(&pos) = res.downcast_ref::<pumpkin_util::math::position::BlockPos>() {
            map.insert(name.clone(), OwnedArg::BlockPos(pos));
        } else if let Some(&v) = res.downcast_ref::<pumpkin_util::math::vector3::Vector3<f64>>() {
            map.insert(name.clone(), OwnedArg::Pos3D(v));
        } else if let Some(&v) = res.downcast_ref::<pumpkin_util::math::vector2::Vector2<f64>>() {
            map.insert(name.clone(), OwnedArg::Pos2D(v));
        } else if let Some(&mode) = res.downcast_ref::<pumpkin_util::GameMode>() {
            map.insert(name.clone(), OwnedArg::GameMode(mode));
        } else if let Some(&diff) = res.downcast_ref::<pumpkin_util::Difficulty>() {
            map.insert(name.clone(), OwnedArg::Difficulty(diff));
        } else if let Some(t) = res.downcast_ref::<TextComponent>() {
            map.insert(name.clone(), OwnedArg::TextComponent(t.clone()));
        } else if let Some(&anchor) = res.downcast_ref::<EntityAnchor>() {
            map.insert(name.clone(), OwnedArg::EntityAnchor(anchor));
        } else if let Some(&coords) = res.downcast_ref::<Coordinates>() {
            // `block_pos`, `column_pos`, `vec2`, `vec3` and `rotation` all parse
            // to `Coordinates` and the original type cannot be detected here
            if let Some(arg) = coordinates_to_owned_arg(context, name, coords) {
                map.insert(name.clone(), arg);
            }
        } else if let Some(selector) =
            res.downcast_ref::<crate::command::argument_types::entity_selector::EntitySelector>()
        {
            if let Ok(players) = selector.find_players(&context.source) {
                map.insert(name.clone(), OwnedArg::Players(players));
            } else if let Ok(entities) = selector.find_entities(&context.source) {
                map.insert(name.clone(), OwnedArg::Entities(entities));
            }
        }
    }
    map
}

/// The Java client-side parser declared for argument `name`
fn declared_parser(context: &CommandContext, name: &str) -> Option<JavaClientArgumentType> {
    // 同名参数由最后一次解析覆盖，声明类型也必须取实际路径中的最后一个节点。
    context
        .nodes
        .iter()
        .rev()
        .find_map(|parsed| match &context.tree[parsed.node] {
            AttachedNode::Argument(argument) if argument.meta.name == name => {
                Some(argument.meta.argument_type.client_side_parser())
            }
            _ => None,
        })
}

/// Converts a parsed `Coordinates` into the `OwnedArg` shape matching the
/// argument type it was declared as
fn coordinates_to_owned_arg(
    context: &CommandContext,
    name: &str,
    coords: Coordinates,
) -> Option<OwnedArg> {
    let source = context.source.as_ref();
    match declared_parser(context, name)? {
        JavaClientArgumentType::BlockPos => {
            let resolved: Vector3<f64> = coords.resolve(source);
            Some(OwnedArg::BlockPos(BlockPos::floored_v(resolved)))
        }
        JavaClientArgumentType::ColumnPos => {
            let resolved: Vector3<f64> = coords.resolve(source);
            let pos = BlockPos::floored_v(resolved);
            Some(OwnedArg::Pos2D(Vector2::new(
                f64::from(pos.0.x),
                f64::from(pos.0.z),
            )))
        }
        JavaClientArgumentType::Vec3 => Some(OwnedArg::Pos3D(coords.resolve(source))),
        JavaClientArgumentType::Vec2 => {
            let resolved: Vector3<f64> = coords.resolve(source);
            Some(OwnedArg::Pos2D(Vector2::new(resolved.x, resolved.z)))
        }
        JavaClientArgumentType::Rotation => {
            let rotation = coords.rotation(source);
            Some(OwnedArg::Rotation(
                rotation.x,
                coords.is_relative(Axis::X),
                rotation.y,
                coords.is_relative(Axis::Y),
            ))
        }
        _ => None,
    }
}

pub struct ConsumedArgsResource {
    pub provider: HashMap<String, OwnedArg>,
}

#[cfg(test)]
mod tests {
    use super::{OwnedArg, build_consumed_args_from_context, declared_parser};
    use crate::command::{
        argument_builder::{ArgumentBuilder, argument, command},
        argument_types::coordinates::{block_pos::BlockPosArgumentType, vec3::Vec3ArgumentType},
        context::command_source::CommandSource,
        dispatcher::CommandDispatcher,
    };
    use pumpkin_util::math::vector3::Vector3;

    #[test]
    fn coordinates_use_the_last_parsed_declaration_when_names_collide() {
        let mut dispatcher = CommandDispatcher::new();
        dispatcher.register(
            command("probe", "test command").then(
                argument("pos", BlockPosArgumentType)
                    .then(argument("pos", Vec3ArgumentType::Uncorrected)),
            ),
        );
        let source = CommandSource::dummy();
        let input = "probe 1 2 3 4.25 5.5 -6.75";
        let parsed = dispatcher.parse_input(input, &source);
        assert!(parsed.errors.is_empty());
        assert!(!parsed.reader.can_read_char());
        let context = parsed.context.build(input);

        // 同一路径中的后一个 pos 覆盖前值，转换时不能再按前一个方块坐标取整。
        assert_eq!(context.arguments.len(), 1);
        let args = build_consumed_args_from_context(&context);
        assert!(matches!(
            args.get("pos"),
            Some(OwnedArg::Pos3D(pos)) if *pos == Vector3::new(4.25, 5.5, -6.75)
        ));
    }

    #[test]
    fn coordinates_without_parsed_nodes_do_not_use_tree_fallback() {
        let mut dispatcher = CommandDispatcher::new();
        dispatcher.register(
            command("probe", "test command").then(argument("pos", Vec3ArgumentType::Uncorrected)),
        );
        let source = CommandSource::dummy();
        let input = "probe 1.25 2.5 3.75";
        let parsed = dispatcher.parse_input(input, &source);
        assert!(parsed.errors.is_empty());
        assert!(!parsed.reader.can_read_char());
        let mut context = parsed.context.build(input);
        assert!(build_consumed_args_from_context(&context).contains_key("pos"));

        // 即使命令树和值仍在，没有实际解析节点也不能从全树猜测参数类型。
        context.nodes.clear();
        assert!(declared_parser(&context, "pos").is_none());
        assert!(!build_consumed_args_from_context(&context).contains_key("pos"));
    }
}
