pub mod behavior;
pub mod brain;
pub mod control;
pub mod goal;
pub mod pathfinder;
pub mod random_pos;
pub mod sensing;
pub mod target_match;
pub mod target_predicate;
pub mod util;

#[cfg(test)]
pub(crate) mod test_support {
    use std::sync::{Arc, Weak};

    use arc_swap::ArcSwap;
    use pumpkin_config::{
        AdvancedConfiguration, BasicConfiguration, TelemetryConfig, world::LevelConfig,
    };
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::world_seed::Seed;
    use pumpkin_world::{level::Level, world_info::LevelData};

    use crate::{
        block::registry::default_registry, data::VanillaData, server::Server, world::World,
    };

    pub async fn server(
        folder: &tempfile::TempDir,
    ) -> Result<Arc<Server>, pumpkin_world::world_info::WorldInfoError> {
        let basic_config = BasicConfiguration {
            default_level_name: folder.path().to_string_lossy().into_owned(),
            allow_nether: false,
            allow_end: false,
            allow_chat_reports: false,
            ..Default::default()
        };
        let mut advanced_config = AdvancedConfiguration::default();
        advanced_config.update_checker.enabled = false;
        advanced_config.networking.java.online_mode = false;
        advanced_config.networking.bedrock.enabled = false;
        advanced_config.networking.bedrock.authentication.enabled = false;
        Server::new(
            basic_config,
            advanced_config,
            TelemetryConfig::default(),
            VanillaData::default(),
            Vec::new(),
        )
        .await
    }

    pub fn world(folder: &tempfile::TempDir) -> Arc<World> {
        world_in_dimension(folder, Dimension::OVERWORLD.clone())
    }

    pub fn world_in_dimension(folder: &tempfile::TempDir, dimension: Dimension) -> Arc<World> {
        Arc::new(World::load(
            Level::from_root_folder(
                &LevelConfig::default(),
                folder.path().to_path_buf(),
                0,
                dimension.clone(),
            ),
            Arc::new(ArcSwap::from_pointee(LevelData::default(Seed(0)))),
            dimension,
            default_registry(),
            Weak::new(),
        ))
    }
}
