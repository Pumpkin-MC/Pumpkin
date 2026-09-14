use std::borrow::Cow;

use pumpkin_data::biome::Biome;
use pumpkin_data::structures::{StructureKeys, StructurePlacementType, StructureSet};
use pumpkin_data::tag::{self, RegistryKey};
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_world::generation::generator::biome_finder::find_closest_biome_3d;
use pumpkin_world::generation::generator::structure_finder::{
    find_nearest_structure, find_nearest_structure_start,
};
use rustc_hash::FxHashSet;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::resource_key::BIOME_REGISTRY;
use crate::command::argument_types::resource_or_tag::{
    POI_REGISTRY, ResourceOrTag, ResourceOrTagArgument, ResourceOrTagKeyArgument,
    STRUCTURE_REGISTRY,
};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Locates the closest structure, biome, or point of interest.";

const PERMISSION: &str = "minecraft:command.locate";

const ARG_STRUCTURE: &str = "structure";
const ARG_BIOME: &str = "biome";
const ARG_POI: &str = "poi";

/// The maximum structure search radius in chunk regions, matching vanilla's
/// `findNearestMapStructure` call in `LocateCommand`.
const STRUCTURE_SEARCH_RADIUS: i32 = 100;

/// Biome search parameters from vanilla's `LocateCommand`: a 6400 block
/// radius probed every 32 blocks horizontally and every 64 blocks vertically.
const BIOME_SEARCH_RADIUS: i32 = 6400;
const BIOME_SEARCH_HORIZONTAL_STEP: i32 = 32;
const BIOME_SEARCH_VERTICAL_STEP: i32 = 64;

/// The POI search radius in blocks, matching vanilla's `LocateCommand`.
const POI_SEARCH_RADIUS: i32 = 256;

static STRUCTURE_INVALID_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_LOCATE_STRUCTURE_INVALID,
    translation::java::COMMANDS_LOCATE_STRUCTURE_INVALID,
);

static STRUCTURE_NOT_FOUND_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_LOCATE_STRUCTURE_NOT_FOUND,
    translation::bedrock::COMMANDS_LOCATE_STRUCTURE_FAIL_NOSTRUCTUREFOUND,
);

static BIOME_NOT_FOUND_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_LOCATE_BIOME_NOT_FOUND,
    translation::bedrock::COMMANDS_LOCATE_BIOME_FAIL,
);

static POI_NOT_FOUND_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_LOCATE_POI_NOT_FOUND,
    translation::java::COMMANDS_LOCATE_POI_NOT_FOUND,
);

/// Builds the clickable green `[x, ~, z]` (or `[x, y, z]` when `absolute_y`)
/// coordinates component used by vanilla's locate feedback.
fn coordinates_text(pos: &BlockPos, absolute_y: bool) -> TextComponent {
    let x = pos.0.x;
    let z = pos.0.z;
    let y = if absolute_y {
        pos.0.y.to_string()
    } else {
        "~".to_string()
    };

    TextComponent::translate_cross(
        translation::java::CHAT_COORDINATES,
        translation::java::CHAT_COORDINATES,
        [
            TextComponent::text(x.to_string()),
            TextComponent::text(y.clone()),
            TextComponent::text(z.to_string()),
        ],
    )
    .color_named(NamedColor::Green)
    .click_event(ClickEvent::SuggestCommand {
        command: Cow::from(format!("/tp @s {x} {y} {z}")),
    })
    .hover_event(HoverEvent::show_text(TextComponent::translate_cross(
        translation::java::CHAT_COORDINATES_TOOLTIP,
        translation::java::CHAT_COORDINATES_TOOLTIP,
        [],
    )))
}

/// The first argument of the success messages: the searched id, with the
/// concretely found entry appended for tag searches, like vanilla's
/// `LocateCommand.showLocateResult`.
fn result_name(searched: &ResourceOrTag, found: &str) -> String {
    match searched {
        ResourceOrTag::Resource(_) => searched.printable(),
        ResourceOrTag::Tag(_) => format!("{} ({found})", searched.printable()),
    }
}

