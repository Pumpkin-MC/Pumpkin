use super::Player;
use crate::command::client_suggestions;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::entity::EntityBase;
use crate::entity::NBTStorage;
use crate::entity::NBTStorageInit;
use crate::entity::NbtFuture;
use crate::net::ClientPlatform;
use crate::plugin::player::player_gamemode_change::PlayerGamemodeChangeEvent;
use crate::plugin::player::player_permission_check::PlayerPermissionCheckEvent;
use crate::server::Server;
use pumpkin_data::entity::EntityStatus;
use pumpkin_macros::send_cancellable;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::client::AbilityLayer;
use pumpkin_protocol::bedrock::client::update_abilities::Ability;
use pumpkin_protocol::bedrock::client::update_abilities::CUpdateAbilities;
use pumpkin_protocol::java::client::play::CGameEvent;
use pumpkin_protocol::java::client::play::CPlayerAbilities;
use pumpkin_protocol::java::client::play::CPlayerInfoUpdate;
use pumpkin_protocol::java::client::play::CSetCamera;
use pumpkin_protocol::java::client::play::GameEvent;
use pumpkin_protocol::java::client::play::PlayerAction;
use pumpkin_protocol::java::client::play::PlayerInfoFlags;
use pumpkin_util::GameMode;
use pumpkin_util::permission::PermissionLvl;
use std::sync::Arc;
use std::sync::atomic::Ordering;

impl Player {
    /// Updates the current abilities the player has.
    pub async fn send_abilities_update(&self) {
        match self.client.as_ref() {
            ClientPlatform::Java(java) => {
                let mut b = 0;
                let abilities = &self.abilities.lock().await;

                if abilities.invulnerable {
                    b |= 1;
                }
                if abilities.flying {
                    b |= 2;
                }
                if abilities.allow_flying {
                    b |= 4;
                }
                if abilities.creative {
                    b |= 8;
                }
                java.enqueue_packet(&CPlayerAbilities::new(
                    b,
                    abilities.fly_speed,
                    abilities.walk_speed,
                ))
                .await;
            }
            ClientPlatform::Bedrock(bedrock) => {
                let abilities = self.abilities.lock().await;
                let is_op = self.permission_lvl.load() == PermissionLvl::Four;
                let is_spectator = self.gamemode.load() == GameMode::Spectator;

                // 1. Permission Mapping
                let player_perm = if is_op { 2 } else { 1 }; // 1: Member, 2: Operator
                let command_perm = u8::from(is_op); // 0: Normal, 1: Operator

                // 2. Build the Ability Bitmask
                let mut ability_value: u32 = 0;

                // Helper closure to set bits using your enum
                let mut set_ability = |ability: Ability, enabled: bool| {
                    if enabled {
                        ability_value |= 1 << (ability as u32);
                    }
                };

                // Base Permissions
                set_ability(Ability::MayFly, abilities.allow_flying);
                set_ability(Ability::Flying, abilities.flying);
                set_ability(
                    Ability::Invulnerable,
                    abilities.invulnerable || abilities.creative,
                );

                // Operator Specifics
                set_ability(Ability::OperatorCommands, is_op);
                set_ability(Ability::Teleport, is_op);

                // Interaction Permissions (Disabled for Spectators)
                let can_interact = !is_spectator;
                set_ability(Ability::Build, can_interact);
                set_ability(Ability::Mine, can_interact);
                set_ability(Ability::DoorsAndSwitches, can_interact);
                set_ability(Ability::OpenContainers, can_interact);
                set_ability(Ability::AttackPlayers, can_interact);
                set_ability(Ability::AttackMobs, can_interact);

                // Creative/Spectator Extras
                set_ability(Ability::Instabuild, abilities.creative);
                set_ability(Ability::NoClip, is_spectator);

                // 3. Construct the Layers
                let mut layers = vec![AbilityLayer {
                    serialized_layer: 0, // LAYER_BASE
                    // 0x3FFFF defines the first 18 bits as "provided" by this packet
                    abilities_set: (1 << Ability::AbilityCount as u32) - 1,
                    ability_value,
                    fly_speed: 0.05,
                    vertical_fly_speed: 1.0,
                    walk_speed: 0.1,
                }];

                if is_spectator {
                    layers.push(AbilityLayer {
                        serialized_layer: 1,
                        abilities_set: 1 << (Ability::Flying as u32),
                        ability_value: 1 << (Ability::Flying as u32),
                        fly_speed: 0.05,
                        vertical_fly_speed: 1.0,
                        walk_speed: 0.1,
                    });
                }

                let packet = CUpdateAbilities {
                    target_player_raw_id: self.entity_id().into(),
                    player_permission: player_perm,
                    command_permission: command_perm,
                    layers,
                };

                bedrock.send_game_packet(&packet).await;
            }
        }
    }

