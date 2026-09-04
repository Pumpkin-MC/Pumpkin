use crate::plugin::{
    PluginMetadata,
    loader::wasm::wasm_host::{
        PluginInitError, PluginInstance, concurrent_store::LegacySyncReentry,
        state::PluginHostState,
    },
};
use wasmtime::component::{HasSelf, InstancePre, Linker, bindgen};
use wasmtime::{Engine, Store};

pub mod advancement;
// wasmtime's `bindgen!` requires every Host trait method to be `async fn`, even the ones whose
// implementation here happens not to need to `.await` anything - so `unused_async_trait_impl`
// can't be avoided without breaking the generated trait signatures.
#[allow(clippy::unused_async_trait_impl)]
pub mod block_entity;
#[allow(clippy::unused_async_trait_impl)]
pub mod boss_bar;
#[allow(clippy::unused_async_trait_impl)]
pub mod commands;
pub mod common;
#[allow(clippy::unused_async_trait_impl)]
pub mod context;
#[allow(clippy::unused_async_trait_impl)]
pub mod datapack;
#[allow(clippy::unused_async_trait_impl)]
pub mod display;
#[allow(clippy::unused_async_trait_impl)]
pub mod enchantment;
#[allow(clippy::unused_async_trait_impl)]
pub mod entity;
pub mod events;
pub mod forms;
pub mod generated_packets;
#[allow(clippy::unused_async_trait_impl)]
pub mod gui;
#[allow(clippy::unused_async_trait_impl)]
pub mod i18n;
#[allow(clippy::unused_async_trait_impl)]
pub mod inventory;
pub mod ipc;
#[allow(clippy::unused_async_trait_impl)]
pub mod item_stack;
pub mod java_dialogs;
#[allow(clippy::unused_async_trait_impl)]
pub mod living_entity;
#[allow(clippy::unused_async_trait_impl)]
pub mod logging;
#[allow(clippy::unused_async_trait_impl)]
pub mod mob;
pub mod permission;
#[allow(clippy::unused_async_trait_impl)]
pub mod player;
#[allow(clippy::unused_async_trait_impl)]
pub mod recipe;
pub mod scheduler;
#[allow(clippy::unused_async_trait_impl)]
pub mod scoreboard;
#[allow(clippy::unused_async_trait_impl)]
pub mod server;
pub mod status_effect;
#[allow(clippy::unused_async_trait_impl)]
pub mod text;
#[allow(clippy::unused_async_trait_impl)]
pub mod uuid;
#[allow(clippy::unused_async_trait_impl)]
pub mod world;

bindgen!({
    path: "../pumpkin-plugin-wit/v0.1",
    world: "plugin",
    imports: {
        "pumpkin:plugin/command@0.1.0.[method]command-sender.has-permission": async | store | trappable,
        "pumpkin:plugin/datapack@0.1.0.[method]datapack-manager.disable-pack": async | store | trappable,
        "pumpkin:plugin/datapack@0.1.0.[method]datapack-manager.enable-pack": async | store | trappable,
        "pumpkin:plugin/datapack@0.1.0.[method]datapack-manager.execute-function": async | store | trappable,
        "pumpkin:plugin/datapack@0.1.0.[method]datapack-manager.reload": async | store | trappable,
        "pumpkin:plugin/ipc@0.1.0.send-ipc-message": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.add-effect": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.add-experience-levels": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.add-experience-points": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.award-advancement": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.award-advancement-criterion": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.ban": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.ban-ip": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.damage": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.has-permission": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.heal": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.kill": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.open-ender-chest": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.open-gui": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.respawn": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.set-experience-level": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.set-experience-points": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.set-experience-progress": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.set-food-level": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.set-gamemode": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.set-permission-level": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.teleport": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]player.teleport-world": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]java-player.show-dialog": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]java-player.clear-dialog": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]java-player.kick": async | store | trappable,
        "pumpkin:plugin/player@0.1.0.[method]bedrock-player.kick": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]ban-manager.ban-ip": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]ban-manager.ban-player": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]op-manager.deop-player": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]op-manager.op-player": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]server.broadcast": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]server.create-world": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]server.execute-command": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]server.save-all": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]server.unload-world": async | store | trappable,
        "pumpkin:plugin/server@0.1.0.[method]whitelist-manager.set-enabled": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]entity.add-passenger": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]entity.eject-passengers": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]entity.remove-passenger": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]entity.set-swimming": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]entity.set-vehicle": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]entity.teleport": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]living-entity.damage": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]mob.clear-ai-goals": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.create-explosion": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.save": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.set-block": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.set-block-by-id": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.set-block-by-name": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.set-block-state": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.set-raining": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.set-thundering": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.spawn-entity": async | store | trappable,
        "pumpkin:plugin/world@0.1.0.[method]world.strike-lightning": async | store | trappable,
        default: async | trappable,
    },
    exports: { default: async | store | trappable},
});

