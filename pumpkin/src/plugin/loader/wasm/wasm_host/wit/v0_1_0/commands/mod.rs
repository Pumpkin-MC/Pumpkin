use wasmtime::component::Resource;

use crate::{
    command::{
        args::{
            GetClientSideArgParser,
            block::{BlockArgumentConsumer, BlockPredicateArgumentConsumer},
            bool::BoolArgConsumer,
            bounded_num::{BoundedNumArgumentConsumer, ToFromNumber},
            difficulty::DifficultyArgumentConsumer,
            entities::EntitiesArgumentConsumer,
            entity::EntityArgumentConsumer,
            entity_anchor::EntityAnchorArgumentConsumer,
            gamemode::GamemodeArgumentConsumer,
            message::MsgArgConsumer,
            players::PlayersArgumentConsumer,
            position_2d::Position2DArgumentConsumer,
            position_3d::Position3DArgumentConsumer,
            position_block::BlockPosArgumentConsumer,
            resource::item::{ItemArgumentConsumer, ItemPredicateArgumentConsumer},
            resource_location::ResourceLocationArgumentConsumer,
            rotation::RotationArgumentConsumer,
            simple::SimpleArgConsumer,
            textcomponent::TextComponentArgConsumer,
            time::TimeArgumentConsumer,
        },
        tree::{
            CommandTree,
            builder::{argument, literal},
        },
    },
    plugin::loader::wasm::wasm_host::{
        DowncastResourceExt,
        state::{
            CommandNodeResource, CommandSenderResource, PluginHostState, TextComponentResource,
        },
        wit::v0_1_0::{
            commands::executor::WasmCommandExecutor,
            pumpkin::{
                self,
                plugin::{
                    command::{
                        Arg, ArgumentType, Command, CommandNode, CommandSender, CommandSenderType,
                        ConsumedArgs, PermissionLevel, StringType,
                    },
                    common::{Locale, Position},
                    entity::EntityBase,
                    player::Player,
                    server::Server,
                    text::TextComponent,
                    world::World,
                },
            },
        },
    },
};

pub mod executor;

impl pumpkin::plugin::command::Host for PluginHostState {}

