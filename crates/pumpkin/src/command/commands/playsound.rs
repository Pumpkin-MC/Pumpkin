use std::sync::Arc;

use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::translation;
use pumpkin_protocol::IdOr;
use pumpkin_protocol::java::client::play::CSoundEffect;
use pumpkin_util::PermissionLvl;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{
    ArgumentBuilder, LiteralArgumentBuilder, argument, command, literal,
};
use crate::command::argument_types::coordinates::vec3::Vec3ArgumentType;
use crate::command::argument_types::core::float::FloatArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::identifier::IdentifierArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::player::Player;

const DESCRIPTION: &str = "Plays a sound at a position.";
const PERMISSION: &str = "minecraft:command.playsound";

const ERROR_TOO_FAR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_PLAYSOUND_FAILED,
    translation::java::COMMANDS_PLAYSOUND_FAILED,
);

const SOUND_SOURCES: [(&str, SoundCategory); 10] = [
    ("master", SoundCategory::Master),
    ("music", SoundCategory::Music),
    ("record", SoundCategory::Records),
    ("weather", SoundCategory::Weather),
    ("block", SoundCategory::Blocks),
    ("hostile", SoundCategory::Hostile),
    ("neutral", SoundCategory::Neutral),
    ("player", SoundCategory::Players),
    ("ambient", SoundCategory::Ambient),
    ("voice", SoundCategory::Voice),
];

#[allow(clippy::too_many_arguments)]
fn play_sound(
    source: &CommandSource,
    players: &[Arc<Player>],
    sound: &Identifier,
    sound_source: SoundCategory,
    position: Vector3<f64>,
    volume: f32,
    pitch: f32,
    min_volume: f32,
) -> Result<i32, CommandSyntaxError> {
    let range = 16.0 * if volume > 1.0 { volume } else { 1.0 };
    let max_dist_sq = f64::from(range * range);
    let world = source.world().clone();
    let seed = rand::random::<f64>();
    let mut played_for = Vec::new();

    let sound_event = Sound::from_name(sound.path())
        .or_else(|| Sound::from_name(&sound.to_string()))
        .map_or_else(
            || {
                IdOr::Value(pumpkin_protocol::SoundEvent {
                    sound_name: sound.to_string(),
                    range: None,
                })
            },
            |sound_enum| IdOr::Id(sound_enum as u16),
        );

    for player in players {
        if !Arc::ptr_eq(&player.world(), &world) {
            continue;
        }

        let player_pos = player.position();
        let delta_x = position.x - player_pos.x;
        let delta_y = position.y - player_pos.y;
        let delta_z = position.z - player_pos.z;
        let dist_sq = delta_x * delta_x + delta_y * delta_y + delta_z * delta_z;

        let (local_position, local_volume) = if dist_sq > max_dist_sq {
            if min_volume <= 0.0 {
                continue;
            }

            let distance = dist_sq.sqrt();
            (
                Vector3::new(
                    player_pos.x + delta_x / distance * 2.0,
                    player_pos.y + delta_y / distance * 2.0,
                    player_pos.z + delta_z / distance * 2.0,
                ),
                min_volume,
            )
        } else {
            (position, volume)
        };

        let packet = CSoundEffect::new(
            sound_event.clone(),
            sound_source,
            &local_position,
            local_volume,
            pitch,
            seed,
        );
        player.try_send_client_packet(&packet);
        played_for.push(player.clone());
    }

    let count = played_for.len();
    if count == 0 {
        return Err(ERROR_TOO_FAR.create_without_context());
    }

    let sound_str = sound.to_string();
    if count == 1 {
        let player_name = played_for[0].gameprofile.name.clone();
        source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_PLAYSOUND_SUCCESS_SINGLE,
                translation::java::COMMANDS_PLAYSOUND_SUCCESS_SINGLE,
                [
                    TextComponent::text(sound_str),
                    TextComponent::text(player_name),
                ],
            ),
            true,
        );
    } else {
        source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_PLAYSOUND_SUCCESS_MULTIPLE,
                translation::java::COMMANDS_PLAYSOUND_SUCCESS_MULTIPLE,
                [
                    TextComponent::text(sound_str),
                    TextComponent::text(count.to_string()),
                ],
            ),
            true,
        );
    }

    Ok(count as i32)
}

#[derive(Clone, Copy)]
enum PlaySoundStep {
    SoundOnly,
    SourceOnly { source: SoundCategory },
    WithTargets { source: SoundCategory },
    WithPos { source: SoundCategory },
    WithVolume { source: SoundCategory },
    WithPitch { source: SoundCategory },
    WithMinVolume { source: SoundCategory },
}