/// Vanilla reports the horizontal block distance for structures and POIs.
fn horizontal_distance(origin: &BlockPos, target: &BlockPos) -> i32 {
    let dx = f64::from(target.0.x - origin.0.x);
    let dz = f64::from(target.0.z - origin.0.z);
    dx.hypot(dz).floor().max(0.0) as i32
}

/// ... and the full 3D block distance for biomes.
fn absolute_distance(origin: &BlockPos, target: &BlockPos) -> i32 {
    let dx = f64::from(target.0.x - origin.0.x);
    let dy = f64::from(target.0.y - origin.0.y);
    let dz = f64::from(target.0.z - origin.0.z);
    (dx * dx + dy * dy + dz * dz).sqrt().floor().max(0.0) as i32
}

fn send_success(
    context: &CommandContext<'_>,
    java_key: &'static str,
    bedrock_key: &'static str,
    name: String,
    target: &BlockPos,
    absolute_y: bool,
    distance: i32,
) {
    context.source.send_feedback(
        TextComponent::translate_cross(
            java_key,
            bedrock_key,
            [
                TextComponent::text(name),
                coordinates_text(target, absolute_y),
                TextComponent::text(distance.to_string()),
            ],
        ),
        false,
    );
}

struct LocateStructureExecutor;

impl CommandExecutor for LocateStructureExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let searched = context.get_argument::<ResourceOrTag>(ARG_STRUCTURE)?;

        // The generator's placement data models vanilla's structure sets,
        // so ids resolve against those. There is no structure tag data,
        // hence tags cannot name any known structure (yet).
        let set = if let ResourceOrTag::Resource(id) = searched
            && id.is_vanilla()
        {
            StructureSet::get(id.path())
        } else {
            None
        };

        let Some(set) = set else {
            return Err(STRUCTURE_INVALID_ERROR_TYPE
                .create_without_context(TextComponent::text(searched.printable())));
        };

        let origin = BlockPos::floored_v(context.source.position);

        let world = context.source.world();
        let seed = world.level.seed.0;
        let world_gen = world.level.world_gen.load_full();

        let found = match &set.placement.placement_type {
            // Strongholds come out of the pre-computed ring cache, which
            // already holds positions they really occupy.
            StructurePlacementType::ConcentricRings(_) => {
                world_gen.global_structure_cache().and_then(|global_cache| {
                    find_nearest_structure(
                        origin,
                        &[&set.placement],
                        STRUCTURE_SEARCH_RADIUS,
                        seed as i64,
                        global_cache,
                    )
                })
            }
            // Everything else is spread over a grid whose candidate chunks
            // are only *possible* sites: the biome at a candidate can still
            // reject every structure in the set. Resolving the start makes
            // sure the reported position actually holds one.
            StructurePlacementType::RandomSpread(_) => {
                let targets: Vec<StructureKeys> =
                    set.structures.iter().map(|entry| entry.structure).collect();
                find_nearest_structure_start(
                    origin,
                    set,
                    &targets,
                    STRUCTURE_SEARCH_RADIUS,
                    &world_gen,
                )
            }
        };

        let Some(target) = found else {
            return Err(STRUCTURE_NOT_FOUND_ERROR_TYPE
                .create_without_context(TextComponent::text(searched.printable())));
        };

        let distance = horizontal_distance(&origin, &target);
        send_success(
            context,
            translation::java::COMMANDS_LOCATE_STRUCTURE_SUCCESS,
            translation::bedrock::COMMANDS_LOCATE_STRUCTURE_SUCCESS,
            searched.printable(),
            &target,
            false,
            distance,
        );

        Ok(distance)
    }
}

struct LocateBiomeExecutor;

