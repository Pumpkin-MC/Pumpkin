#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_data::entity::EntityPose;
use pumpkin_protocol::bedrock::server::player_auth_input::{InputMode, InteractionModel};
use pumpkin_protocol::java::server::play::SPlayerInput;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::player::ClientMove;

impl BedrockClient {
    pub fn handle_player_auth_input(
        &self,
        player: &Arc<Player>,
        packet: SPlayerAuthInput,
        server: &Arc<Server>,
    ) {
        if player.living_entity.dead.load(Ordering::Relaxed)
            || player.living_entity.health.load() <= 0.0
        {
            return;
        }
        if self.await_teleport(player, &packet) {
            // Stale poses before a respawn teleport must not count as client motion.
            player.get_entity().movement.store(Vector3::default());
            return;
        }
        if !player.has_client_loaded() {
            return;
        }

        let entity = player.get_entity();
        let flags = &packet.input_data;
        // Like Geyser: the motion this tick started with was downward. Landing can already
        // have cleared the end delta (gliding into the ground).
        let on_ground = flags.get(InputData::VerticalCollision as usize)
            && self.client_delta.load().y < 0.0
            && !entity.has_vehicle();
        let horizontal_collision = flags.get(InputData::HorizontalCollision as usize);

        self.input_tick.store(packet.tick.0, Ordering::Relaxed);
        // Client predicted end-of-tick velocity. Server `Entity.velocity` stays knockback/push
        // owned; `send_velocity_changes` sends that, then later pushes layer on this delta.
        self.client_delta.store(packet.delta.to_f64());

        let sneaking = sneaking_from_auth(player, flags);
        player.apply_client_input(server, java_input_from_auth(&packet, sneaking));

        let started_sprint = flags.get(InputData::StartSprinting as usize);
        let stopped_sprint = flags.get(InputData::StopSprinting as usize);
        if started_sprint && !stopped_sprint {
            player.apply_sprint_input(server, true);
        } else if stopped_sprint && !started_sprint {
            player.apply_sprint_input(server, false);
        }

        if flags.get(InputData::StartSwimming as usize) {
            entity.set_swimming(true);
        } else if flags.get(InputData::StopSwimming as usize) {
            entity.set_swimming(false);
        }

        if flags.get(InputData::StartCrawling as usize) {
            entity.set_pose(EntityPose::Swimming);
        } else if flags.get(InputData::StopCrawling as usize) {
            player.update_player_pose();
        }

        if flags.get(InputData::StartFlying as usize) {
            player.apply_flight_input(server, true);
        } else if flags.get(InputData::StopFlying as usize) {
            player.apply_flight_input(server, false);
        }

        // Bedrock can stop in the same input it started, then it never glided.
        if flags.get(InputData::StartGliding as usize)
            && !flags.get(InputData::StopGliding as usize)
        {
            Self::start_gliding(player, server);
        } else if flags.get(InputData::StopGliding as usize) && entity.is_fall_flying() {
            // Java cannot cancel elytra in mid-air; landing / `canGlide` already stops it.
            if on_ground || !player.can_glide() {
                entity.set_fall_flying(false);
            }
        }

        let new_pos = player.feet_from_bedrock_pos(packet.position.to_f64());
        player.apply_client_move(
            server,
            ClientMove {
                position: Some(new_pos),
                yaw: Some(packet.yaw),
                pitch: Some(packet.pitch),
                head_yaw: Some(packet.head_yaw),
                on_ground,
                horizontal_collision,
            },
        );

        if let Some(block_actions) = packet.block_actions {
            for action in &block_actions {
                self.handle_player_block_action(player, server, action);
            }
        }
    }
}

impl BedrockClient {
    /// The Bedrock client glides straight out of creative flight, which vanilla `canGlide`
    /// forbids, so flight ends first and comes back if the glide is refused.
    fn start_gliding(player: &Arc<Player>, server: &Arc<Server>) {
        if player.get_entity().is_fall_flying() {
            return;
        }
        let was_flying = player.is_flying();
        let set_flying = |flying: bool| {
            player
                .abilities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .flying = flying;
        };
        if was_flying {
            set_flying(false);
        }
        if !player.try_to_start_fall_flying(server) {
            if was_flying {
                set_flying(true);
            }
            player.get_entity().stop_fall_flying();
        }
        if was_flying {
            player.send_abilities_update();
        }
    }

