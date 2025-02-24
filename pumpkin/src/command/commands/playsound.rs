use async_trait::async_trait;
use pumpkin_data::sound::SoundCategory;
use pumpkin_util::text::TextComponent;
use rand::{Rng, thread_rng};

use crate::command::{
    CommandError, CommandExecutor, CommandSender,
    args::{
        Arg, ConsumedArgs, FindArg, bounded_num::BoundedNumArgumentConsumer,
        players::PlayersArgumentConsumer, position_3d::Position3DArgumentConsumer,
        sound::SoundArgumentConsumer, sound_category::SoundCategoryArgumentConsumer,
    },
    tree::CommandTree,
    tree::builder::argument,
};

/// Command: playsound <sound> [<source>] [<targets>] [<pos>] [<volume>] [<pitch>] [<minVolume>]
///
/// Plays a sound at specified position for target players.
/// - sound: The sound identifier to play
/// - source: Sound category (master, music, record, etc.)
/// - targets: Players who will hear the sound
/// - pos: Position to play the sound from
/// - volume: Sound volume (>=0, default: 1.0)
/// - pitch: Sound pitch (0.5-2.0, default: 1.0)
/// - minVolume: Minimum volume for distant players (0.0-1.0, default: 0.0)
const NAMES: [&str; 1] = ["playsound"];
const DESCRIPTION: &str = "Plays a sound at a position.";
const ARG_SOUND: &str = "sound";
const ARG_SOURCE: &str = "source";
const ARG_TARGETS: &str = "targets";
const ARG_POS: &str = "pos";
const ARG_VOLUME: &str = "volume";
const ARG_PITCH: &str = "pitch";
const ARG_MIN_VOLUME: &str = "minVolume";

// Volume must be >= 0, no upper limit
fn volume_consumer() -> BoundedNumArgumentConsumer<f32> {
    BoundedNumArgumentConsumer::new().name(ARG_VOLUME).min(0.0)
}

// Pitch must be between 0.0 and 2.0
// Values below 0.5 are treated as 0.5
fn pitch_consumer() -> BoundedNumArgumentConsumer<f32> {
    BoundedNumArgumentConsumer::new()
        .name(ARG_PITCH)
        .min(0.0)
        .max(2.0)
}

// Minimum volume must be between 0.0 and 1.0
// Controls the volume for players outside normal hearing range
fn min_volume_consumer() -> BoundedNumArgumentConsumer<f32> {
    BoundedNumArgumentConsumer::new()
        .name(ARG_MIN_VOLUME)
        .min(0.0)
        .max(1.0)
}

struct Executor;

#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        // Get required sound argument
        let sound = SoundArgumentConsumer::find_arg(args, ARG_SOUND)?;

        // Get optional sound category, defaults to Master
        let source = args
            .get(ARG_SOURCE)
            .map_or(SoundCategory::Master, |arg| match arg {
                Arg::SoundCategory(category) => *category,
                _ => SoundCategory::Master,
            });

        // Get target players, defaults to sender if not specified
        let targets = if let Ok(players) = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS) {
            players
        } else if let Some(player) = sender.as_player() {
            &[player.clone()]
        } else {
            return Ok(());
        };

        // Get optional position, defaults to target's position
        let position = Position3DArgumentConsumer::find_arg(args, ARG_POS).ok();

        // Get optional volume parameter
        let volume = match BoundedNumArgumentConsumer::<f32>::find_arg(args, ARG_VOLUME) {
            Ok(Ok(v)) => v,
            _ => 1.0, // Default volume
        };

        // Get optional pitch parameter
        let pitch = match BoundedNumArgumentConsumer::<f32>::find_arg(args, ARG_PITCH) {
            Ok(Ok(p)) => p.max(0.5), // Values below 0.5 are clamped
            _ => 1.0,                // Default pitch
        };

        // Get optional minimum volume (currently unused in implementation)
        let _min_volume = match BoundedNumArgumentConsumer::<f32>::find_arg(args, ARG_MIN_VOLUME) {
            Ok(Ok(v)) => v,
            _ => 0.0, // Default minimum volume
        };

        // Use same random seed for all targets to ensure sound synchronization
        let seed = thread_rng().r#gen::<f64>();

        // Track how many players actually received the sound
        let mut players_who_heard = 0;

        // Play sound for each target player
        for target in targets {
            let pos = position.unwrap_or(target.living_entity.entity.pos.load());

            // Check if player can hear the sound based on volume and distance
            let player_pos = target.living_entity.entity.pos.load();
            let distance = player_pos.squared_distance_to_vec(pos);
            let max_distance = 16.0 * volume; // 16 blocks is base distance at volume 1.0

            if distance <= max_distance.into() || _min_volume > 0.0 {
                target
                    .play_sound(sound as u16, source, &pos, volume, pitch, seed)
                    .await;
                players_who_heard += 1;
            }
        }

        // Send appropriate message based on results
        if players_who_heard == 0 {
            sender
                .send_message(TextComponent::translate("commands.playsound.failed", []))
                .await;
        } else {
            let sound_name = sound.to_name();
            if players_who_heard == 1 {
                sender
                    .send_message(TextComponent::translate(
                        "commands.playsound.success.single",
                        [
                            TextComponent::text(sound_name),
                            TextComponent::text(targets[0].gameprofile.name.clone()),
                        ],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        "commands.playsound.success.multiple",
                        [
                            TextComponent::text(sound_name),
                            TextComponent::text(players_who_heard.to_string()),
                        ],
                    ))
                    .await;
            }
        }

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_SOUND, SoundArgumentConsumer)
            .then(
                argument(ARG_SOURCE, SoundCategoryArgumentConsumer)
                    .then(
                        argument(ARG_TARGETS, PlayersArgumentConsumer)
                            .then(
                                argument(ARG_POS, Position3DArgumentConsumer)
                                    .then(
                                        argument(ARG_VOLUME, volume_consumer())
                                            .then(
                                                argument(ARG_PITCH, pitch_consumer())
                                                    .then(
                                                        argument(
                                                            ARG_MIN_VOLUME,
                                                            min_volume_consumer(),
                                                        )
                                                        .execute(Executor),
                                                    )
                                                    .execute(Executor),
                                            )
                                            .execute(Executor),
                                    )
                                    .execute(Executor),
                            )
                            .execute(Executor),
                    )
                    .execute(Executor),
            )
            .execute(Executor),
    )
}
