use super::{
    ARG_TEAM, ARG_VALUE, COLLISION_RULE_UNCHANGED_ERROR, COLOR_UNCHANGED_ERROR,
    FRIENDLY_FIRE_ALREADY_DISABLED_ERROR, FRIENDLY_FIRE_ALREADY_ENABLED_ERROR,
    NAME_UNCHANGED_ERROR, NAMETAG_VISIBILITY_UNCHANGED_ERROR,
    SEE_FRIENDLY_INVISIBLES_ALREADY_DISABLED_ERROR, SEE_FRIENDLY_INVISIBLES_ALREADY_ENABLED_ERROR,
    TEAM_NOT_FOUND_ERROR,
};
use crate::command::argument_builder::{
    ArgumentBuilder, LiteralArgumentBuilder, argument, literal,
};
use crate::command::argument_types::core::bool::BoolArgumentType;
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::team::TeamArgumentType;
use crate::command::argument_types::team_color::TeamColorArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::world::scoreboard::{CollisionRule, NameTagVisibility};
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

struct TeamModifyColorExecutor;

impl CommandExecutor for TeamModifyColorExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let team_name = TeamArgumentType::get(context, ARG_TEAM)?;
            let new_color = TeamColorArgumentType::get(context, ARG_VALUE)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let mut team = scoreboard
                .get_teams()
                .get(team_name)
                .ok_or_else(|| {
                    TEAM_NOT_FOUND_ERROR
                        .create_without_context(TextComponent::text(team_name.to_string()))
                })?
                .clone();

            if team.color == new_color {
                return Err(COLOR_UNCHANGED_ERROR.create_without_context());
            }

            team.color = new_color;
            let team_display_name = team.display_name.clone();
            scoreboard.update_team(world, team);

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_TEAM_OPTION_COLOR_SUCCESS,
                        translation::java::COMMANDS_TEAM_OPTION_COLOR_SUCCESS,
                        [
                            team_display_name,
                            TextComponent::text(format!("{new_color:?}").to_lowercase()),
                        ],
                    ),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

struct TeamModifyDisplayNameExecutor;

impl CommandExecutor for TeamModifyDisplayNameExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let team_name = TeamArgumentType::get(context, ARG_TEAM)?;
            let new_name_str = StringArgumentType::get(context, ARG_VALUE)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let mut team = scoreboard
                .get_teams()
                .get(team_name)
                .ok_or_else(|| {
                    TEAM_NOT_FOUND_ERROR
                        .create_without_context(TextComponent::text(team_name.to_string()))
                })?
                .clone();

            let new_display_name = TextComponent::text(new_name_str.to_string());
            if team.display_name == new_display_name {
                return Err(NAME_UNCHANGED_ERROR.create_without_context());
            }

            team.display_name = new_display_name.clone();
            let team_display_name = team.display_name.clone();
            scoreboard.update_team(world, team);

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_TEAM_OPTION_NAME_SUCCESS,
                        translation::java::COMMANDS_TEAM_OPTION_NAME_SUCCESS,
                        [team_display_name, new_display_name],
                    ),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

struct TeamModifyPrefixExecutor;

impl CommandExecutor for TeamModifyPrefixExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let team_name = TeamArgumentType::get(context, ARG_TEAM)?;
            let new_prefix_str = StringArgumentType::get(context, ARG_VALUE)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let mut team = scoreboard
                .get_teams()
                .get(team_name)
                .ok_or_else(|| {
                    TEAM_NOT_FOUND_ERROR
                        .create_without_context(TextComponent::text(team_name.to_string()))
                })?
                .clone();

            let new_prefix = TextComponent::text(new_prefix_str.to_string());
            team.player_prefix = new_prefix.clone();
            let team_display_name = team.display_name.clone();
            scoreboard.update_team(world, team);

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_TEAM_OPTION_PREFIX_SUCCESS,
                        translation::java::COMMANDS_TEAM_OPTION_PREFIX_SUCCESS,
                        [team_display_name, new_prefix],
                    ),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

