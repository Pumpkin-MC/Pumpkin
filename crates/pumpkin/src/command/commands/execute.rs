use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::block::BlockArgumentType;
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::argument_types::coordinates::rotation::RotationArgumentType;
use crate::command::argument_types::coordinates::vec3::Vec3ArgumentType;
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::entity_anchor::EntityAnchorArgumentType;
use crate::command::argument_types::resource::{ENTITY_TYPE_ARGUMENT, ResourceArgument};
use crate::command::argument_types::resource_key::ResourceKeyArgument;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::attached::{CommandNodeId, NodeId};
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::tree::Tree;
use crate::command::node::{RedirectModifier, Redirection};
use crate::entity::r#type::from_type;
use pumpkin_util::PermissionLvl;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use std::sync::Arc;
use uuid::Uuid;

const DESCRIPTION: &str = "Execute a command with a modified context.";
const PERMISSION: &str = "minecraft:command.execute";

static ERROR_INVALID_DIMENSION: CommandErrorType<1> =
    CommandErrorType::new("argument.dimension.invalid", "argument.dimension.invalid");

static DIMENSION_REGISTRY: &Identifier = &Identifier::vanilla_static("dimension");

fn execute_as_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let targets = EntityArgumentType::get_optional_entities(context, "targets").await?;
        let mut sources = Vec::new();
        for target in targets {
            let mut source = context.source.as_ref().clone();
            let display_name = target.get_display_name().await;
            let name = target.get_name().get_text();
            source.entity = Some(target.clone());
            source.name = name;
            source.display_name = display_name;
            sources.push(Arc::new(source));
        }
        Ok(sources)
    })
}

fn execute_at_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let targets = EntityArgumentType::get_optional_entities(context, "targets").await?;
        let mut sources = Vec::new();
        for target in targets {
            let entity = target.get_entity();
            let mut source = context.source.as_ref().clone();
            source.position = entity.pos.load();
            source.rotation = Vector2::new(entity.yaw.load(), entity.pitch.load());
            source.world = Some(entity.world.load().clone());
            sources.push(Arc::new(source));
        }
        Ok(sources)
    })
}

fn execute_in_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let dimension_key = ResourceKeyArgument::get_registry_key(
            context,
            "dimension",
            &Identifier::vanilla_static("dimension"),
            &ERROR_INVALID_DIMENSION,
        )?;
        let dimension_name = dimension_key.identifier.to_string();
        let server = context.server();
        let worlds = server.worlds.load();
        let target_world = worlds
            .iter()
            .find(|w| w.dimension.minecraft_name == dimension_name);

        target_world.map_or_else(
            || {
                Err(ERROR_INVALID_DIMENSION
                    .create_without_context(TextComponent::text(dimension_name)))
            },
            |target_world| {
                let mut source = context.source.as_ref().clone();
                source.world = Some(target_world.clone());
                Ok(vec![Arc::new(source)])
            },
        )
    })
}

fn execute_positioned_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let pos = Vec3ArgumentType::get_vector3(context, "pos")?;
        let mut source = context.source.as_ref().clone();
        source.position = pos;
        Ok(vec![Arc::new(source)])
    })
}

fn execute_positioned_as_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let targets = EntityArgumentType::get_optional_entities(context, "targets").await?;
        let mut sources = Vec::new();
        for target in targets {
            let mut source = context.source.as_ref().clone();
            source.position = target.get_entity().pos.load();
            sources.push(Arc::new(source));
        }
        Ok(sources)
    })
}

fn execute_rotated_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let rot_coords = RotationArgumentType::get(context, "rotation")?;
        let rot = rot_coords.rotation(&context.source);
        let mut source = context.source.as_ref().clone();
        source.rotation = rot;
        Ok(vec![Arc::new(source)])
    })
}