mod resource_with {
    use super::pumpkin::plugin;
    use crate::{
        command::CommandSender,
        entity::EntityBase,
        entity::player::Player,
        plugin::{
            Context,
            api::gui::PluginGui,
            loader::wasm::wasm_host::{
                args::OwnedArg,
                state::{
                    ChunkBuffer, FromResource, InventoryProvider, ScoreboardProvider, WasmCommand,
                    WasmCommandNode,
                },
            },
        },
        server::{RecipeManager, Server},
        world::World,
    };
    use pumpkin_util::text::TextComponent;
    use std::collections::HashMap;
    use std::sync::{Arc, Weak};
    use tokio::sync::Mutex;

    impl FromResource for plugin::server::Server {
        type Internal = Arc<Server>;
    }
    impl FromResource for plugin::context::Context {
        type Internal = Arc<Context>;
    }
    impl FromResource for plugin::player::Player {
        type Internal = Arc<Player>;
    }
    impl FromResource for plugin::player::JavaPlayer {
        type Internal = Arc<Player>;
    }
    impl FromResource for plugin::player::BedrockPlayer {
        type Internal = Arc<Player>;
    }
    impl FromResource for plugin::world::Entity {
        type Internal = Arc<dyn EntityBase>;
    }
    impl FromResource for plugin::world::World {
        type Internal = Arc<World>;
    }
    impl FromResource for plugin::world::Chunk {
        type Internal = (Arc<World>, Weak<pumpkin_world::chunk::ChunkData>);
    }
    impl FromResource for plugin::world::WorldBorder {
        type Internal = Arc<World>;
    }

    impl FromResource for plugin::scoreboard::Scoreboard {
        type Internal = ScoreboardProvider;
    }
    impl FromResource for plugin::scoreboard::BedrockScoreboard {
        type Internal = Arc<Player>;
    }
    impl FromResource for plugin::gui::Gui {
        type Internal = Arc<Mutex<PluginGui>>;
    }
    impl FromResource for plugin::boss_bar::BossBar {
        type Internal =
            Arc<Mutex<crate::plugin::loader::wasm::wasm_host::wit::v0_1::boss_bar::PluginBossBar>>;
    }
    impl FromResource for plugin::text::TextComponent {
        type Internal = TextComponent;
    }
    impl FromResource for plugin::command::Command {
        type Internal = WasmCommand;
    }
    impl FromResource for plugin::command::CommandSender {
        type Internal = CommandSender;
    }
    impl FromResource for plugin::command::ConsumedArgs {
        type Internal = HashMap<String, OwnedArg>;
    }
    impl FromResource for plugin::command::CommandNode {
        type Internal = WasmCommandNode;
    }
    impl FromResource for plugin::item_stack::ItemStack {
        type Internal = Arc<Mutex<pumpkin_data::item_stack::ItemStack>>;
    }
    impl FromResource for plugin::recipe::RecipeManager {
        type Internal = Arc<RecipeManager>;
    }
    impl FromResource for plugin::enchantments::EnchantmentManager {
        type Internal = Arc<crate::server::enchantment::EnchantmentManager>;
    }
    impl FromResource for plugin::server::OpManager {
        type Internal = Arc<Server>;
    }
    impl FromResource for plugin::server::BanManager {
        type Internal = Arc<Server>;
    }
    impl FromResource for plugin::server::WhitelistManager {
        type Internal = Arc<Server>;
    }
    impl FromResource for plugin::server::DatapackManager {
        type Internal = Arc<Server>;
    }

    #[allow(clippy::wildcard_imports)]
    mod block_entities {
        use super::super::pumpkin::plugin;
        use crate::block::entities::*;
        use crate::plugin::loader::wasm::wasm_host::state::{ContainerBlockEntity, FromResource};
        use std::sync::Arc;

