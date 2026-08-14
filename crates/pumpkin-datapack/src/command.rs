use crate::DataPackManager;
use crate::pack::format::{PackCompatibility, PackFormat};

/// Implements the `/datapack` command.
///
/// Subcommands:
///   /datapack enable <name> [after|before <existing>] [last|first]
///   /datapack disable <name>
///   /datapack list [available|enabled]
///   /datapack create <id> <description>
///
/// This module provides the logic; actual command registration
/// into Pumpkin's `CommandDispatcher` should be done via the existing
/// command pattern in `pumpkin/src/command/commands/`.
pub struct DatapackCommand;

impl DatapackCommand {
    /// Enable a datapack.
    pub async fn enable(
        manager: &DataPackManager,
        name: &str,
    ) -> Result<String, crate::DatapackError> {
        manager.enable_pack(name).await?;
        Ok(format!("Enabled datapack '{name}'"))
    }

    /// Enable a datapack at a specific position.
    pub async fn enable_at_position(
        manager: &DataPackManager,
        name: &str,
        position: EnablePosition,
    ) -> Result<String, crate::DatapackError> {
        let mut repo = manager.repository.write().await;
        if !repo.available_ids().contains(&name.to_string()) {
            return Err(crate::DatapackError::PackNotFound(name.to_string()));
        }
        // Remove first so we can re-insert at the right position
        repo.remove_pack(name);

        match position {
            EnablePosition::First => {
                repo.add_pack_at(name, 0);
            }
            EnablePosition::Last => {
                repo.add_pack(name);
            }
            EnablePosition::Before(existing) => {
                let idx = repo
                    .selected_ids()
                    .iter()
                    .position(|s| s.as_str() == existing);
                if let Some(i) = idx {
                    repo.add_pack_at(name, i);
                } else {
                    repo.add_pack(name);
                }
            }
            EnablePosition::After(existing) => {
                let idx = repo
                    .selected_ids()
                    .iter()
                    .position(|s| s.as_str() == existing);
                if let Some(i) = idx {
                    repo.add_pack_at(name, i + 1);
                } else {
                    repo.add_pack(name);
                }
            }
        }
        drop(repo);
        manager.reload().await?;
        Ok(format!("Enabled datapack '{name}'"))
    }

    /// Disable a datapack.
    pub async fn disable(
        manager: &DataPackManager,
        name: &str,
    ) -> Result<String, crate::DatapackError> {
        manager.disable_pack(name).await?;
        Ok(format!("Disabled datapack '{name}'"))
    }

    /// Create a new datapack with boilerplate.
    pub fn create(
        manager: &DataPackManager,
        id: &str,
        description: &str,
    ) -> Result<String, crate::DatapackError> {
        let pack_dir = manager.world_path.join("datapacks").join(id);
        if pack_dir.exists() {
            return Err(crate::DatapackError::Validation(vec![format!(
                "Pack '{id}' already exists"
            )]));
        }
        std::fs::create_dir_all(pack_dir.join("data").join("minecraft"))?;

        let current = PackFormat::CURRENT;
        let mcmeta = serde_json::json!({
            "pack": {
                "description": description,
                "min_format": current.major,
                "max_format": [current.major, current.minor],
            }
        });
        std::fs::write(
            pack_dir.join("pack.mcmeta"),
            serde_json::to_string_pretty(&mcmeta)?,
        )?;

        Ok(format!("Created datapack '{id}'"))
    }

    /// List available or enabled datapacks.
    pub async fn list(
        manager: &DataPackManager,
        mode: ListMode,
    ) -> Result<Vec<String>, crate::DatapackError> {
        let repo = manager.repository.read().await;

        let ids: Vec<String> = match mode {
            ListMode::Available => repo.available_ids(),
            ListMode::Enabled => repo.selected_ids().to_vec(),
        };

        let mut result = Vec::new();
        for id in ids {
            if let Some(pack) = repo.get_pack(&id) {
                let status = match pack.compatibility {
                    PackCompatibility::Compatible => "§a✔",
                    PackCompatibility::TooOld => "§c✘ (too old)",
                    PackCompatibility::TooNew => "§c✘ (too new)",
                    PackCompatibility::Unknown => "§7? (unknown)",
                };
                let enabled = if repo.selected_ids().contains(&id) {
                    "§e[enabled]"
                } else {
                    "§7[disabled]"
                };
                result.push(format!("{status} {enabled} {id}"));
            } else {
                result.push(format!("§7{id} (unknown)"));
            }
        }

        Ok(result)
    }
}

/// Position to enable a datapack at.
pub enum EnablePosition {
    First,
    Last,
    Before(String),
    After(String),
}

/// Which list to show.
pub enum ListMode {
    Available,
    Enabled,
}