impl pumpkin::plugin::command::HostConsumedArgs for PluginHostState {
    async fn get_value(&mut self, consumed_args: Resource<ConsumedArgs>, key: String) -> Arg {
        use crate::plugin::loader::wasm::wasm_host::{args::OwnedArg, state::ConsumedArgsResource};

        let resource = self
            .resource_table
            .get::<ConsumedArgsResource>(&Resource::new_own(consumed_args.rep()))
            .expect("invalid consumed-args resource handle");

        let owned = resource.provider.get(&key).cloned().unwrap_or_else(|| {
            panic!(
                "consumed-args.get-value called with unknown key '{key}' (available keys: {:?})",
                resource.provider.keys().collect::<Vec<_>>()
            )
        });

        match owned {
            // NOTE: The WIT entity API is currently very limited; until a richer entity handle
            // exists, fall back to a stable string representation.
            OwnedArg::Entities(entities) => Arg::Entities(
                entities
                    .into_iter()
                    .map(|e| {
                        self.add_entity::<EntityBase>(e)
                            .expect("failed to add entity resource")
                    })
                    .collect::<Vec<_>>(),
            ),
            OwnedArg::Entity(entity) => Arg::Entity(
                self.add_entity::<EntityBase>(entity)
                    .expect("failed to add entity resource"),
            ),
            OwnedArg::Players(players) => {
                let players = players
                    .into_iter()
                    .map(|p| {
                        self.add_player::<Player>(p)
                            .expect("failed to add player resource")
                    })
                    .collect();
                Arg::Players(players)
            }

            OwnedArg::BlockPos(pos) => Arg::BlockPos((pos.0.x, pos.0.y, pos.0.z)),
            OwnedArg::Pos3D(pos) => Arg::Pos3d((pos.x, pos.y, pos.z)),
            OwnedArg::Pos2D(pos) => Arg::Pos2d((pos.x, pos.y)),
            OwnedArg::Rotation(yaw, yaw_rel, pitch, pitch_rel) => {
                Arg::Rotation((yaw, yaw_rel, pitch, pitch_rel))
            }

            OwnedArg::GameMode(mode) => {
                let mode = match mode {
                    pumpkin_util::GameMode::Survival => {
                        pumpkin::plugin::command::Gamemode::Survival
                    }
                    pumpkin_util::GameMode::Creative => {
                        pumpkin::plugin::command::Gamemode::Creative
                    }
                    pumpkin_util::GameMode::Adventure => {
                        pumpkin::plugin::command::Gamemode::Adventure
                    }
                    pumpkin_util::GameMode::Spectator => {
                        pumpkin::plugin::command::Gamemode::Spectator
                    }
                };
                Arg::GameMode(mode)
            }
            OwnedArg::Difficulty(difficulty) => {
                let difficulty = match difficulty {
                    pumpkin_util::Difficulty::Peaceful => {
                        pumpkin::plugin::server::Difficulty::Peaceful
                    }
                    pumpkin_util::Difficulty::Easy => pumpkin::plugin::server::Difficulty::Easy,
                    pumpkin_util::Difficulty::Normal => pumpkin::plugin::server::Difficulty::Normal,
                    pumpkin_util::Difficulty::Hard => pumpkin::plugin::server::Difficulty::Hard,
                };
                Arg::Difficulty(difficulty)
            }

            OwnedArg::CommandTree(_) => Arg::Simple("<command-tree>".to_string()),

            OwnedArg::Item(s) => Arg::Item(s),
            OwnedArg::ItemPredicate(s) => Arg::ItemPredicate(s),
            OwnedArg::ResourceLocation(s) => Arg::ResourceLocation(s),
            OwnedArg::Block(s) => Arg::Block(s),
            OwnedArg::BlockPredicate(s) => Arg::BlockPredicate(s),

            OwnedArg::BossbarColor(color) => {
                let color = match color {
                    crate::world::bossbar::BossbarColor::Pink => {
                        pumpkin::plugin::command::BossbarColor::Pink
                    }
                    crate::world::bossbar::BossbarColor::Blue => {
                        pumpkin::plugin::command::BossbarColor::Blue
                    }
                    crate::world::bossbar::BossbarColor::Red => {
                        pumpkin::plugin::command::BossbarColor::Red
                    }
                    crate::world::bossbar::BossbarColor::Green => {
                        pumpkin::plugin::command::BossbarColor::Green
                    }
                    crate::world::bossbar::BossbarColor::Yellow => {
                        pumpkin::plugin::command::BossbarColor::Yellow
                    }
                    crate::world::bossbar::BossbarColor::Purple => {
                        pumpkin::plugin::command::BossbarColor::Purple
                    }
                    crate::world::bossbar::BossbarColor::White => {
                        pumpkin::plugin::command::BossbarColor::White
                    }
                };
                Arg::BossbarColor(color)
            }
            OwnedArg::BossbarStyle(style) => {
                let style = match style {
                    crate::world::bossbar::BossbarDivisions::NoDivision => {
                        pumpkin::plugin::command::BossbarStyle::NoDivision
                    }
                    crate::world::bossbar::BossbarDivisions::Notches6 => {
                        pumpkin::plugin::command::BossbarStyle::Notches6
                    }
                    crate::world::bossbar::BossbarDivisions::Notches10 => {
                        pumpkin::plugin::command::BossbarStyle::Notches10
                    }
                    crate::world::bossbar::BossbarDivisions::Notches12 => {
                        pumpkin::plugin::command::BossbarStyle::Notches12
                    }
                    crate::world::bossbar::BossbarDivisions::Notches20 => {
                        pumpkin::plugin::command::BossbarStyle::Notches20
                    }
                };
                Arg::BossbarStyle(style)
            }

            OwnedArg::Particle(particle) => Arg::Particle(particle.to_name().to_string()),
            OwnedArg::Msg(msg) => Arg::Msg(msg),
            OwnedArg::TextComponent(component) => {
                let text = self
                    .add_text_component::<TextComponent>(component)
                    .expect("failed to add text-component resource");
                Arg::TextComponent(text)
            }
            OwnedArg::Time(time) => Arg::Time(time),
            OwnedArg::Num(value) => {
                let value = value
                    .map(|n| match n {
                        crate::command::args::bounded_num::Number::F64(v) => {
                            pumpkin::plugin::command::Number::Float64(v)
                        }
                        crate::command::args::bounded_num::Number::F32(v) => {
                            pumpkin::plugin::command::Number::Float32(v)
                        }
                        crate::command::args::bounded_num::Number::I32(v) => {
                            pumpkin::plugin::command::Number::Int32(v)
                        }
                        crate::command::args::bounded_num::Number::I64(v) => {
                            pumpkin::plugin::command::Number::Int64(v)
                        }
                    })
                    .map_err(|e| match e {
                        crate::command::args::bounded_num::NotInBounds::LowerBound(val, bound) => {
                            pumpkin::plugin::command::NotInBounds::LowerBound((
                                match val {
                                    crate::command::args::bounded_num::Number::F64(v) => {
                                        pumpkin::plugin::command::Number::Float64(v)
                                    }
                                    crate::command::args::bounded_num::Number::F32(v) => {
                                        pumpkin::plugin::command::Number::Float32(v)
                                    }
                                    crate::command::args::bounded_num::Number::I32(v) => {
                                        pumpkin::plugin::command::Number::Int32(v)
                                    }
                                    crate::command::args::bounded_num::Number::I64(v) => {
                                        pumpkin::plugin::command::Number::Int64(v)
                                    }
                                },
                                match bound {
                                    crate::command::args::bounded_num::Number::F64(v) => {
                                        pumpkin::plugin::command::Number::Float64(v)
                                    }
                                    crate::command::args::bounded_num::Number::F32(v) => {
                                        pumpkin::plugin::command::Number::Float32(v)
                                    }
                                    crate::command::args::bounded_num::Number::I32(v) => {
                                        pumpkin::plugin::command::Number::Int32(v)
                                    }
                                    crate::command::args::bounded_num::Number::I64(v) => {
                                        pumpkin::plugin::command::Number::Int64(v)
                                    }
                                },
                            ))
                        }
                        crate::command::args::bounded_num::NotInBounds::UpperBound(val, bound) => {
                            pumpkin::plugin::command::NotInBounds::UpperBound((
                                match val {
                                    crate::command::args::bounded_num::Number::F64(v) => {
                                        pumpkin::plugin::command::Number::Float64(v)
                                    }
                                    crate::command::args::bounded_num::Number::F32(v) => {
                                        pumpkin::plugin::command::Number::Float32(v)
                                    }
                                    crate::command::args::bounded_num::Number::I32(v) => {
                                        pumpkin::plugin::command::Number::Int32(v)
                                    }
                                    crate::command::args::bounded_num::Number::I64(v) => {
                                        pumpkin::plugin::command::Number::Int64(v)
                                    }
                                },
                                match bound {
                                    crate::command::args::bounded_num::Number::F64(v) => {
                                        pumpkin::plugin::command::Number::Float64(v)
                                    }
                                    crate::command::args::bounded_num::Number::F32(v) => {
                                        pumpkin::plugin::command::Number::Float32(v)
                                    }
                                    crate::command::args::bounded_num::Number::I32(v) => {
                                        pumpkin::plugin::command::Number::Int32(v)
                                    }
                                    crate::command::args::bounded_num::Number::I64(v) => {
                                        pumpkin::plugin::command::Number::Int64(v)
                                    }
                                },
                            ))
                        }
                    });
                Arg::Num(value)
            }
            OwnedArg::Bool(value) => Arg::Bool(value),
            OwnedArg::Simple(value) => Arg::Simple(value),
            OwnedArg::SoundCategory(category) => {
                let category = match category {
                    pumpkin_data::sound::SoundCategory::Master => {
                        pumpkin::plugin::command::SoundCategory::Master
                    }
                    pumpkin_data::sound::SoundCategory::Music => {
                        pumpkin::plugin::command::SoundCategory::Music
                    }
                    pumpkin_data::sound::SoundCategory::Records => {
                        pumpkin::plugin::command::SoundCategory::Records
                    }
                    pumpkin_data::sound::SoundCategory::Weather => {
                        pumpkin::plugin::command::SoundCategory::Weather
                    }
                    pumpkin_data::sound::SoundCategory::Blocks => {
                        pumpkin::plugin::command::SoundCategory::Blocks
                    }
                    pumpkin_data::sound::SoundCategory::Hostile => {
                        pumpkin::plugin::command::SoundCategory::Hostile
                    }
                    pumpkin_data::sound::SoundCategory::Neutral => {
                        pumpkin::plugin::command::SoundCategory::Neutral
                    }
                    pumpkin_data::sound::SoundCategory::Players => {
                        pumpkin::plugin::command::SoundCategory::Players
                    }
                    pumpkin_data::sound::SoundCategory::Ambient => {
                        pumpkin::plugin::command::SoundCategory::Ambient
                    }
                    pumpkin_data::sound::SoundCategory::Voice => {
                        pumpkin::plugin::command::SoundCategory::Voice
                    }
                    pumpkin_data::sound::SoundCategory::Ui => {
                        pumpkin::plugin::command::SoundCategory::Ui
                    }
                };
                Arg::SoundCategory(category)
            }
            OwnedArg::DamageType(damage_type) => {
                Arg::DamageType(damage_type.message_id.to_string())
            }
            OwnedArg::Effect(effect) => Arg::Effect(effect.minecraft_name.to_string()),
            OwnedArg::Enchantment(enchantment) => {
                Arg::Enchantment(enchantment.registry_key.to_string())
            }
            OwnedArg::EntityAnchor(anchor) => {
                let anchor = match anchor {
                    crate::command::args::EntityAnchor::Feet => {
                        pumpkin::plugin::command::EntityAnchor::Feet
                    }
                    crate::command::args::EntityAnchor::Eyes => {
                        pumpkin::plugin::command::EntityAnchor::Eyes
                    }
                };
                Arg::EntityAnchor(anchor)
            }
        }
    }