    /// Inputs are ignored until the client reaches a pending teleport, so stale
    /// positions (e.g. the death spot before a respawn) never count as movement or a fall.
    /// Returns true while still waiting.
    fn await_teleport(&self, player: &Arc<Player>, packet: &SPlayerAuthInput) -> bool {
        const TELEPORT_ERROR: f64 = 0.1;
        const RESEND_INPUTS: u32 = 20;

        let mut awaiting = player
            .awaiting_teleport
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some((_, target)) = *awaiting else {
            return false;
        };
        let position = player.feet_from_bedrock_pos(packet.position.to_f64());
        if (position.x - target.x).abs() < TELEPORT_ERROR
            && (position.y - target.y).abs() < TELEPORT_ERROR
            && (position.z - target.z).abs() < TELEPORT_ERROR
        {
            *awaiting = None;
            drop(awaiting);
            // Bedrock sends no `PlayerLoaded` after a respawn, reaching the spawn stands in
            player.set_client_loaded(true);
        } else if self
            .teleport_unconfirmed_inputs
            .fetch_add(1, Ordering::Relaxed)
            + 1
            >= RESEND_INPUTS
        {
            drop(awaiting);
            player.send_bedrock_teleport(self);
        }
        true
    }
}

fn sneaking_from_auth(
    player: &Player,
    flags: &pumpkin_protocol::codec::bitset::Bitset<66>,
) -> bool {
    if player.is_flying() {
        return flags.get(InputData::Descend as usize) || flags.get(InputData::SneakDown as usize);
    }
    // Stop wins when both edges arrive on the same tick.
    if flags.get(InputData::StopSneaking as usize) {
        false
    } else if flags.get(InputData::StartSneaking as usize) {
        true
    } else {
        player.get_entity().is_sneaking()
    }
}

fn java_input_from_auth(packet: &SPlayerAuthInput, sneaking: bool) -> i8 {
    let flags = &packet.input_data;
    let mouse = packet.input_mode.0 == InputMode::Mouse as u32;
    let classic_touch = packet.input_mode.0 == InputMode::Touch as u32
        && packet.interaction_model.0 == InteractionModel::Classic as i32;
    let (up, down, left, right) = if mouse || classic_touch {
        digital_move(flags, classic_touch)
    } else {
        analog_move(packet.analog_move)
    };

    let mut input = 0i8;
    if up {
        input |= SPlayerInput::FORWARD;
    }
    if down {
        input |= SPlayerInput::BACKWARD;
    }
    if left {
        input |= SPlayerInput::LEFT;
    }
    if right {
        input |= SPlayerInput::RIGHT;
    }
    if flags.get(InputData::JumpCurrentRaw as usize)
        || flags.get(InputData::JumpDown as usize)
        || flags.get(InputData::AutoJumpingInWater as usize)
    {
        input |= SPlayerInput::JUMP;
    }
    if sneaking {
        input |= SPlayerInput::SNEAK;
    }
    if flags.get(InputData::SprintDown as usize) {
        input |= SPlayerInput::SPRINT;
    }
    input
}

fn digital_move(
    flags: &pumpkin_protocol::codec::bitset::Bitset<66>,
    classic_touch: bool,
) -> (bool, bool, bool, bool) {
    let mut up = flags.get(InputData::Up as usize);
    let mut down = flags.get(InputData::Down as usize);
    let mut left = flags.get(InputData::Left as usize);
    let mut right = flags.get(InputData::Right as usize);
    if classic_touch {
        if flags.get(InputData::UpLeft as usize) {
            up = true;
            left = true;
        }
        if flags.get(InputData::UpRight as usize) {
            up = true;
            right = true;
        }
        if flags.get(InputData::DownLeft as usize) {
            down = true;
            left = true;
        }
        if flags.get(InputData::DownRight as usize) {
            down = true;
            right = true;
        }
    }
    (up, down, left, right)
}

fn analog_move(analog: Vector2<f32>) -> (bool, bool, bool, bool) {
    (
        analog.y > 0.0,
        analog.y < 0.0,
        analog.x > 0.0,
        analog.x < 0.0,
    )
}
