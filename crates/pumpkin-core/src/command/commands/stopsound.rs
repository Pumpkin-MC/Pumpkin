use pumpkin_data::sound::SoundCategory;
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::identifier::IdentifierArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Stops a currently playing sound.";
const PERMISSION: &str = "minecraft:command.stopsound";

enum StopSoundMode {
    All,
    Category(SoundCategory),
    Sound,
    CategoryAndSound(SoundCategory),
}

/// `SoundSource`'s names, which vanilla lists as literals.
const SOURCES: [(&str, SoundCategory); 11] = [
    ("ambient", SoundCategory::Ambient),
    ("block", SoundCategory::Blocks),
    ("hostile", SoundCategory::Hostile),
    ("master", SoundCategory::Master),
    ("music", SoundCategory::Music),
    ("neutral", SoundCategory::Neutral),
    ("player", SoundCategory::Players),
    ("record", SoundCategory::Records),
    ("ui", SoundCategory::Ui),
    ("voice", SoundCategory::Voice),
    ("weather", SoundCategory::Weather),
];

struct StopSoundExecutor(StopSoundMode);

impl CommandExecutor for StopSoundExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_players(context, "targets")?;

        let (category, sound) = match self.0 {
            StopSoundMode::All => (None, None),
            StopSoundMode::Category(cat) => (Some(cat), None),
            StopSoundMode::Sound => {
                let snd = IdentifierArgumentType::get(context, "sound")?;
                (None, Some(snd.to_string()))
            }
            StopSoundMode::CategoryAndSound(cat) => {
                let snd = IdentifierArgumentType::get(context, "sound")?;
                (Some(cat), Some(snd.to_string()))
            }
        };

        for target in &targets {
            target.stop_sound(sound.clone(), category);
        }

        let text = match (category, &sound) {
            (Some(c), Some(s)) => TextComponent::translate_cross(
                translation::java::COMMANDS_STOPSOUND_SUCCESS_SOURCE_SOUND,
                translation::java::COMMANDS_STOPSOUND_SUCCESS_SOURCE_SOUND,
                [
                    TextComponent::text(s.clone()),
                    TextComponent::text(c.to_name()),
                ],
            ),
            (Some(c), None) => TextComponent::translate_cross(
                translation::java::COMMANDS_STOPSOUND_SUCCESS_SOURCE_ANY,
                translation::java::COMMANDS_STOPSOUND_SUCCESS_SOURCE_ANY,
                [TextComponent::text(c.to_name())],
            ),
            (None, Some(s)) => TextComponent::translate_cross(
                translation::java::COMMANDS_STOPSOUND_SUCCESS_SOURCELESS_SOUND,
                translation::java::COMMANDS_STOPSOUND_SUCCESS_SOURCELESS_SOUND,
                [TextComponent::text(s.clone())],
            ),
            (None, None) => TextComponent::translate_cross(
                translation::java::COMMANDS_STOPSOUND_SUCCESS_SOURCELESS_ANY,
                translation::java::COMMANDS_STOPSOUND_SUCCESS_SOURCELESS_ANY,
                [],
            ),
        };
        context.source.send_feedback(text, true);

        Ok(targets.len() as i32)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("stopsound", DESCRIPTION).requires(PERMISSION).then(
            SOURCES.iter().fold(
                argument("targets", EntityArgumentType::Players)
                    .executes(StopSoundExecutor(StopSoundMode::All))
                    .then(
                        literal("*").then(
                            argument("sound", IdentifierArgumentType)
                                .executes(StopSoundExecutor(StopSoundMode::Sound)),
                        ),
                    ),
                |targets, &(name, category)| {
                    targets.then(
                        literal(name)
                            .executes(StopSoundExecutor(StopSoundMode::Category(category)))
                            .then(argument("sound", IdentifierArgumentType).executes(
                                StopSoundExecutor(StopSoundMode::CategoryAndSound(category)),
                            )),
                    )
                },
            ),
        ),
    );
}
