use std::{borrow::Cow, sync::Arc, time::Instant};

use pumpkin_data::{
    biome::Biome,
    tag::{RegistryKey, get_tag_ids},
    translation,
};
use pumpkin_util::{
    PermissionLvl,
    math::position::BlockPos,
    permission::{Permission, PermissionDefault, PermissionRegistry},
    text::{TextComponent, click::ClickEvent, color::NamedColor, hover::HoverEvent},
};
use tokio::sync::oneshot;

use crate::command::{
    argument_builder::{ArgumentBuilder, argument, command, literal},
    argument_types::resource_or_tag::{ResourceOrTag, ResourceOrTagArgument},
    context::command_context::CommandContext,
    errors::error_types::CommandErrorType,
    node::{CommandExecutor, CommandExecutorResult, dispatcher::CommandDispatcher},
};

const DESCRIPTION: &str = "Locates the closest biome.";
const PERMISSION: &str = "minecraft:command.locate";

const MAX_BIOME_SEARCH_RADIUS: i32 = 6400;
const BIOME_SAMPLE_RESOLUTION_HORIZONTAL: i32 = 32;
const BIOME_SAMPLE_RESOLUTION_VERTICAL: i32 = 64;

static ERROR_BIOME_NOT_FOUND: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_LOCATE_BIOME_NOT_FOUND,
    translation::java::COMMANDS_LOCATE_BIOME_NOT_FOUND,
);

struct LocateBiomeExecutor;

impl CommandExecutor for LocateBiomeExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let requested = ResourceOrTagArgument::get(context, "biome")?;
            let target_biomes = resolve_biomes(requested).ok_or_else(|| {
                ERROR_BIOME_NOT_FOUND
                    .create_without_context(TextComponent::text(requested.printable()))
            })?;

            let source_position = context.source.position;
            let origin = BlockPos::new(
                source_position.x.floor() as i32,
                source_position.y.floor() as i32,
                source_position.z.floor() as i32,
            );
            let world_gen = Arc::clone(&context.world().level.world_gen);
            let generation_pool = context.world().level.gen_pool.clone();
            let (sender, receiver) = oneshot::channel();
            let started = Instant::now();

            let locate = move || {
                let result =
                    pumpkin_world::generation::generator::biome_finder::find_closest_biome_parallel(
                        &world_gen,
                        origin,
                        MAX_BIOME_SEARCH_RADIUS,
                        BIOME_SAMPLE_RESOLUTION_HORIZONTAL,
                        BIOME_SAMPLE_RESOLUTION_VERTICAL,
                        &target_biomes,
                    );
                let _ = sender.send(result);
            };

            if let Some(pool) = generation_pool {
                pool.spawn(locate);
            } else {
                drop(tokio::task::spawn_blocking(locate));
            }

            let result = receiver.await.ok().flatten().ok_or_else(|| {
                ERROR_BIOME_NOT_FOUND
                    .create_without_context(TextComponent::text(requested.printable()))
            })?;

            let distance = distance_3d(origin, result.position);
            let coordinates = coordinate_component(result.position);
            let found_name = if requested.is_tag {
                format!(
                    "{} (minecraft:{})",
                    requested.printable(),
                    result.biome.registry_id
                )
            } else {
                requested.printable()
            };

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_LOCATE_BIOME_SUCCESS,
                        translation::java::COMMANDS_LOCATE_BIOME_SUCCESS,
                        [
                            TextComponent::text(found_name.clone()),
                            coordinates,
                            TextComponent::text(distance.to_string()),
                        ],
                    ),
                    false,
                )
                .await;

            tracing::info!(
                "Locating element {found_name} took {} ms",
                started.elapsed().as_millis()
            );

            Ok(distance)
        })
    }
}

fn resolve_biomes(requested: &ResourceOrTag) -> Option<Vec<u8>> {
    if requested.is_tag {
        get_tag_ids(
            RegistryKey::WorldgenBiome,
            &requested.identifier.to_string(),
        )
        .map(|ids| ids.iter().filter_map(|&id| u8::try_from(id).ok()).collect())
    } else {
        (requested.identifier.namespace() == "minecraft")
            .then(|| Biome::from_name(requested.identifier.path()))
            .flatten()
            .map(|biome| vec![biome.id])
    }
}

fn coordinate_component(position: BlockPos) -> TextComponent {
    let y = position.0.y.to_string();
    let coordinates = format!("{} {y} {}", position.0.x, position.0.z);

    TextComponent::wrap_in_square_brackets(
        TextComponent::translate_cross(
            translation::java::CHAT_COORDINATES,
            translation::java::CHAT_COORDINATES,
            [
                TextComponent::text(position.0.x.to_string()),
                TextComponent::text(y),
                TextComponent::text(position.0.z.to_string()),
            ],
        )
        .color_named(NamedColor::Green)
        .click_event(ClickEvent::SuggestCommand {
            command: Cow::Owned(coordinates),
        })
        .hover_event(HoverEvent::show_text(TextComponent::translate_cross(
            translation::java::CHAT_COORDINATES_TOOLTIP,
            translation::java::CHAT_COORDINATES_TOOLTIP,
            [],
        ))),
    )
}

fn distance_3d(from: BlockPos, to: BlockPos) -> i32 {
    let dx = i64::from(to.0.x) - i64::from(from.0.x);
    let dy = i64::from(to.0.y) - i64::from(from.0.y);
    let dz = i64::from(to.0.z) - i64::from(from.0.z);
    ((dx * dx + dy * dy + dz * dz) as f32).sqrt().floor() as i32
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("locate", DESCRIPTION).requires(PERMISSION).then(
            literal("biome").then(
                argument("biome", ResourceOrTagArgument::biome()).executes(LocateBiomeExecutor),
            ),
        ),
    );
}

#[cfg(test)]
mod tests {
    use pumpkin_util::math::position::BlockPos;

    use super::distance_3d;

    #[test]
    fn biome_distance_includes_height() {
        assert_eq!(
            distance_3d(BlockPos::new(0, 0, 0), BlockPos::new(3, 4, 0)),
            5
        );
    }
}