    async fn drop(&mut self, rep: Resource<ConsumedArgs>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<crate::plugin::loader::wasm::wasm_host::state::ConsumedArgsResource>(
                Resource::new_own(rep.rep()),
            )?;
        Ok(())
    }
}

impl pumpkin::plugin::command::HostCommand for PluginHostState {
    async fn new(&mut self, names: Vec<String>, description: String) -> Resource<Command> {
        self.add_command(CommandTree::new(names, description))
            .unwrap()
    }

    async fn then(&mut self, command: Resource<Command>, node: Resource<CommandNode>) -> () {
        let node_resource = node.consume(self);
        let command_resource = self
            .resource_table
            .get_mut::<crate::plugin::loader::wasm::wasm_host::state::CommandResource>(
                &Resource::new_own(command.rep()),
            )
            .expect("invalid command resource handle");

        command_resource.provider = command_resource
            .provider
            .clone()
            .then(node_resource.provider);
    }

    async fn execute_with_handler_id(&mut self, command: Resource<Command>, handler_id: u32) -> () {
        let plugin = self
            .plugin
            .as_ref()
            .expect("plugin should always be initialized here")
            .upgrade()
            .expect("plugin has been dropped");

        let server = self
            .server
            .clone()
            .expect("server should be set before command registration");

        let executor = WasmCommandExecutor {
            handler_id,
            plugin,
            server,
        };

        let command_resource = self
            .resource_table
            .get_mut::<crate::plugin::loader::wasm::wasm_host::state::CommandResource>(
                &Resource::new_own(command.rep()),
            )
            .expect("invalid command resource handle");

        command_resource.provider = command_resource.provider.clone().execute(executor);
    }

