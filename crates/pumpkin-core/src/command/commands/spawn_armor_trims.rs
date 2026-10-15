use std::sync::Arc;

use pumpkin_data::data_component_impl::{EquipmentSlot, TrimImpl};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::trim_material::TrimMaterial;
use pumpkin_data::trim_pattern::TrimPattern;
use pumpkin_data::FacingExt;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::{
    PermissionLvl,
    permission::{Permission, PermissionDefault, PermissionRegistry},
    text::TextComponent,
};

use crate::command::{
    argument_builder::{ArgumentBuilder, argument, command, literal},
    argument_types::core::string::StringArgumentType,
    context::command_context::CommandContext,
    errors::error_types::CommandErrorType,
    node::{CommandExecutor, CommandExecutorResult, dispatcher::CommandDispatcher},
};
use crate::entity::{Entity, EntityBase, decoration::armor_stand::ArmorStandEntity};

const DESCRIPTION: &str = "Spawns armor stands showcasing armor trims.";
const PERMISSION: &str = "minecraft:command.spawn_armor_trims";

const ERROR_INVALID_PATTERN: CommandErrorType<1> =
    CommandErrorType::new("Invalid pattern: %s", "Invalid pattern: %s");

const VANILLA_TRIM_PATTERNS: &[TrimPattern] = &[
    TrimPattern::Sentry,
    TrimPattern::Dune,
    TrimPattern::Coast,
    TrimPattern::Wild,
    TrimPattern::Ward,
    TrimPattern::Eye,
    TrimPattern::Vex,
    TrimPattern::Tide,
    TrimPattern::Snout,
    TrimPattern::Rib,
    TrimPattern::Spire,
    TrimPattern::Wayfinder,
    TrimPattern::Shaper,
    TrimPattern::Silence,
    TrimPattern::Raiser,
    TrimPattern::Host,
    TrimPattern::Flow,
    TrimPattern::Bolt,
];

const VANILLA_TRIM_MATERIALS: &[TrimMaterial] = &[
    TrimMaterial::Quartz,
    TrimMaterial::Iron,
    TrimMaterial::Netherite,
    TrimMaterial::Redstone,
    TrimMaterial::Copper,
    TrimMaterial::Gold,
    TrimMaterial::Emerald,
    TrimMaterial::Diamond,
    TrimMaterial::Lapis,
    TrimMaterial::Amethyst,
    TrimMaterial::Resin,
];

const EQUIPPABLE_ARMOR: &[(&Item, EquipmentSlot)] = &[
    (&Item::NETHERITE_HELMET, EquipmentSlot::HEAD),
    (&Item::NETHERITE_CHESTPLATE, EquipmentSlot::CHEST),
    (&Item::NETHERITE_LEGGINGS, EquipmentSlot::LEGS),
    (&Item::NETHERITE_BOOTS, EquipmentSlot::FEET),
    (&Item::DIAMOND_HELMET, EquipmentSlot::HEAD),
    (&Item::DIAMOND_CHESTPLATE, EquipmentSlot::CHEST),
    (&Item::DIAMOND_LEGGINGS, EquipmentSlot::LEGS),
    (&Item::DIAMOND_BOOTS, EquipmentSlot::FEET),
    (&Item::IRON_HELMET, EquipmentSlot::HEAD),
    (&Item::IRON_CHESTPLATE, EquipmentSlot::CHEST),
    (&Item::IRON_LEGGINGS, EquipmentSlot::LEGS),
    (&Item::IRON_BOOTS, EquipmentSlot::FEET),
    (&Item::GOLDEN_HELMET, EquipmentSlot::HEAD),
    (&Item::GOLDEN_CHESTPLATE, EquipmentSlot::CHEST),
    (&Item::GOLDEN_LEGGINGS, EquipmentSlot::LEGS),
    (&Item::GOLDEN_BOOTS, EquipmentSlot::FEET),
    (&Item::CHAINMAIL_HELMET, EquipmentSlot::HEAD),
    (&Item::CHAINMAIL_CHESTPLATE, EquipmentSlot::CHEST),
    (&Item::CHAINMAIL_LEGGINGS, EquipmentSlot::LEGS),
    (&Item::CHAINMAIL_BOOTS, EquipmentSlot::FEET),
    (&Item::TURTLE_HELMET, EquipmentSlot::HEAD),
];

fn spawn_trims(context: &CommandContext, patterns: &[TrimPattern]) -> CommandExecutorResult {
    let player = context.source.player_or_err()?;
    let world = context.source.world();
    let facing = player.get_entity().get_facing();
    let origin = player
        .get_entity()
        .block_pos
        .load()
        .offset_dir(facing.to_block_direction().to_offset(), 5);

    let origin_x = f64::from(origin.0.x);
    let origin_y = f64::from(origin.0.y);
    let origin_z = f64::from(origin.0.z);

    for (material_index, &material) in VANILLA_TRIM_MATERIALS.iter().enumerate() {
        for (pattern_index, &pattern) in patterns.iter().enumerate() {
            let trim = TrimImpl::new(material, pattern);

            for (item_index, &(item, ref slot)) in EQUIPPABLE_ARMOR.iter().enumerate() {
                let x = origin_x + 0.5 - (item_index as f64) * 3.0;
                let y = origin_y + 0.5 + (material_index as f64) * 3.0;
                let z = origin_z + 0.5 + (pattern_index as f64) * 10.0;
                let pos = Vector3::new(x, y, z);

                let entity = Entity::new(world.clone(), pos, &EntityType::ARMOR_STAND);
                entity.set_rotation(180.0, 0.0);
                entity.set_has_no_gravity(true);

                let mut stack = ItemStack::new(1, item);
                stack.set_data_component(trim.clone());

                let armor_stand = Arc::new(ArmorStandEntity::new(entity));
                armor_stand.set_item_slot(slot.clone(), stack);

                if item_index == 0 {
                    let label = format!("{} & {}", pattern.to_name(), material.to_name());
                    armor_stand
                        .get_entity()
                        .set_custom_name(TextComponent::text(label));
                    armor_stand.get_entity().set_custom_name_visible(true);
                } else {
                    armor_stand.get_entity().set_invisible(true);
                }

                world.spawn_entity(armor_stand);
            }
        }
    }

    context.source.send_feedback(
        TextComponent::text("Armorstands with trimmed armor spawned around you"),
        true,
    );
    Ok(1)
}

struct AllTrimsExecutor;

impl CommandExecutor for AllTrimsExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        spawn_trims(context, VANILLA_TRIM_PATTERNS)
    }
}

struct PatternTrimExecutor;

impl CommandExecutor for PatternTrimExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let pattern_str = StringArgumentType::get(context, "pattern")?;
        if pattern_str == "*_lag_my_game" {
            return spawn_trims(context, VANILLA_TRIM_PATTERNS);
        }
        let pattern = TrimPattern::from_name(&pattern_str).ok_or_else(|| {
            ERROR_INVALID_PATTERN
                .create_without_context(TextComponent::text(pattern_str.to_string()))
        })?;
        spawn_trims(context, &[pattern])
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("spawn_armor_trims", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("*_lag_my_game").executes(AllTrimsExecutor))
            .then(argument("pattern", StringArgumentType::SingleWord).executes(PatternTrimExecutor)),
    );
}
