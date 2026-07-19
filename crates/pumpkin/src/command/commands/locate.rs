use std::borrow::Cow;
use std::pin::Pin;

use pumpkin_data::structures::StructureSet;
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_world::generation::generator::structure_finder::find_nearest_structure;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::suggestion::provider::SuggestionProvider;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};

const DESCRIPTION: &str = "Locates the closest structure.";

const PERMISSION: &str = "minecraft:command.locate";

const ARG_STRUCTURE: &str = "structure";

/// The maximum search radius in chunk regions, matching vanilla's
/// `findNearestMapStructure` call in `LocateCommand`.
const MAX_SEARCH_RADIUS: i32 = 100;

/// The structure set names known to the generator, offered as suggestions.
const STRUCTURE_SET_NAMES: &[&str] = &[
    "ancient_cities",
    "buried_treasures",
    "desert_pyramids",
    "end_cities",
    "igloos",
    "jungle_temples",
    "mineshafts",
    "nether_complexes",
    "nether_fossils",
    "ocean_monuments",
    "ocean_ruins",
    "pillager_outposts",
    "ruined_portals",
    "shipwrecks",
    "strongholds",
    "swamp_huts",
    "trail_ruins",
    "trial_chambers",
    "villages",
    "woodland_mansions",
];

const STRUCTURE_INVALID_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_LOCATE_STRUCTURE_INVALID,
    translation::java::COMMANDS_LOCATE_STRUCTURE_INVALID,
);

const STRUCTURE_NOT_FOUND_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_LOCATE_STRUCTURE_NOT_FOUND,
    translation::bedrock::COMMANDS_LOCATE_STRUCTURE_FAIL_NOSTRUCTUREFOUND,
);

struct StructureSuggestionProvider;

impl SuggestionProvider for StructureSuggestionProvider {
    fn suggest(
        &self,
        _context: &CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send>> {
        Box::pin(async move {
            for name in STRUCTURE_SET_NAMES {
                builder = builder.suggest(*name);
            }
            builder.build()
        })
    }
}

/// Builds the clickable green `[x, ~, z]` coordinates component used by
/// vanilla's locate feedback.
fn coordinates_text(pos: &BlockPos) -> TextComponent {
    let x = pos.0.x;
    let z = pos.0.z;

    TextComponent::translate_cross(
        translation::java::CHAT_COORDINATES,
        translation::java::CHAT_COORDINATES,
        [
            TextComponent::text(x.to_string()),
            TextComponent::text("~"),
            TextComponent::text(z.to_string()),
        ],
    )
    .color_named(NamedColor::Green)
    .click_event(ClickEvent::SuggestCommand {
        command: Cow::from(format!("/tp @s {x} ~ {z}")),
    })
    .hover_event(HoverEvent::show_text(TextComponent::translate_cross(
        translation::java::CHAT_COORDINATES_TOOLTIP,
        translation::java::CHAT_COORDINATES_TOOLTIP,
        [],
    )))
}

struct LocateStructureExecutor;

impl CommandExecutor for LocateStructureExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let input = StringArgumentType::get(context, ARG_STRUCTURE)?;
            let name = input.strip_prefix("minecraft:").unwrap_or(input);

            let Some(set) = StructureSet::get(name) else {
                return Err(STRUCTURE_INVALID_ERROR_TYPE
                    .create_without_context(TextComponent::text(input.to_string())));
            };

            let source_pos = context.source.position;
            let origin = BlockPos::floored_v(source_pos);

            let world = context.source.world().clone();
            let level = &world.level;
            let seed = level.seed.0;

            let found = level
                .world_gen
                .global_structure_cache()
                .and_then(|global_cache| {
                    find_nearest_structure(
                        origin,
                        &[&set.placement],
                        MAX_SEARCH_RADIUS,
                        seed as i64,
                        global_cache,
                    )
                });

            let Some(target) = found else {
                return Err(STRUCTURE_NOT_FOUND_ERROR_TYPE
                    .create_without_context(TextComponent::text(name.to_string())));
            };

            // Vanilla reports the horizontal distance only.
            let dx = f64::from(target.0.x) - source_pos.x;
            let dz = f64::from(target.0.z) - source_pos.z;
            let distance = dx.hypot(dz).floor().max(0.0) as i32;

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_LOCATE_STRUCTURE_SUCCESS,
                        translation::bedrock::COMMANDS_LOCATE_STRUCTURE_SUCCESS,
                        [
                            TextComponent::text(name.to_string()),
                            coordinates_text(&target),
                            TextComponent::text(distance.to_string()),
                        ],
                    ),
                    false,
                )
                .await;

            Ok(distance)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("locate", DESCRIPTION).requires(PERMISSION).then(
            literal("structure").then(
                argument(ARG_STRUCTURE, StringArgumentType::SingleWord)
                    .suggests(StructureSuggestionProvider)
                    .executes(LocateStructureExecutor),
            ),
        ),
    );
}