    /// Updates the client of the player's current permission level.
    pub fn send_permission_lvl_update(&self) {
        let status = match self.permission_lvl.load() {
            PermissionLvl::Zero => EntityStatus::PermissionLevelAll,
            PermissionLvl::One => EntityStatus::PermissionLevelModerators,
            PermissionLvl::Two => EntityStatus::PermissionLevelGamemasters,
            PermissionLvl::Three => EntityStatus::PermissionLevelAdmins,
            PermissionLvl::Four => EntityStatus::PermissionLevelOwners,
        };
        self.world()
            .send_entity_status(&self.living_entity.entity, status);
    }

    /// Sets the player's permission level and notifies the client.
    pub async fn set_permission_lvl(
        self: &Arc<Self>,
        server: &Server,
        lvl: PermissionLvl,
        command_dispatcher: &CommandDispatcher,
    ) {
        self.permission_lvl.store(lvl);
        self.send_permission_lvl_update();

        if let ClientPlatform::Bedrock(_) = self.client.as_ref() {
            client_suggestions::send_bedrock_commands_packet(self, server, command_dispatcher)
                .await;
        } else {
            client_suggestions::send_c_commands_packet(self, server, command_dispatcher).await;
        }
    }

    pub async fn set_gamemode(self: &Arc<Self>, gamemode: GameMode) -> bool {
        // We could send the same gamemode without any problems. But why waste bandwidth?
        // assert_ne!(
        //    self.gamemode.load(),
        //    gamemode,
        //    "Attempt to set the gamemode to the already current gamemode"
        // );
        // Why are we panicking if the gamemodes are the same? Vanilla just exits early.
        if self.gamemode.load() == gamemode {
            return false;
        }
        let server = self.world().server.upgrade().unwrap();
        send_cancellable! {{
            server;
            PlayerGamemodeChangeEvent {
                player: self.clone(),
                new_gamemode: gamemode,
                previous_gamemode: self.gamemode.load(),
                cancelled: false,
            };

            'after: {
                let gamemode = event.new_gamemode;
                self.gamemode.store(gamemode);
                // TODO: Fix this when mojang fixes it
                // This is intentional to keep the pure vanilla mojang experience
                // self.previous_gamemode.store(self.previous_gamemode.load());
                {
                    // Use another scope so that we instantly unlock `abilities`.
                    let mut abilities = self.abilities.lock().await;
                    abilities.set_for_gamemode(gamemode);
                };
                self.send_abilities_update().await;

                if gamemode == GameMode::Creative {
                    self.get_entity().extinguish();
                    self.get_entity().set_on_fire(false).await;
                }

                // Stop elytra flight and reset sneaking when switching to spectator mode
                if gamemode == GameMode::Spectator {
                    let entity = self.get_entity();
                    if entity.is_fall_flying() {
                        entity.set_fall_flying(false).await;
                    }
                    if entity.is_sneaking() {
                        entity.set_sneaking(false).await;
                    }
                }

                if gamemode != GameMode::Spectator && self.camera_target_id.load().is_some() {
                    self.camera_target_id.store(None);
                    self.client.send_packet_now(&CSetCamera::new(
                        self.entity_id().into()
                    )).await;
                }

                self.living_entity.entity.invulnerable.store(
                    matches!(gamemode, GameMode::Creative | GameMode::Spectator),
                    Ordering::Relaxed,
                );
                self.living_entity
                    .entity
                    .world
                    .load()
                    .broadcast_packet_all(&CPlayerInfoUpdate::new(
                        PlayerInfoFlags::UPDATE_GAME_MODE.bits(),
                        &[pumpkin_protocol::java::client::play::Player {
                            uuid: self.gameprofile.id,
                            actions: &[PlayerAction::UpdateGameMode((gamemode as i32).into())],
                        }],
                    ));

                self.client
                    .enqueue_packet_editioned(
                        &CGameEvent::new(GameEvent::ChangeGameMode, gamemode as i32 as f32),
                        &pumpkin_protocol::bedrock::client::set_player_gamemode::CSetPlayerGamemode {
                            gamemode,
                        },
                    )
                    .await;

                true
            }

            'cancelled: {
                false
            }
        }}
    }

    pub async fn has_permission(self: &Arc<Self>, server: &Server, node: &str) -> bool {
        let perm_manager = server.permission_manager.read().await;
        let result = perm_manager
            .has_permission(&self.gameprofile.id, node, self.permission_lvl.load())
            .await;
        drop(perm_manager);

        let event = server
            .plugin_manager
            .fire(PlayerPermissionCheckEvent::new(
                self.clone(),
                node.to_string(),
                result,
            ))
            .await;
        event.result
    }

    pub fn is_creative(&self) -> bool {
        self.gamemode.load() == GameMode::Creative
    }
}