    async fn drop(&mut self, rep: Resource<Command>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<crate::plugin::loader::wasm::wasm_host::state::CommandResource>(
                Resource::new_own(rep.rep()),
            )?;
        Ok(())
    }
}

impl pumpkin::plugin::command::HostCommandSender for PluginHostState {
    async fn get_command_sender_type(
        &mut self,
        _command_sender: Resource<CommandSender>,
    ) -> CommandSenderType {
        todo!()
    }

    async fn send_message(
        &mut self,
        command_sender: Resource<CommandSender>,
        text: Resource<TextComponent>,
    ) -> () {
        let text_resource = self
            .resource_table
            .get::<TextComponentResource>(&Resource::new_own(text.rep()))
            .expect("invalid text-component resource handle");
        let component = text_resource.provider.clone();

        let sender_resource = self
            .resource_table
            .get::<CommandSenderResource>(&Resource::new_own(command_sender.rep()))
            .expect("invalid command-sender resource handle");

        sender_resource.provider.send_message(component).await;
    }

    async fn set_success_count(&mut self, command_sender: Resource<CommandSender>, count: i32) {
        let resource = self
            .resource_table
            .get_mut::<CommandSenderResource>(&Resource::new_own(command_sender.rep()))
            .expect("invalid command-sender resource handle");

        resource.provider.set_success_count(count as u32);
    }

    async fn is_player(&mut self, command_sender: Resource<CommandSender>) -> bool {
        let resource = self
            .resource_table
            .get::<CommandSenderResource>(&Resource::new_own(command_sender.rep()))
            .expect("invalid command-sender resource handle");

        matches!(resource.provider, crate::command::CommandSender::Player(_))
    }

    async fn is_console(&mut self, command_sender: Resource<CommandSender>) -> bool {
        let resource = self
            .resource_table
            .get::<CommandSenderResource>(&Resource::new_own(command_sender.rep()))
            .expect("invalid command-sender resource handle");

        matches!(
            resource.provider,
            crate::command::CommandSender::Console | crate::command::CommandSender::Rcon(_)
        )
    }

    async fn as_player(
        &mut self,
        command_sender: Resource<CommandSender>,
    ) -> Option<Resource<Player>> {
        let resource = self
            .resource_table
            .get::<CommandSenderResource>(&Resource::new_own(command_sender.rep()))
            .expect("invalid command-sender resource handle");

        if let crate::command::CommandSender::Player(player) = &resource.provider {
            let player = player.clone();
            Some(self.add_player(player).unwrap())
        } else {
            None
        }
    }

    async fn permission_level(
        &mut self,
        command_sender: Resource<CommandSender>,
    ) -> PermissionLevel {
        let resource = self
            .resource_table
            .get::<CommandSenderResource>(&Resource::new_own(command_sender.rep()))
            .expect("invalid command-sender resource handle");

        match resource.provider.permission_lvl() {
            pumpkin_util::PermissionLvl::Zero => PermissionLevel::Zero,
            pumpkin_util::PermissionLvl::One => PermissionLevel::One,
            pumpkin_util::PermissionLvl::Two => PermissionLevel::Two,
            pumpkin_util::PermissionLvl::Three => PermissionLevel::Three,
            pumpkin_util::PermissionLvl::Four => PermissionLevel::Four,
        }
    }