fn execute_rotated_as_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let targets = EntityArgumentType::get_optional_entities(context, "targets").await?;
        let mut sources = Vec::new();
        for target in targets {
            let entity = target.get_entity();
            let mut source = context.source.as_ref().clone();
            source.rotation = Vector2::new(entity.yaw.load(), entity.pitch.load());
            sources.push(Arc::new(source));
        }
        Ok(sources)
    })
}

fn execute_if_entity_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let targets = EntityArgumentType::get_optional_entities(context, "targets").await?;
        if targets.is_empty() {
            Ok(vec![])
        } else {
            Ok(vec![context.source.clone()])
        }
    })
}

fn execute_unless_entity_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let targets = EntityArgumentType::get_optional_entities(context, "targets").await?;
        if targets.is_empty() {
            Ok(vec![context.source.clone()])
        } else {
            Ok(vec![])
        }
    })
}

fn execute_align_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let axes = StringArgumentType::get(context, "axes")?;
        let mut source = context.source.as_ref().clone();

        let has_x = axes.contains('x');
        let has_y = axes.contains('y');
        let has_z = axes.contains('z');

        if has_x {
            source.position.x = source.position.x.floor();
        }
        if has_y {
            source.position.y = source.position.y.floor();
        }
        if has_z {
            source.position.z = source.position.z.floor();
        }

        Ok(vec![Arc::new(source)])
    })
}

fn execute_anchored_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let anchor = EntityAnchorArgumentType::get(context, "anchor")?;
        let mut source = context.source.as_ref().clone();
        source.entity_anchor = anchor;
        Ok(vec![Arc::new(source)])
    })
}

fn execute_facing_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let pos = Vec3ArgumentType::get_vector3(context, "pos")?;
        let mut source = context.source.as_ref().clone();

        let dx = pos.x - source.position.x;
        let dy = pos.y - source.position.y;
        let dz = pos.z - source.position.z;

        let xz_dist = dx.hypot(dz);
        let yaw = (dz.atan2(dx).to_degrees() as f32) - 90.0;
        let pitch = -(dy.atan2(xz_dist).to_degrees() as f32);

        source.rotation = Vector2::new(yaw, pitch);
        Ok(vec![Arc::new(source)])
    })
}

fn execute_facing_entity_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let targets = EntityArgumentType::get_optional_entities(context, "targets").await?;
        let anchor = EntityAnchorArgumentType::get(context, "anchor")?;
        let mut sources = Vec::new();

        for target in targets {
            let target_pos = anchor.position_at_entity(target.get_entity());
            let mut source = context.source.as_ref().clone();

            let dx = target_pos.x - source.position.x;
            let dy = target_pos.y - source.position.y;
            let dz = target_pos.z - source.position.z;

            let xz_dist = dx.hypot(dz);
            let yaw = (dz.atan2(dx).to_degrees() as f32) - 90.0;
            let pitch = -(dy.atan2(xz_dist).to_degrees() as f32);

            source.rotation = Vector2::new(yaw, pitch);
            sources.push(Arc::new(source));
        }
        Ok(sources)
    })
}

fn execute_if_block_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let pos = BlockPosArgumentType::get_block_pos(context, "pos")?;
        let expected_block = BlockArgumentType::get(context, "block")?;

        if let Some(ref world) = context.source.world {
            let block = world.get_block(&pos);
            if block == expected_block {
                return Ok(vec![context.source.clone()]);
            }
        }
        Ok(vec![])
    })
}

fn execute_unless_block_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let pos = BlockPosArgumentType::get_block_pos(context, "pos")?;
        let expected_block = BlockArgumentType::get(context, "block")?;

        if let Some(ref world) = context.source.world {
            let block = world.get_block(&pos);
            if block != expected_block {
                return Ok(vec![context.source.clone()]);
            }
        } else {
            return Ok(vec![context.source.clone()]);
        }
        Ok(vec![])
    })
}