struct TeamModifySuffixExecutor;

impl CommandExecutor for TeamModifySuffixExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let team_name = TeamArgumentType::get(context, ARG_TEAM)?;
            let new_suffix_str = StringArgumentType::get(context, ARG_VALUE)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let mut team = scoreboard
                .get_teams()
                .get(team_name)
                .ok_or_else(|| {
                    TEAM_NOT_FOUND_ERROR
                        .create_without_context(TextComponent::text(team_name.to_string()))
                })?
                .clone();

            let new_suffix = TextComponent::text(new_suffix_str.to_string());
            team.player_suffix = new_suffix.clone();
            let team_display_name = team.display_name.clone();
            scoreboard.update_team(world, team);

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_TEAM_OPTION_SUFFIX_SUCCESS,
                        translation::java::COMMANDS_TEAM_OPTION_SUFFIX_SUCCESS,
                        [team_display_name, new_suffix],
                    ),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

struct TeamModifyFriendlyFireExecutor;

impl CommandExecutor for TeamModifyFriendlyFireExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let team_name = TeamArgumentType::get(context, ARG_TEAM)?;
            let value = BoolArgumentType::get(context, ARG_VALUE)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let mut team = scoreboard
                .get_teams()
                .get(team_name)
                .ok_or_else(|| {
                    TEAM_NOT_FOUND_ERROR
                        .create_without_context(TextComponent::text(team_name.to_string()))
                })?
                .clone();

            let is_enabled = (team.options & 0x01) != 0;
            if value == is_enabled {
                if value {
                    return Err(FRIENDLY_FIRE_ALREADY_ENABLED_ERROR.create_without_context());
                }
                return Err(FRIENDLY_FIRE_ALREADY_DISABLED_ERROR.create_without_context());
            }

            if value {
                team.options |= 0x01;
            } else {
                team.options &= !0x01;
            }

            let team_display_name = team.display_name.clone();
            scoreboard.update_team(world, team);

            let key = if value {
                translation::java::COMMANDS_TEAM_OPTION_FRIENDLYFIRE_ENABLED
            } else {
                translation::java::COMMANDS_TEAM_OPTION_FRIENDLYFIRE_DISABLED
            };

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(key, key, [team_display_name]),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

struct TeamModifySeeFriendlyInvisiblesExecutor;

impl CommandExecutor for TeamModifySeeFriendlyInvisiblesExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let team_name = TeamArgumentType::get(context, ARG_TEAM)?;
            let value = BoolArgumentType::get(context, ARG_VALUE)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let mut team = scoreboard
                .get_teams()
                .get(team_name)
                .ok_or_else(|| {
                    TEAM_NOT_FOUND_ERROR
                        .create_without_context(TextComponent::text(team_name.to_string()))
                })?
                .clone();

            let is_enabled = (team.options & 0x02) != 0;
            if value == is_enabled {
                if value {
                    return Err(
                        SEE_FRIENDLY_INVISIBLES_ALREADY_ENABLED_ERROR.create_without_context()
                    );
                }
                return Err(SEE_FRIENDLY_INVISIBLES_ALREADY_DISABLED_ERROR.create_without_context());
            }

            if value {
                team.options |= 0x02;
            } else {
                team.options &= !0x02;
            }

            let team_display_name = team.display_name.clone();
            scoreboard.update_team(world, team);

            let key = if value {
                translation::java::COMMANDS_TEAM_OPTION_SEEFRIENDLYINVISIBLES_ENABLED
            } else {
                translation::java::COMMANDS_TEAM_OPTION_SEEFRIENDLYINVISIBLES_DISABLED
            };

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(key, key, [team_display_name]),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

struct TeamModifyNametagVisibilityExecutor {
    value: NameTagVisibility,
}