    async fn has_permission_level(
        &mut self,
        command_sender: Resource<CommandSender>,
        level: PermissionLevel,
    ) -> bool {
        let resource = self
            .resource_table
            .get::<CommandSenderResource>(&Resource::new_own(command_sender.rep()))
            .expect("invalid command-sender resource handle");

        let required = match level {
            PermissionLevel::Zero => pumpkin_util::PermissionLvl::Zero,
            PermissionLevel::One => pumpkin_util::PermissionLvl::One,
            PermissionLevel::Two => pumpkin_util::PermissionLvl::Two,
            PermissionLevel::Three => pumpkin_util::PermissionLvl::Three,
            PermissionLevel::Four => pumpkin_util::PermissionLvl::Four,
        };

        resource.provider.permission_lvl() >= required
    }

    async fn has_permission(
        &mut self,
        command_sender: Resource<CommandSender>,
        server: Resource<Server>,
        node: String,
    ) -> bool {
        let sender_resource = self
            .resource_table
            .get::<CommandSenderResource>(&Resource::new_own(command_sender.rep()))
            .expect("invalid command-sender resource handle");

        let server_resource = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::ServerResource>(
                &Resource::new_own(server.rep()),
            )
            .expect("invalid server resource handle");

        sender_resource
            .provider
            .has_permission(&server_resource.provider, &node)
            .await
    }

    async fn position(&mut self, command_sender: Resource<CommandSender>) -> Option<Position> {
        let resource = self
            .resource_table
            .get::<CommandSenderResource>(&Resource::new_own(command_sender.rep()))
            .expect("invalid command-sender resource handle");

        resource
            .provider
            .position()
            .map(|pos| (pos.x, pos.y, pos.z))
    }

    async fn world(&mut self, command_sender: Resource<CommandSender>) -> Option<Resource<World>> {
        let resource = self
            .resource_table
            .get::<CommandSenderResource>(&Resource::new_own(command_sender.rep()))
            .expect("invalid command-sender resource handle");

        if let Some(world) = resource.provider.world() {
            Some(
                self.resource_table
                    .push(
                        crate::plugin::loader::wasm::wasm_host::state::WasmResource {
                            provider: world,
                        },
                    )
                    .map(|r| wasmtime::component::Resource::new_own(r.rep()))
                    .unwrap(),
            )
        } else {
            None
        }
    }

