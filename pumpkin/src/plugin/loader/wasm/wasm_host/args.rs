use std::{collections::HashMap, sync::Arc};

use pumpkin_util::text::TextComponent;

use crate::{command::tree::CommandTree, entity::player::Player};

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
    Advancement(&'static pumpkin_data::Advancement),
}

impl OwnedArg {
    #[must_use]
    pub fn from_arg(arg: &crate::command::args::Arg<'_>) -> Self {
        use crate::command::args::Arg;
        match arg {
            Arg::Entities(v) => Self::Entities(v.clone()),
            Arg::Entity(e) => Self::Entity(e.clone()),
            Arg::Players(v) => Self::Players(v.clone()),
            Arg::GameProfiles(v) => Self::GameProfiles(v.clone()),
            Arg::BlockPos(p) => Self::BlockPos(*p),
            Arg::Pos3D(v) => Self::Pos3D(*v),
            Arg::Pos2D(v) => Self::Pos2D(*v),
            Arg::Rotation(a, b, c, d) => Self::Rotation(*a, *b, *c, *d),
            Arg::GameMode(g) => Self::GameMode(*g),
            Arg::Difficulty(d) => Self::Difficulty(*d),
            Arg::CommandTree(t) => Self::CommandTree(t.clone()),
            Arg::Item(s) => Self::Item(s.to_string()),
            Arg::ItemPredicate(s) => Self::ItemPredicate(s.to_string()),
            Arg::ResourceLocation(s) => Self::ResourceLocation(s.to_string()),
            Arg::Block(s) => Self::Block(s.to_string()),
            Arg::BlockPredicate(s) => Self::BlockPredicate(s.to_string()),
            Arg::BossbarColor(c) => Self::BossbarColor(c.clone()),
            Arg::BossbarStyle(s) => Self::BossbarStyle(s.clone()),
            Arg::Particle(p) => Self::Particle(*p),
            Arg::Msg(m) => Self::Msg(m.clone()),
            Arg::TextComponent(t) => Self::TextComponent(t.clone()),
            Arg::Time(t) => Self::Time(*t),
            Arg::Num(n) => Self::Num(*n),
            Arg::Bool(b) => Self::Bool(*b),
            Arg::Simple(s) => Self::Simple(s.to_string()),
            Arg::SoundCategory(s) => Self::SoundCategory(*s),
            Arg::DamageType(d) => Self::DamageType(*d),
            Arg::Effect(e) => Self::Effect(e),
            Arg::Enchantment(e) => Self::Enchantment(e),
            Arg::EntityAnchor(a) => Self::EntityAnchor(*a),
            Arg::Advancement(a) => Self::Advancement(a)
        }
    }
}

pub struct ConsumedArgsResource {
    pub provider: HashMap<String, OwnedArg>,
}