/// Represents a player's abilities and special powers.
///
/// This struct contains information about the player's current abilities, such as flight, invulnerability, and creative mode.
pub struct Abilities {
    /// Indicates whether the player is invulnerable to damage.
    pub invulnerable: bool,
    /// Indicates whether the player is currently flying.
    pub flying: bool,
    /// Indicates whether the player is allowed to fly (if enabled).
    pub allow_flying: bool,
    /// Indicates whether the player is in creative mode.
    pub creative: bool,
    /// Indicates whether the player is allowed to modify the world.
    pub allow_modify_world: bool,
    /// The player's flying speed.
    pub fly_speed: f32,
    /// The field of view adjustment when the player is walking or sprinting.
    pub walk_speed: f32,
}

impl NBTStorage for Abilities {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            let mut component = NbtCompound::new();
            component.put_bool("invulnerable", self.invulnerable);
            component.put_bool("flying", self.flying);
            component.put_bool("mayfly", self.allow_flying);
            component.put_bool("instabuild", self.creative);
            component.put_bool("mayBuild", self.allow_modify_world);
            component.put_float("flySpeed", self.fly_speed);
            component.put_float("walkSpeed", self.walk_speed);
            nbt.put_compound("abilities", component);
        })
    }

    fn read_nbt<'a>(&'a mut self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            if let Some(component) = nbt.get_compound("abilities") {
                self.invulnerable = component.get_bool("invulnerable").unwrap_or(false);
                self.flying = component.get_bool("flying").unwrap_or(false);
                self.allow_flying = component.get_bool("mayfly").unwrap_or(false);
                self.creative = component.get_bool("instabuild").unwrap_or(false);
                self.allow_modify_world = component.get_bool("mayBuild").unwrap_or(false);
                self.fly_speed = component.get_float("flySpeed").unwrap_or(0.05);
                self.walk_speed = component.get_float("walkSpeed").unwrap_or(0.1);
            }
        })
    }
}

impl NBTStorageInit for Abilities {}

impl Default for Abilities {
    fn default() -> Self {
        Self {
            invulnerable: false,
            flying: false,
            allow_flying: false,
            creative: false,
            allow_modify_world: true,
            fly_speed: 0.05,
            walk_speed: 0.1,
        }
    }
}

impl Abilities {
    pub const fn set_for_gamemode(&mut self, gamemode: GameMode) {
        match gamemode {
            GameMode::Creative => {
                // self.flying = false; // Start not flying
                self.allow_flying = true;
                self.creative = true;
                self.invulnerable = true;
            }
            GameMode::Spectator => {
                self.flying = true;
                self.allow_flying = true;
                self.creative = false;
                self.invulnerable = true;
            }
            _ => {
                self.flying = false;
                self.allow_flying = false;
                self.creative = false;
                self.invulnerable = false;
            }
        }
    }
}