    #[allow(clippy::too_many_lines)]
    async fn get_locale(&mut self, command_sender: Resource<CommandSender>) -> Locale {
        let resource = self
            .resource_table
            .get::<CommandSenderResource>(&Resource::new_own(command_sender.rep()))
            .expect("invalid command-sender resource handle");

        match resource.provider.get_locale() {
            pumpkin_util::translation::Locale::AfZa => Locale::AfZa,
            pumpkin_util::translation::Locale::ArSa => Locale::ArSa,
            pumpkin_util::translation::Locale::AstEs => Locale::AstEs,
            pumpkin_util::translation::Locale::AzAz => Locale::AzAz,
            pumpkin_util::translation::Locale::BaRu => Locale::BaRu,
            pumpkin_util::translation::Locale::Bar => Locale::Bar,
            pumpkin_util::translation::Locale::BeBy => Locale::BeBy,
            pumpkin_util::translation::Locale::BgBg => Locale::BgBg,
            pumpkin_util::translation::Locale::BrFr => Locale::BrFr,
            pumpkin_util::translation::Locale::Brb => Locale::Brb,
            pumpkin_util::translation::Locale::BsBa => Locale::BsBa,
            pumpkin_util::translation::Locale::CaEs => Locale::CaEs,
            pumpkin_util::translation::Locale::CsCz => Locale::CsCz,
            pumpkin_util::translation::Locale::CyGb => Locale::CyGb,
            pumpkin_util::translation::Locale::DaDk => Locale::DaDk,
            pumpkin_util::translation::Locale::DeAt => Locale::DeAt,
            pumpkin_util::translation::Locale::DeCh => Locale::DeCh,
            pumpkin_util::translation::Locale::DeDe => Locale::DeDe,
            pumpkin_util::translation::Locale::ElGr => Locale::ElGr,
            pumpkin_util::translation::Locale::EnAu => Locale::EnAu,
            pumpkin_util::translation::Locale::EnCa => Locale::EnCa,
            pumpkin_util::translation::Locale::EnGb => Locale::EnGb,
            pumpkin_util::translation::Locale::EnNz => Locale::EnNz,
            pumpkin_util::translation::Locale::EnPt => Locale::EnPt,
            pumpkin_util::translation::Locale::EnUd => Locale::EnUd,
            pumpkin_util::translation::Locale::EnUs => Locale::EnUs,
            pumpkin_util::translation::Locale::Enp => Locale::Enp,
            pumpkin_util::translation::Locale::Enws => Locale::Enws,
            pumpkin_util::translation::Locale::EoUy => Locale::EoUy,
            pumpkin_util::translation::Locale::EsAr => Locale::EsAr,
            pumpkin_util::translation::Locale::EsCl => Locale::EsCl,
            pumpkin_util::translation::Locale::EsEc => Locale::EsEc,
            pumpkin_util::translation::Locale::EsEs => Locale::EsEs,
            pumpkin_util::translation::Locale::EsMx => Locale::EsMx,
            pumpkin_util::translation::Locale::EsUy => Locale::EsUy,
            pumpkin_util::translation::Locale::EsVe => Locale::EsVe,
            pumpkin_util::translation::Locale::Esan => Locale::Esan,
            pumpkin_util::translation::Locale::EtEe => Locale::EtEe,
            pumpkin_util::translation::Locale::EuEs => Locale::EuEs,
            pumpkin_util::translation::Locale::FaIr => Locale::FaIr,
            pumpkin_util::translation::Locale::FiFi => Locale::FiFi,
            pumpkin_util::translation::Locale::FilPh => Locale::FilPh,
            pumpkin_util::translation::Locale::FoFo => Locale::FoFo,
            pumpkin_util::translation::Locale::FrCa => Locale::FrCa,
            pumpkin_util::translation::Locale::FrFr => Locale::FrFr,
            pumpkin_util::translation::Locale::FraDe => Locale::FraDe,
            pumpkin_util::translation::Locale::FurIt => Locale::FurIt,
            pumpkin_util::translation::Locale::FyNl => Locale::FyNl,
            pumpkin_util::translation::Locale::GaIe => Locale::GaIe,
            pumpkin_util::translation::Locale::GdGb => Locale::GdGb,
            pumpkin_util::translation::Locale::GlEs => Locale::GlEs,
            pumpkin_util::translation::Locale::HawUs => Locale::HawUs,
            pumpkin_util::translation::Locale::HeIl => Locale::HeIl,
            pumpkin_util::translation::Locale::HiIn => Locale::HiIn,
            pumpkin_util::translation::Locale::HrHr => Locale::HrHr,
            pumpkin_util::translation::Locale::HuHu => Locale::HuHu,
            pumpkin_util::translation::Locale::HyAm => Locale::HyAm,
            pumpkin_util::translation::Locale::IdId => Locale::IdId,
            pumpkin_util::translation::Locale::IgNg => Locale::IgNg,
            pumpkin_util::translation::Locale::IoEn => Locale::IoEn,
            pumpkin_util::translation::Locale::IsIs => Locale::IsIs,
            pumpkin_util::translation::Locale::Isv => Locale::Isv,
            pumpkin_util::translation::Locale::ItIt => Locale::ItIt,
            pumpkin_util::translation::Locale::JaJp => Locale::JaJp,
            pumpkin_util::translation::Locale::JboEn => Locale::JboEn,
            pumpkin_util::translation::Locale::KaGe => Locale::KaGe,
            pumpkin_util::translation::Locale::KkKz => Locale::KkKz,
            pumpkin_util::translation::Locale::KnIn => Locale::KnIn,
            pumpkin_util::translation::Locale::KoKr => Locale::KoKr,
            pumpkin_util::translation::Locale::Ksh => Locale::Ksh,
            pumpkin_util::translation::Locale::KwGb => Locale::KwGb,
            pumpkin_util::translation::Locale::LaLa => Locale::LaLa,
            pumpkin_util::translation::Locale::LbLu => Locale::LbLu,
            pumpkin_util::translation::Locale::LiLi => Locale::LiLi,
            pumpkin_util::translation::Locale::Lmo => Locale::Lmo,
            pumpkin_util::translation::Locale::LoLa => Locale::LoLa,
            pumpkin_util::translation::Locale::LolUs => Locale::LolUs,
            pumpkin_util::translation::Locale::LtLt => Locale::LtLt,
            pumpkin_util::translation::Locale::LvLv => Locale::LvLv,
            pumpkin_util::translation::Locale::Lzh => Locale::Lzh,
            pumpkin_util::translation::Locale::MkMk => Locale::MkMk,
            pumpkin_util::translation::Locale::MnMn => Locale::MnMn,
            pumpkin_util::translation::Locale::MsMy => Locale::MsMy,
            pumpkin_util::translation::Locale::MtMt => Locale::MtMt,
            pumpkin_util::translation::Locale::Nah => Locale::Nah,
            pumpkin_util::translation::Locale::NdsDe => Locale::NdsDe,
            pumpkin_util::translation::Locale::NlBe => Locale::NlBe,
            pumpkin_util::translation::Locale::NlNl => Locale::NlNl,
            pumpkin_util::translation::Locale::NnNo => Locale::NnNo,
            pumpkin_util::translation::Locale::NoNo => Locale::NoNo,
            pumpkin_util::translation::Locale::OcFr => Locale::OcFr,
            pumpkin_util::translation::Locale::Ovd => Locale::Ovd,
            pumpkin_util::translation::Locale::PlPl => Locale::PlPl,
            pumpkin_util::translation::Locale::PtBr => Locale::PtBr,
            pumpkin_util::translation::Locale::PtPt => Locale::PtPt,
            pumpkin_util::translation::Locale::QyaAa => Locale::QyaAa,
            pumpkin_util::translation::Locale::RoRo => Locale::RoRo,
            pumpkin_util::translation::Locale::Rpr => Locale::Rpr,
            pumpkin_util::translation::Locale::RuRu => Locale::RuRu,
            pumpkin_util::translation::Locale::RyUa => Locale::RyUa,
            pumpkin_util::translation::Locale::SahSah => Locale::SahSah,
            pumpkin_util::translation::Locale::SeNo => Locale::SeNo,
            pumpkin_util::translation::Locale::SkSk => Locale::SkSk,
            pumpkin_util::translation::Locale::SlSi => Locale::SlSi,
            pumpkin_util::translation::Locale::SoSo => Locale::SoSo,
            pumpkin_util::translation::Locale::SqAl => Locale::SqAl,
            pumpkin_util::translation::Locale::SrCs => Locale::SrCs,
            pumpkin_util::translation::Locale::SrSp => Locale::SrSp,
            pumpkin_util::translation::Locale::SvSe => Locale::SvSe,
            pumpkin_util::translation::Locale::Sxu => Locale::Sxu,
            pumpkin_util::translation::Locale::Szl => Locale::Szl,
            pumpkin_util::translation::Locale::TaIn => Locale::TaIn,
            pumpkin_util::translation::Locale::ThTh => Locale::ThTh,
            pumpkin_util::translation::Locale::TlPh => Locale::TlPh,
            pumpkin_util::translation::Locale::TlhAa => Locale::TlhAa,
            pumpkin_util::translation::Locale::Tok => Locale::Tok,
            pumpkin_util::translation::Locale::TrTr => Locale::TrTr,
            pumpkin_util::translation::Locale::TtRu => Locale::TtRu,
            pumpkin_util::translation::Locale::UkUa => Locale::UkUa,
            pumpkin_util::translation::Locale::ValEs => Locale::ValEs,
            pumpkin_util::translation::Locale::VecIt => Locale::VecIt,
            pumpkin_util::translation::Locale::ViVn => Locale::ViVn,
            pumpkin_util::translation::Locale::YiDe => Locale::YiDe,
            pumpkin_util::translation::Locale::YoNg => Locale::YoNg,
            pumpkin_util::translation::Locale::ZhCn => Locale::ZhCn,
            pumpkin_util::translation::Locale::ZhHk => Locale::ZhHk,
            pumpkin_util::translation::Locale::ZhTw => Locale::ZhTw,
            pumpkin_util::translation::Locale::ZlmArab => Locale::ZlmArab,
        }
    }

