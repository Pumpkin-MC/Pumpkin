use std::{collections::HashMap, sync::Arc};

use pumpkin_util::text::TextComponent;

use crate::{command::tree::CommandTree, entity::player::Player};

#[derive(Clone)]
pub enum OwnedArg {
    Entities(Vec<Arc<dyn crate::entity::EntityBase>>),
    Entity(Arc<dyn crate::entity::EntityBase>),
    Players(Vec<Arc<Player>>),
    BlockPos(pumpkin_util::math::position::BlockPos),
    Pos3D(pumpkin_util::math::vector3::Vector3<f64>),
    Pos2D(pumpkin_util::math::vector2::Vector2<f64>),
    Rotation(f32, bool, f32, bool),
    GameMode(pumpkin_util::GameMode),
    Difficulty(pumpkin_util::Difficulty),
    CommandTree(CommandTree),
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
    Num(
        Result<
            crate::command::args::bounded_num::Number,
            crate::command::args::bounded_num::NotInBounds,
        >,
    ),
    Bool(bool),
    Simple(String),
    SoundCategory(pumpkin_data::sound::SoundCategory),
    DamageType(pumpkin_data::damage::DamageType),
    Effect(&'static pumpkin_data::effect::StatusEffect),
    Enchantment(&'static pumpkin_data::Enchantment),
    EntityAnchor(crate::command::args::EntityAnchor),
}

impl OwnedArg {
    pub fn from_arg(arg: &crate::command::args::Arg<'_>) -> Self {
        use crate::command::args::Arg;
        match arg {
            Arg::Entities(v) => OwnedArg::Entities(v.clone()),
            Arg::Entity(e) => OwnedArg::Entity(e.clone()),
            Arg::Players(v) => OwnedArg::Players(v.clone()),
            Arg::BlockPos(p) => OwnedArg::BlockPos(*p),
            Arg::Pos3D(v) => OwnedArg::Pos3D(*v),
            Arg::Pos2D(v) => OwnedArg::Pos2D(*v),
            Arg::Rotation(a, b, c, d) => OwnedArg::Rotation(*a, *b, *c, *d),
            Arg::GameMode(g) => OwnedArg::GameMode(*g),
            Arg::Difficulty(d) => OwnedArg::Difficulty(*d),
            Arg::CommandTree(t) => OwnedArg::CommandTree(t.clone()),
            Arg::Item(s) => OwnedArg::Item(s.to_string()),
            Arg::ItemPredicate(s) => OwnedArg::ItemPredicate(s.to_string()),
            Arg::ResourceLocation(s) => OwnedArg::ResourceLocation(s.to_string()),
            Arg::Block(s) => OwnedArg::Block(s.to_string()),
            Arg::BlockPredicate(s) => OwnedArg::BlockPredicate(s.to_string()),
            Arg::BossbarColor(c) => OwnedArg::BossbarColor(c.clone()),
            Arg::BossbarStyle(s) => OwnedArg::BossbarStyle(s.clone()),
            Arg::Particle(p) => OwnedArg::Particle(p.clone()),
            Arg::Msg(m) => OwnedArg::Msg(m.clone()),
            Arg::TextComponent(t) => OwnedArg::TextComponent(t.clone()),
            Arg::Time(t) => OwnedArg::Time(*t),
            Arg::Num(n) => OwnedArg::Num(n.clone()),
            Arg::Bool(b) => OwnedArg::Bool(*b),
            Arg::Simple(s) => OwnedArg::Simple(s.to_string()),
            Arg::SoundCategory(s) => OwnedArg::SoundCategory(*s),
            Arg::DamageType(d) => OwnedArg::DamageType(*d),
            Arg::Effect(e) => OwnedArg::Effect(e),
            Arg::Enchantment(e) => OwnedArg::Enchantment(e),
            Arg::EntityAnchor(a) => OwnedArg::EntityAnchor(*a),
        }
    }
}

pub struct ConsumedArgsResource {
    pub provider: HashMap<String, OwnedArg>,
}
