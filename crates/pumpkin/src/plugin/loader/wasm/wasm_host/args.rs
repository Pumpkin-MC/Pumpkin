use std::{collections::HashMap, sync::Arc};

use pumpkin_util::text::TextComponent;

use crate::{
    command::{
        argument_types::entity_anchor::EntityAnchor, context::command_context::CommandContext,
    },
    entity::player::Player,
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

pub struct ConsumedArgsResource {
    pub provider: HashMap<String, OwnedArg>,
}
