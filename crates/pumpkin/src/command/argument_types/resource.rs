use crate::command::argument_types::FromStringReader;
use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::{CommandErrorType, DISPATCHER_PARSE_EXCEPTION};
use crate::command::node::attached::AttachedNode;
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_data::entity::EntityType;
use pumpkin_data::translation;
use pumpkin_protocol::java::client::play::SuggestionProviders;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::text::TextComponent;
use std::any::Any;
use std::iter::Iterator;

pub static ENTITY_TYPE_REGISTRY: &Identifier = &Identifier::vanilla_static("entity_type");
pub static ENCHANTMENT_REGISTRY: &Identifier = &Identifier::vanilla_static("enchantment");
pub static MOB_EFFECT_REGISTRY: &Identifier = &Identifier::vanilla_static("mob_effect");
pub static DAMAGE_TYPE_REGISTRY: &Identifier = &Identifier::vanilla_static("damage_type");

static DAMAGE_TYPES: [pumpkin_data::damage::DamageType; 51] = [
    pumpkin_data::damage::DamageType::ARROW,
    pumpkin_data::damage::DamageType::BAD_RESPAWN_POINT,
    pumpkin_data::damage::DamageType::CACTUS,
    pumpkin_data::damage::DamageType::CAMPFIRE,
    pumpkin_data::damage::DamageType::CRAMMING,
    pumpkin_data::damage::DamageType::DRAGON_BREATH,
    pumpkin_data::damage::DamageType::DROWN,
    pumpkin_data::damage::DamageType::DRY_OUT,
    pumpkin_data::damage::DamageType::ENDER_PEARL,
    pumpkin_data::damage::DamageType::EXPLOSION,
    pumpkin_data::damage::DamageType::FALL,
    pumpkin_data::damage::DamageType::FALLING_ANVIL,
    pumpkin_data::damage::DamageType::FALLING_BLOCK,
    pumpkin_data::damage::DamageType::FALLING_STALACTITE,
    pumpkin_data::damage::DamageType::FIREBALL,
    pumpkin_data::damage::DamageType::FIREWORKS,
    pumpkin_data::damage::DamageType::FLY_INTO_WALL,
    pumpkin_data::damage::DamageType::FREEZE,
    pumpkin_data::damage::DamageType::GENERIC,
    pumpkin_data::damage::DamageType::GENERIC_KILL,
    pumpkin_data::damage::DamageType::HOT_FLOOR,
    pumpkin_data::damage::DamageType::IN_FIRE,
    pumpkin_data::damage::DamageType::IN_WALL,
    pumpkin_data::damage::DamageType::INDIRECT_MAGIC,
    pumpkin_data::damage::DamageType::LAVA,
    pumpkin_data::damage::DamageType::LIGHTNING_BOLT,
    pumpkin_data::damage::DamageType::MACE_SMASH,
    pumpkin_data::damage::DamageType::MAGIC,
    pumpkin_data::damage::DamageType::MOB_ATTACK,
    pumpkin_data::damage::DamageType::MOB_ATTACK_NO_AGGRO,
    pumpkin_data::damage::DamageType::MOB_PROJECTILE,
    pumpkin_data::damage::DamageType::ON_FIRE,
    pumpkin_data::damage::DamageType::OUT_OF_WORLD,
    pumpkin_data::damage::DamageType::OUTSIDE_BORDER,
    pumpkin_data::damage::DamageType::PLAYER_ATTACK,
    pumpkin_data::damage::DamageType::PLAYER_EXPLOSION,
    pumpkin_data::damage::DamageType::SONIC_BOOM,
    pumpkin_data::damage::DamageType::SPEAR,
    pumpkin_data::damage::DamageType::SPIT,
    pumpkin_data::damage::DamageType::STALAGMITE,
    pumpkin_data::damage::DamageType::STARVE,
    pumpkin_data::damage::DamageType::STING,
    pumpkin_data::damage::DamageType::SULFUR_CUBE_HOT,
    pumpkin_data::damage::DamageType::SWEET_BERRY_BUSH,
    pumpkin_data::damage::DamageType::THORNS,
    pumpkin_data::damage::DamageType::THROWN,
    pumpkin_data::damage::DamageType::TRIDENT,
    pumpkin_data::damage::DamageType::UNATTRIBUTED_FIREBALL,
    pumpkin_data::damage::DamageType::WIND_CHARGE,
    pumpkin_data::damage::DamageType::WITHER,
    pumpkin_data::damage::DamageType::WITHER_SKULL,
];