    async fn should_receive_feedback(&mut self, command_sender: Resource<CommandSender>) -> bool {
        let resource = self
            .resource_table
            .get::<CommandSenderResource>(&Resource::new_own(command_sender.rep()))
            .expect("invalid command-sender resource handle");

        resource.provider.should_receive_feedback()
    }

    async fn should_broadcast_console_to_ops(
        &mut self,
        command_sender: Resource<CommandSender>,
    ) -> bool {
        let resource = self
            .resource_table
            .get::<CommandSenderResource>(&Resource::new_own(command_sender.rep()))
            .expect("invalid command-sender resource handle");

        resource.provider.should_broadcast_console_to_ops()
    }

    async fn should_track_output(&mut self, command_sender: Resource<CommandSender>) -> bool {
        let resource = self
            .resource_table
            .get::<CommandSenderResource>(&Resource::new_own(command_sender.rep()))
            .expect("invalid command-sender resource handle");

        resource.provider.should_track_output()
    }

    async fn drop(&mut self, rep: Resource<CommandSender>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<CommandSenderResource>(Resource::new_own(rep.rep()))?;
        Ok(())
    }
}

impl DowncastResourceExt<CommandNodeResource> for Resource<CommandNode> {
    fn downcast_ref<'a>(&'a self, state: &'a mut PluginHostState) -> &'a CommandNodeResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid command-node resource handle")
            .downcast_ref()
            .expect("resource type mismatch")
    }

    fn downcast_mut<'a>(&'a self, state: &'a mut PluginHostState) -> &'a mut CommandNodeResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid command-node resource handle")
            .downcast_mut()
            .expect("resource type mismatch")
    }

    fn consume(self, state: &mut PluginHostState) -> CommandNodeResource {
        state
            .resource_table
            .delete(Resource::new_own(self.rep()))
            .expect("invalid command-node resource handle")
    }
}

fn bounded_num_argument<T: ToFromNumber + 'static>(
    state: &mut PluginHostState,
    name: String,
    min: Option<T>,
    max: Option<T>,
) -> Resource<CommandNode>
where
    BoundedNumArgumentConsumer<T>: GetClientSideArgParser,
{
    let mut consumer = BoundedNumArgumentConsumer::<T>::new();
    if let Some(min) = min {
        consumer = consumer.min(min);
    }
    if let Some(max) = max {
        consumer = consumer.max(max);
    }
    state.add_command_node(argument(name, consumer)).unwrap()
}