struct PlaySoundExecutor {
    step: PlaySoundStep,
}

impl CommandExecutor for PlaySoundExecutor {
    #[allow(clippy::too_many_lines)]
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let sound = IdentifierArgumentType::get(context, "sound")?;
        let caller_players = context
            .source
            .output
            .as_player()
            .map_or_else(Vec::new, |p| vec![p]);

        match self.step {
            PlaySoundStep::SoundOnly => play_sound(
                &context.source,
                &caller_players,
                &sound,
                SoundCategory::Master,
                context.source.position,
                1.0,
                1.0,
                0.0,
            ),
            PlaySoundStep::SourceOnly { source } => play_sound(
                &context.source,
                &caller_players,
                &sound,
                source,
                context.source.position,
                1.0,
                1.0,
                0.0,
            ),
            PlaySoundStep::WithTargets { source } => {
                let targets = EntityArgumentType::get_players(context, "targets")?;
                play_sound(
                    &context.source,
                    &targets,
                    &sound,
                    source,
                    context.source.position,
                    1.0,
                    1.0,
                    0.0,
                )
            }
            PlaySoundStep::WithPos { source } => {
                let targets = EntityArgumentType::get_players(context, "targets")?;
                let pos = Vec3ArgumentType::get_vector3(context, "pos")?;
                play_sound(
                    &context.source,
                    &targets,
                    &sound,
                    source,
                    pos,
                    1.0,
                    1.0,
                    0.0,
                )
            }
            PlaySoundStep::WithVolume { source } => {
                let targets = EntityArgumentType::get_players(context, "targets")?;
                let pos = Vec3ArgumentType::get_vector3(context, "pos")?;
                let volume = FloatArgumentType::get(context, "volume")?;
                play_sound(
                    &context.source,
                    &targets,
                    &sound,
                    source,
                    pos,
                    volume,
                    1.0,
                    0.0,
                )
            }
            PlaySoundStep::WithPitch { source } => {
                let targets = EntityArgumentType::get_players(context, "targets")?;
                let pos = Vec3ArgumentType::get_vector3(context, "pos")?;
                let volume = FloatArgumentType::get(context, "volume")?;
                let pitch = FloatArgumentType::get(context, "pitch")?;
                play_sound(
                    &context.source,
                    &targets,
                    &sound,
                    source,
                    pos,
                    volume,
                    pitch,
                    0.0,
                )
            }
            PlaySoundStep::WithMinVolume { source } => {
                let targets = EntityArgumentType::get_players(context, "targets")?;
                let pos = Vec3ArgumentType::get_vector3(context, "pos")?;
                let volume = FloatArgumentType::get(context, "volume")?;
                let pitch = FloatArgumentType::get(context, "pitch")?;
                let min_volume = FloatArgumentType::get(context, "minVolume")?;
                play_sound(
                    &context.source,
                    &targets,
                    &sound,
                    source,
                    pos,
                    volume,
                    pitch,
                    min_volume,
                )
            }
        }
    }
}

fn build_source_branch(name: &'static str, source: SoundCategory) -> LiteralArgumentBuilder {
    literal(name)
        .executes(PlaySoundExecutor {
            step: PlaySoundStep::SourceOnly { source },
        })
        .then(
            argument("targets", EntityArgumentType::Players)
                .executes(PlaySoundExecutor {
                    step: PlaySoundStep::WithTargets { source },
                })
                .then(
                    argument("pos", Vec3ArgumentType::Default)
                        .executes(PlaySoundExecutor {
                            step: PlaySoundStep::WithPos { source },
                        })
                        .then(
                            argument("volume", FloatArgumentType::with_min(0.0))
                                .executes(PlaySoundExecutor {
                                    step: PlaySoundStep::WithVolume { source },
                                })
                                .then(
                                    argument("pitch", FloatArgumentType::new(0.0, 2.0))
                                        .executes(PlaySoundExecutor {
                                            step: PlaySoundStep::WithPitch { source },
                                        })
                                        .then(
                                            argument("minVolume", FloatArgumentType::new(0.0, 1.0))
                                                .executes(PlaySoundExecutor {
                                                    step: PlaySoundStep::WithMinVolume { source },
                                                }),
                                        ),
                                ),
                        ),
                ),
        )
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let mut sound_arg = argument("sound", IdentifierArgumentType).executes(PlaySoundExecutor {
        step: PlaySoundStep::SoundOnly,
    });

    for (name, source) in SOUND_SOURCES {
        sound_arg = sound_arg.then(build_source_branch(name, source));
    }

    dispatcher.register(
        command("playsound", DESCRIPTION)
            .requires(PERMISSION)
            .then(sound_arg),
    );
}