        impl FromResource for plugin::block_entity::BlockEntity {
            type Internal = Arc<dyn BlockEntity>;
        }
        impl FromResource for plugin::block_entity::BannerBlockEntity {
            type Internal = Arc<banner::BannerBlockEntity>;
        }
        impl FromResource for plugin::block_entity::BarrelBlockEntity {
            type Internal = Arc<barrel::BarrelBlockEntity>;
        }
        impl FromResource for plugin::block_entity::BeaconBlockEntity {
            type Internal = Arc<beacon::BeaconBlockEntity>;
        }
        impl FromResource for plugin::block_entity::BedBlockEntity {
            type Internal = Arc<bed::BedBlockEntity>;
        }
        impl FromResource for plugin::block_entity::BeehiveBlockEntity {
            type Internal = Arc<beehive::BeehiveBlockEntity>;
        }
        impl FromResource for plugin::block_entity::BellBlockEntity {
            type Internal = Arc<bell::BellBlockEntity>;
        }
        impl FromResource for plugin::block_entity::BlastingFurnaceBlockEntity {
            type Internal = Arc<blasting_furnace::BlastingFurnaceBlockEntity>;
        }
        impl FromResource for plugin::block_entity::BrewingStandBlockEntity {
            type Internal = Arc<brewing_stand::BrewingStandBlockEntity>;
        }
        impl FromResource for plugin::block_entity::BrushableBlockBlockEntity {
            type Internal = Arc<brushable_block::BrushableBlockBlockEntity>;
        }
        impl FromResource for plugin::block_entity::CalibratedSculkSensorBlockEntity {
            type Internal = Arc<calibrated_sculk_sensor::CalibratedSculkSensorBlockEntity>;
        }
        impl FromResource for plugin::block_entity::CampfireBlockEntity {
            type Internal = Arc<campfire::CampfireBlockEntity>;
        }
        impl FromResource for plugin::block_entity::ChestBlockEntity {
            type Internal = Arc<chest::ChestBlockEntity>;
        }
        impl FromResource for plugin::block_entity::ChiseledBookshelfBlockEntity {
            type Internal = Arc<chiseled_bookshelf::ChiseledBookshelfBlockEntity>;
        }
        impl FromResource for plugin::block_entity::CommandBlockEntity {
            type Internal = Arc<command_block::CommandBlockEntity>;
        }
        impl FromResource for plugin::block_entity::ComparatorBlockEntity {
            type Internal = Arc<comparator::ComparatorBlockEntity>;
        }
        impl FromResource for plugin::block_entity::ConduitBlockEntity {
            type Internal = Arc<conduit::ConduitBlockEntity>;
        }
        impl FromResource for plugin::block_entity::CopperGolemStatueBlockEntity {
            type Internal = Arc<copper_golem_statue::CopperGolemStatueBlockEntity>;
        }
        impl FromResource for plugin::block_entity::CrafterBlockEntity {
            type Internal = Arc<crafter::CrafterBlockEntity>;
        }
        impl FromResource for plugin::block_entity::CreakingHeartBlockEntity {
            type Internal = Arc<creaking_heart::CreakingHeartBlockEntity>;
        }
        impl FromResource for plugin::block_entity::DaylightDetectorBlockEntity {
            type Internal = Arc<daylight_detector::DaylightDetectorBlockEntity>;
        }
        impl FromResource for plugin::block_entity::DecoratedPotBlockEntity {
            type Internal = Arc<decorated_pot::DecoratedPotBlockEntity>;
        }
        impl FromResource for plugin::block_entity::DispenserBlockEntity {
            type Internal = Arc<dispenser::DispenserBlockEntity>;
        }
        impl FromResource for plugin::block_entity::DropperBlockEntity {
            type Internal = Arc<dropper::DropperBlockEntity>;
        }
        impl FromResource for plugin::block_entity::EnchantingTableBlockEntity {
            type Internal = Arc<enchanting_table::EnchantingTableBlockEntity>;
        }
        impl FromResource for plugin::block_entity::EnderChestBlockEntity {
            type Internal = Arc<ender_chest::EnderChestBlockEntity>;
        }
        impl FromResource for plugin::block_entity::EndGatewayBlockEntity {
            type Internal = Arc<end_gateway::EndGatewayBlockEntity>;
        }
        impl FromResource for plugin::block_entity::EndPortalBlockEntity {
            type Internal = Arc<end_portal::EndPortalBlockEntity>;
        }
        impl FromResource for plugin::block_entity::FurnaceBlockEntity {
            type Internal = Arc<furnace::FurnaceBlockEntity>;
        }
        impl FromResource for plugin::block_entity::HangingSignBlockEntity {
            type Internal = Arc<hanging_sign::HangingSignBlockEntity>;
        }
        impl FromResource for plugin::block_entity::HopperBlockEntity {
            type Internal = Arc<hopper::HopperBlockEntity>;
        }
        impl FromResource for plugin::block_entity::JigsawBlockEntity {
            type Internal = Arc<jigsaw_block::JigsawBlockEntity>;
        }
        impl FromResource for plugin::block_entity::JukeboxBlockEntity {
            type Internal = Arc<jukebox::JukeboxBlockEntity>;
        }
        impl FromResource for plugin::block_entity::LecternBlockEntity {
            type Internal = Arc<lectern::LecternBlockEntity>;
        }
        impl FromResource for plugin::block_entity::MapBlockEntity {
            type Internal = Arc<map::MapBlockEntity>;
        }
        impl FromResource for plugin::block_entity::MobSpawnerBlockEntity {
            type Internal = Arc<mob_spawner::MobSpawnerBlockEntity>;
        }
        impl FromResource for plugin::block_entity::PistonBlockEntity {
            type Internal = Arc<piston::PistonBlockEntity>;
        }
        impl FromResource for plugin::block_entity::PotentSulfurBlockEntity {
            type Internal = Arc<potent_sulfur::PotentSulfurBlockEntity>;
        }
        impl FromResource for plugin::block_entity::SculkCatalystBlockEntity {
            type Internal = Arc<sculk_catalyst::SculkCatalystBlockEntity>;
        }
        impl FromResource for plugin::block_entity::SculkSensorBlockEntity {
            type Internal = Arc<sculk_sensor::SculkSensorBlockEntity>;
        }
        impl FromResource for plugin::block_entity::SculkShriekerBlockEntity {
            type Internal = Arc<sculk_shrieker::SculkShriekerBlockEntity>;
        }
        impl FromResource for plugin::block_entity::ShelfBlockEntity {
            type Internal = Arc<shelf::ShelfBlockEntity>;
        }
        impl FromResource for plugin::block_entity::ShulkerBoxBlockEntity {
            type Internal = Arc<shulker_box::ShulkerBoxBlockEntity>;
        }
        impl FromResource for plugin::block_entity::SignBlockEntity {
            type Internal = Arc<sign::SignBlockEntity>;
        }
        impl FromResource for plugin::block_entity::SkullBlockEntity {
            type Internal = Arc<skull::SkullBlockEntity>;
        }
        impl FromResource for plugin::block_entity::SmokerBlockEntity {
            type Internal = Arc<smoker::SmokerBlockEntity>;
        }
        impl FromResource for plugin::block_entity::StructureBlockBlockEntity {
            type Internal = Arc<structure_block::StructureBlockBlockEntity>;
        }
        impl FromResource for plugin::block_entity::TestBlockBlockEntity {
            type Internal = Arc<test_block::TestBlockBlockEntity>;
        }
        impl FromResource for plugin::block_entity::TestInstanceBlockBlockEntity {
            type Internal = Arc<test_instance_block::TestInstanceBlockBlockEntity>;
        }
        impl FromResource for plugin::block_entity::TrappedChestBlockEntity {
            type Internal = Arc<trapped_chest::TrappedChestBlockEntity>;
        }
        impl FromResource for plugin::block_entity::TrialSpawnerBlockEntity {
            type Internal = Arc<trial_spawner::TrialSpawnerBlockEntity>;
        }
        impl FromResource for plugin::block_entity::VaultBlockEntity {
            type Internal = Arc<vault::VaultBlockEntity>;
        }
        impl FromResource for plugin::block_entity::ContainerBlockEntity {
            type Internal = ContainerBlockEntity;
        }
    }