impl CommandExecutor for TeamModifyNametagVisibilityExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let team_name = TeamArgumentType::get(context, ARG_TEAM)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let mut team = scoreboard
                .get_teams()
                .get(team_name)
                .ok_or_else(|| {
                    TEAM_NOT_FOUND_ERROR
                        .create_without_context(TextComponent::text(team_name.to_string()))
                })?
                .clone();

            let is_unchanged = matches!(
                (&team.nametag_visibility, &self.value),
                (NameTagVisibility::Always, NameTagVisibility::Always)
                    | (NameTagVisibility::Never, NameTagVisibility::Never)
                    | (
                        NameTagVisibility::HideForOtherTeams,
                        NameTagVisibility::HideForOtherTeams
                    )
                    | (
                        NameTagVisibility::HideForOwnTeam,
                        NameTagVisibility::HideForOwnTeam
                    )
            );

            if is_unchanged {
                return Err(NAMETAG_VISIBILITY_UNCHANGED_ERROR.create_without_context());
            }

            team.nametag_visibility = match self.value {
                NameTagVisibility::Always => NameTagVisibility::Always,
                NameTagVisibility::Never => NameTagVisibility::Never,
                NameTagVisibility::HideForOtherTeams => NameTagVisibility::HideForOtherTeams,
                NameTagVisibility::HideForOwnTeam => NameTagVisibility::HideForOwnTeam,
            };

            let team_display_name = team.display_name.clone();
            scoreboard.update_team(world, team);

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_TEAM_OPTION_NAMETAGVISIBILITY_SUCCESS,
                        translation::java::COMMANDS_TEAM_OPTION_NAMETAGVISIBILITY_SUCCESS,
                        [
                            team_display_name,
                            TextComponent::text(self.value.to_str().to_string()),
                        ],
                    ),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

struct TeamModifyDeathMessageVisibilityExecutor {
    value: &'static str,
}

impl CommandExecutor for TeamModifyDeathMessageVisibilityExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let team_name = TeamArgumentType::get(context, ARG_TEAM)?;

            let world = context.world();
            let scoreboard = world.scoreboard.lock().await;

            let team = scoreboard.get_teams().get(team_name).ok_or_else(|| {
                TEAM_NOT_FOUND_ERROR
                    .create_without_context(TextComponent::text(team_name.to_string()))
            })?;

            let team_display_name = team.display_name.clone();

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_TEAM_OPTION_DEATHMESSAGEVISIBILITY_SUCCESS,
                        translation::java::COMMANDS_TEAM_OPTION_DEATHMESSAGEVISIBILITY_SUCCESS,
                        [
                            team_display_name,
                            TextComponent::text(self.value.to_string()),
                        ],
                    ),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

struct TeamModifyCollisionRuleExecutor {
    value: CollisionRule,
}

impl CommandExecutor for TeamModifyCollisionRuleExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let team_name = TeamArgumentType::get(context, ARG_TEAM)?;

            let world = context.world();
            let mut scoreboard = world.scoreboard.lock().await;

            let mut team = scoreboard
                .get_teams()
                .get(team_name)
                .ok_or_else(|| {
                    TEAM_NOT_FOUND_ERROR
                        .create_without_context(TextComponent::text(team_name.to_string()))
                })?
                .clone();

            let is_unchanged = matches!(
                (&team.collision_rule, &self.value),
                (CollisionRule::Always, CollisionRule::Always)
                    | (CollisionRule::Never, CollisionRule::Never)
                    | (CollisionRule::PushOtherTeams, CollisionRule::PushOtherTeams)
                    | (CollisionRule::PushOwnTeam, CollisionRule::PushOwnTeam)
            );

            if is_unchanged {
                return Err(COLLISION_RULE_UNCHANGED_ERROR.create_without_context());
            }

            team.collision_rule = match self.value {
                CollisionRule::Always => CollisionRule::Always,
                CollisionRule::Never => CollisionRule::Never,
                CollisionRule::PushOtherTeams => CollisionRule::PushOtherTeams,
                CollisionRule::PushOwnTeam => CollisionRule::PushOwnTeam,
            };

            let team_display_name = team.display_name.clone();
            scoreboard.update_team(world, team);

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_TEAM_OPTION_COLLISIONRULE_SUCCESS,
                        translation::java::COMMANDS_TEAM_OPTION_COLLISIONRULE_SUCCESS,
                        [
                            team_display_name,
                            TextComponent::text(self.value.to_str().to_string()),
                        ],
                    ),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