impl pumpkin::plugin::command::HostCommandNode for PluginHostState {
    async fn literal(&mut self, name: String) -> Resource<CommandNode> {
        self.add_command_node(literal(name)).unwrap()
    }

    async fn argument(&mut self, name: String, arg_type: ArgumentType) -> Resource<CommandNode> {
        match arg_type {
            ArgumentType::Bool => self
                .add_command_node(argument(name, BoolArgConsumer))
                .unwrap(),
            ArgumentType::Float((min, max)) => bounded_num_argument(self, name, min, max),
            ArgumentType::Double((min, max)) => bounded_num_argument(self, name, min, max),
            ArgumentType::Integer((min, max)) => bounded_num_argument(self, name, min, max),
            ArgumentType::Long((min, max)) => bounded_num_argument(self, name, min, max),
            ArgumentType::String(string_type) => match string_type {
                StringType::SingleWord | StringType::Quotable => self
                    .add_command_node(argument(name, SimpleArgConsumer))
                    .unwrap(),
                StringType::Greedy => self
                    .add_command_node(argument(name, MsgArgConsumer))
                    .unwrap(),
            },
            ArgumentType::Entities => self
                .add_command_node(argument(name, EntitiesArgumentConsumer))
                .unwrap(),
            ArgumentType::Entity => self
                .add_command_node(argument(name, EntityArgumentConsumer))
                .unwrap(),
            ArgumentType::Players | ArgumentType::GameProfile => self
                .add_command_node(argument(name, PlayersArgumentConsumer))
                .unwrap(),
            ArgumentType::BlockPos => self
                .add_command_node(argument(name, BlockPosArgumentConsumer))
                .unwrap(),
            ArgumentType::Position3d => self
                .add_command_node(argument(name, Position3DArgumentConsumer))
                .unwrap(),
            ArgumentType::Position2d => self
                .add_command_node(argument(name, Position2DArgumentConsumer))
                .unwrap(),
            ArgumentType::BlockState => self
                .add_command_node(argument(name, BlockArgumentConsumer))
                .unwrap(),
            ArgumentType::BlockPredicate => self
                .add_command_node(argument(name, BlockPredicateArgumentConsumer))
                .unwrap(),
            ArgumentType::Item => self
                .add_command_node(argument(name, ItemArgumentConsumer))
                .unwrap(),
            ArgumentType::ItemPredicate => self
                .add_command_node(argument(name, ItemPredicateArgumentConsumer))
                .unwrap(),
            ArgumentType::Component => self
                .add_command_node(argument(name, TextComponentArgConsumer))
                .unwrap(),
            ArgumentType::Rotation => self
                .add_command_node(argument(name, RotationArgumentConsumer))
                .unwrap(),
            ArgumentType::ResourceLocation | ArgumentType::Resource(_) => self
                .add_command_node(argument(name, ResourceLocationArgumentConsumer))
                .unwrap(),
            ArgumentType::EntityAnchor => self
                .add_command_node(argument(name, EntityAnchorArgumentConsumer))
                .unwrap(),
            ArgumentType::Gamemode => self
                .add_command_node(argument(name, GamemodeArgumentConsumer))
                .unwrap(),
            ArgumentType::Difficulty => self
                .add_command_node(argument(name, DifficultyArgumentConsumer))
                .unwrap(),
            ArgumentType::Time(_) => self
                .add_command_node(argument(name, TimeArgumentConsumer))
                .unwrap(),
        }
    }

    async fn then(
        &mut self,
        self_command_node: Resource<CommandNode>,
        node: Resource<CommandNode>,
    ) {
        let child_resource = node.consume(self);
        let parent_resource = self_command_node.downcast_mut(self);
        let builder = std::mem::replace(&mut parent_resource.provider, literal(""));
        parent_resource.provider = builder.then(child_resource.provider);
    }

    async fn execute_with_handler_id(
        &mut self,
        command_node: Resource<CommandNode>,
        handler_id: u32,
    ) {
        let plugin = self
            .plugin
            .as_ref()
            .expect("plugin should always be initialized here")
            .upgrade()
            .expect("plugin has been dropped");

        let server = self
            .server
            .clone()
            .expect("server should be set before command registration");

        let executor = WasmCommandExecutor {
            handler_id,
            plugin,
            server,
        };

        let resource = command_node.downcast_mut(self);
        // Unless we make the native command registration code less convenient to use, this is our best option
        let builder = std::mem::replace(&mut resource.provider, literal(""));
        resource.provider = builder.execute(executor);
    }

    async fn require_with_handler_id(
        &mut self,
        _command_node: Resource<CommandNode>,
        _handler_id: u32,
    ) {
        todo!()
    }

    async fn drop(&mut self, rep: Resource<CommandNode>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<CommandNodeResource>(Resource::new_own(rep.rep()))?;
        Ok(())
    }
}