    impl FromResource for plugin::inventory::Inventory {
        type Internal = InventoryProvider;
    }
    impl FromResource for plugin::inventory::PlayerInventory {
        type Internal = Arc<Player>;
    }
    impl FromResource for plugin::entity::LivingEntity {
        type Internal = Arc<dyn EntityBase>;
    }
    impl FromResource for plugin::entity::Mob {
        type Internal = Arc<dyn EntityBase>;
    }

    impl FromResource for plugin::display::DisplayEntity {
        type Internal = Arc<dyn EntityBase>;
    }
    impl FromResource for plugin::display::BlockDisplayEntity {
        type Internal = Arc<crate::entity::decoration::display::BlockDisplayEntity>;
    }
    impl FromResource for plugin::display::ItemDisplayEntity {
        type Internal = Arc<crate::entity::decoration::display::ItemDisplayEntity>;
    }
    impl FromResource for plugin::display::TextDisplayEntity {
        type Internal = Arc<crate::entity::decoration::display::TextDisplayEntity>;
    }
    impl FromResource for plugin::display::InteractionEntity {
        type Internal = Arc<crate::entity::interaction::InteractionEntity>;
    }

    impl FromResource for plugin::world::ChunkBuffer {
        type Internal = ChunkBuffer;
    }
}