fn execute_if_loaded_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let pos = BlockPosArgumentType::get_block_pos(context, "pos")?;

        if let Some(ref world) = context.source.world
            && world.is_loaded(&pos)
        {
            return Ok(vec![context.source.clone()]);
        }
        Ok(vec![])
    })
}

fn execute_unless_loaded_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let pos = BlockPosArgumentType::get_block_pos(context, "pos")?;

        if let Some(ref world) = context.source.world {
            if !world.is_loaded(&pos) {
                return Ok(vec![context.source.clone()]);
            }
        } else {
            return Ok(vec![context.source.clone()]);
        }
        Ok(vec![])
    })
}

fn execute_if_dimension_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let dimension_key = ResourceKeyArgument::get_registry_key(
            context,
            "dimension",
            &Identifier::vanilla_static("dimension"),
            &ERROR_INVALID_DIMENSION,
        )?;
        let dimension_name = dimension_key.identifier.to_string();

        if let Some(ref world) = context.source.world
            && world.dimension.minecraft_name == dimension_name
        {
            return Ok(vec![context.source.clone()]);
        }
        Ok(vec![])
    })
}

fn execute_unless_dimension_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let dimension_key = ResourceKeyArgument::get_registry_key(
            context,
            "dimension",
            &Identifier::vanilla_static("dimension"),
            &ERROR_INVALID_DIMENSION,
        )?;
        let dimension_name = dimension_key.identifier.to_string();

        if let Some(ref world) = context.source.world {
            if world.dimension.minecraft_name != dimension_name {
                return Ok(vec![context.source.clone()]);
            }
        } else {
            return Ok(vec![context.source.clone()]);
        }
        Ok(vec![])
    })
}

fn execute_summon_modifier<'a>(
    context: &'a CommandContext,
) -> crate::command::node::RedirectModifierResult<'a> {
    Box::pin(async move {
        let entity_type = ResourceArgument::get_summonable_entity_type(context, "entity_type")?;
        let entity = from_type(
            entity_type,
            context.source.position,
            context.source.world(),
            Uuid::new_v4(),
        );
        context.source.world().spawn_entity(entity.clone()).await;
        let mut source = context.source.as_ref().clone();
        source.entity = Some(entity);
        Ok(vec![Arc::new(source)])
    })
}

