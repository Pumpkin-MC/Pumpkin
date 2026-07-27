use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_util::translation::Locale;

use crate::command::argument_builder::{ArgumentBuilder, command};
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Displays build information about this Pumpkin server.";
const PERMISSION: &str = "pumpkin:command.about";

/// Crate version of the running binary, for example `0.1.0-dev+26.2`.
///
/// This is the same source the startup banner uses, so `/about` and the log
/// line can never disagree.
const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Short commit hash captured by `build.rs`.
const GIT_HASH: &str = env!("GIT_HASH");
/// Full commit hash captured by `build.rs`.
const GIT_HASH_FULL: &str = env!("GIT_HASH_FULL");
/// Branch name captured by `build.rs`, or `detached` / `unknown`.
const GIT_BRANCH: &str = env!("GIT_BRANCH");
/// Whether the work tree was `clean`, `dirty` or `unknown` at build time.
const GIT_DIRTY: &str = env!("GIT_DIRTY");
/// Target triple this binary was compiled for.
const BUILD_TARGET: &str = env!("BUILD_TARGET");
/// Full `rustc --version` output of the compiler that built this binary.
const BUILD_RUSTC_VERSION: &str = env!("BUILD_RUSTC_VERSION");

/// Placeholder `build.rs` emits when a field could not be determined.
const UNKNOWN: &str = "unknown";

/// Marker `build.rs` emits when the work tree had uncommitted changes.
const DIRTY: &str = "dirty";

struct AboutCommandExecutor;

/// Returns the Cargo profile this binary was built with.
const fn build_profile() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}

/// Returns the global allocator actually linked into this binary.
///
/// The allocator is picked at compile time by the `mimalloc` Cargo feature, so
/// it can differ from `advanced.performance.allocator` in `pumpkin.toml`. This
/// reports what is really running.
const fn active_allocator() -> &'static str {
    if cfg!(feature = "mimalloc") {
        "mimalloc"
    } else {
        "system"
    }
}

/// Returns the `flate2` compression backend linked into this binary.
const fn active_compression() -> &'static str {
    if cfg!(feature = "zlib-ng") {
        "zlib-ng"
    } else {
        "rust (miniz_oxide)"
    }
}

/// Formats the commit descriptor, for example `1a2b3c4 (main, dirty)`.
///
/// Fields `build.rs` could not resolve are dropped rather than shown as
/// `unknown`, so a build from a source tarball without git still reads well.
fn commit_summary(hash: &str, branch: &str, dirty: &str) -> String {
    let mut details = Vec::new();
    if branch != UNKNOWN {
        details.push(branch);
    }
    if dirty == DIRTY {
        details.push(dirty);
    }

    if details.is_empty() {
        hash.to_string()
    } else {
        format!("{hash} ({})", details.join(", "))
    }
}

/// Builds one localized line of the report from a `pumpkin` namespace key.
fn line(key: &'static str, locale: Locale, values: Vec<TextComponent>) -> TextComponent {
    TextComponent::custom("pumpkin", key, locale, values)
        .color_named(NamedColor::Gray)
        .new_line()
}

/// Assembles the `/about` report for the given locale.
fn about_message(locale: Locale) -> TextComponent {
    TextComponent::empty()
        .add_child(
            TextComponent::custom(
                "pumpkin",
                "commands.about.version",
                locale,
                vec![TextComponent::text(CARGO_PKG_VERSION).color_named(NamedColor::Green)],
            )
            .color_named(NamedColor::Gold)
            .bold()
            .new_line(),
        )
        .add_child(line(
            "commands.about.minecraft",
            locale,
            vec![
                TextComponent::text(CURRENT_MC_VERSION.to_string()),
                TextComponent::text(CURRENT_MC_VERSION.protocol_version().to_string()),
            ],
        ))
        .add_child(
            line(
                "commands.about.commit",
                locale,
                vec![
                    TextComponent::text(commit_summary(GIT_HASH, GIT_BRANCH, GIT_DIRTY))
                        .color_named(NamedColor::Aqua),
                ],
            )
            .hover_event(HoverEvent::show_text(TextComponent::custom(
                "pumpkin",
                "commands.about.commit.hover",
                locale,
                vec![TextComponent::text(GIT_HASH_FULL)],
            ))),
        )
        .add_child(line(
            "commands.about.build",
            locale,
            vec![
                TextComponent::text(build_profile()),
                TextComponent::text(BUILD_TARGET),
            ],
        ))
        .add_child(line(
            "commands.about.compiler",
            locale,
            vec![TextComponent::text(BUILD_RUSTC_VERSION)],
        ))
        .add_child(line(
            "commands.about.backends",
            locale,
            vec![
                TextComponent::text(active_allocator()),
                TextComponent::text(active_compression()),
            ],
        ))
}

impl CommandExecutor for AboutCommandExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let locale = context.source.output.get_locale();
            context
                .source
                .send_feedback(about_message(locale), false)
                .await;

            Ok(1)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        // Purely informational, like `/pumpkin`, so everyone may run it.
        PermissionDefault::Allow,
    ));

    dispatcher.register(
        command("about", DESCRIPTION)
            .requires(PERMISSION)
            .executes(AboutCommandExecutor),
    );
}

#[cfg(test)]
mod tests {
    use super::{DIRTY, UNKNOWN, commit_summary};

    #[test]
    fn commit_summary_includes_branch_and_dirty_flag() {
        assert_eq!(commit_summary("1a2b3c4", "main", "clean"), "1a2b3c4 (main)");
        assert_eq!(
            commit_summary("1a2b3c4", "main", DIRTY),
            "1a2b3c4 (main, dirty)"
        );
    }

    #[test]
    fn commit_summary_omits_unresolved_fields() {
        assert_eq!(commit_summary("1a2b3c4", UNKNOWN, UNKNOWN), "1a2b3c4");
        assert_eq!(commit_summary(UNKNOWN, UNKNOWN, UNKNOWN), UNKNOWN);
        assert_eq!(commit_summary("1a2b3c4", UNKNOWN, DIRTY), "1a2b3c4 (dirty)");
    }
}