static ERROR_UNKNOWN_RESOURCE: CommandErrorType<2> = CommandErrorType::new(
    translation::java::ARGUMENT_RESOURCE_NOT_FOUND,
    translation::java::ARGUMENT_RESOURCE_NOT_FOUND,
);

static ERROR_INVALID_RESOURCE_TYPE: CommandErrorType<3> = CommandErrorType::new(
    translation::java::ARGUMENT_RESOURCE_INVALID_TYPE,
    translation::java::ARGUMENT_RESOURCE_INVALID_TYPE,
);

static ERROR_NOT_SUMMONABLE_ENTITY: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ENTITY_NOT_SUMMONABLE,
    translation::java::ENTITY_NOT_SUMMONABLE,
);

pub static ENTITY_TYPE_ARGUMENT: ResourceArgument =
    ResourceArgument(ENTITY_TYPE_REGISTRY, &|id: Identifier| {
        EntityType::from_name(id.path()).map(|value| value as &'static (dyn Any + Send + Sync))
    });

pub static ENCHANTMENT_ARGUMENT: ResourceArgument =
    ResourceArgument(ENCHANTMENT_REGISTRY, &|id: Identifier| {
        pumpkin_data::Enchantment::from_name(id.path())
            .or_else(|| pumpkin_data::Enchantment::from_name(&id.to_string()))
            .map(|value| value as &'static (dyn Any + Send + Sync))
    });

pub static MOB_EFFECT_ARGUMENT: ResourceArgument =
    ResourceArgument(MOB_EFFECT_REGISTRY, &|id: Identifier| {
        pumpkin_data::effect::StatusEffect::from_name(id.path())
            .or_else(|| pumpkin_data::effect::StatusEffect::from_minecraft_name(&id.to_string()))
            .map(|value| value as &'static (dyn Any + Send + Sync))
    });

pub static DAMAGE_TYPE_ARGUMENT: ResourceArgument =
    ResourceArgument(DAMAGE_TYPE_REGISTRY, &|id: Identifier| {
        let dt = pumpkin_data::damage::DamageType::from_name(id.path())
            .or_else(|| pumpkin_data::damage::DamageType::from_name(&id.to_string()))?;
        let idx = dt.id as usize;
        DAMAGE_TYPES
            .get(idx)
            .map(|val| val as &'static (dyn Any + Send + Sync))
    });

#[derive(Clone)]
pub struct ResourceArgument(
    pub &'static Identifier,
    pub &'static (dyn Fn(Identifier) -> Option<&'static (dyn Any + Send + Sync)> + Send + Sync),
);

impl ArgumentType for ResourceArgument {
    type Item = &'static (dyn Any + Send + Sync);

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let identifier = Identifier::from_reader(reader)?;
        self.1(identifier.clone()).ok_or_else(|| {
            ERROR_UNKNOWN_RESOURCE.create(
                reader,
                TextComponent::text(identifier.path().to_string()),
                TextComponent::text(self.0.to_string()),
            )
        })
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext,
        suggestions_builder: SuggestionsBuilder,
    ) -> Suggestions {
        if self.0 == ENTITY_TYPE_REGISTRY {
            let entity_types = EntityType::ALL
                .iter()
                .filter(|entity_type| entity_type.summonable)
                .map(|entity_type| format!("minecraft:{}", entity_type.resource_name));
            suggestions_builder
                .filter_and_suggest_iter(entity_types)
                .build()
        } else if self.0 == ENCHANTMENT_REGISTRY {
            let enchantments = pumpkin_data::Enchantment::ALL
                .iter()
                .map(|e| format!("minecraft:{}", e.name));
            suggestions_builder
                .filter_and_suggest_iter(enchantments)
                .build()
        } else if self.0 == MOB_EFFECT_REGISTRY {
            let effects = [
                &pumpkin_data::effect::StatusEffect::ABSORPTION,
                &pumpkin_data::effect::StatusEffect::BAD_OMEN,
                &pumpkin_data::effect::StatusEffect::BLINDNESS,
                &pumpkin_data::effect::StatusEffect::BREATH_OF_THE_NAUTILUS,
                &pumpkin_data::effect::StatusEffect::CONDUIT_POWER,
                &pumpkin_data::effect::StatusEffect::DARKNESS,
                &pumpkin_data::effect::StatusEffect::DOLPHINS_GRACE,
                &pumpkin_data::effect::StatusEffect::FIRE_RESISTANCE,
                &pumpkin_data::effect::StatusEffect::GLOWING,
                &pumpkin_data::effect::StatusEffect::HASTE,
                &pumpkin_data::effect::StatusEffect::HEALTH_BOOST,
                &pumpkin_data::effect::StatusEffect::HERO_OF_THE_VILLAGE,
                &pumpkin_data::effect::StatusEffect::HUNGER,
                &pumpkin_data::effect::StatusEffect::INFESTED,
                &pumpkin_data::effect::StatusEffect::INSTANT_DAMAGE,
                &pumpkin_data::effect::StatusEffect::INSTANT_HEALTH,
                &pumpkin_data::effect::StatusEffect::INVISIBILITY,
                &pumpkin_data::effect::StatusEffect::JUMP_BOOST,
                &pumpkin_data::effect::StatusEffect::LEVITATION,
                &pumpkin_data::effect::StatusEffect::LUCK,
                &pumpkin_data::effect::StatusEffect::MINING_FATIGUE,
                &pumpkin_data::effect::StatusEffect::NAUSEA,
                &pumpkin_data::effect::StatusEffect::NIGHT_VISION,
                &pumpkin_data::effect::StatusEffect::OOZING,
                &pumpkin_data::effect::StatusEffect::POISON,
                &pumpkin_data::effect::StatusEffect::RAID_OMEN,
                &pumpkin_data::effect::StatusEffect::REGENERATION,
                &pumpkin_data::effect::StatusEffect::RESISTANCE,
                &pumpkin_data::effect::StatusEffect::SATURATION,
                &pumpkin_data::effect::StatusEffect::SLOW_FALLING,
                &pumpkin_data::effect::StatusEffect::SLOWNESS,
                &pumpkin_data::effect::StatusEffect::SPEED,
                &pumpkin_data::effect::StatusEffect::STRENGTH,
                &pumpkin_data::effect::StatusEffect::TRIAL_OMEN,
                &pumpkin_data::effect::StatusEffect::UNLUCK,
                &pumpkin_data::effect::StatusEffect::WATER_BREATHING,
                &pumpkin_data::effect::StatusEffect::WEAKNESS,
                &pumpkin_data::effect::StatusEffect::WEAVING,
                &pumpkin_data::effect::StatusEffect::WIND_CHARGED,
                &pumpkin_data::effect::StatusEffect::WITHER,
            ]
            .iter()
            .map(|e| e.minecraft_name.to_string());
            suggestions_builder.filter_and_suggest_iter(effects).build()
        } else if self.0 == DAMAGE_TYPE_REGISTRY {
            let types = DAMAGE_TYPES
                .iter()
                .map(|dt| format!("minecraft:{}", dt.message_id));
            suggestions_builder.filter_and_suggest_iter(types).build()
        } else {
            Suggestions::empty()
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::Resource {
            identifier: self.0.clone(),
        }
    }

    fn override_suggestion_providers(&self) -> Option<SuggestionProviders> {
        (self.0 == ENTITY_TYPE_REGISTRY).then_some(SuggestionProviders::SummonableEntities)
    }
}

impl ResourceArgument {
    pub fn get_resource<T: 'static>(
        context: &CommandContext,
        name: &str,
        registry_key: &Identifier,
    ) -> Result<&'static T, CommandSyntaxError> {
        let missing_argument = DISPATCHER_PARSE_EXCEPTION.create_without_context(
            TextComponent::text(format!("Could not find argument with name '{name}'")),
        );
        let node = context
            .nodes
            .iter()
            .rev()
            .find_map(|parsed| {
                if let AttachedNode::Argument(cur) = &context.tree[parsed.node]
                    && cur.meta.name == name
                {
                    Some(cur)
                } else {
                    None
                }
            })
            .or_else(|| {
                context.tree.iter().find_map(|node| {
                    if let AttachedNode::Argument(cur) = node
                        && cur.meta.name == name
                    {
                        Some(cur)
                    } else {
                        None
                    }
                })
            })
            .ok_or(missing_argument.clone())?;
        let invalid_argument =
            DISPATCHER_PARSE_EXCEPTION.create_without_context(TextComponent::text(format!(
                "argument with name '{name}' isn't a ResourceArgument"
            )));
        let result_argument = node
            .meta
            .argument_type
            .as_any()
            .downcast_ref::<Self>()
            .ok_or(invalid_argument)?;
        let registry_name = result_argument.0;
        let identifier = context
            .arguments
            .get(name)
            .ok_or(missing_argument)?
            .range
            .substring_slice(context.input.as_str())
            .to_string();
        let err = ERROR_INVALID_RESOURCE_TYPE.create_without_context(
            TextComponent::text(identifier),
            TextComponent::text(registry_name.to_string()),
            TextComponent::text(registry_key.to_string()),
        );
        if registry_name == registry_key {
            context
                .get_argument::<&'static (dyn Any + Send + Sync)>(name)?
                .downcast_ref::<T>()
                .ok_or(err)
        } else {
            Err(err)
        }
    }

    pub fn get_entity_type(
        context: &CommandContext,
        name: &str,
    ) -> Result<&'static EntityType, CommandSyntaxError> {
        Self::get_resource(context, name, ENTITY_TYPE_REGISTRY)
    }

    pub fn get_enchantment(
        context: &CommandContext,
        name: &str,
    ) -> Result<&'static pumpkin_data::Enchantment, CommandSyntaxError> {
        Self::get_resource(context, name, ENCHANTMENT_REGISTRY)
    }

    pub fn get_mob_effect(
        context: &CommandContext,
        name: &str,
    ) -> Result<&'static pumpkin_data::effect::StatusEffect, CommandSyntaxError> {
        Self::get_resource(context, name, MOB_EFFECT_REGISTRY)
    }

    pub fn get_damage_type(
        context: &CommandContext,
        name: &str,
    ) -> Result<&'static pumpkin_data::damage::DamageType, CommandSyntaxError> {
        Self::get_resource(context, name, DAMAGE_TYPE_REGISTRY)
    }

    pub fn get_summonable_entity_type(
        context: &CommandContext,
        name: &str,
    ) -> Result<&'static EntityType, CommandSyntaxError> {
        let val: &'static EntityType = Self::get_resource(context, name, ENTITY_TYPE_REGISTRY)?;
        if val.summonable {
            Ok(val)
        } else {
            Err(ERROR_NOT_SUMMONABLE_ENTITY
                .create_without_context(TextComponent::text(val.resource_name)))
        }
    }
}