impl pumpkin::plugin::java_packets::Host for PluginHostState {}
impl pumpkin::plugin::bedrock_packets::Host for PluginHostState {}
impl pumpkin::plugin::data_components::Host for PluginHostState {}
impl pumpkin::plugin::enchantments::Host for PluginHostState {}
impl pumpkin::plugin::biomes::Host for PluginHostState {}
impl pumpkin::plugin::attributes::Host for PluginHostState {}
impl pumpkin::plugin::advancement::Host for PluginHostState {}
impl pumpkin::plugin::damage_types::Host for PluginHostState {}
impl pumpkin::plugin::screens::Host for PluginHostState {}
impl pumpkin::plugin::statistics::Host for PluginHostState {}
impl pumpkin::plugin::game_rules::Host for PluginHostState {}
impl pumpkin::plugin::game_events::Host for PluginHostState {}
impl pumpkin::plugin::potions::Host for PluginHostState {}
impl pumpkin::plugin::entity_statuses::Host for PluginHostState {}

pub fn add_to_linker(linker: &mut Linker<PluginHostState>) -> wasmtime::Result<()> {
    Plugin::add_to_linker::<_, HasSelf<_>>(linker, |state: &mut PluginHostState| state)?;
    Ok(())
}

pub fn prepare_plugin(
    instance_pre: &InstancePre<PluginHostState>,
) -> wasmtime::Result<PluginPre<PluginHostState>> {
    PluginPre::new(instance_pre.clone())
}

pub async fn init_plugin(
    engine: &Engine,
    plugin_pre: PluginPre<PluginHostState>,
    legacy_sync_reentry: &LegacySyncReentry,
) -> Result<(PluginInstance, Store<PluginHostState>, PluginMetadata), PluginInitError> {
    let mut store = Store::new(engine, PluginHostState::new());
    store.limiter(|state| &mut state.limits);
    let plugin = legacy_sync_reentry
        .scope_bootstrap(plugin_pre.instantiate_async(&mut store))
        .await
        .map_err(PluginInitError::InstantiationFailed)?;

    store
        .run_concurrent(async |accessor| {
            legacy_sync_reentry
                .scope_bootstrap(plugin.call_init_plugin(accessor))
                .await
        })
        .await
        .map_err(PluginInitError::CallInitPluginFailed)?
        .map_err(PluginInitError::CallInitPluginFailed)?;

    let metadata = store
        .run_concurrent(async |accessor| {
            legacy_sync_reentry
                .scope_bootstrap(plugin.pumpkin_plugin_metadata().call_get_metadata(accessor))
                .await
        })
        .await
        .map_err(PluginInitError::CallGetMetadataFailed)?
        .map_err(PluginInitError::CallGetMetadataFailed)?;

    let metadata = PluginMetadata {
        name: metadata.name,
        version: metadata.version,
        authors: metadata.authors,
        description: metadata.description,
        dependencies: metadata.dependencies,
        permissions: metadata.permissions,
    };

    store
        .data_mut()
        .permissions
        .clone_from(&metadata.permissions);

    Ok((PluginInstance::V0_1(plugin), store, metadata))
}