#[allow(clippy::too_many_lines)]
pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let builder = command("execute", DESCRIPTION)
        .requires(PERMISSION)
        .then(literal("run").redirect(Redirection::Root))
        .then(
            literal("as").then(argument("targets", EntityArgumentType::Entities).fork(
                Redirection::Root,
                RedirectModifier::Custom(Arc::new(execute_as_modifier)),
            )),
        )
        .then(
            literal("at").then(argument("targets", EntityArgumentType::Entities).fork(
                Redirection::Root,
                RedirectModifier::Custom(Arc::new(execute_at_modifier)),
            )),
        )
        .then(literal("in").then(
            argument("dimension", ResourceKeyArgument(DIMENSION_REGISTRY)).redirect_with_modifier(
                Redirection::Root,
                RedirectModifier::Custom(Arc::new(execute_in_modifier)),
            ),
        ))
        .then(
            literal("positioned")
                .then(
                    literal("as").then(argument("targets", EntityArgumentType::Entities).fork(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_positioned_as_modifier)),
                    )),
                )
                .then(
                    argument("pos", Vec3ArgumentType::Default).redirect_with_modifier(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_positioned_modifier)),
                    ),
                ),
        )
        .then(
            literal("rotated")
                .then(
                    literal("as").then(argument("targets", EntityArgumentType::Entities).fork(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_rotated_as_modifier)),
                    )),
                )
                .then(
                    argument("rotation", RotationArgumentType).redirect_with_modifier(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_rotated_modifier)),
                    ),
                ),
        )
        .then(literal("align").then(
            argument("axes", StringArgumentType::SingleWord).redirect_with_modifier(
                Redirection::Root,
                RedirectModifier::Custom(Arc::new(execute_align_modifier)),
            ),
        ))
        .then(literal("anchored").then(
            argument("anchor", EntityAnchorArgumentType).redirect_with_modifier(
                Redirection::Root,
                RedirectModifier::Custom(Arc::new(execute_anchored_modifier)),
            ),
        ))
        .then(
            literal("facing")
                .then(literal("entity").then(
                    argument("targets", EntityArgumentType::Entities).then(
                        argument("anchor", EntityAnchorArgumentType).fork(
                            Redirection::Root,
                            RedirectModifier::Custom(Arc::new(execute_facing_entity_modifier)),
                        ),
                    ),
                ))
                .then(
                    argument("pos", Vec3ArgumentType::Default).redirect_with_modifier(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_facing_modifier)),
                    ),
                ),
        )
        .then(
            literal("if")
                .then(literal("entity").then(
                    argument("targets", EntityArgumentType::Entities).redirect_with_modifier(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_if_entity_modifier)),
                    ),
                ))
                .then(
                    literal("block").then(argument("pos", BlockPosArgumentType).then(
                        argument("block", BlockArgumentType).redirect_with_modifier(
                            Redirection::Root,
                            RedirectModifier::Custom(Arc::new(execute_if_block_modifier)),
                        ),
                    )),
                )
                .then(literal("loaded").then(
                    argument("pos", BlockPosArgumentType).redirect_with_modifier(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_if_loaded_modifier)),
                    ),
                ))
                .then(
                    literal("dimension").then(
                        argument("dimension", ResourceKeyArgument(DIMENSION_REGISTRY))
                            .redirect_with_modifier(
                                Redirection::Root,
                                RedirectModifier::Custom(Arc::new(execute_if_dimension_modifier)),
                            ),
                    ),
                ),
        )
        .then(
            literal("unless")
                .then(literal("entity").then(
                    argument("targets", EntityArgumentType::Entities).redirect_with_modifier(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_unless_entity_modifier)),
                    ),
                ))
                .then(
                    literal("block").then(argument("pos", BlockPosArgumentType).then(
                        argument("block", BlockArgumentType).redirect_with_modifier(
                            Redirection::Root,
                            RedirectModifier::Custom(Arc::new(execute_unless_block_modifier)),
                        ),
                    )),
                )
                .then(literal("loaded").then(
                    argument("pos", BlockPosArgumentType).redirect_with_modifier(
                        Redirection::Root,
                        RedirectModifier::Custom(Arc::new(execute_unless_loaded_modifier)),
                    ),
                ))
                .then(
                    literal("dimension").then(
                        argument("dimension", ResourceKeyArgument(DIMENSION_REGISTRY))
                            .redirect_with_modifier(
                                Redirection::Root,
                                RedirectModifier::Custom(Arc::new(
                                    execute_unless_dimension_modifier,
                                )),
                            ),
                    ),
                ),
        )
        .then(literal("summon").then(
            argument("entity_type", ENTITY_TYPE_ARGUMENT.clone()).redirect_with_modifier(
                Redirection::Root,
                RedirectModifier::Custom(Arc::new(execute_summon_modifier)),
            ),
        ));

    let execute_node_id = dispatcher.register(builder);

    set_redirects_to_execute(
        &mut dispatcher.tree,
        NodeId::from(execute_node_id),
        execute_node_id,
    );
}

fn set_redirects_to_execute(tree: &mut Tree, parent: NodeId, execute_id: CommandNodeId) {
    for child_id in tree.get_children(parent) {
        if let Some(redirect) = tree[child_id].redirect()
            && matches!(redirect, Redirection::Root)
            && tree[child_id].name() != "run"
        {
            tree[child_id].set_redirect(Some(Redirection::Local(NodeId::from(execute_id))));
        }
        set_redirects_to_execute(tree, child_id, execute_id);
    }
}