impl CommandExecutor for LocateBiomeExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let searched = context.get_argument::<ResourceOrTag>(ARG_BIOME)?.clone();

        let targets: FxHashSet<u8> = match &searched {
            ResourceOrTag::Resource(id) => Biome::from_name(id.path())
                .map(|biome| biome.id)
                .into_iter()
                .collect(),
            ResourceOrTag::Tag(id) => {
                tag::get_tag_values(RegistryKey::WorldgenBiome, &id.to_string())
                    .into_iter()
                    .flatten()
                    .filter_map(|name| Biome::from_name(name))
                    .map(|biome| biome.id)
                    .collect()
            }
        };

        let not_found = || {
            BIOME_NOT_FOUND_ERROR_TYPE
                .create_without_context(TextComponent::text(searched.printable()))
        };
        if targets.is_empty() {
            return Err(not_found());
        }

        let origin = BlockPos::floored_v(context.source.position);
        let world = context.source.world().clone();
        let world_gen = world.level.world_gen.load_full();

        let found = find_closest_biome_3d(
            &world_gen,
            origin,
            &targets,
            BIOME_SEARCH_RADIUS,
            BIOME_SEARCH_HORIZONTAL_STEP,
            BIOME_SEARCH_VERTICAL_STEP,
        );

        let Some((target, biome)) = found else {
            return Err(not_found());
        };

        let distance = absolute_distance(&origin, &target);
        send_success(
            context,
            translation::java::COMMANDS_LOCATE_BIOME_SUCCESS,
            translation::bedrock::COMMANDS_LOCATE_BIOME_SUCCESS,
            result_name(&searched, &format!("minecraft:{}", biome.registry_id)),
            &target,
            true,
            distance,
        );

        Ok(distance)
    }
}

struct LocatePoiExecutor;

impl CommandExecutor for LocatePoiExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let searched = context.get_argument::<ResourceOrTag>(ARG_POI)?;

        // POI entries store namespaced type ids, tag data uses bare
        // vanilla names; normalize everything to `namespace:path`.
        let targets: FxHashSet<String> = match searched {
            ResourceOrTag::Resource(id) => std::iter::once(id.to_string()).collect(),
            ResourceOrTag::Tag(id) => {
                tag::get_tag_values(RegistryKey::PointOfInterestType, &id.to_string())
                    .into_iter()
                    .flatten()
                    .map(|name| {
                        if name.contains(':') {
                            (*name).to_string()
                        } else {
                            format!("minecraft:{name}")
                        }
                    })
                    .collect()
            }
        };

        let origin = BlockPos::floored_v(context.source.position);
        let world = context.source.world().clone();

        let found = {
            let mut poi_storage = world
                .portal_poi
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            poi_storage.find_closest_matching(origin, POI_SEARCH_RADIUS, |poi_type| {
                targets.contains(poi_type)
            })
        };

        let Some((target, poi_type)) = found else {
            return Err(POI_NOT_FOUND_ERROR_TYPE
                .create_without_context(TextComponent::text(searched.printable())));
        };

        let distance = horizontal_distance(&origin, &target);
        send_success(
            context,
            translation::java::COMMANDS_LOCATE_POI_SUCCESS,
            translation::java::COMMANDS_LOCATE_POI_SUCCESS,
            result_name(searched, &poi_type),
            &target,
            false,
            distance,
        );

        Ok(distance)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("locate", DESCRIPTION)
            .requires(PERMISSION)
            .then(
                literal("structure").then(
                    argument(
                        ARG_STRUCTURE,
                        ResourceOrTagKeyArgument(STRUCTURE_REGISTRY.clone()),
                    )
                    .executes(LocateStructureExecutor),
                ),
            )
            .then(
                literal("biome").then(
                    argument(ARG_BIOME, ResourceOrTagArgument(BIOME_REGISTRY.clone()))
                        .executes(LocateBiomeExecutor),
                ),
            )
            .then(
                literal("poi").then(
                    argument(ARG_POI, ResourceOrTagArgument(POI_REGISTRY.clone()))
                        .executes(LocatePoiExecutor),
                ),
            ),
    );
}
