use std::sync::Arc;

use pumpkin_data::particle::Particle;
use pumpkin_data::translation;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::CParticle;
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::coordinates::vec3::Vec3ArgumentType;
use crate::command::argument_types::core::float::FloatArgumentType;
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::particle::ParticleArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::player::Player;

const DESCRIPTION: &str = "Creates particles in the world.";
const PERMISSION: &str = "minecraft:command.particle";

const ERROR_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_PARTICLE_FAILED,
    translation::java::COMMANDS_PARTICLE_FAILED,
);

#[allow(clippy::too_many_arguments)]
fn send_particles(
    source: &CommandSource,
    particle: Particle,
    pos: Vector3<f64>,
    delta: Vector3<f64>,
    speed: f32,
    count: i32,
    force: bool,
    viewers: Option<Vec<Arc<Player>>>,
) -> Result<i32, CommandSyntaxError> {
    let world = source.world().clone();
    let players: Vec<Arc<Player>> = viewers.unwrap_or_else(|| world.players.load().to_vec());

    let offset = Vector3::new(delta.x as f32, delta.y as f32, delta.z as f32);
    let packet = CParticle::new(
        force,
        false,
        pos,
        offset,
        speed,
        count,
        VarInt(i32::from(particle.to_id())),
        &[],
    );

    let max_dist_sq = if force { 512.0 * 512.0 } else { 32.0 * 32.0 };
    let mut sent_count = 0;

    for player in &players {
        if !Arc::ptr_eq(&player.world(), &world) {
            continue;
        }
        let player_pos = player.position();
        let dx = player_pos.x - pos.x;
        let dy = player_pos.y - pos.y;
        let dz = player_pos.z - pos.z;
        let dist_sq = dx * dx + dy * dy + dz * dz;
        if dist_sq <= max_dist_sq {
            player.try_send_client_packet(&packet);
            sent_count += 1;
        }
    }

    if sent_count == 0 {
        return Err(ERROR_FAILED.create_without_context());
    }

    let particle_name = format!("{particle:?}").to_lowercase();
    source.send_feedback(
        TextComponent::translate_cross(
            translation::java::COMMANDS_PARTICLE_SUCCESS,
            translation::java::COMMANDS_PARTICLE_SUCCESS,
            [TextComponent::text(particle_name)],
        ),
        true,
    );

    Ok(sent_count)
}

#[derive(Clone, Copy)]
enum ParticleMode {
    Simple,
    WithPos,
    Full { force: bool, has_viewers: bool },
}

struct ParticleExecutor {
    mode: ParticleMode,
}

impl CommandExecutor for ParticleExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let particle = ParticleArgumentType::get(context, "name")?;

        match self.mode {
            ParticleMode::Simple => {
                let pos = context.source.position;
                send_particles(
                    &context.source,
                    particle,
                    pos,
                    Vector3::new(0.0, 0.0, 0.0),
                    0.0,
                    0,
                    false,
                    None,
                )
            }
            ParticleMode::WithPos => {
                let pos = Vec3ArgumentType::get_vector3(context, "pos")?;
                send_particles(
                    &context.source,
                    particle,
                    pos,
                    Vector3::new(0.0, 0.0, 0.0),
                    0.0,
                    0,
                    false,
                    None,
                )
            }
            ParticleMode::Full { force, has_viewers } => {
                let pos = Vec3ArgumentType::get_vector3(context, "pos")?;
                let delta = Vec3ArgumentType::get_vector3(context, "delta")?;
                let speed = FloatArgumentType::get(context, "speed")?;
                let count = IntegerArgumentType::get(context, "count")?;
                let viewers = if has_viewers {
                    Some(EntityArgumentType::get_players(context, "viewers")?)
                } else {
                    None
                };

                send_particles(
                    &context.source,
                    particle,
                    pos,
                    delta,
                    speed,
                    count,
                    force,
                    viewers,
                )
            }
        }
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("particle", DESCRIPTION).requires(PERMISSION).then(
            argument("name", ParticleArgumentType)
                .executes(ParticleExecutor {
                    mode: ParticleMode::Simple,
                })
                .then(
                    argument("pos", Vec3ArgumentType::Default)
                        .executes(ParticleExecutor {
                            mode: ParticleMode::WithPos,
                        })
                        .then(
                            argument("delta", Vec3ArgumentType::Uncorrected).then(
                                argument("speed", FloatArgumentType::with_min(0.0)).then(
                                    argument("count", IntegerArgumentType::with_min(0))
                                        .executes(ParticleExecutor {
                                            mode: ParticleMode::Full {
                                                force: false,
                                                has_viewers: false,
                                            },
                                        })
                                        .then(
                                            literal("force")
                                                .executes(ParticleExecutor {
                                                    mode: ParticleMode::Full {
                                                        force: true,
                                                        has_viewers: false,
                                                    },
                                                })
                                                .then(
                                                    argument(
                                                        "viewers",
                                                        EntityArgumentType::Players,
                                                    )
                                                    .executes(ParticleExecutor {
                                                        mode: ParticleMode::Full {
                                                            force: true,
                                                            has_viewers: true,
                                                        },
                                                    }),
                                                ),
                                        )
                                        .then(
                                            literal("normal")
                                                .executes(ParticleExecutor {
                                                    mode: ParticleMode::Full {
                                                        force: false,
                                                        has_viewers: false,
                                                    },
                                                })
                                                .then(
                                                    argument(
                                                        "viewers",
                                                        EntityArgumentType::Players,
                                                    )
                                                    .executes(ParticleExecutor {
                                                        mode: ParticleMode::Full {
                                                            force: false,
                                                            has_viewers: true,
                                                        },
                                                    }),
                                                ),
                                        ),
                                ),
                            ),
                        ),
                ),
        ),
    );
}