pub(super) fn modify_branch() -> LiteralArgumentBuilder {
    literal("modify").then(
        argument(ARG_TEAM, TeamArgumentType)
            .then(
                literal("color").then(
                    argument(ARG_VALUE, TeamColorArgumentType).executes(TeamModifyColorExecutor),
                ),
            )
            .then(
                literal("displayName").then(
                    argument(ARG_VALUE, StringArgumentType::GreedyPhrase)
                        .executes(TeamModifyDisplayNameExecutor),
                ),
            )
            .then(
                literal("prefix").then(
                    argument(ARG_VALUE, StringArgumentType::GreedyPhrase)
                        .executes(TeamModifyPrefixExecutor),
                ),
            )
            .then(
                literal("suffix").then(
                    argument(ARG_VALUE, StringArgumentType::GreedyPhrase)
                        .executes(TeamModifySuffixExecutor),
                ),
            )
            .then(literal("friendlyFire").then(
                argument(ARG_VALUE, BoolArgumentType).executes(TeamModifyFriendlyFireExecutor),
            ))
            .then(
                literal("seeFriendlyInvisibles").then(
                    argument(ARG_VALUE, BoolArgumentType)
                        .executes(TeamModifySeeFriendlyInvisiblesExecutor),
                ),
            )
            .then(
                literal("nametagVisibility")
                    .then(
                        literal("always").executes(TeamModifyNametagVisibilityExecutor {
                            value: NameTagVisibility::Always,
                        }),
                    )
                    .then(
                        literal("never").executes(TeamModifyNametagVisibilityExecutor {
                            value: NameTagVisibility::Never,
                        }),
                    )
                    .then(literal("hideForOtherTeams").executes(
                        TeamModifyNametagVisibilityExecutor {
                            value: NameTagVisibility::HideForOtherTeams,
                        },
                    ))
                    .then(literal("hideForOwnTeam").executes(
                        TeamModifyNametagVisibilityExecutor {
                            value: NameTagVisibility::HideForOwnTeam,
                        },
                    )),
            )
            .then(
                literal("deathMessageVisibility")
                    .then(
                        literal("always")
                            .executes(TeamModifyDeathMessageVisibilityExecutor { value: "always" }),
                    )
                    .then(
                        literal("never")
                            .executes(TeamModifyDeathMessageVisibilityExecutor { value: "never" }),
                    )
                    .then(literal("hideForOtherTeams").executes(
                        TeamModifyDeathMessageVisibilityExecutor {
                            value: "hideForOtherTeams",
                        },
                    ))
                    .then(literal("hideForOwnTeam").executes(
                        TeamModifyDeathMessageVisibilityExecutor {
                            value: "hideForOwnTeam",
                        },
                    )),
            )
            .then(
                literal("collisionRule")
                    .then(literal("always").executes(TeamModifyCollisionRuleExecutor {
                        value: CollisionRule::Always,
                    }))
                    .then(literal("never").executes(TeamModifyCollisionRuleExecutor {
                        value: CollisionRule::Never,
                    }))
                    .then(
                        literal("pushOtherTeams").executes(TeamModifyCollisionRuleExecutor {
                            value: CollisionRule::PushOtherTeams,
                        }),
                    )
                    .then(
                        literal("pushOwnTeam").executes(TeamModifyCollisionRuleExecutor {
                            value: CollisionRule::PushOwnTeam,
                        }),
                    ),
            ),
    )
}
